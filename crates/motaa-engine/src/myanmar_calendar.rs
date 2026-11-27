//! Myanmar calendar conversion and daily observance rules from MOTAA V31.1
//! (`L0/AbstractC0029c.java`, `L0/C0035f.java`, `L0/C0047q.java`, `L0/C0025a.java`,
//! and `motaa/myanmar/main.java` `A0`).
//! Negative/ancient years and out-of-range dates are intentionally unsupported.

use serde::{Deserialize, Serialize};
const SOLAR_YEAR: f64 = 1577917828.0 / 4320000.0;
const LUNAR_MONTH: f64 = 1577917828.0 / 53433336.0;
const MYANMAR_EPOCH: f64 = 1954168.050623;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyanmarDate {
    pub year: i32,
    pub year_type: i32,
    pub month: i32,
    pub month_english: String,
    pub month_burmese: String,
    pub day: i32,
    pub phase: String,
    pub fortnight_day: i32,
    pub source: String,
}
fn month_name(m: i32, year_type: i32) -> (&'static str, &'static str) {
    match m {
        0 => ("First Waso", "ပထမ ဝါဆို"),
        1 => ("Tagu", "တန်ခူး"),
        2 => ("Kason", "ကဆုန်"),
        3 => ("Nayon", "နယုန်"),
        4 if year_type > 0 => ("Second Waso", "ဒုဝါဆို"),
        4 => ("Waso", "ဝါဆို"),
        5 => ("Wagaung", "ဝါခေါင်"),
        6 => ("Tawthalin", "တော်သလင်း"),
        7 => ("Thadingyut", "သီတင်းကျွတ်"),
        8 => ("Tazaungmon", "တန်ဆောင်မုန်း"),
        9 => ("Nadaw", "နတ်တော်"),
        10 => ("Pyatho", "ပြာသို"),
        11 => ("Tabodwe", "တပို့တွဲ"),
        12 => ("Tabaung", "တပေါင်း"),
        13 => ("Late Tagu", "နှောင်းတန်ခူး"),
        14 => ("Late Kason", "နှောင်းကဆုန်"),
        _ => ("Unknown", "မသိ"),
    }
}
#[derive(Debug, Copy, Clone)]
struct Era {
    ei: f64,
    wo: f64,
    nm: f64,
    fm_exceptions: &'static [(i32, i32)],
    watat_overrides: &'static [(i32, i32)],
}
type EraRules = (f64, f64, f64, &'static [(i32, i32)], &'static [(i32, i32)]);
fn constants(year: i32) -> Era {
    let (ei, wo, nm, fm_exceptions, watat_overrides): EraRules = if year >= 1312 {
        (3.0, -0.5, 8.0, &[(1377, 1)], &[(1344, 1), (1345, 0)])
    } else if year >= 1217 {
        (
            2.0,
            -1.0,
            4.0,
            &[(1234, 1), (1261, -1)],
            &[(1263, 1), (1264, 0)],
        )
    } else if year >= 1100 {
        (
            1.3,
            -0.85,
            -1.0,
            &[(1120, 1), (1126, -1), (1150, 1), (1172, -1), (1207, 1)],
            &[(1201, 1), (1202, 0)],
        )
    } else if year >= 798 {
        (
            1.2,
            -1.1,
            -1.0,
            &[
                (813, -1),
                (849, -1),
                (851, -1),
                (854, -1),
                (927, -1),
                (933, -1),
                (936, -1),
                (938, -1),
                (949, -1),
                (952, -1),
                (963, -1),
                (968, -1),
                (1039, -1),
            ],
            &[],
        )
    } else {
        (
            1.1,
            -1.1,
            -1.0,
            &[
                (205, 1),
                (246, 1),
                (471, 1),
                (572, -1),
                (651, 1),
                (653, 2),
                (656, 1),
                (672, 1),
                (729, 1),
                (767, -1),
            ],
            &[],
        )
    };
    Era {
        ei,
        wo,
        nm,
        fm_exceptions,
        watat_overrides,
    }
}
#[derive(Debug, Copy, Clone)]
struct Watat {
    fm: i64,
    watat: i32,
}
fn check_watat(year: i32) -> Watat {
    let era = constants(year);
    let threshold = (SOLAR_YEAR / 12.0 - LUNAR_MONTH) * (12.0 - era.nm);
    let mut excess = (SOLAR_YEAR * (year as f64 + 3739.0)) % LUNAR_MONTH;
    if excess < threshold {
        excess += LUNAR_MONTH;
    }
    let mut fm = (SOLAR_YEAR * year as f64 + MYANMAR_EPOCH - excess + 4.5 * LUNAR_MONTH + era.wo)
        .round() as i64;
    let mut watat = if era.ei >= 2.0 {
        i32::from(excess >= LUNAR_MONTH - (SOLAR_YEAR / 12.0 - LUNAR_MONTH) * era.nm)
    } else {
        ((year * 7 + 2).rem_euclid(19)) / 12
    };
    if let Some(&(_, forced)) = era
        .watat_overrides
        .iter()
        .find(|&&(candidate, _)| candidate == year)
    {
        watat = forced;
    }
    if watat > 0
        && let Some(&(_, offset)) = era
            .fm_exceptions
            .iter()
            .find(|&&(candidate, _)| candidate == year)
    {
        fm += i64::from(offset);
    }
    Watat { fm, watat }
}

