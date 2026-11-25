use std::{
    cell::Cell,
    f64::consts::PI,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, OnceLock},
    time::Duration,
};

use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::{OffsetComponents, OffsetName, TZ_VARIANTS, Tz};
use gpui::{
    AnyElement, Bounds, Context, Div, Entity, Image, ImageFormat, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollDelta, ScrollWheelEvent, canvas, div, img,
    prelude::*, px, rgb,
};
use serde::Deserialize;

use crate::{
    DesktopRoot, GOLD, INK, LINE, MUTED, PAPER, RADIUS_CONTROL, RADIUS_FIELD, SELECTED, SOFT,
    SUBTLE_LINE, action_control, button, caption, column, error_banner, field, input::TextInput,
    panel, parse_coordinate, row, section_label, text_input,
};

const USER_AGENT: &str = concat!(
    "MOTAA-Desktop/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/AungMyoKyaw/motaa-desktop)"
);
const MAP_HEIGHT: f32 = 280.;
const DEFAULT_MAP_WIDTH: f32 = 560.;
const MAX_MERCATOR_LAT: f64 = 85.051_128_78;
const MIN_ZOOM: u8 = 2;
const MAX_ZOOM: u8 = 16;

pub const COMMON_TIMEZONES: [&str; 7] = [
    "Asia/Yangon",
    "Asia/Bangkok",
    "Asia/Singapore",
    "Europe/London",
    "America/New_York",
    "America/Los_Angeles",
    "UTC",
];

static TZ_FINDER: OnceLock<tzf_rs::DefaultFinder> = OnceLock::new();

#[derive(Clone)]
pub enum TileState {
    Loading,
    Loaded(Arc<Image>),
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OsmPlace {
    pub display_name: String,
    pub short_name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimezoneOption {
    pub name: &'static str,
    pub offset_label: String,
    pub abbr: String,
}

pub struct LocationPickerState {
    pub osm_open: bool,
    pub osm_query: Entity<TextInput>,
    pub osm_loading: bool,
    pub osm_error: Option<String>,
    pub osm_results: Vec<OsmPlace>,
    pub osm_request_id: u64,
    pub map_open: bool,
    pub map_center_lat: f64,
    pub map_center_lon: f64,
    pub map_zoom: u8,
    pub map_drag: Option<(Point<Pixels>, f64, f64, bool)>,
    pub map_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub reverse_request_id: u64,
    pub tz_open: bool,
    pub tz_query: Entity<TextInput>,
}

impl LocationPickerState {
    pub fn new(initial_lat: f64, initial_lon: f64, cx: &mut Context<DesktopRoot>) -> Self {
        let osm_query = text_input(
            "",
            "Search city, town, or address (e.g. Milton Massachusetts)",
            cx,
        );
        let tz_query = text_input(
            "",
            "Filter by city, region, or UTC offset (e.g. +06:30)",
            cx,
        );
        cx.observe(&osm_query, |_, _, cx| cx.notify()).detach();
        cx.observe(&tz_query, |_, _, cx| cx.notify()).detach();
        Self {
            osm_open: false,
            osm_query,
            osm_loading: false,
            osm_error: None,
            osm_results: Vec::new(),
            osm_request_id: 0,
            map_open: false,
            map_center_lat: initial_lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT),
            map_center_lon: initial_lon.clamp(-180., 180.),
            map_zoom: 11,
            map_drag: None,
            map_bounds: Rc::new(Cell::new(None)),
            reverse_request_id: 0,
            tz_open: false,
            tz_query,
        }
    }
}

pub fn timezone_for_coords(lat: f64, lon: f64) -> Option<String> {
    if !(-90. ..=90.).contains(&lat) || !(-180. ..=180.).contains(&lon) {
        return None;
    }
    let finder = TZ_FINDER.get_or_init(tzf_rs::DefaultFinder::new);
    // CRITICAL: tzf-rs takes (longitude, latitude).
    let tz = finder.get_tz_name(lon, lat).trim();
    if tz.is_empty() || tz.parse::<Tz>().is_err() {
        None
    } else {
        Some(tz.to_owned())
    }
}

pub fn format_utc_offset(offset_seconds: i32) -> String {
    let sign = if offset_seconds < 0 { '-' } else { '+' };
    let total_minutes = offset_seconds.unsigned_abs() / 60;
    format!(
        "UTC{sign}{:02}:{:02}",
        total_minutes / 60,
        total_minutes % 60
    )
}

fn tz_offset_and_abbr_at(tz: Tz, dt: NaiveDateTime) -> (i32, String) {
    let mapped = tz
        .from_local_datetime(&dt)
        .earliest()
        .unwrap_or_else(|| tz.from_utc_datetime(&dt));
    let offset = mapped.offset();
    let total_seconds =
        offset.base_utc_offset().num_seconds() as i32 + offset.dst_offset().num_seconds() as i32;
    let abbr = offset
        .abbreviation()
        .map(str::to_owned)
        .unwrap_or_else(|| format_utc_offset(total_seconds));
    (total_seconds, abbr)
}

pub fn describe_timezone_at(value: &str, dt: NaiveDateTime) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(
            "Enter an IANA timezone or UTC offset in minutes, or pick from the list.".into(),
        );
    }
    if let Ok(minutes) = trimmed.parse::<i32>() {
        if !(-840..=840).contains(&minutes) {
            return Err("UTC offset in minutes must be between -840 and 840.".into());
        }
        let formatted = format_utc_offset(minutes * 60);
        return Ok(format!("Fixed offset · {formatted} ({minutes} min)"));
    }
    let tz: Tz = trimmed.parse().map_err(|_| {
        "Unknown IANA timezone or UTC offset in minutes — pick from the list.".to_owned()
    })?;
    let (seconds, abbr) = tz_offset_and_abbr_at(tz, dt);
    let offset_label = format_utc_offset(seconds);
    let date_label = dt.format("%Y-%m-%d");
    Ok(format!("{offset_label} · {abbr} on {date_label}"))
}

