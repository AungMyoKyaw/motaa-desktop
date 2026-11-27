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
    if let Ok(time) = jiff::civil::Time::strptime("%H:%M:%S", trimmed) {
        return Ok(time);
    }
    if let Ok(time) = jiff::civil::Time::strptime("%H:%M", trimmed) {
        return Ok(time);
    }
    Err("Enter time as HH:MM:SS.".into())
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

pub(super) fn time_scroll_offset_for(val: i8, count: i8) -> f32 {
    const ITEM_H: f32 = 30.0;
    const GAP: f32 = 2.0;
    const STRIDE: f32 = ITEM_H + GAP;
    const VIEWPORT_H: f32 = 210.0;
    let total_h = (f32::from(count) * STRIDE) - GAP;
    let max_offset = (total_h - VIEWPORT_H).max(0.0);
    let centered = (f32::from(val) * STRIDE) - ((VIEWPORT_H - ITEM_H) / 2.0);
    centered.clamp(0.0, max_offset)
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

pub(super) fn apply_time_digit(
    dt: DateTime,
    seg: TimeSegment,
    buf: Option<u8>,
    digit: u8,
) -> (DateTime, Option<u8>, TimeSegment) {
    let max_val: u8 = match seg {
        TimeSegment::Hour => 23,
        TimeSegment::Minute | TimeSegment::Second => 59,
    };
    let next_seg = match seg {
        TimeSegment::Hour => TimeSegment::Minute,
        TimeSegment::Minute => TimeSegment::Second,
        TimeSegment::Second => TimeSegment::Second,
    };
    let set_seg = |val: u8| -> DateTime {
        let v = val as i8;
        let (h, m, s) = match seg {
            TimeSegment::Hour => (v, dt.minute(), dt.second()),
            TimeSegment::Minute => (dt.hour(), v, dt.second()),
            TimeSegment::Second => (dt.hour(), dt.minute(), v),
        };
        jiff::civil::Time::new(h, m, s, 0)
            .map(|t| DateTime::from_parts(dt.date(), t))
            .unwrap_or(dt)
    };
    if let Some(first) = buf {
        let candidate = first * 10 + digit;
        if candidate <= max_val {
            return (set_seg(candidate), None, next_seg);
        }
    }
    let updated = set_seg(digit);
    if digit * 10 <= max_val {
        (updated, Some(digit), seg)
    } else {
        (updated, None, next_seg)
    }
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
    fn time_scroll_offset_centers_and_clamps() {
        assert_eq!(time_scroll_offset_for(0, 24), 0.0);
        assert_eq!(time_scroll_offset_for(12, 24), (12.0 * 32.0) - 90.0);
        let max_24 = (24.0 * 32.0 - 2.0) - 210.0;
        assert_eq!(time_scroll_offset_for(23, 24), max_24);
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
    fn time_digit_entry_advances_and_validates() {
        let base = DateTime::constant(2000, 1, 1, 0, 0, 0, 0);
        let (d1, buf1, seg1) = apply_time_digit(base, TimeSegment::Hour, None, 1);
        assert_eq!((d1.hour(), buf1, seg1), (1, Some(1), TimeSegment::Hour));
        let (d2, buf2, seg2) = apply_time_digit(d1, seg1, buf1, 9);
        assert_eq!((d2.hour(), buf2, seg2), (19, None, TimeSegment::Minute));

        let (d3, buf3, seg3) = apply_time_digit(d2, seg2, buf2, 7);
        assert_eq!((d3.minute(), buf3, seg3), (7, None, TimeSegment::Second));
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