/// Expose `AbstractC0029c.a(year)` `(fm, watat)` for differential APK tests.
pub fn watat_for_year(year: i32) -> (i64, i32) {
    let entry = check_watat(year);
    (entry.fm, entry.watat)
}
#[derive(Debug, Copy, Clone)]
struct Year {
    year_type: i32,
    first_tagu: i64,
}
fn check_year(my: i32) -> Year {
    let current = check_watat(my);
    let mut delta_years: i64 = 0;
    let prev = loop {
        delta_years += 1;
        let previous = check_watat(my - delta_years as i32);
        if previous.watat != 0 || delta_years >= 3 {
            break previous;
        }
    };
    let year_type = if current.watat != 0 {
        let n = (current.fm - prev.fm).rem_euclid(354);
        (n / 31 + 1) as i32
    } else {
        0
    };
    Year {
        year_type,
        first_tagu: prev.fm + 354 * delta_years - 102,
    }
}

/// Calculate Myanmar date from a Gregorian calendar date represented as JDN.
/// Only supported for Myanmar Era years 2 through 1500. This is a reference
/// implementation; discrepancies with original APK outputs must be investigated.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CivilCalendar {
    #[default]
    Gregorian,
    Julian,
    Hybrid,
}

impl CivilCalendar {
    pub const OPTIONS: [Self; 3] = [Self::Gregorian, Self::Julian, Self::Hybrid];

    pub fn label(self) -> &'static str {
        match self {
            Self::Gregorian => "Gregorian",
            Self::Julian => "Julian",
            Self::Hybrid => "Hybrid",
        }
    }

    /// Day-number convention from MOTAA main.A0, including its 1752 reform clamp.
    pub fn numbered_day(self, year: i32, month: u8, day: u8) -> i64 {
        let month = i64::from(month);
        let adjustment = (14 - month).div_euclid(12);
        let year = i64::from(year) + 4800 - adjustment;
        let month = month + adjustment * 12 - 3;
        let common =
            i64::from(day) + (153 * month + 2).div_euclid(5) + 365 * year + year.div_euclid(4);
        let gregorian = common - year.div_euclid(100) + year.div_euclid(400) - 32045;
        let julian = common - 32083;
        match self {
            Self::Gregorian => gregorian,
            Self::Julian => julian,
            Self::Hybrid if gregorian < 2361222 => julian.min(2361222),
            Self::Hybrid => gregorian,
        }
    }
}