pub fn filter_timezones(query: &str, dt: NaiveDateTime, limit: usize) -> Vec<TimezoneOption> {
    let q = query.trim().to_ascii_lowercase();
    let mut ranked: Vec<(u8, TimezoneOption)> = Vec::new();
    for &tz in TZ_VARIANTS.iter() {
        let name = tz.name();
        let (seconds, abbr) = tz_offset_and_abbr_at(tz, dt);
        let offset_label = format_utc_offset(seconds);
        let rank = if q.is_empty() {
            if COMMON_TIMEZONES.contains(&name) {
                0
            } else {
                2
            }
        } else {
            let name_lower = name.to_ascii_lowercase();
            let name_spaced = name_lower.replace('_', " ");
            let city_lower = name_lower.rsplit('/').next().unwrap_or(&name_lower);
            let city_spaced = city_lower.replace('_', " ");
            let offset_lower = offset_label.to_ascii_lowercase();
            let offset_short = offset_lower.trim_start_matches("utc");
            let abbr_lower = abbr.to_ascii_lowercase();

            if city_lower.starts_with(&q)
                || city_spaced.starts_with(&q)
                || name_lower.starts_with(&q)
            {
                0
            } else if name_lower.contains(&q)
                || name_spaced.contains(&q)
                || abbr_lower == q
                || offset_lower.contains(&q)
                || offset_short.contains(&q)
            {
                1
            } else {
                continue;
            }
        };
        ranked.push((
            rank,
            TimezoneOption {
                name,
                offset_label,
                abbr,
            },
        ));
    }
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.cmp(b.1.name)));
    ranked.into_iter().take(limit).map(|(_, opt)| opt).collect()
}

pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

fn http_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .build();
    ureq::Agent::new_with_config(config)
}

#[derive(Deserialize)]
struct NominatimResult {
    display_name: String,
    #[serde(default)]
    name: String,
    lat: String,
    lon: String,
}

fn concise_place_name(name: &str, display_name: &str) -> String {
    let parts: Vec<&str> = display_name
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return name.trim().to_owned();
    }
    if parts.len() <= 3 {
        parts.join(", ")
    } else {
        format!("{}, {}, {}", parts[0], parts[1], parts[parts.len() - 1])
    }
}

pub fn search_osm_places(query: &str) -> Result<Vec<OsmPlace>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let url = format!(
        "https://nominatim.openstreetmap.org/search?format=jsonv2&limit=6&q={}",
        percent_encode(trimmed)
    );
    let mut response = http_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .call()
        .map_err(|error| format!("OpenStreetMap search failed: {error}"))?;
    let rows: Vec<NominatimResult> = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Invalid OpenStreetMap response: {error}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let lat = row.lat.parse::<f64>().ok()?;
            let lon = row.lon.parse::<f64>().ok()?;
            let short_name = concise_place_name(&row.name, &row.display_name);
            Some(OsmPlace {
                display_name: row.display_name,
                short_name,
                lat,
                lon,
            })
        })
        .collect())
}

#[derive(Deserialize)]
struct NominatimReverseResult {
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    name: String,
}

pub fn reverse_osm_place(lat: f64, lon: f64) -> Result<String, String> {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=jsonv2&zoom=10&lat={lat:.5}&lon={lon:.5}"
    );
    let mut response = http_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .call()
        .map_err(|error| format!("Reverse geocoding failed: {error}"))?;
    let row: NominatimReverseResult = response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Invalid reverse geocode response: {error}"))?;
    let name = concise_place_name(&row.name, &row.display_name);
    if name.is_empty() {
        Err("No place name found at coordinates.".into())
    } else {
        Ok(name)
    }
}

pub fn lat_lon_to_tile_xy(lat: f64, lon: f64, zoom: u8) -> (f64, f64) {
    let n = (1u64 << zoom) as f64;
    let clamped_lat = lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
    let lat_rad = clamped_lat.to_radians();
    let x = ((lon + 180.) / 360.) * n;
    let y = (1. - (lat_rad.tan() + 1. / lat_rad.cos()).ln() / PI) * 0.5 * n;
    (x, y)
}

pub fn tile_xy_to_lat_lon(tx: f64, ty: f64, zoom: u8) -> (f64, f64) {
    let n = (1u64 << zoom) as f64;
    let lon = (tx / n) * 360. - 180.;
    let lat_rad = (PI * (1. - 2. * ty / n)).sinh().atan();
    let lat = lat_rad
        .to_degrees()
        .clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
    (lat, lon.clamp(-180., 180.))
}

pub fn pixel_offset_to_lat_lon(
    center_lat: f64,
    center_lon: f64,
    zoom: u8,
    dx: f64,
    dy: f64,
) -> (f64, f64) {
    let (cx, cy) = lat_lon_to_tile_xy(center_lat, center_lon, zoom);
    let n = (1u64 << zoom) as f64;
    let tx = (cx + dx / 256.).rem_euclid(n);
    let ty = (cy + dy / 256.).clamp(0., n);
    tile_xy_to_lat_lon(tx, ty, zoom)
}

