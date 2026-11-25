//! Recovered MOTAA main.W/X/Y period timing, driven by a supplied natal Moon longitude.
use chrono::{Datelike, NaiveDate, NaiveDateTime, TimeDelta, Timelike};

use crate::{ChartRequest, original_geometry::original_p0};

const LORDS: [(&str, i32); 9] = [
    ("Ketu", 7),
    ("Venus", 20),
    ("Sun", 6),
    ("Moon", 10),
    ("Mars", 7),
    ("Rahu", 18),
    ("Jupiter", 16),
    ("Saturn", 19),
    ("Mercury", 17),
];
const SIDEREAL_YEAR: f64 = 365.25636305;

#[derive(Clone, Copy, Debug)]
pub struct Period {
    pub lord: usize,
    pub start: f64,
    pub end: f64,
    years: f64,
}

impl Period {
    pub fn name(self) -> &'static str {
        LORDS[self.lord].0
    }

    pub fn contains(self, day: f64) -> bool {
        self.start <= day && day < self.end
    }

    pub fn children(self) -> [Self; 9] {
        let mut cursor = self.start;
        std::array::from_fn(|index| {
            let lord = (self.lord + index) % LORDS.len();
            let years = self.years * f64::from(LORDS[lord].1) / 120.;
            // The source anchors the final child to the parent's explicit end.
            let end = if index == 8 {
                self.end
            } else {
                (cursor + years * SIDEREAL_YEAR).min(self.end)
            };
            let period = Self {
                lord,
                start: cursor,
                end,
                years,
            };
            cursor = end;
            period
        })
    }
}

#[derive(Debug)]
pub struct Timeline {
    pub major: [Period; 10],
    pub balance_days: f64,
}

pub fn calculate(birth: &ChartRequest, moon_longitude: f64) -> Result<Timeline, String> {
    crate::validate(birth)?;
    if birth.unknown_birth_time {
        return Err("Dasha requires a known birth time.".into());
    }
    if !(1800..=2100).contains(&birth.year) || !(0. ..360.).contains(&moon_longitude) {
        return Err("Dasha requires a supported birth date and Moon longitude.".into());
    }
    let birth_date =
        NaiveDate::from_ymd_opt(birth.year, u32::from(birth.month), u32::from(birth.day))
            .and_then(|date| {
                date.and_hms_opt(
                    u32::from(birth.hour),
                    u32::from(birth.minute),
                    u32::from(birth.second),
                )
            })
            .ok_or("Invalid civil birth timestamp.")?;
    let mansion = moon_longitude / (360. / 27.);
    let lord = mansion.floor() as usize % LORDS.len();
    let progress = mansion.fract();
    let years = f64::from(LORDS[lord].1);
    let start = day_number(birth_date) - SIDEREAL_YEAR * years * progress;
    let civil =
        from_day_number((start + 0.5 / 86_400.).floor()).ok_or("Invalid first period date.")?;
    let mut elapsed = 0;
    let mut cursor = start;
    let major = std::array::from_fn(|index| {
        let lord = (lord + index) % LORDS.len();
        let years = LORDS[lord].1;
        elapsed += years;
        let end = original_p0(
            civil.day() as i32,
            civil.month() as i32,
            civil.year() + elapsed,
        );
        let period = Period {
            lord,
            start: cursor,
            end,
            years: f64::from(years),
        };
        cursor = end;
        period
    });
    Ok(Timeline {
        major,
        balance_days: years * 365.2422 * (1. - progress),
    })
}

/// MOTAA's civil day number, with midnight as the start of the numbered day.
pub fn day_number(time: NaiveDateTime) -> f64 {
    original_p0(time.day() as i32, time.month() as i32, time.year())
        + f64::from(time.num_seconds_from_midnight()) / 86_400.
}

pub fn from_day_number(day: f64) -> Option<NaiveDateTime> {
    if !day.is_finite() {
        return None;
    }
    let origin = NaiveDate::from_ymd_opt(2000, 1, 1)?.and_hms_opt(0, 0, 0)?;
    let seconds = ((day - original_p0(1, 1, 2000)) * 86_400.).round() as i64;
    origin.checked_add_signed(TimeDelta::try_seconds(seconds)?)
}

/// KP star, sub and sub-sub lords. Each subdivision starts from its parent's
/// lord and uses the Vimshottari weights. Boundaries are start-inclusive.
pub fn kp_lords(longitude: f64) -> Option<[&'static str; 3]> {
    if !(0. ..360.).contains(&longitude) {
        return None;
    }
    let mansion_width = 360. / 27.;
    let mansion = (longitude / mansion_width).floor() as usize;
    let star = mansion % LORDS.len();
    let (sub, within_sub) = subdivide(
        star,
        longitude - mansion as f64 * mansion_width,
        mansion_width,
    );
    let (sub_sub, _) = subdivide(sub, within_sub.0, within_sub.1);
    Some([LORDS[star].0, LORDS[sub].0, LORDS[sub_sub].0])
}