pub fn from_julian_day_number(jdn: i64) -> Option<MyanmarDate> {
    let my = ((jdn as f64 - 0.5 - MYANMAR_EPOCH) / SOLAR_YEAR).floor() as i32;
    if !(2..=1500).contains(&my) {
        return None;
    }
    let info = check_year(my);
    let mut daycount = jdn - info.first_tagu + 1;
    let b = (info.year_type / 2) as i64;
    let c = (1 / (info.year_type + 1)) as i64;
    let year_length = 354 + (1 - c) * 30 + b;
    let late = (daycount - 1).div_euclid(year_length);
    daycount -= late * year_length;
    let a = (daycount + 423).div_euclid(512);
    let mut mm = ((daycount - b * a + c * a * 30) as f64 / 29.544 + 29.26 / 29.544).floor() as i64;
    let e = (mm + 12).div_euclid(16);
    let f = (mm + 11).div_euclid(16);
    let md = daycount - (29.544 * mm as f64 - 29.26).floor() as i64 - b * e + c * f * 30;
    mm += f * 3 - e * 4 + 12 * late;
    let month = mm as i32;
    let mml = 30 - month.rem_euclid(2) + if month == 3 { info.year_type / 2 } else { 0 };
    let phase = (md + 1).div_euclid(16) + md.div_euclid(16) + md.div_euclid(mml as i64);
    let name = month_name(month, info.year_type);
    Some(MyanmarDate {
        year: my,
        year_type: info.year_type,
        month,
        month_english: name.0.into(),
        month_burmese: name.1.into(),
        day: md as i32,
        phase: match phase {
            0 => "Waxing",
            1 => "Full moon",
            2 => "Waning",
            3 => "New moon",
            _ => "Unknown",
        }
        .into(),
        fortnight_day: (md - 15 * md.div_euclid(16)) as i32,
        source: "MOTAA V31.1 AbstractC0029c / C0035f / main.A0 Myanmar calendar".into(),
    })
}
impl MyanmarDate {
    /// Normalize First Waso (`0 -> 4`), Late Tagu (`13 -> 1`), and Late Kason (`14 -> 2`)
    /// to the 1..=12 base month index used by Android `main.A0` observances and dragon head.
    pub fn base_month(&self) -> i32 {
        match self.month {
            0 => 4,
            13 => 1,
            14 => 2,
            month => month,
        }
    }

    pub fn sasana_year(&self) -> i32 {
        self.year + 1182
    }

