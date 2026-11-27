//! MOTAA domain engine: evidence-backed calendar algorithm and nakshatra metadata.
//! Source: reversed APK motaa.myanmar.main.w0 and Q3, motaa-circle/circle.tex.
//! No fabricated ephemeris positions: Moon can be classified ONLY from manual longitude.
use jiff::civil::DateTime;
use jiff::tz::{AmbiguousOffset, TimeZone};
use serde::{Deserialize, Serialize};
pub mod ayanamsha;
pub mod dasha;
pub mod eclipses;
pub mod houses;
pub mod myanmar_calendar;
pub mod original_aspects;
pub mod original_colors;
pub mod original_geometry;
pub mod original_m0;
pub mod original_mansions;
pub mod original_positions;
pub mod original_strengths;
pub mod original_text;
pub mod panchanga;
pub mod reference_ephemeris;
pub const EN: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];
pub const MY: [&str; 27] = [
    "အဿဝဏီ",
    "ဘရဏီ",
    "ကြတ္တိကာ",
    "ရောဟိဏီ",
    "မိဂသီ",
    "အဒြ",
    "ပုဏ္ဏဖုသျှု",
    "ဖုသျှ",
    "အဿလိသ",
    "မာဃ",
    "ပြုဗ္ဗာဖလဂုဏီ",
    "ဥတ္တရာဖလဂုဏီ",
    "ဟဿတ",
    "စိတြ",
    "သွာတိ",
    "ဝိသာခါ",
    "အနုရာဓ",
    "ဇေဋ္ဌ",
    "မူလ",
    "ပြုဗ္ဗာသဠ်",
    "ဥတ္တရာသဠ်",
    "သရဝဏ်",
    "ဓနသိဒ္ဓ",
    "သတ္တဘိသျှ",
    "ပြုဗ္ဗာဘဒြပိုဒ်",
    "ဥတ္တရာဘဒြပိုဒ်",
    "ရေဝတီ",
];
pub const APK_CYCLE: [u8; 9] = [9, 6, 1, 2, 3, 8, 5, 0, 4]; // original APK Q3 contains zero
pub const CORRECTED_CYCLE: [u8; 9] = [9, 6, 1, 2, 3, 8, 5, 7, 4]; // explicit alternate profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartRequest {
    pub name: String,
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub timezone_offset_minutes: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub birthplace: String,
    #[serde(default)]
    pub timezone_id: Option<String>,
    pub unknown_birth_time: bool,
    pub nakshatra_count: u8,
    pub corrected_rulers: bool,
    pub manual_moon_longitude: Option<f64>,
}
impl Default for ChartRequest {
    fn default() -> Self {
        Self {
            name: "New chart".into(),
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            minute: 0,
            second: 0,
            timezone_offset_minutes: 390,
            latitude: 16.8409,
            longitude: 96.1735,
            birthplace: "Yangon, Myanmar".into(),
            timezone_id: Some("Asia/Yangon".into()),
            unknown_birth_time: false,
            nakshatra_count: 27,
            corrected_rulers: false,
            manual_moon_longitude: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nakshatra {
    pub number: u8,
    pub english: String,
    pub burmese: String,
    pub ruler_code: u8,
    pub ruler_name: String,
    pub start_degree: Option<f64>,
    pub end_degree: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonClassification {
    pub longitude: f64,
    pub nakshatra_number: u8,
    pub quarter: u8,
    pub ruler_code: u8,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartResult {
    pub name: String,
    pub julian_day_utc: Option<f64>,
    pub julian_day_local_midnight: f64,
    pub effective_utc_offset_minutes: i32,
    pub myanmar_date: Option<myanmar_calendar::MyanmarDate>,
    pub reference_ephemeris: Option<reference_ephemeris::ReferenceEphemeris>,
    pub reference_moon: Option<MoonClassification>,
    pub weekday_iso: u8,
    pub nakshatras: Vec<Nakshatra>,
    pub moon: Option<MoonClassification>,
    pub computation_status: String,
    pub evidence: Vec<String>,
    pub warnings: Vec<String>,
}
pub fn leap_year(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
pub fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        4 | 6 | 9 | 11 => 30,
        2 if leap_year(y) => 29,
        2 => 28,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}
/// Exact arithmetic sequence from original Java main.w0 / JADX m2250w0.
pub fn julian_midnight(year: i32, month: u8, day: u8) -> f64 {
    let mut y = year as f64;
    if month < 3 {
        y -= 1.0
    }
    let mut m = month as f64 + 1.0;
    if m < 4.0 {
        m += 12.0
    }
    let b = (m * 30.6 + 1e-6).floor() + ((4712.0 + y) * 365.25).floor() + day as f64 - 63.5;
    let mut century = (y.abs() / 100.0).floor() - (y.abs() / 400.0).floor();
    if y < 0.0 {
        century = -century
    }
    let j = b - century + 2.0;
    if y >= 0.0 || y / 100.0 != (y / 100.0).floor() || y / 400.0 == (y / 400.0).floor() {
        j
    } else {
        j - 1.0
    }
}
pub fn validate(r: &ChartRequest) -> Result<(), String> {
    if !(-4712..=9999).contains(&r.year) {
        return Err("Year must be between -4712 and 9999".into());
    }
    if r.day == 0 || r.day > days_in_month(r.year, r.month) {
        return Err("Invalid calendar date".into());
    }
    if r.hour > 23 || r.minute > 59 || r.second > 59 {
        return Err("Invalid time".into());
    }
    if !(-840..=840).contains(&r.timezone_offset_minutes) {
        return Err("UTC offset out of range".into());
    }
    if !r.latitude.is_finite() || r.latitude.abs() > 90.0 {
        return Err("Latitude out of range".into());
    }
    if !r.longitude.is_finite() || r.longitude.abs() > 180.0 {
        return Err("Longitude out of range".into());
    }
    if r.nakshatra_count != 27 && r.nakshatra_count != 28 {
        return Err("Use 27 or 28 nakshatras".into());
    }
    if let Some(m) = r.manual_moon_longitude
        && (!m.is_finite() || !(0.0..360.0).contains(&m))
    {
        return Err("Manual moon longitude must be in [0, 360)".into());
    }
    if r.name.len() > 200 || r.birthplace.len() > 300 {
        return Err("Profile text too long".into());
    }
    if let Some(id) = &r.timezone_id {
        TimeZone::get(id).map_err(|_| "Unknown IANA timezone".to_string())?;
    }
    Ok(())
}
fn ruler_name(r: u8) -> &'static str {
    match r {
        1 => "Sun",
        2 => "Moon",
        3 => "Mars",
        4 => "Mercury",
        5 => "Jupiter",
        6 => "Venus",
        7 => "Saturn",
        8 => "Rahu",
        9 => "Ketu",
        _ => "0 (APK literal)",
    }
}
pub fn effective_utc_offset(r: &ChartRequest) -> Result<i32, String> {
    let Some(ref zone) = r.timezone_id else {
        return Ok(r.timezone_offset_minutes);
    };
    if r.unknown_birth_time {
        return Ok(r.timezone_offset_minutes);
    }
    let tz = TimeZone::get(zone).map_err(|_| format!("Unknown timezone {zone}"))?;
    let year = i16::try_from(r.year).map_err(|_| "Invalid local date".to_string())?;
    let dt = DateTime::new(
        year,
        r.month as i8,
        r.day as i8,
        r.hour as i8,
        r.minute as i8,
        r.second as i8,
        0,
    )
    .map_err(|_| "Invalid local date or time".to_string())?;
    match tz.to_ambiguous_zoned_datetime(dt).offset() {
        AmbiguousOffset::Unambiguous { offset } => Ok(offset.seconds() / 60),
        AmbiguousOffset::Fold { .. } => Err("Ambiguous local time at daylight-saving transition; choose a numeric UTC offset and clear the timezone ID".into()),
        AmbiguousOffset::Gap { .. } => Err("Local time does not exist in the selected timezone; choose a valid time or a numeric offset".into()),
    }
}
pub fn calculate(r: &ChartRequest) -> Result<ChartResult, String> {
    validate(r)?;
    let offset = effective_utc_offset(r)?;
    let j = julian_midnight(r.year, r.month, r.day);
    let utc = if r.unknown_birth_time {
        None
    } else {
        Some(
            j + (r.hour as f64 * 3600.0 + r.minute as f64 * 60.0 + r.second as f64) / 86400.0
                - offset as f64 / 1440.0,
        )
    };
    let cycle = if r.corrected_rulers {
        CORRECTED_CYCLE
    } else {
        APK_CYCLE
    };
    let mut stars = (0..27)
        .map(|i| {
            let k = cycle[i % 9];
            Nakshatra {
                number: (i + 1) as u8,
                english: EN[i].into(),
                burmese: MY[i].into(),
                ruler_code: k,
                ruler_name: ruler_name(k).into(),
                start_degree: Some(i as f64 * 360.0 / 27.0),
                end_degree: Some((i + 1) as f64 * 360.0 / 27.0),
            }
        })
        .collect::<Vec<_>>();
    let mut warnings=vec!["Supported dates (1800–2100) follow V31.1 Delta T, ayanamsha, house, and Myanmar calendar conventions; true-node state-vector residual remains tracked.".to_string()];
    if r.nakshatra_count == 28 {
        stars.insert(
            21,
            Nakshatra {
                number: 0,
                english: "Abhijit".into(),
                burmese: "အဘိဇိ".into(),
                ruler_code: 0,
                ruler_name: "Unverified".into(),
                start_degree: None,
                end_degree: None,
            },
        );
        warnings.push(
            "28 mode: Abhijit shown as reference; V31.1 Rahu wheel maps 28-division Y0 with sentinel 30."
                .into(),
        );
    }
    let moon = if r.nakshatra_count == 27 {
        r.manual_moon_longitude.map(|m| {
            let i = (m / (360.0 / 27.0)).floor() as usize;
            let q = ((m - i as f64 * 360.0 / 27.0) / (360.0 / 108.0)).floor() as u8 + 1;
            MoonClassification {
                longitude: m,
                nakshatra_number: (i + 1) as u8,
                quarter: q.min(4),
                ruler_code: cycle[i % 9],
                source: "Manual longitude supplied by user".into(),
            }
        })
    } else {
        None
    };
    if moon.is_some() {
        warnings.push("Manual Moon longitude is not an automatic astronomical result.".into())
    }
    if r.unknown_birth_time {
        warnings.push("Unknown time: time-dependent output suppressed.".into())
    }
    let reference_ephemeris = utc.and_then(|jd| {
        reference_ephemeris::calculate(jd, r.year, r.month, r.latitude, r.longitude).ok()
    });
    let reference_moon = if r.nakshatra_count == 27 {
        reference_ephemeris
            .as_ref()
            .and_then(|ephem| ephem.bodies.iter().find(|b| b.name == "Moon"))
            .map(|body| {
                let longitude = body.sidereal_longitude_deg;
                let index = (longitude / (360.0 / 27.0)).floor() as usize;
                let quarter =
                    ((longitude - index as f64 * 360.0 / 27.0) / (360.0 / 108.0)).floor() as u8 + 1;
                MoonClassification {
                    longitude,
                    nakshatra_number: (index + 1) as u8,
                    quarter: quarter.min(4),
                    ruler_code: cycle[index % 9],
                    source: "Reference sidereal Moon position (V31.1 Delta T and ayanamsha)".into(),
                }
            })
    } else {
        None
    };
    if reference_ephemeris.is_none() {
        warnings.push(
            "Reference planetary positions unavailable: unsupported date or unknown birth time."
                .into(),
        )
    }
    Ok(ChartResult {
        name: r.name.clone(),
        julian_day_utc: utc,
        julian_day_local_midnight: j,
        effective_utc_offset_minutes: offset,
        myanmar_date: myanmar_calendar::from_julian_day_number((j + 0.5).round() as i64),
        reference_ephemeris,
        reference_moon,
        weekday_iso: ((j + 0.5).floor() as i64).rem_euclid(7) as u8 + 1,
        nakshatras: stars,
        moon,
        computation_status:
            "MOTAA V31.1 Myanmar calendar and Delta T / ayanamsha / house conventions".into(),
        evidence: vec![
            "original APK main.w0 (main.java:15451)".into(),
            "original APK Q3 (main.java:56)".into(),
            "motaa-circle/circle.tex mansion labels".into(),
        ],
        warnings,
    })
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn j2000() {
        assert_eq!(julian_midnight(2000, 1, 1), 2451544.5)
    }
    #[test]
    fn known_weekdays_are_iso_correct() {
        for (y, m, d, expected) in [(2000, 1, 1, 6), (2024, 4, 17, 3), (2026, 10, 22, 4)] {
            let mut r = ChartRequest::default();
            r.year = y;
            r.month = m;
            r.day = d;
            assert_eq!(calculate(&r).unwrap().weekday_iso, expected, "{y}-{m}-{d}");
        }
    }
    #[test]
    fn reference_moon_uses_independent_positions_not_manual_input() {
        let request = ChartRequest {
            year: 2000,
            month: 1,
            day: 1,
            hour: 12,
            timezone_id: None,
            timezone_offset_minutes: 0,
            ..ChartRequest::default()
        };
        let result = calculate(&request).unwrap();
        assert!(result.moon.is_none());
        let calculated = result.reference_moon.expect("reference Moon expected");
        assert_eq!(calculated.nakshatra_number, 15);
        assert!(calculated.longitude > 195.0 && calculated.longitude < 205.0);
        assert!(calculated.source.contains("V31.1 Delta T and ayanamsha"));
    }
    #[test]
    fn utc_noon() {
        let mut a = ChartRequest::default();
        a.timezone_offset_minutes = 0;
        a.timezone_id = None;
        assert_eq!(calculate(&a).unwrap().julian_day_utc, Some(2451545.0))
    }
    #[test]
    fn leap() {
        let mut a = ChartRequest::default();
        a.month = 2;
        a.day = 29;
        assert!(validate(&a).is_ok());
        a.year = 1900;
        assert!(validate(&a).is_err())
    }
    #[test]
    fn no_fabrication() {
        assert!(calculate(&ChartRequest::default()).unwrap().moon.is_none())
    }
    #[test]
    fn ruling_modes() {
        let a = ChartRequest::default();
        assert_eq!(calculate(&a).unwrap().nakshatras[7].ruler_code, 0);
        let mut b = a;
        b.corrected_rulers = true;
        assert_eq!(calculate(&b).unwrap().nakshatras[7].ruler_code, 7)
    }
    #[test]
    fn manual_longitude() {
        let mut a = ChartRequest::default();
        a.manual_moon_longitude = Some(359.99);
        assert_eq!(calculate(&a).unwrap().moon.unwrap().nakshatra_number, 27)
    }
    #[test]
    fn twenty_eight() {
        let mut a = ChartRequest::default();
        a.nakshatra_count = 28;
        let o = calculate(&a).unwrap();
        assert_eq!(o.nakshatras.len(), 28);
        assert_eq!(o.nakshatras[21].start_degree, None)
    }
    #[test]
    fn historical_timezone_offset() {
        let mut r = ChartRequest::default();
        r.year = 2024;
        r.month = 1;
        r.day = 1;
        assert_eq!(effective_utc_offset(&r).unwrap(), 390);
        r.timezone_id = Some("America/New_York".into());
        r.month = 7;
        assert_eq!(effective_utc_offset(&r).unwrap(), -240);
        r.month = 1;
        assert_eq!(effective_utc_offset(&r).unwrap(), -300)
    }
    #[test]
    fn ambiguous_time_rejected() {
        let mut r = ChartRequest::default();
        r.timezone_id = Some("America/New_York".into());
        r.year = 2024;
        r.month = 11;
        r.day = 3;
        r.hour = 1;
        r.minute = 30;
        assert!(effective_utc_offset(&r).unwrap_err().contains("Ambiguous"));
    }
}