fn subdivide(first: usize, offset: f64, width: f64) -> (usize, (f64, f64)) {
    let mut start = 0.;
    let mut cumulative_years = 0;
    for index in 0..LORDS.len() {
        let lord = (first + index) % LORDS.len();
        cumulative_years += LORDS[lord].1;
        let end = width * f64::from(cumulative_years) / 120.;
        if offset < end || index == LORDS.len() - 1 {
            return (lord, (offset - start, end - start));
        }
        start = end;
    }
    unreachable!("nonempty lord cycle")
}

/// Standard 249-subdivision KP Horary table (`1..=249`), splitting any sub
/// that crosses a 30° sign boundary into two entries. Returns `(start_deg, [star_lord, sub_lord])`.
pub fn kp_249_longitude(slot: usize) -> Option<(f64, [&'static str; 2])> {
    if !(1..=249).contains(&slot) {
        return None;
    }
    let mansion_width = 360. / 27.;
    let mut current = 0_usize;
    for mansion in 0..27_usize {
        let star = mansion % LORDS.len();
        let mansion_start = mansion as f64 * mansion_width;
        let mut sub_start = mansion_start;
        for index in 0..LORDS.len() {
            let sub = (star + index) % LORDS.len();
            let sub_width = mansion_width * f64::from(LORDS[sub].1) / 120.;
            let sub_end = sub_start + sub_width;
            let start_sign = (sub_start / 30.).floor() as i32;
            let end_sign = ((sub_end - 1e-9) / 30.).floor() as i32;
            if start_sign != end_sign {
                let boundary = f64::from(end_sign) * 30.;
                current += 1;
                if current == slot {
                    return Some((sub_start, [LORDS[star].0, LORDS[sub].0]));
                }
                current += 1;
                if current == slot {
                    return Some((boundary, [LORDS[star].0, LORDS[sub].0]));
                }
            } else {
                current += 1;
                if current == slot {
                    return Some((sub_start, [LORDS[star].0, LORDS[sub].0]));
                }
            }
            sub_start = sub_end;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kp_ashwini_and_swati_reference_boundaries() {
        assert_eq!(kp_lords(0.), Some(["Ketu", "Ketu", "Ketu"]));
        assert_eq!(kp_lords(0.8).unwrap()[1], "Venus");
        assert_eq!(kp_lords(3.01).unwrap()[1], "Sun");
        assert_eq!(kp_lords(187.).unwrap()[..2], ["Rahu", "Rahu"]);
        assert_eq!(kp_lords(360.), None);
        assert_eq!(kp_249_longitude(1).unwrap().0, 0.);
        assert_eq!(kp_249_longitude(1).unwrap().1, ["Ketu", "Ketu"]);
        assert!(kp_249_longitude(249).is_some());
        assert!(kp_249_longitude(250).is_none());
    }

    #[test]
    fn source_weights_and_civil_anniversaries_are_preserved() {
        let birth = ChartRequest {
            year: 2000,
            month: 1,
            day: 1,
            hour: 0,
            ..ChartRequest::default()
        };
        let dasha = calculate(&birth, 0.).unwrap();
        let first = dasha.major[0];
        assert_eq!(first.name(), "Ketu");
        assert_eq!(dasha.major[1].name(), "Venus");
        assert_eq!(dasha.major[9].name(), "Ketu");
        assert_eq!(first.start, original_p0(1, 1, 2000));
        assert_eq!(first.end, original_p0(1, 1, 2007));
        assert_eq!(dasha.major[8].end, original_p0(1, 1, 2120));
        assert!(first.contains(first.start));
        assert!(!first.contains(first.end));
        let children = first.children();
        assert_eq!(children[0].name(), "Ketu");
        assert!((children[0].end - first.start - 49. / 120. * SIDEREAL_YEAR).abs() < 1e-8);
        assert_eq!(children[8].end, first.end);
        assert_eq!(children[1].children()[8].end, children[1].end);
    }

    #[test]
    fn mansion_boundaries_rotate_the_first_lord() {
        let birth = ChartRequest::default();
        for index in 0..27 {
            let inside = (index as f64 + 0.5) * (360. / 27.);
            let timeline = calculate(&birth, inside).unwrap();
            assert_eq!(timeline.major[0].lord, index % 9);
            assert!(timeline.major[0].start < original_p0(1, 1, 2000));
        }
        assert!(calculate(&birth, 360.).is_err());
        assert!(calculate(&birth, f64::NAN).is_err());
    }
}
