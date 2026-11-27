use motaa_engine::{ChartRequest, calculate, houses::System};
use serde::Deserialize;

#[derive(Deserialize)]
struct RuntimeCase {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    offset: i32,
    latitude: f64,
    longitude: f64,
    tropical: bool,
    mode: usize,
    positions: [f64; 12],
    #[serde(default)]
    tropical_positions: Option<[f64; 12]>,
    angle: f64,
    houses: [[f64; 12]; 8],
    western: [[u8; 13]; 13],
    dasha: (u8, f64),
    #[serde(default)]
    moon_clock: Option<[f64; 3]>,
    #[serde(default)]
    moon_state: Option<[f64; 6]>,
    speeds: [f64; 12],
    declinations: [f64; 12],
    distances: [f64; 12],
    house_midpoints: [[f64; 12]; 8],
    western_all_houses: [[[u8; 13]; 12]; 8],
}

impl RuntimeCase {
    fn birth(&self) -> ChartRequest {
        ChartRequest {
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            timezone_offset_minutes: self.offset,
            timezone_id: None,
            latitude: self.latitude,
            longitude: self.longitude,
            ..Default::default()
        }
    }
}

fn cases() -> Vec<RuntimeCase> {
    serde_json::from_str(include_str!("../data/motaa-v31.1-runtime-oracle.json")).unwrap()
}

fn error(observed: f64, expected: f64) -> f64 {
    ((observed - expected + 180.).rem_euclid(360.) - 180.).abs()
}