fn tile_cache_root() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Caches/MOTAA-Desktop/osm-tiles"))
        .unwrap_or_else(|| std::env::temp_dir().join("motaa-osm-tiles"))
}

pub fn fetch_osm_tile(z: u8, x: u32, y: u32) -> Result<Vec<u8>, String> {
    let cache_path = tile_cache_root()
        .join(z.to_string())
        .join(x.to_string())
        .join(format!("{y}.png"));
    if let Ok(bytes) = std::fs::read(&cache_path)
        && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        return Ok(bytes);
    }
    let url = format!("https://tile.openstreetmap.org/{z}/{x}/{y}.png");
    let mut response = http_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| format!("Tile fetch failed: {error}"))?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|error| format!("Tile read failed: {error}"))?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("Tile response was not a PNG image.".into());
    }
    if let Some(parent) = cache_path.parent()
        && std::fs::create_dir_all(parent).is_ok()
    {
        let _ = std::fs::write(&cache_path, &bytes);
    }
    Ok(bytes)
}

fn map_ctrl_button(
    id: (&'static str, usize),
    label: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control(id, cx, action)
        .px_2()
        .h(px(28.))
        .rounded(px(RADIUS_FIELD))
        .border_1()
        .border_color(rgb(LINE))
        .bg(rgb(PAPER))
        .text_size(px(12.))
        .text_color(rgb(INK))
        .hover(|s| s.bg(rgb(SOFT)))
        .child(label.into())
}

struct DraftHandles<'a> {
    place: &'a Entity<TextInput>,
    latitude: &'a Entity<TextInput>,
    longitude: &'a Entity<TextInput>,
    timezone: &'a Entity<TextInput>,
    town_search: &'a Option<Entity<TextInput>>,
    location: &'a LocationPickerState,
}

struct DraftHandlesMut<'a> {
    place: &'a Entity<TextInput>,
    latitude: &'a Entity<TextInput>,
    longitude: &'a Entity<TextInput>,
    timezone: &'a Entity<TextInput>,
    location: &'a mut LocationPickerState,
}