    pub fn dragon_head(&self) -> &'static str {
        ["West", "North", "East", "South"][((self.base_month() % 12) / 3) as usize]
    }

    pub fn birth_lord(&self, weekday: jiff::civil::Weekday) -> &'static str {
        let android_weekday = i32::from((weekday.to_sunday_zero_offset() + 1) % 7);
        [
            "Binga", "Atun", "Yaza", "Adipati", "Marana", "Thike", "Puti",
        ][(self.year - android_weekday).rem_euclid(7) as usize]
    }

    pub fn nat_class(&self) -> &'static str {
        ["Orc", "Elf", "Human"][self.year.rem_euclid(3) as usize]
    }

    pub fn year_name_burmese(&self) -> &'static str {
        [
            "ပုဿနှစ်",
            "မာခနှစ်",
            "ဖ္လကိုန်သံဝစ္ဆိုဝ်ရနှစ်",
            "စယ်နှစ်",
            "ပိသျက်နှစ်",
            "စိဿနှစ်",
            "အာသတ်နှစ်",
            "သရဝန်နှစ်",
            "ဘဒ္ဒြသံဝစ္ဆုံရ်နှစ်",
            "အာသိန်နှစ်",
            "ကြတိုက်နှစ်",
            "မြိက္ကသိုဝ်နှစ်",
        ][self.year.rem_euclid(12) as usize]
    }

    /// Daily classifications transcribed from MOTAA V31.1 `main.A0` and `C0025a`.
    pub fn observances(&self, weekday: jiff::civil::Weekday) -> Vec<&'static str> {
        let weekday = ((weekday.to_sunday_zero_offset() + 1) % 7) as usize;
        let month = self.base_month();
        let day = self.day;
        let fortnight = self.fortnight_day;
        let month_length =
            30 - month.rem_euclid(2) + if month == 3 { self.year_type / 2 } else { 0 };
        let month_mod = month % 4;
        let half_month_mod = month_mod / 2;
        let parity = month_mod % 2;
        let weekday_number = weekday as i32;
        let thamanyo_month = month - 1 - month / 9;
        let shifted_weekday =
            (weekday_number + 7 - (thamanyo_month * 2 - thamanyo_month / 8) % 7) % 7;
        let yatyotema_month = if month % 2 == 0 {
            (month + 9) % 12
        } else {
            month
        };
        let is_afternoon = month_mod == 0 && weekday == 4;
        let nagapor = day == [26, 21, 2, 10, 18, 2, 21][weekday]
            || day == [17, 19, 1, 0, 9, 0, 0][weekday]
            || (weekday == 1 && day == 2)
            || (weekday == 2 && matches!(day, 4 | 12 | 18));

        [
            ("Sabbath", matches!(day, 8 | 15 | 23) || day == month_length),
            (
                "Sabbath Eve",
                matches!(day, 7 | 14 | 22) || day == month_length - 1,
            ),
            (
                "Yatyaza",
                weekday_number == half_month_mod + 4
                    || weekday_number == (2 * parity + 1) * (1 - half_month_mod + parity),
            ),
            (
                "Pyathada",
                !is_afternoon && month_mod == [1, 3, 3, 0, 2, 1, 2][weekday],
            ),
            ("Afternoon Pyathada", is_afternoon),
            ("Thamanyo", shifted_weekday <= 1),
            ("Amyeittasote", fortnight == [5, 8, 3, 7, 2, 4, 1][weekday]),
            ("Warameittugyi", fortnight == [7, 1, 4, 8, 9, 6, 3][weekday]),
            ("Warameittunge", 12 - fortnight == (weekday_number + 6) % 7),
            ("Yatpote", fortnight == [8, 1, 4, 6, 9, 8, 7][weekday]),
            (
                "Thamaphyu",
                fortnight == [0, 1, 0, 0, 0, 3, 3][weekday]
                    || fortnight == [1, 2, 6, 6, 5, 6, 7][weekday]
                    || (fortnight == 4 && weekday == 5),
            ),
            ("Nagapor", nagapor),
            ("Yatyotema", fortnight == (yatyotema_month + 4) % 12 + 1),
            ("Mahayatkyan", fortnight == (month % 12 / 2 + 4) % 6 + 1),
            (
                "Shanyat",
                (1..=12).contains(&month)
                    && fortnight == [8, 8, 2, 2, 9, 3, 3, 5, 1, 4, 7, 4][(month - 1) as usize],
            ),
        ]
        .into_iter()
        .filter_map(|(label, applies)| applies.then_some(label))
        .collect()
    }

    pub fn format_burmese_birth_line(
        &self,
        weekday: jiff::civil::Weekday,
        moon_longitude: Option<f64>,
    ) -> String {
        let phase_text = match self.phase.as_str() {
            "Waxing" => format!("လဆန်း {}", burmese_digits(self.fortnight_day)),
            "Full moon" => "လပြည့်".into(),
            "Waning" => format!("လဆုတ် {}", burmese_digits(self.fortnight_day)),
            "New moon" => "လကွယ်".into(),
            other => other.into(),
        };
        let weekday_text = match weekday {
            jiff::civil::Weekday::Sunday => "တနင်္ဂနွေနေ့",
            jiff::civil::Weekday::Monday => "တနင်္လာနေ့",
            jiff::civil::Weekday::Tuesday => "အင်္ဂါနေ့",
            jiff::civil::Weekday::Wednesday => "ဗုဒ္ဓဟူးနေ့",
            jiff::civil::Weekday::Thursday => "ကြာသပတေးနေ့",
            jiff::civil::Weekday::Friday => "သောကြာနေ့",
            jiff::civil::Weekday::Saturday => "စနေနေ့",
        };
        let lord_text = match self.birth_lord(weekday) {
            "Binga" => "ဘင်္ဂ",
            "Atun" => "အထွန်း",
            "Yaza" => "ရာဇ",
            "Adipati" => "အဓိပတိ",
            "Marana" => "မရဏ",
            "Thike" => "သိုက်",
            "Puti" => "ပုတိ",
            other => other,
        };
        let dragon_dir = match self.dragon_head() {
            "West" => "အနောက်",
            "North" => "မြောက်",
            "East" => "အရှေ့",
            "South" => "တောင်",
            other => other,
        };
        let observances = self.observances(weekday);
        let primary_obs: Vec<&str> = observances
            .iter()
            .filter_map(|&obs| match obs {
                "Yatyaza" => Some("ရက်ရာဇာ"),
                "Pyathada" => Some("ပြဿဒါး"),
                "Afternoon Pyathada" => Some("မွန်းလွဲပြဿဒါး"),
                _ => None,
            })
            .collect();
        let secondary_obs: Vec<&str> = observances
            .iter()
            .filter_map(|&obs| match obs {
                "Amyeittasote" => Some("အမြိတ္တစုတ်"),
                "Warameittugyi" => Some("ဝါရမိတ္တုကြီး"),
                "Warameittunge" => Some("ဝါရမိတ္တုငယ်"),
                "Thamanyo" => Some("သမားညို"),
                "Thamaphyu" => Some("သမားဖြူ"),
                "Yatpote" => Some("ရက်ပုပ်"),
                "Yatyotema" => Some("ရက်ယုတ်မာ"),
                "Mahayatkyan" => Some("မဟာရက်ကြမ်း"),
                "Shanyat" => Some("ရှမ်းရက်"),
                "Nagapor" => Some("နဂါးပေါ်"),
                "Sabbath" => Some("ဥပုသ်"),
                "Sabbath Eve" => Some("အဖိတ်"),
                _ => None,
            })
            .collect();

        let mut parts = vec![
            format!("{} ခု", burmese_digits(self.year)),
            self.month_burmese.clone(),
            phase_text,
            weekday_text.into(),
            format!("{lord_text}ဖွား၊"),
        ];
        for obs in primary_obs {
            parts.push(obs.into());
        }
        parts.push(format!("နဂါး{dragon_dir}လှည့်၊"));
        for obs in secondary_obs {
            parts.push(obs.into());
        }
        if let Some(lon) = moon_longitude {
            let idx = ((lon.rem_euclid(360.) / (360. / 27.)) as usize).min(26);
            parts.push(format!("စင်[{}]", crate::MY[idx]));
        }
        parts.join(" ")
    }
}

