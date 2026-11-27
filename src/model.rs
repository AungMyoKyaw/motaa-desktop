use std::cell::RefCell;

use jiff::{
    Timestamp,
    civil::DateTime,
    tz::{Offset, TimeZone},
};
use motaa_engine::{ChartRequest, ChartResult};
use motaa_store::Profile;
use rusqlite::Connection;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Workspace {
    Charts,
    Profiles,
    Calculations,
    Time,
    Advanced,
    About,
}

impl Workspace {
    pub const ALL: [Self; 6] = [
        Self::Charts,
        Self::Profiles,
        Self::Calculations,
        Self::Time,
        Self::Advanced,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Charts => "Charts",
            Self::Profiles => "Profiles",
            Self::Calculations => "Calculations",
            Self::Time => "Time & Analysis",
            Self::Advanced => "Advanced",
            Self::About => "About",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl Step {
    pub const ALL: [Self; 6] = [
        Self::Year,
        Self::Month,
        Self::Day,
        Self::Hour,
        Self::Minute,
        Self::Second,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Year => "Year",
            Self::Month => "Month",
            Self::Day => "Day",
            Self::Hour => "Hour",
            Self::Minute => "Minute",
            Self::Second => "Second",
        }
    }

    pub fn offset(self, time: DateTime, count: i32) -> Option<DateTime> {
        let months = match self {
            Self::Year => count.checked_mul(12)?,
            Self::Month => count,
            Self::Day => {
                return time
                    .checked_add(jiff::SignedDuration::from_hours(i64::from(count) * 24))
                    .ok();
            }
            Self::Hour => {
                return time
                    .checked_add(jiff::SignedDuration::from_hours(i64::from(count)))
                    .ok();
            }
            Self::Minute => {
                return time
                    .checked_add(jiff::SignedDuration::from_mins(i64::from(count)))
                    .ok();
            }
            Self::Second => {
                return time
                    .checked_add(jiff::SignedDuration::from_secs(i64::from(count)))
                    .ok();
            }
        };
        time.checked_add(jiff::Span::new().months(months)).ok()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub tropical: bool,
    pub ayanamsha: usize,
    pub houses: usize,
    pub mean_node: bool,
    pub calendar: motaa_engine::myanmar_calendar::CivilCalendar,
    pub fifteen_degree_display: bool,
}

pub const AYANAMSHAS: [&str; 9] = [
    "Lahiri",
    "Fagan–Bradley",
    "De Luce",
    "Raman",
    "Ushashashi",
    "Krishnamurti",
    "Djwhal Khul",
    "Yukteshwar",
    "JN Bhasin",
];
pub const HOUSES: [&str; 8] = [
    "Equal",
    "Vehlow Placidus",
    "Placidus",
    "Koch",
    "Porphyry",
    "Regiomontanus",
    "Campanus",
    "Vehlow Equal",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceSource {
    SelectedProfile(i64),
    NatalProfile(i64),
}

#[derive(Clone, PartialEq, Eq)]
struct ChartCacheKey {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    timezone_offset_minutes: i32,
    timezone_id: Option<String>,
    latitude_bits: u64,
    longitude_bits: u64,
    unknown_birth_time: bool,
    ayanamsha: usize,
    time: Option<DateTime>,
}

impl ChartCacheKey {
    fn new(profile: &ChartRequest, ayanamsha: usize, time: Option<DateTime>) -> Self {
        Self {
            year: profile.year,
            month: profile.month,
            day: profile.day,
            hour: profile.hour,
            minute: profile.minute,
            second: profile.second,
            timezone_offset_minutes: profile.timezone_offset_minutes,
            timezone_id: profile.timezone_id.clone(),
            latitude_bits: profile.latitude.to_bits(),
            longitude_bits: profile.longitude.to_bits(),
            unknown_birth_time: profile.unknown_birth_time,
            ayanamsha,
            time,
        }
    }
}

type ChartCacheSlot = Option<(ChartCacheKey, Result<ChartResult, String>)>;

pub struct WorkspaceSession {
    pub workspace: Workspace,
    pub natal: ChartRequest,
    pub natal_id: Option<i64>,
    pub transit: Option<ChartRequest>,
    pub reference_source: Option<ReferenceSource>,
    pub planet_transit: bool,
    pub ascendant_transit: bool,
    pub settings: Settings,
    pub settings_draft: Settings,
    pub anchor: DateTime,
    pub time: DateTime,
    pub step: Step,
    pub mode: &'static str,
    pub tool: Option<&'static str>,
    pub store: Option<Connection>,
    pub status: String,
    calc_cache: RefCell<[ChartCacheSlot; 2]>,
}

impl WorkspaceSession {
    pub fn new(store: Result<Connection, String>, now: DateTime) -> Self {
        let (store, status) = match store {
            Ok(store) => (Some(store), String::new()),
            Err(error) => (None, format!("Profile library unavailable: {error}")),
        };
        Self {
            workspace: Workspace::Charts,
            natal: ChartRequest {
                name: "Anonymous sample".into(),
                ..ChartRequest::default()
            },
            natal_id: None,
            transit: None,
            reference_source: None,
            planet_transit: false,
            ascendant_transit: false,
            settings: Settings::default(),
            settings_draft: Settings::default(),
            anchor: now,
            time: now,
            step: Step::Day,
            mode: "Rasi",
            tool: None,
            store,
            status,
            calc_cache: RefCell::new([None, None]),
        }
    }

    pub fn profiles(&self) -> Result<Vec<Profile>, String> {
        motaa_store::list(self.connection()?)
    }

    pub fn connection(&self) -> Result<&Connection, String> {
        self.store
            .as_ref()
            .ok_or_else(|| "Profile library unavailable".into())
    }

    pub fn clear_reference_location(&mut self) {
        self.transit = None;
        self.reference_source = None;
    }

    pub fn activate_natal(
        &mut self,
        id: i64,
        profile: ChartRequest,
        reference: Option<ChartRequest>,
    ) {
        if let Some(reference) = reference {
            self.transit = Some(reference);
            self.reference_source = Some(ReferenceSource::NatalProfile(id));
        } else if matches!(
            self.reference_source,
            Some(ReferenceSource::NatalProfile(_))
        ) {
            self.clear_reference_location();
        }
        self.natal = profile;
        self.natal_id = Some(id);
    }

    pub fn reference_profile(&self) -> &ChartRequest {
        self.transit.as_ref().unwrap_or(&self.natal)
    }

    pub fn now(&self) -> DateTime {
        let profile = self.reference_profile();
        let now = Timestamp::now();
        if let Some(zone) = profile
            .timezone_id
            .as_ref()
            .and_then(|zone| TimeZone::get(zone).ok())
        {
            return zone.to_datetime(now);
        }
        Offset::from_seconds(profile.timezone_offset_minutes * 60)
            .map_or(TimeZone::UTC.to_datetime(now), |offset| {
                TimeZone::fixed(offset).to_datetime(now)
            })
    }

    pub fn profile_reference_location(&self, id: i64) -> Result<Option<ChartRequest>, String> {
        Ok(self.profile_reference_preference(id)?.flatten())
    }

    pub fn profile_reference_preference(
        &self,
        id: i64,
    ) -> Result<Option<Option<ChartRequest>>, String> {
        motaa_store::setting(
            self.connection()?,
            &format!("profile_reference_location_{id}"),
        )?
        .map(|value| serde_json::from_str(&value).map_err(|error| error.to_string()))
        .transpose()
    }

    pub fn store_reference_for_active_profile(
        &self,
        reference: Option<&ChartRequest>,
    ) -> Result<(), String> {
        if let Some(id) = self.natal_id {
            let value = serde_json::to_string(&reference).map_err(|error| error.to_string())?;
            motaa_store::put_setting(
                self.connection()?,
                &format!("profile_reference_location_{id}"),
                &value,
            )?;
        }
        Ok(())
    }

    pub fn profile_settings(&self, id: i64) -> Result<Settings, String> {
        let stored =
            motaa_store::setting(self.connection()?, &format!("profile_calculations_{id}"))?;
        let Some(stored) = stored else {
            return Ok(Settings::default());
        };
        let settings: Settings =
            serde_json::from_str(&stored).map_err(|error| error.to_string())?;
        if settings.ayanamsha >= AYANAMSHAS.len() || settings.houses >= HOUSES.len() {
            return Err("Saved calculation options are out of range.".into());
        }
        Ok(settings)
    }
    pub fn calculate(&self, transit: bool) -> Result<ChartResult, String> {
        let profile = if transit {
            self.reference_profile()
        } else {
            &self.natal
        };
        let key = ChartCacheKey::new(
            profile,
            self.settings.ayanamsha,
            transit.then_some(self.time),
        );
        let slot = usize::from(transit);
        if let Some((cached_key, cached_result)) = &self.calc_cache.borrow()[slot]
            && cached_key == &key
        {
            return cached_result.clone();
        }
        let result = (|| {
            let mut chart = if transit {
                calculate_at(profile, self.time)?
            } else {
                motaa_engine::calculate(profile)?
            };
            if let Some(ephemeris) = chart.reference_ephemeris.as_mut() {
                ephemeris.select_ayanamsha(self.settings.ayanamsha)?;
            }
            Ok(chart)
        })();
        self.calc_cache.borrow_mut()[slot] = Some((key, result.clone()));
        result
    }
}

pub fn chart_request_at(profile: &ChartRequest, time: DateTime) -> ChartRequest {
    let mut request = profile.clone();
    request.year = i32::from(time.year());
    request.month = time.month() as u8;
    request.day = time.day() as u8;
    request.hour = time.hour() as u8;
    request.minute = time.minute() as u8;
    request.second = time.second() as u8;
    request.unknown_birth_time = false;
    request
}

pub fn calculate_at(profile: &ChartRequest, time: DateTime) -> Result<ChartResult, String> {
    motaa_engine::calculate(&chart_request_at(profile, time))
}

pub fn julian_at(profile: &ChartRequest, time: DateTime) -> Result<f64, String> {
    let request = chart_request_at(profile, time);
    let offset = motaa_engine::effective_utc_offset(&request)?;
    let midnight =
        motaa_engine::julian_midnight(i32::from(time.year()), time.month() as u8, time.day() as u8);
    let seconds_from_midnight =
        i32::from(time.hour()) * 3600 + i32::from(time.minute()) * 60 + i32::from(time.second());
    Ok(midnight + f64::from(seconds_from_midnight) / 86_400. - f64::from(offset) / 1440.)
}

pub fn reference_datetime(profile: &ChartRequest, jdut: f64) -> Option<DateTime> {
    if !jdut.is_finite() {
        return None;
    }
    let seconds = ((jdut - 2_440_587.5) * 86_400.).round() as i64;
    let time = Timestamp::from_second(seconds).ok()?;
    let tz = if let Some(zone) = profile.timezone_id.as_deref() {
        TimeZone::get(zone).ok()?
    } else {
        TimeZone::fixed(
            Offset::from_seconds(profile.timezone_offset_minutes.checked_mul(60)?).ok()?,
        )
    };
    Some(tz.to_datetime(time))
}

pub fn display_reference_time(profile: &ChartRequest, jdut: f64) -> Option<String> {
    if !jdut.is_finite() {
        return None;
    }
    let seconds = ((jdut - 2_440_587.5) * 86_400.).round() as i64;
    let time = Timestamp::from_second(seconds).ok()?;
    let tz = if let Some(zone) = profile.timezone_id.as_deref() {
        TimeZone::get(zone).ok()?
    } else {
        TimeZone::fixed(
            Offset::from_seconds(profile.timezone_offset_minutes.checked_mul(60)?).ok()?,
        )
    };
    let dt = tz.to_datetime(time);
    let offset_secs = tz.to_offset(time).seconds();
    let sign = if offset_secs < 0 { '-' } else { '+' };
    let abs_mins = offset_secs.abs() / 60;
    Some(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}{:02}:{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        sign,
        abs_mins / 60,
        abs_mins % 60
    ))
}

pub fn daily_timing(
    profile: &ChartRequest,
    selection: DateTime,
    ayanamsha: usize,
) -> Result<([f64; 2], motaa_engine::panchanga::DailyTiming), String> {
    let start = selection.date().at(0, 0, 0, 0);
    let end = selection
        .date()
        .tomorrow()
        .map_err(|_| "Invalid next civil date.")?
        .at(0, 0, 0, 0);
    let request = chart_request_at(profile, selection);
    let bounds = [julian_at(profile, start)?, julian_at(profile, end)?];
    let timing = motaa_engine::panchanga::calculate_with_ayanamsha(
        julian_at(profile, selection)?,
        bounds[0],
        bounds[1],
        &request,
        ayanamsha,
    )?;
    Ok((bounds, timing))
}