impl DesktopRoot {
    fn draft_handles(&self, reference: bool) -> Option<DraftHandles<'_>> {
        if reference {
            self.reference_location_draft
                .as_ref()
                .map(|d| DraftHandles {
                    place: &d.place,
                    latitude: &d.latitude,
                    longitude: &d.longitude,
                    timezone: &d.timezone,
                    town_search: &d.town_search,
                    location: &d.location,
                })
        } else {
            self.draft.as_ref().map(|d| DraftHandles {
                place: &d.place,
                latitude: &d.latitude,
                longitude: &d.longitude,
                timezone: &d.timezone,
                town_search: &d.town_search,
                location: &d.location,
            })
        }
    }

    fn draft_handles_mut(&mut self, reference: bool) -> Option<DraftHandlesMut<'_>> {
        if reference {
            self.reference_location_draft
                .as_mut()
                .map(|d| DraftHandlesMut {
                    place: &d.place,
                    latitude: &d.latitude,
                    longitude: &d.longitude,
                    timezone: &d.timezone,
                    location: &mut d.location,
                })
        } else {
            self.draft.as_mut().map(|d| DraftHandlesMut {
                place: &d.place,
                latitude: &d.latitude,
                longitude: &d.longitude,
                timezone: &d.timezone,
                location: &mut d.location,
            })
        }
    }

    pub(super) fn location_and_timezone_section(
        &self,
        reference: bool,
        effective_dt: NaiveDateTime,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(handles) = self.draft_handles(reference) else {
            return column();
        };
        let prefix = if reference { "ref" } else { "prof" };
        let place_label = if reference {
            "Place name"
        } else {
            "Birthplace"
        };
        let place_input = handles.place.clone();
        let lat_input = handles.latitude.clone();
        let lon_input = handles.longitude.clone();
        let tz_input = handles.timezone.clone();
        let town_open = handles.town_search.is_some();
        let osm_open = handles.location.osm_open;
        let map_open = handles.location.map_open;
        let tz_open = handles.location.tz_open;
        let tz_value = tz_input.read(cx).value().to_owned();
        let tz_status = describe_timezone_at(&tz_value, effective_dt);

        let mut section = column().child(field(place_label, place_input)).child(
            row()
                .flex_wrap()
                .child(
                    action_control((prefix, 100usize), cx, move |view, cx| {
                        view.toggle_osm_search(reference, cx);
                    })
                    .px_3()
                    .h(px(34.))
                    .rounded(px(RADIUS_CONTROL))
                    .border_1()
                    .border_color(rgb(if osm_open { GOLD } else { LINE }))
                    .bg(rgb(if osm_open { SELECTED } else { PAPER }))
                    .text_size(px(13.))
                    .text_color(rgb(INK))
                    .hover(|s| s.bg(rgb(SOFT)))
                    .child("Search OpenStreetMap"),
                )
                .child(
                    action_control((prefix, 101usize), cx, move |view, cx| {
                        view.toggle_map_picker(reference, cx);
                    })
                    .px_3()
                    .h(px(34.))
                    .rounded(px(RADIUS_CONTROL))
                    .border_1()
                    .border_color(rgb(if map_open { GOLD } else { LINE }))
                    .bg(rgb(if map_open { SELECTED } else { PAPER }))
                    .text_size(px(13.))
                    .text_color(rgb(INK))
                    .hover(|s| s.bg(rgb(SOFT)))
                    .child("Pick on map"),
                )
                .child(
                    action_control((prefix, 102usize), cx, move |view, cx| {
                        view.toggle_town_picker(reference, cx);
                    })
                    .px_3()
                    .h(px(34.))
                    .rounded(px(RADIUS_CONTROL))
                    .border_1()
                    .border_color(rgb(if town_open { GOLD } else { LINE }))
                    .bg(rgb(if town_open { SELECTED } else { PAPER }))
                    .text_size(px(13.))
                    .text_color(rgb(INK))
                    .hover(|s| s.bg(rgb(SOFT)))
                    .child("Myanmar town directory"),
                ),
        );

        if osm_open {
            section = section.child(self.render_osm_search_panel(reference, cx));
        }
        if map_open {
            section = section.child(self.render_osm_map_panel(reference, cx));
        }
        if handles.town_search.is_some() {
            section = section.child(self.town_picker_panel(reference, cx));
        }

        section = section
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(field("Latitude · decimal or D M S N/S", lat_input))
                    .child(field("Longitude · decimal or D M S E/W", lon_input)),
            )
            .child(
                column()
                    .gap_1()
                    .child(caption("Historical IANA timezone or UTC offset in minutes"))
                    .child(
                        row()
                            .w_full()
                            .child(div().flex_1().min_w(px(0.)).child(tz_input))
                            .child(
                                action_control((prefix, 103usize), cx, move |view, _| {
                                    if let Some(h) = view.draft_handles_mut(reference) {
                                        h.location.tz_open = !h.location.tz_open;
                                    }
                                })
                                .px_3()
                                .h(px(38.))
                                .rounded(px(RADIUS_CONTROL))
                                .border_1()
                                .border_color(rgb(if tz_open { GOLD } else { LINE }))
                                .bg(rgb(if tz_open { SELECTED } else { PAPER }))
                                .text_size(px(13.))
                                .text_color(rgb(INK))
                                .hover(|s| s.bg(rgb(SOFT)))
                                .child(if tz_open {
                                    "Hide list"
                                } else {
                                    "Choose timezone"
                                }),
                            )
                            .child(
                                action_control((prefix, 104usize), cx, move |view, cx| {
                                    view.autodetect_timezone(reference, cx);
                                })
                                .px_3()
                                .h(px(38.))
                                .rounded(px(RADIUS_CONTROL))
                                .border_1()
                                .border_color(rgb(LINE))
                                .bg(rgb(PAPER))
                                .text_size(px(13.))
                                .text_color(rgb(INK))
                                .hover(|s| s.bg(rgb(SOFT)))
                                .child("Auto-detect"),
                            ),
                    )
                    .child(match tz_status {
                        Ok(summary) => caption(summary).into_any_element(),
                        Err(err) => error_banner(err).into_any_element(),
                    }),
            );

        if tz_open {
            section = section.child(self.render_timezone_panel(reference, effective_dt, cx));
        }

        section
    }

    pub(super) fn toggle_osm_search(&mut self, reference: bool, cx: &mut Context<Self>) {
        let Some(handles) = self.draft_handles_mut(reference) else {
            return;
        };
        let next = !handles.location.osm_open;
        handles.location.osm_open = next;
        if next {
            let place = handles.place.read(cx).value().trim().to_owned();
            if handles
                .location
                .osm_query
                .read(cx)
                .value()
                .trim()
                .is_empty()
                && !place.is_empty()
            {
                handles
                    .location
                    .osm_query
                    .update(cx, |input, cx| input.set_value(place, cx));
            }
        }
    }

    pub(super) fn toggle_map_picker(&mut self, reference: bool, cx: &mut Context<Self>) {
        let (lat, lon) = {
            let Some(handles) = self.draft_handles(reference) else {
                return;
            };
            let lat = parse_coordinate(handles.latitude.read(cx).value(), true).unwrap_or(16.8409);
            let lon =
                parse_coordinate(handles.longitude.read(cx).value(), false).unwrap_or(96.1735);
            (lat, lon)
        };
        if let Some(handles) = self.draft_handles_mut(reference) {
            handles.location.map_open = !handles.location.map_open;
            if handles.location.map_open {
                handles.location.map_center_lat = lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
                handles.location.map_center_lon = lon.clamp(-180., 180.);
            }
        }
    }

    fn trigger_osm_search(&mut self, reference: bool, cx: &mut Context<Self>) {
        let (query, req_id) = {
            let Some(handles) = self.draft_handles_mut(reference) else {
                return;
            };
            let q = handles
                .location
                .osm_query
                .read(cx)
                .value()
                .trim()
                .to_owned();
            if q.is_empty() {
                handles.location.osm_error =
                    Some("Enter a city, town, or address to search.".into());
                handles.location.osm_results.clear();
                return;
            }
            handles.location.osm_request_id += 1;
            handles.location.osm_loading = true;
            handles.location.osm_error = None;
            (q, handles.location.osm_request_id)
        };
        cx.spawn(async move |view, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { search_osm_places(&query) })
                .await;
            let _ = view.update(cx, |root, cx| {
                let Some(handles) = root.draft_handles_mut(reference) else {
                    return;
                };
                if handles.location.osm_request_id != req_id {
                    return;
                }
                handles.location.osm_loading = false;
                match result {
                    Ok(places) => {
                        if places.is_empty() {
                            handles.location.osm_error =
                                Some("No matching places found on OpenStreetMap.".into());
                        }
                        handles.location.osm_results = places;
                    }
                    Err(err) => {
                        handles.location.osm_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_osm_place(&mut self, reference: bool, index: usize, cx: &mut Context<Self>) {
        let Some(place) = self
            .draft_handles(reference)
            .and_then(|h| h.location.osm_results.get(index).cloned())
        else {
            return;
        };
        let tz = timezone_for_coords(place.lat, place.lon);
        let lat_str = format!("{:.4}", place.lat);
        let lon_str = format!("{:.4}", place.lon);
        let Some(handles) = self.draft_handles_mut(reference) else {
            return;
        };
        handles
            .place
            .update(cx, |input, cx| input.set_value(place.short_name, cx));
        handles
            .latitude
            .update(cx, |input, cx| input.set_value(lat_str, cx));
        handles
            .longitude
            .update(cx, |input, cx| input.set_value(lon_str, cx));
        if let Some(tz_name) = tz {
            handles
                .timezone
                .update(cx, |input, cx| input.set_value(tz_name, cx));
        }
        handles.location.map_center_lat = place.lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
        handles.location.map_center_lon = place.lon.clamp(-180., 180.);
        handles.location.osm_open = false;
    }

    pub(super) fn autodetect_timezone(&mut self, reference: bool, cx: &mut Context<Self>) {
        let (lat_str, lon_str) = {
            let Some(handles) = self.draft_handles(reference) else {
                return;
            };
            (
                handles.latitude.read(cx).value().to_owned(),
                handles.longitude.read(cx).value().to_owned(),
            )
        };
        let Ok(lat) = parse_coordinate(&lat_str, true) else {
            self.set_status("Enter a valid latitude first to auto-detect timezone.", cx);
            return;
        };
        let Ok(lon) = parse_coordinate(&lon_str, false) else {
            self.set_status("Enter a valid longitude first to auto-detect timezone.", cx);
            return;
        };
        match timezone_for_coords(lat, lon) {
            Some(tz_name) => {
                if let Some(handles) = self.draft_handles_mut(reference) {
                    handles
                        .timezone
                        .update(cx, |input, cx| input.set_value(tz_name.clone(), cx));
                }
                self.set_status(format!("Detected timezone: {tz_name}"), cx);
            }
            None => self.set_status("Could not detect timezone for those coordinates.", cx),
        }
    }

    pub(super) fn select_timezone_name(
        &mut self,
        reference: bool,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(handles) = self.draft_handles_mut(reference) {
            handles
                .timezone
                .update(cx, |input, cx| input.set_value(name.to_owned(), cx));
            handles.location.tz_open = false;
        }
    }

    fn set_pin_from_map(&mut self, reference: bool, lat: f64, lon: f64, cx: &mut Context<Self>) {
        let lat = lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
        let lon = lon.clamp(-180., 180.);
        let tz = timezone_for_coords(lat, lon);
        let req_id = {
            let Some(handles) = self.draft_handles_mut(reference) else {
                return;
            };
            handles
                .latitude
                .update(cx, |input, cx| input.set_value(format!("{lat:.4}"), cx));
            handles
                .longitude
                .update(cx, |input, cx| input.set_value(format!("{lon:.4}"), cx));
            if let Some(tz_name) = tz {
                handles
                    .timezone
                    .update(cx, |input, cx| input.set_value(tz_name, cx));
            }
            handles.location.reverse_request_id += 1;
            handles.location.reverse_request_id
        };
        cx.spawn(async move |view, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { reverse_osm_place(lat, lon) })
                .await;
            if let Ok(place_name) = result {
                let _ = view.update(cx, |root, cx| {
                    let Some(handles) = root.draft_handles(reference) else {
                        return;
                    };
                    if handles.location.reverse_request_id != req_id {
                        return;
                    }
                    handles
                        .place
                        .update(cx, |input, cx| input.set_value(place_name, cx));
                });
            }
        })
        .detach();
    }

    fn request_map_tile(&self, z: u8, x: u32, y: u32, cx: &mut Context<Self>) {
        if self.tile_cache.contains_key(&(z, x, y)) {
            return;
        }
        cx.spawn(async move |view, cx| {
            let _ = view.update(cx, |root, _| {
                root.tile_cache
                    .entry((z, x, y))
                    .or_insert(TileState::Loading);
            });
            let result = cx
                .background_executor()
                .spawn(async move { fetch_osm_tile(z, x, y) })
                .await;
            let _ = view.update(cx, |root, cx| {
                match result {
                    Ok(bytes) => {
                        let image = Arc::new(Image::from_bytes(ImageFormat::Png, bytes));
                        root.tile_cache.insert((z, x, y), TileState::Loaded(image));
                    }
                    Err(_) => {
                        root.tile_cache.insert((z, x, y), TileState::Failed);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn render_osm_search_panel(&self, reference: bool, cx: &mut Context<Self>) -> Div {
        let Some(handles) = self.draft_handles(reference) else {
            return panel();
        };
        let prefix = if reference { "ref-osm" } else { "prof-osm" };
        let query_input = handles.location.osm_query.clone();
        let loading = handles.location.osm_loading;
        let error = handles.location.osm_error.clone();
        let results = handles.location.osm_results.clone();

        panel()
            .bg(rgb(SOFT))
            .gap_2()
            .child(section_label("OPENSTREETMAP PLACE SEARCH"))
            .child(
                row()
                    .w_full()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .on_key_down(cx.listener(move |view, event: &gpui::KeyDownEvent, _, cx| {
                                if event.keystroke.key == "enter" {
                                    view.trigger_osm_search(reference, cx);
                                    cx.stop_propagation();
                                }
                            }))
                            .child(query_input),
                    )
                    .child(button(
                        if loading { "Searching…" } else { "Search" },
                        true,
                        cx,
                        move |view, cx| view.trigger_osm_search(reference, cx),
                    )),
            )
            .when_some(error, |p, err| p.child(error_banner(err)))
            .when(!results.is_empty(), |p| {
                p.child(
                    column().gap_1().children(results.into_iter().enumerate().map(
                        |(idx, place)| {
                            let coords = format!("{:.4}°, {:.4}°", place.lat, place.lon);
                            action_control((prefix, idx), cx, move |view, cx| {
                                view.apply_osm_place(reference, idx, cx);
                            })
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded(px(RADIUS_FIELD))
                            .border_1()
                            .border_color(rgb(SUBTLE_LINE))
                            .bg(rgb(PAPER))
                            .hover(|s| s.bg(rgb(SELECTED)))
                            .justify_between()
                            .child(
                                column()
                                    .gap_0()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(INK))
                                            .child(place.short_name),
                                    )
                                    .child(caption(place.display_name)),
                            )
                            .child(
                                div()
                                    .pl_3()
                                    .font_family("Menlo")
                                    .text_size(px(11.5))
                                    .text_color(rgb(MUTED))
                                    .child(coords),
                            )
                        },
                    )),
                )
            })
            .child(caption(
                "Uses OpenStreetMap Nominatim. Selecting a result fills place name, coordinates, and timezone.",
            ))
    }

    fn render_osm_map_panel(&self, reference: bool, cx: &mut Context<Self>) -> Div {
        let Some(handles) = self.draft_handles(reference) else {
            return panel();
        };
        let prefix = if reference { "ref-map" } else { "prof-map" };
        let center_lat = handles.location.map_center_lat;
        let center_lon = handles.location.map_center_lon;
        let zoom = handles.location.map_zoom;
        let bounds_cell = handles.location.map_bounds.clone();
        let pin_lat =
            parse_coordinate(handles.latitude.read(cx).value(), true).unwrap_or(center_lat);
        let pin_lon =
            parse_coordinate(handles.longitude.read(cx).value(), false).unwrap_or(center_lon);

        let viewport_w = bounds_cell
            .get()
            .map(|b| f32::from(b.size.width))
            .filter(|w| *w > 100.)
            .unwrap_or(DEFAULT_MAP_WIDTH);
        let viewport_h = MAP_HEIGHT;

        let (center_tx, center_ty) = lat_lon_to_tile_xy(center_lat, center_lon, zoom);
        let max_tile = (1u64 << zoom) as i64;
        let half_tiles_x = f64::from(viewport_w) / 512. + 1.;
        let half_tiles_y = f64::from(viewport_h) / 512. + 1.;
        let min_x = (center_tx - half_tiles_x).floor() as i64;
        let max_x = (center_tx + half_tiles_x).floor() as i64;
        let min_y = ((center_ty - half_tiles_y).floor() as i64).max(0);
        let max_y = ((center_ty + half_tiles_y).floor() as i64).min(max_tile - 1);

        let mut tile_elements: Vec<AnyElement> = Vec::new();
        for ty in min_y..=max_y {
            for tx in min_x..=max_x {
                let wrapped_x = tx.rem_euclid(max_tile) as u32;
                let tile_y = ty as u32;
                let left = (tx as f64 - center_tx) * 256. + f64::from(viewport_w) * 0.5;
                let top = (ty as f64 - center_ty) * 256. + f64::from(viewport_h) * 0.5;
                match self.tile_cache.get(&(zoom, wrapped_x, tile_y)) {
                    Some(TileState::Loaded(image)) => {
                        tile_elements.push(
                            img(image.clone())
                                .absolute()
                                .left(px(left as f32))
                                .top(px(top as f32))
                                .w(px(256.))
                                .h(px(256.))
                                .into_any_element(),
                        );
                    }
                    Some(TileState::Loading | TileState::Failed) => {}
                    None => {
                        self.request_map_tile(zoom, wrapped_x, tile_y, cx);
                    }
                }
            }
        }

        let (pin_tx, pin_ty) = lat_lon_to_tile_xy(pin_lat, pin_lon, zoom);
        let pin_x = ((pin_tx - center_tx) * 256. + f64::from(viewport_w) * 0.5) as f32;
        let pin_y = ((pin_ty - center_ty) * 256. + f64::from(viewport_h) * 0.5) as f32;
        let pin_visible = (-16. ..=viewport_w + 16.).contains(&pin_x)
            && (-16. ..=viewport_h + 16.).contains(&pin_y);

        let map_viewport = div()
            .id((prefix, 1usize))
            .relative()
            .w_full()
            .h(px(viewport_h))
            .overflow_hidden()
            .rounded(px(RADIUS_CONTROL))
            .border_1()
            .border_color(rgb(LINE))
            .bg(rgb(SOFT))
            .cursor_crosshair()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, event: &MouseDownEvent, _, _| {
                    if let Some(h) = view.draft_handles_mut(reference) {
                        h.location.map_drag = Some((
                            event.position,
                            h.location.map_center_lat,
                            h.location.map_center_lon,
                            false,
                        ));
                    }
                }),
            )
            .on_mouse_move(cx.listener(move |view, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    return;
                }
                let Some(h) = view.draft_handles_mut(reference) else {
                    return;
                };
                if let Some((start_pos, start_lat, start_lon, moved)) = h.location.map_drag {
                    let dx = f32::from(event.position.x - start_pos.x);
                    let dy = f32::from(event.position.y - start_pos.y);
                    if moved || dx.hypot(dy) > 4. {
                        let (next_lat, next_lon) = pixel_offset_to_lat_lon(
                            start_lat,
                            start_lon,
                            h.location.map_zoom,
                            -f64::from(dx),
                            -f64::from(dy),
                        );
                        h.location.map_center_lat = next_lat;
                        h.location.map_center_lon = next_lon;
                        h.location.map_drag = Some((start_pos, start_lat, start_lon, true));
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |view, event: &MouseUpEvent, _, cx| {
                    let drag = view
                        .draft_handles_mut(reference)
                        .and_then(|h| h.location.map_drag.take());
                    let Some((_, _, _, moved)) = drag else {
                        return;
                    };
                    if moved {
                        return;
                    }
                    let Some(h) = view.draft_handles(reference) else {
                        return;
                    };
                    let Some(bounds) = h.location.map_bounds.get() else {
                        return;
                    };
                    let dx = f64::from(f32::from(
                        event.position.x - (bounds.origin.x + bounds.size.width * 0.5),
                    ));
                    let dy = f64::from(f32::from(
                        event.position.y - (bounds.origin.y + bounds.size.height * 0.5),
                    ));
                    let (lat, lon) = pixel_offset_to_lat_lon(
                        h.location.map_center_lat,
                        h.location.map_center_lon,
                        h.location.map_zoom,
                        dx,
                        dy,
                    );
                    view.set_pin_from_map(reference, lat, lon, cx);
                }),
            )
            .on_scroll_wheel(cx.listener(move |view, event: &ScrollWheelEvent, _, cx| {
                let dy = match event.delta {
                    ScrollDelta::Lines(delta) => delta.y,
                    ScrollDelta::Pixels(delta) => f32::from(delta.y) / 40.,
                };
                if dy.abs() < 0.05 {
                    return;
                }
                if let Some(h) = view.draft_handles_mut(reference) {
                    if dy > 0. {
                        h.location.map_zoom = (h.location.map_zoom + 1).min(MAX_ZOOM);
                    } else {
                        h.location.map_zoom = h.location.map_zoom.saturating_sub(1).max(MIN_ZOOM);
                    }
                    cx.notify();
                }
                cx.stop_propagation();
            }))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, _, _| {
                        bounds_cell.set(Some(bounds));
                    },
                )
                .absolute()
                .size_full(),
            )
            .children(tile_elements)
            .when(pin_visible, |vp| {
                vp.child(
                    div()
                        .absolute()
                        .left(px(pin_x - 9.))
                        .top(px(pin_y - 9.))
                        .w(px(18.))
                        .h(px(18.))
                        .rounded(px(9.))
                        .border_2()
                        .border_color(rgb(INK))
                        .bg(rgb(GOLD))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().w(px(4.)).h(px(4.)).rounded(px(2.)).bg(rgb(PAPER))),
                )
            })
            .child(
                row()
                    .absolute()
                    .left(px(10.))
                    .top(px(10.))
                    .gap_1()
                    .child(map_ctrl_button(
                        (prefix, 10usize),
                        "−",
                        cx,
                        move |view, _| {
                            if let Some(h) = view.draft_handles_mut(reference) {
                                h.location.map_zoom =
                                    h.location.map_zoom.saturating_sub(1).max(MIN_ZOOM);
                            }
                        },
                    ))
                    .child(
                        div()
                            .px_2()
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .rounded(px(RADIUS_FIELD))
                            .border_1()
                            .border_color(rgb(LINE))
                            .bg(rgb(PAPER))
                            .font_family("Menlo")
                            .text_size(px(11.5))
                            .text_color(rgb(MUTED))
                            .child(format!("z{zoom}")),
                    )
                    .child(map_ctrl_button(
                        (prefix, 11usize),
                        "+",
                        cx,
                        move |view, _| {
                            if let Some(h) = view.draft_handles_mut(reference) {
                                h.location.map_zoom = (h.location.map_zoom + 1).min(MAX_ZOOM);
                            }
                        },
                    ))
                    .child(map_ctrl_button(
                        (prefix, 12usize),
                        "Center on pin",
                        cx,
                        move |view, _| {
                            if let Some(h) = view.draft_handles_mut(reference) {
                                h.location.map_center_lat = pin_lat;
                                h.location.map_center_lon = pin_lon;
                            }
                        },
                    )),
            )
            .child(
                div()
                    .absolute()
                    .right(px(8.))
                    .bottom(px(8.))
                    .px_2()
                    .py(px(2.))
                    .rounded(px(4.))
                    .bg(rgb(PAPER))
                    .border_1()
                    .border_color(rgb(LINE))
                    .text_size(px(10.5))
                    .text_color(rgb(MUTED))
                    .child("© OpenStreetMap contributors"),
            );

        panel()
            .bg(rgb(SOFT))
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .child(section_label("OPENSTREETMAP LOCATION PICKER"))
                    .child(caption(format!("Pin · {pin_lat:.4}°, {pin_lon:.4}°"))),
            )
            .child(map_viewport)
            .child(caption(
                "Click anywhere to drop pin · Drag to pan · Scroll or +/− to zoom",
            ))
    }

    fn render_timezone_panel(
        &self,
        reference: bool,
        effective_dt: NaiveDateTime,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(handles) = self.draft_handles(reference) else {
            return panel();
        };
        let prefix = if reference { "ref-tz" } else { "prof-tz" };
        let tz_query = handles.location.tz_query.clone();
        let current_tz = handles.timezone.read(cx).value().trim().to_owned();
        let query_text = tz_query.read(cx).value().to_owned();
        let matches = filter_timezones(&query_text, effective_dt, 12);

        panel()
            .bg(rgb(SOFT))
            .gap_2()
            .child(section_label("TIMEZONE DIRECTORY (IANA)"))
            .child(row().flex_wrap().gap_1().children(
                COMMON_TIMEZONES.into_iter().enumerate().map(|(idx, zone)| {
                    let active = current_tz == zone;
                    action_control((prefix, 200usize + idx), cx, move |view, cx| {
                        view.select_timezone_name(reference, zone, cx);
                    })
                    .px_2()
                    .py(px(4.))
                    .rounded(px(RADIUS_FIELD))
                    .border_1()
                    .border_color(rgb(if active { GOLD } else { LINE }))
                    .bg(rgb(if active { SELECTED } else { PAPER }))
                    .text_size(px(12.))
                    .text_color(rgb(INK))
                    .hover(|s| s.bg(rgb(SELECTED)))
                    .child(zone)
                }),
            ))
            .child(tz_query)
            .when(matches.is_empty(), |p| {
                p.child(caption("No matching IANA timezones."))
            })
            .when(!matches.is_empty(), |p| {
                p.child(
                    column()
                        .gap_1()
                        .children(matches.into_iter().enumerate().map(|(idx, opt)| {
                            let active = current_tz == opt.name;
                            action_control((prefix, idx), cx, move |view, cx| {
                                view.select_timezone_name(reference, opt.name, cx);
                            })
                            .w_full()
                            .px_3()
                            .py(px(6.))
                            .rounded(px(RADIUS_FIELD))
                            .border_1()
                            .border_color(rgb(if active { GOLD } else { SUBTLE_LINE }))
                            .bg(rgb(if active { SELECTED } else { PAPER }))
                            .hover(|s| s.bg(rgb(SELECTED)))
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(rgb(INK))
                                    .child(opt.name),
                            )
                            .child(
                                div()
                                    .font_family("Menlo")
                                    .text_size(px(12.))
                                    .text_color(rgb(MUTED))
                                    .child(format!("{} · {}", opt.offset_label, opt.abbr)),
                            )
                        })),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn mercator_round_trip_and_clamping() {
        for (lat, lon) in [
            (16.8409, 96.1735),   // Yangon
            (42.2495, -71.0662),  // Milton MA
            (-33.8688, 151.2093), // Sydney
            (51.5074, -0.1278),   // London
            (89.5, 179.9),        // Polar clamp
        ] {
            for zoom in [2u8, 8, 11, 15] {
                let (tx, ty) = lat_lon_to_tile_xy(lat, lon, zoom);
                let (rt_lat, rt_lon) = tile_xy_to_lat_lon(tx, ty, zoom);
                let expected_lat = lat.clamp(-MAX_MERCATOR_LAT, MAX_MERCATOR_LAT);
                assert!(
                    (rt_lat - expected_lat).abs() < 1e-6,
                    "lat mismatch at zoom {zoom}: {rt_lat} vs {expected_lat}"
                );
                assert!(
                    (rt_lon - lon).abs() < 1e-6,
                    "lon mismatch at zoom {zoom}: {rt_lon} vs {lon}"
                );
            }
        }
    }

    #[test]
    fn offline_timezone_lookup_for_known_cities() {
        assert_eq!(
            timezone_for_coords(16.8409, 96.1735).as_deref(),
            Some("Asia/Yangon")
        );
        assert_eq!(
            timezone_for_coords(42.2495, -71.0662).as_deref(),
            Some("America/New_York")
        );
        assert_eq!(
            timezone_for_coords(51.5074, -0.1278).as_deref(),
            Some("Europe/London")
        );
        assert_eq!(
            timezone_for_coords(35.6762, 139.6503).as_deref(),
            Some("Asia/Tokyo")
        );
    }

    #[test]
    fn filter_timezones_matches_name_and_offset() {
        let dt = NaiveDate::from_ymd_opt(2024, 6, 12)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
        let ny = filter_timezones("new york", dt, 10);
        assert!(ny.iter().any(|opt| opt.name == "America/New_York"));

        let yangon = filter_timezones("yangon", dt, 10);
        assert!(yangon.iter().any(|opt| opt.name == "Asia/Yangon"));

        let offset = filter_timezones("+06:30", dt, 20);
        assert!(offset.iter().any(|opt| opt.name == "Asia/Yangon"));
    }

    #[test]
    fn describe_timezone_at_formats_historical_and_fixed_offsets() {
        let dt = NaiveDate::from_ymd_opt(1924, 6, 12)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
        let ny = describe_timezone_at("America/New_York", dt).unwrap();
        assert!(ny.contains("UTC-04:00"), "unexpected summary: {ny}");

        let fixed = describe_timezone_at("390", dt).unwrap();
        assert!(fixed.contains("UTC+06:30"), "unexpected summary: {fixed}");

        assert!(describe_timezone_at("Invalid/Zone", dt).is_err());
    }

    #[test]
    fn percent_encode_handles_spaces_commas_and_unicode() {
        assert_eq!(
            percent_encode("Milton, Massachusetts"),
            "Milton%2C%20Massachusetts"
        );
        assert_eq!(
            percent_encode("ရန်ကုန်"),
            "%E1%80%9B%E1%80%94%E1%80%BA%E1%80%80%E1%80%AF%E1%80%94%E1%80%BA"
        );
    }
}