#[test]
fn compare_reference_to_shipped_motaa_runtime() {
    let cases = if let Some(path) = std::env::var_os("MOTAA_RUNTIME_ORACLE") {
        let captured = std::fs::read(path).expect("read live Android oracle");
        let cases: Vec<RuntimeCase> =
            serde_json::from_slice(&captured).expect("decode live Android oracle");
        assert!(
            cases.len() >= 14,
            "live oracle must include the expanded edge cases"
        );
        cases
    } else {
        let cases = cases();
        assert_eq!(cases.len(), 14);
        cases
    };
    let mut house_mismatches = Vec::new();
    let mut max_distance: f64 = 0.;
    let mut max_declination: f64 = 0.;
    let mut max_motion: f64 = 0.;
    let mut max_cusp: f64 = 0.;
    let mut max_body = [0_f64; 12];
    let mut max_ascendant: f64 = 0.;
    let mut max_dasha_start_drift: f64 = 0.;
    let mut max_dasha_balance_drift: f64 = 0.;
    let mut dasha_civil_date_mismatches = 0;
    let mut sidereal_moon_square_error = [0.; 2];
    let mut moon_clock_count = 0;
    let mut tropical_clock_error = [0.0_f64; 2];
    let mut clock_dasha_mismatches = [0; 3];
    let mut worst_true_node_case: Option<(f64, i32, u8, u8, usize)> = None;
    for case in cases {
        if !case.tropical
            && let Some(tropical) = case.tropical_positions
        {
            let offsets = std::array::from_fn::<_, 12, _>(|index| {
                (tropical[index] - case.positions[index]).rem_euclid(360.)
            });
            let range = offsets
                .iter()
                .copied()
                .fold([f64::INFINITY, f64::NEG_INFINITY], |[low, high], offset| {
                    [low.min(offset), high.max(offset)]
                });
            println!(
                "APK offset {}-{}-{} mode {}: Sun {:.8}°, Moon {:.8}°, body spread {:.8}°",
                case.year,
                case.month,
                case.day,
                case.mode,
                offsets[0],
                offsets[1],
                range[1] - range[0]
            );
        }
        let mut chart = calculate(&case.birth())
            .unwrap()
            .reference_ephemeris
            .unwrap();
        chart.select_ayanamsha(case.mode).unwrap();
        if !case.tropical
            && let Some(tropical) = case.tropical_positions
        {
            let expected = (tropical[1] - case.positions[1]).rem_euclid(360.);
            let actual = (chart.longitude("Moon", true).unwrap()
                - chart.longitude("Moon", false).unwrap())
            .rem_euclid(360.);
            assert!(
                (actual - expected).abs() < 0.00001,
                "sidereal frame: {actual} vs {expected}"
            );
        }

        {
            let (code, expected_balance) = case.dasha;
            let lord = [
                "Saturn", "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "", "Rahu", "Ketu",
            ][usize::from(code)];
            let timeline =
                motaa_engine::dasha::calculate(&case.birth(), case.positions[1]).unwrap();
            assert_eq!(timeline.major[0].name(), lord);
            assert!((timeline.balance_days - expected_balance).abs() < 1e-8);
            if let Some([original_ut, original_delta, original_tropical_moon]) = case.moon_clock {
                let desktop_ut = chart.julian_ephemeris_day - chart.delta_t_seconds / 86400.;
                assert!((desktop_ut - original_ut).abs() * 86400. < 0.01);
                assert!((chart.delta_t_seconds - original_delta).abs() < 0.003);
                let tropical = chart.longitude("Moon", true).unwrap();
                let apparent_error =
                    (tropical - original_tropical_moon + 180.).rem_euclid(360.) - 180.;
                let original_tt = original_ut + original_delta / 86400.;
                let apparent = |tt| {
                    (astro::lunar::geocent_ecl_pos(tt).0.long + astro::nutation::nutation(tt).0)
                        .to_degrees()
                };
                let clock_adjustment = apparent(original_tt) - apparent(chart.julian_ephemeris_day);
                let observed = chart.longitude("Moon", case.tropical).unwrap();
                let frame_adjustment = if case.tropical {
                    0.
                } else {
                    astro::nutation::nutation(original_tt).0.to_degrees()
                };
                moon_clock_count += 1;
                tropical_clock_error[0] =
                    tropical_clock_error[0].max(error(tropical, original_tropical_moon));
                tropical_clock_error[1] = tropical_clock_error[1]
                    .max(error(tropical + clock_adjustment, original_tropical_moon));
                for (mismatches, candidate) in clock_dasha_mismatches.iter_mut().zip([
                    observed,
                    observed + clock_adjustment,
                    observed + clock_adjustment - frame_adjustment,
                ]) {
                    let candidate =
                        motaa_engine::dasha::calculate(&case.birth(), candidate.rem_euclid(360.))
                            .unwrap();
                    for (period, original) in candidate.major.iter().zip(&timeline.major) {
                        let date =
                            |start| motaa_engine::dasha::from_day_number(start).unwrap().date();
                        *mismatches += usize::from(date(period.start) != date(original.start));
                    }
                }
                println!(
                    "Moon clock {}-{}-{}: desktop minus APK Delta T {:+.3} s, tropical Moon {apparent_error:+.6}°",
                    case.year,
                    case.month,
                    case.day,
                    chart.delta_t_seconds - original_delta
                );
            }
            let desktop_moon = chart.longitude("Moon", case.tropical).unwrap();
            if !case.tropical {
                let nutation = astro::nutation::nutation(chart.julian_ephemeris_day)
                    .0
                    .to_degrees();
                sidereal_moon_square_error[0] += error(desktop_moon, case.positions[1]).powi(2);
                sidereal_moon_square_error[1] +=
                    error(desktop_moon - nutation, case.positions[1]).powi(2);
            }
            let desktop = motaa_engine::dasha::calculate(&case.birth(), desktop_moon).unwrap();
            assert_eq!(desktop.major[0].name(), lord);
            max_dasha_balance_drift =
                max_dasha_balance_drift.max((desktop.balance_days - expected_balance).abs());
            for (observed, original) in desktop.major.iter().zip(timeline.major) {
                max_dasha_start_drift =
                    max_dasha_start_drift.max((observed.start - original.start).abs());
                let civil_date =
                    |start| motaa_engine::dasha::from_day_number(start).unwrap().date();
                dasha_civil_date_mismatches +=
                    usize::from(civil_date(observed.start) != civil_date(original.start));
            }
        }
        {
            let rates = chart.longitude_rates(case.tropical, case.mode).unwrap();
            let names = [
                "Sun",
                "Moon",
                "Mars",
                "Mercury",
                "Jupiter",
                "Venus",
                "Saturn",
                "True node",
                "Mean node",
                "Uranus",
                "Neptune",
                "Pluto",
            ];
            for (name, expected) in names.into_iter().zip(case.speeds) {
                let (_, observed) = rates.iter().find(|(body, _)| *body == name).unwrap();
                max_motion = max_motion.max((observed - expected).abs());
                assert!(
                    (observed - expected).abs() < 0.025,
                    "motion {name} on {}-{}-{}: {observed} vs {expected}",
                    case.year,
                    case.month,
                    case.day
                );
            }
        }
        {
            let names = [
                "Sun",
                "Moon",
                "Mars",
                "Mercury",
                "Jupiter",
                "Venus",
                "Saturn",
                "True node",
                "Mean node",
                "Uranus",
                "Neptune",
                "Pluto",
            ];
            for (name, expected) in names.into_iter().zip(case.declinations) {
                let (longitude, latitude) = match name {
                    "True node" => (chart.lunar_nodes(false, true)[0], 0.),
                    "Mean node" => (chart.lunar_nodes(true, true)[0], 0.),
                    _ => {
                        let body = chart.bodies.iter().find(|body| body.name == name).unwrap();
                        (body.tropical_longitude_deg, body.ecliptic_latitude_deg)
                    }
                };
                let observed = chart.declination(longitude, latitude);
                max_declination = max_declination.max((observed - expected).abs());
                assert!(
                    (observed - expected).abs() < 0.025,
                    "declination {name} on {}-{}-{}: {observed} vs {expected}",
                    case.year,
                    case.month,
                    case.day
                );
            }
        }
        let expected = case.distances;
        {
            for (name, expected) in [
                "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
            ]
            .into_iter()
            .zip(expected)
            {
                let body = chart.bodies.iter().find(|body| body.name == name).unwrap();
                let observed = body.distance_au().unwrap();
                let delta = (observed - expected).abs();
                max_distance = max_distance.max(delta);
                assert!(
                    delta < 0.00002,
                    "distance {name} on {}-{}-{}: {observed} vs {expected}",
                    case.year,
                    case.month,
                    case.day
                );
            }
        }
        let asc = if case.tropical {
            chart.ascendant_tropical_deg
        } else {
            chart.ascendant_sidereal_deg
        };
        max_ascendant = max_ascendant.max(error(asc, case.angle));
        let [rahu_true, _] = chart.lunar_nodes(false, case.tropical);
        let [rahu_mean, _] = chart.lunar_nodes(true, case.tropical);
        if let (Some([x, y, z, vx, vy, vz]), Some(tropical)) =
            (case.moon_state, case.tropical_positions)
        {
            let hx = y * vz - z * vy;
            let hy = z * vx - x * vz;
            let nutation = astro::nutation::nutation(chart.julian_ephemeris_day).0;
            let reconstructed = (hx.atan2(-hy) + nutation).to_degrees().rem_euclid(360.);
            assert!(error(reconstructed, tropical[7]) < 0.0001);
            println!(
                "APK lunar state {}-{}-{}: reconstructed true node residual {:.6}°",
                case.year,
                case.month,
                case.day,
                error(reconstructed, tropical[7])
            );
        }
        let node_error = error(rahu_true, case.positions[7]);
        if worst_true_node_case
            .as_ref()
            .is_none_or(|(maximum, ..)| node_error > *maximum)
        {
            worst_true_node_case = Some((node_error, case.year, case.month, case.day, case.mode));
        }

        let mut actual = [
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Uranus", "Neptune",
            "Pluto",
        ]
        .map(|body| chart.longitude(body, case.tropical).unwrap())
        .to_vec();
        {
            let observed_inputs = [
                asc,
                actual[0],
                actual[1],
                actual[2],
                actual[3],
                actual[4],
                actual[5],
                actual[6],
                rahu_true,
                actual[7],
                actual[8],
                actual[9],
                chart.midheaven(case.tropical).expect("Midheaven"),
            ];
            let observed = motaa_engine::original_aspects::original_h(observed_inputs).unwrap();
            assert_eq!(
                observed.codes, case.western,
                "W-Style body codes for {}-{}-{}",
                case.year, case.month, case.day
            );
            for (system, expected) in System::OPTIONS.into_iter().zip(&case.western_all_houses) {
                let midpoints = chart.house_midpoints(system, case.tropical).unwrap();
                for (index, (midpoint, expected)) in midpoints.into_iter().zip(expected).enumerate()
                {
                    let mut inputs = observed_inputs;
                    inputs[0] = midpoint;
                    let observed = motaa_engine::original_aspects::original_g(inputs).unwrap();
                    for (body, (&actual, &expected)) in
                        observed.codes.iter().zip(expected).enumerate()
                    {
                        if actual != expected {
                            house_mismatches.push(format!(
                                "{system:?} H{} body index {body} on {}-{}-{}: expected {expected}, got {actual} (midpoint {midpoint:.9}°, body {:.9}°)",
                                index + 1, case.year, case.month, case.day, inputs[body],
                            ));
                        }
                    }
                }
            }
        }
        actual.splice(7..7, [rahu_true, rahu_mean]);
        for (index, (value, expected)) in actual.into_iter().zip(case.positions).enumerate() {
            max_body[index] = max_body[index].max(error(value, expected));
        }
        {
            for (system, expected) in System::OPTIONS.into_iter().zip(case.house_midpoints) {
                for (index, (actual, expected)) in chart
                    .house_midpoints(system, case.tropical)
                    .unwrap()
                    .into_iter()
                    .zip(expected)
                    .enumerate()
                {
                    assert!(
                        error(actual, expected) < 0.0002,
                        "{system:?} midpoint {}: expected {expected}, got {actual}, {}-{}-{}",
                        index + 1,
                        case.year,
                        case.month,
                        case.day
                    );
                }
            }
        }
        for (system, expected_cusps) in System::OPTIONS.into_iter().zip(case.houses) {
            for (value, expected) in chart
                .cusps(system, case.tropical)
                .unwrap()
                .into_iter()
                .zip(expected_cusps)
            {
                max_cusp = max_cusp.max(error(value, expected));
            }
        }
    }
    println!(
        "MOTAA maximum reference errors in degrees: bodies {max_body:?}, ascendant {max_ascendant:.9}, cusps {max_cusp:.9}"
    );
    println!("MOTAA maximum LatLon distance error: {max_distance:.9} AU");
    println!("MOTAA maximum LatLon declination error: {max_declination:.9} degrees");
    println!("MOTAA maximum LatLon rate error: {max_motion:.9} degrees/day");
    assert!(
        house_mismatches.is_empty(),
        "W-Style house code differences:\n{}",
        house_mismatches.join("\n")
    );
    assert!(max_body.into_iter().all(|degrees| degrees < 0.025));
    assert!(max_cusp < 0.0002);
    println!(
        "MOTAA sidereal Moon sum of squared errors (°²): current {:.8}, without nutation {:.8}",
        sidereal_moon_square_error[0], sidereal_moon_square_error[1]
    );
    if moon_clock_count > 0 {
        println!(
            "MOTAA Delta T experiment ({moon_clock_count} cases): peak tropical Moon error {:.6}° to {:.6}°; Dasha shifted starts: current {}, clock only {}, clock and mean frame {}",
            tropical_clock_error[0],
            tropical_clock_error[1],
            clock_dasha_mismatches[0],
            clock_dasha_mismatches[1],
            clock_dasha_mismatches[2]
        );
    }
    println!(
        "MOTAA Dasha drift: maximum start {max_dasha_start_drift:.3} days, balance {max_dasha_balance_drift:.3} days, {dasha_civil_date_mismatches} shifted civil starts"
    );
    for index in [0, 1, 2, 3, 4, 5, 6, 9, 10, 11] {
        assert!(
            max_body[index] < 0.0015,
            "body {index}: {}°",
            max_body[index]
        );
    }
    if let Some((maximum, year, month, day, mode)) = worst_true_node_case {
        println!(
            "MOTAA maximum true-node error: {maximum:.6}° on {year:04}-{month:02}-{day:02}, ayanamsha index {mode}"
        );
    }
    assert!(max_body[8] < 0.00004, "mean lunar node: {}°", max_body[8]);
    assert_eq!(dasha_civil_date_mismatches, 0, "Dasha major civil starts");
    assert!(
        max_dasha_start_drift < 0.5,
        "Dasha major start drift {max_dasha_start_drift:.4} days"
    );
    assert!(
        max_dasha_balance_drift < 0.5,
        "Dasha birth balance drift {max_dasha_balance_drift:.4} days"
    );
    assert!(max_ascendant < 0.0002);
}

