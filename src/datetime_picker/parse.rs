use jiff::civil::{Date, DateTime};

use super::TimeSegment;

pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub(super) const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub(super) fn parse_time_only(text: &str) -> Result<jiff::civil::Time, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(jiff::civil::Time::constant(0, 0, 0, 0));
    }
    let lower = trimmed.to_ascii_lowercase();
    let (body, meridiem) = if let Some(rest) = lower.strip_suffix("am") {
        (rest.trim(), Some(false))
    } else if let Some(rest) = lower.strip_suffix("pm") {
        (rest.trim(), Some(true))
    } else {
        (lower.as_str(), None)
    };

    let parts: Vec<&str> = body.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err("Enter time as HH:MM:SS.".into());
    }
    let raw_hour: u8 = parts[0]
        .trim()
        .parse()
        .map_err(|_| "Enter time as HH:MM:SS.".to_owned())?;
    let minute: u8 = parts[1]
        .trim()
        .parse()
        .map_err(|_| "Enter time as HH:MM:SS.".to_owned())?;
    let second: u8 = if parts.len() == 3 {
        parts[2]
            .trim()
            .parse()
            .map_err(|_| "Enter time as HH:MM:SS.".to_owned())?
    } else {
        0
    };

    let hour = match meridiem {
        Some(is_pm) => {
            if !(1..=12).contains(&raw_hour) {
                return Err("12-hour time requires an hour from 1 to 12.".into());
            }
            match (raw_hour, is_pm) {
                (12, false) => 0,
                (12, true) => 12,
                (h, true) => h + 12,
                (h, false) => h,
            }
        }
        None => raw_hour,
    };

    jiff::civil::Time::new(hour as i8, minute as i8, second as i8, 0)
        .map_err(|_| "Enter time as HH:MM:SS.".to_owned())
}

pub(super) fn parse_time_part(text: &str, max: u8) -> Option<i8> {
    let trimmed = text.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let val: u8 = trimmed.parse().ok()?;
    (val <= max).then_some(val as i8)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let Ok(year) = i16::try_from(year) else {
        return 30;
    };
    let Ok(month) = i8::try_from(month) else {
        return 30;
    };
    Date::new(year, month, 1)
        .map(|d| u32::from(d.days_in_month() as u8))
        .unwrap_or(30)
}

pub fn parse_flexible_datetime(text: &str, allow_date_only: bool) -> Result<DateTime, String> {
    let trimmed = text.trim();
    if let Ok(dt) = DateTime::strptime("%Y-%m-%d %H:%M:%S", trimmed) {
        return Ok(dt);
    }
    if let Ok(dt) = DateTime::strptime("%Y-%m-%d %H:%M", trimmed) {
        return Ok(dt);
    }
    if allow_date_only && let Ok(date) = Date::strptime("%Y-%m-%d", trimmed) {
        return Ok(date.at(0, 0, 0, 0));
    }
    Err("Enter a valid date and time as YYYY-MM-DD HH:MM:SS.".into())
}

pub(super) fn time_of_day_label(hour: i8) -> &'static str {
    match hour {
        0..=4 => "Late night",
        5..=7 => "Early morning",
        8..=11 => "Morning",
        12..=13 => "Midday",
        14..=16 => "Afternoon",
        17..=20 => "Evening",
        _ => "Night",
    }
}

pub(super) fn nudge_time_seconds(dt: DateTime, delta_secs: i32) -> DateTime {
    let total = i32::from(dt.hour()) * 3600 + i32::from(dt.minute()) * 60 + i32::from(dt.second());
    let wrapped = (total + delta_secs).rem_euclid(86_400);
    let nh = (wrapped / 3600) as i8;
    let nm = ((wrapped % 3600) / 60) as i8;
    let ns = (wrapped % 60) as i8;
    jiff::civil::Time::new(nh, nm, ns, 0)
        .map(|t| DateTime::from_parts(dt.date(), t))
        .unwrap_or(dt)
}

