use chrono::NaiveDateTime;
use motaa_engine::ChartRequest;

use crate::{model, recovered};

pub const BODIES: [&str; 10] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Pluto",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Ingress,
    StationDirect,
    StationRetrograde,
}

impl Change {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ingress => "Sign ingress",
            Self::StationDirect => "Station direct",
            Self::StationRetrograde => "Station retrograde",
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub jdut: f64,
    pub body: &'static str,
    pub change: Change,
    pub sign: usize,
}

pub struct Report {
    pub profile: ChartRequest,
    pub start: f64,
    pub end: f64,
    pub tropical: bool,
    pub ayanamsha: usize,
    pub events: Vec<Event>,
}

pub fn calculate(
    profile: ChartRequest,
    start: NaiveDateTime,
    tropical: bool,
    ayanamsha: usize,
) -> Result<Report, String> {
    use chrono::Datelike;
    const DAYS: f64 = 30.;
    const STEP: f64 = 0.25;
    let end_time = start
        .checked_add_signed(chrono::TimeDelta::days(DAYS as i64))
        .ok_or("Invalid transit search range.")?;
    if !(1800..=2100).contains(&start.year()) || !(1800..=2100).contains(&end_time.year()) {
        return Err("Choose a date whose following 30 days fit in 1800–2100.".into());
    }
    let start_jd = model::julian_at(&profile, start)?;
    let end = start_jd + DAYS;
    let sample = |jd| {
        let mut ephemeris = motaa_engine::reference_ephemeris::calculate(
            jd,
            start.year(),
            start.month() as u8,
            profile.latitude,
            profile.longitude,
        )?;
        if !tropical {
            ephemeris.select_ayanamsha(ayanamsha)?;
        }
        let mut longitudes = [0.; BODIES.len()];
        for (index, body) in BODIES.into_iter().enumerate() {
            longitudes[index] = ephemeris
                .longitude(body, tropical)
                .ok_or("Reference body unavailable.")?;
        }
        Ok::<_, String>(longitudes)
    };
    let motions_at = |jd| {
        let before = sample(jd - 1. / 144.)?;
        let after = sample(jd + 1. / 144.)?;
        Ok::<_, String>(angular_velocity(&before, &after))
    };
    let mut events = Vec::new();
    let mut low = start_jd;
    let mut before = sample(low)?;
    let mut speeds = motions_at(low)?;
    for step in 1..=120 {
        let high = start_jd + f64::from(step) * STEP;
        let after = sample(high)?;
        let new_speeds = motions_at(high)?;
        for (body, name) in BODIES.into_iter().enumerate() {
            let sign = recovered::sign(after[body]);
            if recovered::sign(before[body]) != sign {
                let jdut = refine(low, high, |jd| {
                    Ok(recovered::sign(sample(jd)?[body]) == sign)
                })?;
                events.push(Event {
                    jdut,
                    body: name,
                    change: Change::Ingress,
                    sign,
                });
            }

            if body > 1 && speeds[body].is_sign_negative() != new_speeds[body].is_sign_negative() {
                let retrograde = new_speeds[body].is_sign_negative();
                let jdut = refine(low, high, |jd| {
                    Ok(motions_at(jd)?[body].is_sign_negative() == retrograde)
                })?;
                let change = if retrograde {
                    Change::StationRetrograde
                } else {
                    Change::StationDirect
                };
                events.push(Event {
                    jdut,
                    body: name,
                    change,
                    sign: recovered::sign(sample(jdut)?[body]),
                });
            }
        }
        low = high;
        before = after;
        speeds = new_speeds;
    }
    events.sort_by(|left, right| left.jdut.total_cmp(&right.jdut));
    Ok(Report {
        profile,
        start: start_jd,
        end,
        tropical,
        ayanamsha,
        events,
    })
}

fn signed_delta(before: f64, after: f64) -> f64 {
    (after - before + 180.).rem_euclid(360.) - 180.
}

fn angular_velocity(before: &[f64; 10], after: &[f64; 10]) -> [f64; 10] {
    std::array::from_fn(|index| signed_delta(before[index], after[index]))
}

fn refine(
    mut low: f64,
    mut high: f64,
    changed: impl Fn(f64) -> Result<bool, String>,
) -> Result<f64, String> {
    while (high - low) * 86_400. > 30. {
        let midpoint = (low + high) / 2.;
        if changed(midpoint)? {
            high = midpoint;
        } else {
            low = midpoint;
        }
    }
    Ok(high)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_motion_handles_the_aries_wrap_both_directions() {
        assert_eq!(signed_delta(359.5, 0.5), 1.);
        assert_eq!(signed_delta(0.5, 359.5), -1.);
    }

    #[test]
    fn march_equinox_produces_a_sun_ingress_into_aries() {
        let profile = ChartRequest {
            timezone_id: None,
            timezone_offset_minutes: 0,
            ..Default::default()
        };
        let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let report = calculate(profile, start, true, 0).unwrap();
        let event = report
            .events
            .iter()
            .find(|event| event.body == "Sun" && event.change == Change::Ingress)
            .unwrap();
        assert_eq!(event.sign, 0);
        assert!((event.jdut - motaa_engine::julian_midnight(2024, 3, 20)).abs() < 0.3);
        assert!(
            report
                .events
                .windows(2)
                .all(|pair| pair[0].jdut <= pair[1].jdut)
        );
    }

    #[test]
    fn sidereal_ingress_uses_the_requested_ayanamsha() {
        let profile = ChartRequest {
            timezone_id: None,
            timezone_offset_minutes: 0,
            ..Default::default()
        };
        let start = chrono::NaiveDate::from_ymd_opt(2024, 4, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let aries = |ayanamsha| {
            let report = calculate(profile.clone(), start, false, ayanamsha).unwrap();
            assert_eq!(report.ayanamsha, ayanamsha);
            report
                .events
                .iter()
                .find(|event| {
                    event.body == "Sun" && event.change == Change::Ingress && event.sign == 0
                })
                .unwrap()
                .jdut
        };
        let lahiri = aries(0);
        let raman = aries(3);
        assert!((1. ..2.).contains(&(lahiri - raman)));
    }

    #[test]
    fn finds_mercury_stations_including_the_final_hours() {
        let profile = ChartRequest {
            timezone_id: None,
            timezone_offset_minutes: 0,
            ..Default::default()
        };
        for (month, day, hour, station_day, seconds, change) in [
            (4, 1, 0, 1, 80_040., Change::StationRetrograde),
            (3, 26, 13, 25, 46_440., Change::StationDirect),
        ] {
            let start = chrono::NaiveDate::from_ymd_opt(2024, month, day)
                .unwrap()
                .and_hms_opt(hour, 0, 0)
                .unwrap();
            let report = calculate(profile.clone(), start, true, 0).unwrap();
            let event = report
                .events
                .iter()
                .find(|event| event.body == "Mercury" && event.change == change)
                .unwrap();
            assert_eq!(event.sign, 0);
            let actual =
                (event.jdut - motaa_engine::julian_midnight(2024, 4, station_day)) * 86400.;
            assert!(
                (actual - seconds).abs() < 600.,
                "{actual:.1} vs {seconds:.1}"
            );
        }
    }
}