#[test]
fn dasha_subperiods_and_branches_match_android_moon_across_corpus() {
    for case in cases() {
        let mut chart = calculate(&case.birth())
            .unwrap()
            .reference_ephemeris
            .unwrap();
        chart.select_ayanamsha(case.mode).unwrap();
        let desktop_moon = chart.longitude("Moon", case.tropical).unwrap();
        let expected = motaa_engine::dasha::calculate(&case.birth(), case.positions[1]).unwrap();
        let observed = motaa_engine::dasha::calculate(&case.birth(), desktop_moon).unwrap();
        for (expected_major, observed_major) in expected.major.iter().zip(observed.major) {
            assert_eq!(observed_major.lord, expected_major.lord);
            assert!((observed_major.start - expected_major.start).abs() < 0.5);
            assert_eq!(observed_major.end, expected_major.end);
            for (expected_sub, observed_sub) in expected_major
                .children()
                .iter()
                .zip(observed_major.children())
            {
                assert_eq!(observed_sub.lord, expected_sub.lord);
                assert!((observed_sub.start - expected_sub.start).abs() < 0.5);
                assert!((observed_sub.end - expected_sub.end).abs() < 0.5);
                let expected_thirds = expected_sub.children();
                let observed_thirds = observed_sub.children();
                assert_eq!(observed_thirds[8].end, observed_sub.end);
                for (expected_third, observed_third) in expected_thirds.iter().zip(observed_thirds)
                {
                    assert_eq!(observed_third.lord, expected_third.lord);
                    assert!((observed_third.start - expected_third.start).abs() < 0.5);
                    assert!((observed_third.end - expected_third.end).abs() < 0.5);
                    assert!(observed_third.start <= observed_third.end);
                }
            }
        }
    }
}