pub fn burmese_digits(value: i32) -> String {
    value
        .to_string()
        .chars()
        .map(|ch| match ch {
            '0'..='9' => char::from_u32('၀' as u32 + (ch as u32 - '0' as u32)).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(jdn: i64, my: i32, myt: i32, mm: i32, md: i32) {
        let c = from_julian_day_number(jdn).unwrap();
        assert_eq!(
            (c.year, c.year_type, c.month, c.day),
            (my, myt, mm, md),
            "JDN {jdn}"
        );
    }
    #[test]
    fn calendar_options_keep_motaa_hybrid_reform_rule() {
        assert_eq!(CivilCalendar::Gregorian.numbered_day(2000, 1, 1), 2451545);
        assert_eq!(CivilCalendar::Julian.numbered_day(2000, 1, 1), 2451558);
        for (day, expected) in [
            (2, 2361221),
            (3, 2361222),
            (10, 2361222),
            (14, 2361222),
            (15, 2361223),
        ] {
            assert_eq!(CivilCalendar::Hybrid.numbered_day(1752, 9, day), expected);
        }
    }

    #[test]
    fn observances_respect_sabbath_and_weekday() {
        let mut date = from_julian_day_number(2_451_545).unwrap();
        date.month = 4;
        date.day = 15;
        date.fortnight_day = 15;
        let labels = date.observances(jiff::civil::Weekday::Wednesday);
        assert!(labels.contains(&"Sabbath"));
        assert!(labels.contains(&"Afternoon Pyathada"));
        assert!(!labels.contains(&"Sabbath Eve"));
        date.day = 14;
        date.fortnight_day = 14;
        assert!(
            date.observances(jiff::civil::Weekday::Tuesday)
                .contains(&"Sabbath Eve")
        );
    }

    #[test]
    fn fixtures_against_upstream_20260909() {
        // JDNs verified using the Gregorian Julian-day arithmetic in motaa-engine.
        let cases = [
            (2024, 4, 17, 1386, 0, 1, 9),
            (2026, 10, 22, 1388, 2, 7, 11),
            (2000, 1, 1, 1361, 1, 9, 25),
            (2019, 4, 17, 1381, 0, 1, 13),
            (2024, 1, 1, 1385, 2, 9, 20),
            (1980, 1, 1, 1341, 0, 10, 15),
            (1752, 9, 14, 1114, 0, 7, 7),
            (2025, 4, 13, 1386, 0, 13, 16),
            (2025, 4, 18, 1387, 0, 1, 21),
        ];
        for (y, m, d, me, ty, mm, md) in cases {
            let jdn = (crate::julian_midnight(y, m, d) + 0.5).round() as i64;
            check(jdn, me, ty, mm, md);
        }
    }
    #[test]
    fn late_tagu_and_second_waso_match_android_a0_rules() {
        // 2025-04-13 (Sunday): Late Tagu waning 1, ME 1386, Puti, West, Warameittugyi, Thamanyo, Thamaphyu, Yatpote.
        let jdn = (crate::julian_midnight(2025, 4, 13) + 0.5).round() as i64;
        let date = from_julian_day_number(jdn).unwrap();
        assert_eq!(date.month, 13);
        assert_eq!(date.base_month(), 1);
        assert_eq!(date.dragon_head(), "West");
        assert_eq!(date.birth_lord(jiff::civil::Weekday::Sunday), "Puti");
        assert_eq!(date.nat_class(), "Orc");
        assert_eq!(date.sasana_year(), 2568);
        assert_eq!(
            date.observances(jiff::civil::Weekday::Sunday),
            vec!["Thamanyo", "Warameittugyi", "Yatpote", "Thamaphyu"]
        );

        // 2023-07-18 (Tuesday): Second Waso waxing 1, ME 1385.
        let waso_jdn = (crate::julian_midnight(2023, 7, 18) + 0.5).round() as i64;
        let waso = from_julian_day_number(waso_jdn).unwrap();
        assert_eq!(waso.month_english, "Second Waso");
        assert_eq!(waso.month_burmese, "ဒုဝါဆို");
        assert_eq!(waso.dragon_head(), "North");
        assert_eq!(waso.birth_lord(jiff::civil::Weekday::Tuesday), "Adipati");

        // 1984-01-03 (Tuesday): Pakhangyi sample.
        let birth_jdn = (crate::julian_midnight(1984, 1, 3) + 0.5).round() as i64;
        let birth = from_julian_day_number(birth_jdn).unwrap();
        assert_eq!(birth.year, 1345);
        assert_eq!(birth.year_name_burmese(), "မာခနှစ်");
        assert_eq!(birth.birth_lord(jiff::civil::Weekday::Tuesday), "Thike");
        assert_eq!(birth.dragon_head(), "South");
        assert_eq!(birth.nat_class(), "Elf");
        assert!(birth.observances(jiff::civil::Weekday::Tuesday).is_empty());
    }

    #[test]
    fn watat_overrides_apply_before_full_moon_exceptions() {
        for (year, expected_fm, expected_watat) in [
            (1201, 2392946, 1),
            (1202, 2393300, 0),
            (1263, 2415596, 1),
            (1264, 2415950, 0),
            (1344, 2445186, 1),
            (1345, 2445540, 0),
            (1377, 2457235, 1),
        ] {
            assert_eq!(
                watat_for_year(year),
                (expected_fm, expected_watat),
                "ME {year}"
            );
        }
    }

    #[test]
    fn ancient_unsupported() {
        assert!(from_julian_day_number(1_954_000).is_none());
    }

    #[test]
    fn burmese_birth_line_matches_reference_capture() {
        let jdn = CivilCalendar::Hybrid.numbered_day(1984, 1, 3);
        let date = from_julian_day_number(jdn).unwrap();
        let line = date.format_burmese_birth_line(jiff::civil::Weekday::Tuesday, Some(182.08));
        assert!(line.contains("၁၃၄၅"));
        assert!(line.contains("အင်္ဂါနေ့"));
    }
}