pub(super) fn step_time_segment(dt: DateTime, seg: TimeSegment, delta: i8) -> DateTime {
    let (h, m, s) = match seg {
        TimeSegment::Hour => (
            (i16::from(dt.hour()) + i16::from(delta)).rem_euclid(24) as i8,
            dt.minute(),
            dt.second(),
        ),
        TimeSegment::Minute => (
            dt.hour(),
            (i16::from(dt.minute()) + i16::from(delta)).rem_euclid(60) as i8,
            dt.second(),
        ),
        TimeSegment::Second => (
            dt.hour(),
            dt.minute(),
            (i16::from(dt.second()) + i16::from(delta)).rem_euclid(60) as i8,
        ),
    };
    jiff::civil::Time::new(h, m, s, 0)
        .map(|t| DateTime::from_parts(dt.date(), t))
        .unwrap_or(dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_year_and_month_lengths_are_accurate() {
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 11), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn flexible_datetime_parser_supports_seconds_minutes_and_date_only() {
        let full = parse_flexible_datetime("1995-03-21 14:35:09", false).unwrap();
        assert_eq!(
            (
                full.year(),
                full.month(),
                full.day(),
                full.hour(),
                full.minute(),
                full.second()
            ),
            (1995, 3, 21, 14, 35, 9)
        );
        let no_sec = parse_flexible_datetime("1995-03-21 14:35", false).unwrap();
        assert_eq!(
            (no_sec.hour(), no_sec.minute(), no_sec.second()),
            (14, 35, 0)
        );
        assert!(parse_flexible_datetime("1995-03-21", false).is_err());
        let date_only = parse_flexible_datetime("1995-03-21", true).unwrap();
        assert_eq!(
            (
                date_only.year(),
                date_only.month(),
                date_only.day(),
                date_only.hour()
            ),
            (1995, 3, 21, 0)
        );
    }

    #[test]
    fn flexible_time_parser_supports_24h_12h_and_seconds() {
        assert_eq!(
            parse_time_only("9:05").unwrap(),
            jiff::civil::Time::constant(9, 5, 0, 0)
        );
        assert_eq!(
            parse_time_only("2:30 pm").unwrap(),
            jiff::civil::Time::constant(14, 30, 0, 0)
        );
        assert_eq!(
            parse_time_only("12:00 am").unwrap(),
            jiff::civil::Time::constant(0, 0, 0, 0)
        );
        assert_eq!(
            parse_time_only("12:15:30 PM").unwrap(),
            jiff::civil::Time::constant(12, 15, 30, 0)
        );
        assert!(parse_time_only("25:00").is_err());
        assert_eq!(parse_time_part(" 09 ", 23), Some(9));
        assert_eq!(parse_time_part("24", 23), None);
        assert_eq!(parse_time_part("", 59), None);
    }

    #[test]
    fn time_nudge_and_step_wrap_cleanly() {
        let midnight = DateTime::constant(2000, 1, 1, 0, 5, 30, 0);
        let back = nudge_time_seconds(midnight, -900);
        assert_eq!((back.hour(), back.minute(), back.second()), (23, 50, 30));
        let fwd = nudge_time_seconds(back, 900);
        assert_eq!((fwd.hour(), fwd.minute(), fwd.second()), (0, 5, 30));

        let step_h = step_time_segment(midnight, TimeSegment::Hour, -1);
        assert_eq!(
            (step_h.hour(), step_h.minute(), step_h.second()),
            (23, 5, 30)
        );
        let step_m = step_time_segment(midnight, TimeSegment::Minute, 58);
        assert_eq!(
            (step_m.hour(), step_m.minute(), step_m.second()),
            (0, 3, 30)
        );
    }

    #[test]
    fn time_of_day_labels_cover_all_hours() {
        assert_eq!(time_of_day_label(2), "Late night");
        assert_eq!(time_of_day_label(6), "Early morning");
        assert_eq!(time_of_day_label(10), "Morning");
        assert_eq!(time_of_day_label(12), "Midday");
        assert_eq!(time_of_day_label(15), "Afternoon");
        assert_eq!(time_of_day_label(19), "Evening");
        assert_eq!(time_of_day_label(23), "Night");
    }
}