#[test]
fn dasha_three_levels_match_visible_motaa_first_launch() {
    let case = cases().remove(0);
    let timeline = motaa_engine::dasha::calculate(&case.birth(), case.positions[1]).unwrap();
    let years: Vec<_> = timeline
        .major
        .into_iter()
        .map(|period| {
            motaa_engine::dasha::from_day_number(period.start)
                .unwrap()
                .date()
                .to_string()
                .to_string()
        })
        .collect();
    assert_eq!(
        years,
        [
            "1981-02-03",
            "2001-02-03",
            "2007-02-03",
            "2017-02-03",
            "2024-02-03",
            "2042-02-03",
            "2058-02-03",
            "2077-02-03",
            "2094-02-03",
            "2101-02-03",
        ]
    );
    let chart = calculate(&case.birth())
        .unwrap()
        .reference_ephemeris
        .unwrap();
    let desktop =
        motaa_engine::dasha::calculate(&case.birth(), chart.longitude("Moon", false).unwrap())
            .unwrap();
    let starts = |periods: &[motaa_engine::dasha::Period]| {
        periods
            .iter()
            .map(|period| {
                motaa_engine::dasha::from_day_number(period.start)
                    .unwrap()
                    .date()
                    .to_string()
                    .to_string()
            })
            .collect::<Vec<_>>()
    };
    let rahu = desktop.major[4].children();
    assert_eq!(
        starts(&rahu),
        [
            "2024-02-03",
            "2026-10-16",
            "2029-03-10",
            "2032-01-15",
            "2034-08-04",
            "2035-08-22",
            "2038-08-22",
            "2039-07-17",
            "2041-01-15",
        ]
    );
    assert_eq!(
        starts(&rahu[1].children()),
        [
            "2026-10-16",
            "2027-02-10",
            "2027-06-28",
            "2027-10-31",
            "2027-12-21",
            "2028-05-15",
            "2028-06-28",
            "2028-09-09",
            "2028-10-30",
        ]
    );
}

#[test]
fn calendar_matches_shipped_motaa_runtime() {
    let data = std::env::var_os("MOTAA_ANDROID_CALENDAR").map_or_else(
        || include_bytes!("../data/motaa-v31.1-calendar-oracle.json").to_vec(),
        |path| std::fs::read(path).expect("read APK calendar capture"),
    );
    let cases: Vec<(i32, u8, u8, f64)> =
        serde_json::from_slice(&data).expect("decode APK calendar capture");
    assert_eq!(cases.len(), 15);
    for (year, month, day, expected) in cases {
        assert_eq!(
            motaa_engine::julian_midnight(year, month, day),
            expected,
            "{year}-{month}-{day}"
        );
    }
}

#[derive(Deserialize)]
struct MyanmarOracle {
    watat: Vec<(i32, i64, i32)>,
    a0: Vec<(i32, u8, u8, String)>,
}

#[test]
fn myanmar_calendar_and_observances_match_shipped_motaa_runtime() {
    let data = std::env::var_os("MOTAA_ANDROID_MYANMAR").map_or_else(
        || include_bytes!("../data/motaa-v31.1-myanmar-oracle.json").to_vec(),
        |path| std::fs::read(path).expect("read APK Myanmar oracle"),
    );
    let oracle: MyanmarOracle = serde_json::from_slice(&data).expect("decode APK Myanmar oracle");
    assert_eq!(oracle.watat.len(), 31);
    assert_eq!(oracle.a0.len(), 20);
    for (year, expected_fm, expected_watat) in oracle.watat {
        assert_eq!(
            motaa_engine::myanmar_calendar::watat_for_year(year),
            (expected_fm, expected_watat),
            "watat ME {year}"
        );
    }
    let burmese_digits = |value: i32| -> String {
        value
            .to_string()
            .chars()
            .map(|ch| match ch {
                '0'..='9' => char::from_u32('၀' as u32 + (ch as u32 - '0' as u32)).unwrap(),
                other => other,
            })
            .collect()
    };
    let burmese_phase = |phase: &str| -> &'static str {
        match phase {
            "Waxing" => "လဆန်း",
            "Full moon" => "လပြည့်",
            "Waning" => "လဆုတ်",
            "New moon" => "လကွယ်",
            other => unreachable!("unexpected phase {other}"),
        }
    };
    let burmese_weekday = |weekday: jiff::civil::Weekday| match weekday {
        jiff::civil::Weekday::Sunday => "တနင်္ဂနွေနေ့",
        jiff::civil::Weekday::Monday => "တနင်္လာနေ့",
        jiff::civil::Weekday::Tuesday => "အင်္ဂါနေ့",
        jiff::civil::Weekday::Wednesday => "ဗုဒ္ဓဟူးနေ့",
        jiff::civil::Weekday::Thursday => "ကြာသပတေးနေ့",
        jiff::civil::Weekday::Friday => "သောကြာနေ့",
        jiff::civil::Weekday::Saturday => "စနေနေ့",
    };
    let burmese_dragon = |head: &str| -> &'static str {
        match head {
            "West" => "အနောက်",
            "North" => "မြောက်",
            "East" => "အရှေ့",
            "South" => "တောင်",
            other => unreachable!("unexpected dragon head {other}"),
        }
    };
    let burmese_lord = |lord: &str| -> &'static str {
        match lord {
            "Binga" => "ဘင်္ဂ",
            "Atun" => "အထွန်း",
            "Yaza" => "ရာဇ",
            "Adipati" => "အဓိပတိ",
            "Marana" => "မရဏ",
            "Thike" => "သိုက်",
            "Puti" => "ပုတိ",
            other => unreachable!("unexpected lord {other}"),
        }
    };
    let burmese_observance = |label: &str| -> &'static str {
        match label {
            "Sabbath" => "ဥပုသ် ",
            "Sabbath Eve" => "အဖိတ် ",
            "Yatyaza" => "ရက်ရာဇာ ",
            "Pyathada" => "ပြဿဒါး ",
            "Afternoon Pyathada" => "မွန်းလွဲပြဿဒါး ",
            "Thamanyo" => "သမားညို ",
            "Amyeittasote" => "အမြိတ္တစုတ် ",
            "Warameittugyi" => "ဝါရမိတ္တုကြီး ",
            "Warameittunge" => "ဝါရမိတ္တုငယ် ",
            "Yatpote" => "ရက်ပုပ် ",
            "Thamaphyu" => "သမားဖြူ ",
            "Nagapor" => "နဂါး\u{200c}ပေါ် ",
            "Yatyotema" => "ရက်ယုတ်မာ ",
            "Mahayatkyan" => "မဟာရက်ကြမ်း ",
            other => unreachable!("unexpected observance {other}"),
        }
    };
    for (year, month, day, expected) in oracle.a0 {
        let jdn =
            motaa_engine::myanmar_calendar::CivilCalendar::Hybrid.numbered_day(year, month, day);
        let mm = motaa_engine::myanmar_calendar::from_julian_day_number(jdn)
            .expect("supported Myanmar date");
        let weekday =
            jiff::civil::Weekday::from_monday_zero_offset(jdn.rem_euclid(7) as i8).unwrap();
        let observances = mm.observances(weekday);
        let has = |label: &str| observances.contains(&label);
        let mut rendered = format!(
            "{} ခု {} {} {} {} {}ဖွား၊ ",
            burmese_digits(mm.year),
            mm.month_burmese,
            burmese_phase(&mm.phase),
            if mm.phase == "Waxing" || mm.phase == "Waning" {
                burmese_digits(mm.fortnight_day)
            } else {
                String::new()
            },
            burmese_weekday(weekday),
            burmese_lord(mm.birth_lord(weekday)),
        );
        if has("Yatyaza") {
            rendered.push_str(burmese_observance("Yatyaza"));
        }
        if has("Pyathada") {
            rendered.push_str(burmese_observance("Pyathada"));
        } else if has("Afternoon Pyathada") {
            rendered.push_str(burmese_observance("Afternoon Pyathada"));
        }
        rendered.push_str(&format!("နဂါး{}လှည့်၊ ", burmese_dragon(mm.dragon_head())));
        for key in [
            "Nagapor",
            "Amyeittasote",
            "Warameittugyi",
            "Warameittunge",
            "Sabbath",
            "Sabbath Eve",
            "Thamanyo",
            "Thamaphyu",
            "Yatpote",
            "Yatyotema",
            "Mahayatkyan",
        ] {
            if has(key) {
                rendered.push_str(burmese_observance(key));
            }
        }
        assert_eq!(
            rendered, expected,
            "main.A0 parity on {year}-{month:02}-{day:02}"
        );
    }
}
