mod advanced;
mod ashtakavarga;
mod backup;
mod chart;
mod chart_view;
mod datetime_picker;
mod input;
mod inspector;
mod interpretations;
mod location_picker;
mod model;
mod name_wheel;
mod orbit;
mod palette;
mod profiles;
mod recovered;
mod sarvatobhadra;
mod sky;
mod strength_timeline;
mod tara;
mod ten_pillars;
mod theme;
mod time_view;
mod transits;
mod varga;

use std::{
    io::Write,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use chrono::{NaiveDateTime, Utc};
use gpui::{
    AnyElement, App, Application, Bounds, ClickEvent, Context, Div, Entity, FocusHandle, Image,
    ImageFormat, KeyBinding, Window, WindowBounds, WindowOptions, actions, div, img, prelude::*,
    px, rgb, size,
};
use input::TextInput;
use model::{AYANAMSHAS, HOUSES, Settings, Step, Workspace, WorkspaceSession};
use motaa_engine::ChartRequest;
pub(crate) use theme::*;

const MODES: [&str; 12] = [
    "Rasi",
    "Tricakka",
    "Traditional aspects",
    "Nakshatra traversal",
    "LatLon",
    "Western aspects",
    "Natal + transit",
    "Dasha periods",
    "Circular chart",
    "Rahu wheel",
    "Rasi strengths",
    "Rasi aspects",
];
const TOOLS: [[&str; 4]; 4] = [
    [
        "Strength timeline",
        "Ashtakavarga",
        "KP subdivisions",
        "Heliocentric model",
    ],
    [
        "Divisional charts",
        "Electional timing",
        "Ephemeris calendar",
        "Eclipses",
    ],
    [
        "Daily Tatkalika",
        "Satapati name wheel",
        "Sarvatobhadra",
        "Yoga / candidate times",
    ],
    [
        "Interpretations",
        "Transit events",
        "Ten pillars / Ayu Moon",
        "Nakshatra groups",
    ],
];
const PALETTES: [(&str, u32); 4] = [
    ("Gold", GOLD),
    ("Pink", 0x9b486b),
    ("Blue", 0x315f9d),
    ("Monochrome", INK),
];

actions!(
    motaa,
    [
        Charts,
        Profiles,
        Calculations,
        TimeAnalysis,
        Advanced,
        TabNext,
        TabPrevious,
        TogglePalette,
        ToggleSidebar,
        ToggleInspector,
        StepTimeBack,
        StepTimeForward,
        JumpNow,
        NewProfile,
        Quit
    ]
);

type Callback = fn(&mut DesktopRoot, &mut Context<DesktopRoot>);

#[derive(serde::Deserialize)]
struct TownDirectory {
    towns: Vec<Town>,
}

#[derive(serde::Deserialize)]
struct Town {
    name_my: String,
    latitude: f64,
    longitude: f64,
}

struct ProfileDraft {
    copy_from: Option<i64>,
    original: ChartRequest,
    id: Option<i64>,
    name: Entity<TextInput>,
    datetime: Entity<TextInput>,
    picker: datetime_picker::DateTimePicker,
    place: Entity<TextInput>,
    latitude: Entity<TextInput>,
    longitude: Entity<TextInput>,
    timezone: Entity<TextInput>,
    unknown: bool,
    town_search: Option<Entity<TextInput>>,
    location: location_picker::LocationPickerState,
}

struct ReferenceLocationDraft {
    town_search: Option<Entity<TextInput>>,
    place: Entity<TextInput>,
    latitude: Entity<TextInput>,
    longitude: Entity<TextInput>,
    timezone: Entity<TextInput>,
    location: location_picker::LocationPickerState,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RahuReference {
    Node,
    Moon,
    Manual,
}

struct EphemerisMonth {
    title: String,
    rows: Vec<[String; 7]>,
    planets: Vec<[String; 8]>,
    settings: Settings,
    birthplace: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransitFilter {
    All,
    Ingress,
    Station,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Interpretation {
    Period,
    Planet,
    House,
    Nakshatra,
    Yoga,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LightboxTarget {
    ActiveChart,
    TricakkaSubchart(usize),
    RasiInset,
    ComparisonCombined,
    ComparisonSplit(usize),
}

struct DesktopRoot {
    session: Entity<WorkspaceSession>,
    focus: FocusHandle,
    sidebar_collapsed: bool,
    inspector_open: bool,
    palette_open: bool,
    palette_query: Entity<TextInput>,
    palette_index: usize,
    date: Entity<TextInput>,
    time_picker: datetime_picker::DateTimePicker,
    search: Entity<TextInput>,
    tool_search: Entity<TextInput>,
    draft: Option<ProfileDraft>,
    show_profile_editor: bool,
    reference_location_draft: Option<ReferenceLocationDraft>,
    selected: Option<i64>,
    delete_confirmation: Option<i64>,
    deleted_profile: Option<motaa_store::Profile>,
    pending_import: Option<Vec<backup::Record>>,
    reading_backup: bool,
    show_modes: bool,
    show_munp: bool,
    circular_detail: bool,
    split_comparison: bool,
    compact: bool,
    chart_palette: usize,
    show_birth: bool,
    chart_zoom: f32,
    chart_scroll: gpui::ScrollHandle,
    chart_drag: Option<(gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>)>,
    chart_press_origin: Option<gpui::Point<gpui::Pixels>>,
    chart_lightbox: Option<LightboxTarget>,
    lightbox_zoom: f32,
    lightbox_scroll: gpui::ScrollHandle,
    lightbox_drag: Option<(gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>)>,
    scrub_offset: i32,
    scrub_dragging: bool,
    dasha_major: Option<usize>,
    dasha_sub: Option<usize>,
    all_strengths: bool,
    rahu_28: bool,
    rahu_reference: RahuReference,
    rahu_manual_input: Entity<TextInput>,
    rahu_manual_angle: Option<f64>,
    sky_rotation: f64,
    tara_from_ascendant: bool,
    vedha_body: usize,
    vedha_transit: bool,
    vedha_abhijit: bool,
    selected_varga: varga::Varga,
    name_reference: Entity<TextInput>,
    name_matches: Vec<usize>,
    name_sector: Option<usize>,
    transit_report: Option<Result<transits::Report, String>>,
    transit_searching: bool,
    transit_filter: TransitFilter,
    eclipse_lunar: bool,
    eclipse_visible_only: bool,
    eclipse_selection: Option<f64>,
    strength_body: usize,
    strength_pending: bool,
    strength_report: Option<Result<strength_timeline::Timeline, String>>,
    candidate_inputs: [Entity<TextInput>; 3],
    candidate_pickers: [datetime_picker::DateTimePicker; 3],
    candidate_rows: Option<Vec<[String; 5]>>,
    interpretation: Interpretation,
    interpretation_body: usize,
    interpretation_bhava: bool,
    interpretation_transit: bool,
    satapati_lagna: Option<usize>,
    kp_horary: Option<usize>,
    strength_age: Option<u8>,
    ephemeris_month: Option<Result<EphemerisMonth, String>>,
    ephemeris_progress: bool,
    ephemeris_request_id: u64,
    towns: TownDirectory,
    tile_cache: std::collections::HashMap<(u8, u32, u32), location_picker::TileState>,
}

// Navigation geometry exported from the master’s Lucide v0.468.0 components.
const RAIL_ICONS: [(&str, &[u8]); 5] = [
    ("orbit", include_bytes!("../assets/rail/orbit.svg")),
    (
        "users-round",
        include_bytes!("../assets/rail/users-round.svg"),
    ),
    (
        "sliders-horizontal",
        include_bytes!("../assets/rail/sliders-horizontal.svg"),
    ),
    (
        "calendar-clock",
        include_bytes!("../assets/rail/calendar-clock.svg"),
    ),
    ("telescope", include_bytes!("../assets/rail/telescope.svg")),
];

fn app_icon() -> Arc<Image> {
    static ICON: OnceLock<Arc<Image>> = OnceLock::new();
    ICON.get_or_init(|| {
        Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../assets/app-icon-256.png").to_vec(),
        ))
    })
    .clone()
}

struct DesktopAssets;

impl gpui::AssetSource for DesktopAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(RAIL_ICONS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| std::borrow::Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(RAIL_ICONS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| (*name).into())
            .collect())
    }
}

fn rail_icon(index: usize, selected: bool) -> impl IntoElement {
    gpui::svg()
        .path(RAIL_ICONS[index].0)
        .size(px(16.))
        .text_color(rgb(if selected { GOLD } else { MUTED }))
}

fn field(label: &'static str, input: Entity<TextInput>) -> Div {
    column().gap_1().child(caption(label)).child(input)
}
fn text_input(value: &str, placeholder: &str, cx: &mut Context<DesktopRoot>) -> Entity<TextInput> {
    cx.new(|cx| TextInput::new(value, placeholder, cx))
}
fn button(
    label: impl Into<gpui::SharedString>,
    primary: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    let label = label.into();
    action_control(label.clone(), cx, action)
        .px_3()
        .h(px(38.))
        .justify_center()
        .rounded(px(6.))
        .border_1()
        .border_color(rgb(if primary { GOLD } else { LINE }))
        .bg(rgb(if primary { GOLD } else { PAPER }))
        .text_color(rgb(if primary { PAPER } else { INK }))
        .hover(|style| style.bg(rgb(if primary { ACTION_HOVER } else { SOFT })))
        .active(|style| style.bg(rgb(if primary { ACTION_PRESSED } else { SELECTED })))
        .focus(|style| {
            style
                .border_2()
                .border_color(rgb(if primary { INK } else { GOLD }))
        })
        .child(label)
        .into_any_element()
}

fn action_control(
    id: impl Into<gpui::ElementId>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    let action = Rc::new(action);
    let keyboard_action = action.clone();
    div()
        .id(id)
        .flex()
        .items_center()
        .cursor_pointer()
        .focusable()
        .tab_stop(true)
        .border_1()
        .border_color(gpui::transparent_black())
        .focus(|style| style.border_color(rgb(GOLD)))
        .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
            action(view, cx);
            cx.notify();
        }))
        .on_key_down(cx.listener(move |view, event: &gpui::KeyDownEvent, _, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                keyboard_action(view, cx);
                cx.stop_propagation();
                cx.notify();
            }
        }))
}

fn directory_link(
    label: &'static str,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control(label, cx, action)
        .justify_between()
        .px_2()
        .h(px(39.))
        .border_b_1()
        .border_color(rgb(0xeae5dc))
        .hover(|item| item.bg(rgb(SELECTED)))
        .child(label)
        .child(caption("→"))
}

impl DesktopRoot {
    fn new(store: Result<rusqlite::Connection, String>, cx: &mut Context<Self>) -> Self {
        let session = cx.new(|_| {
            let mut session = WorkspaceSession::new(store, Utc::now().naive_utc());
            session.time = session.now();
            session.anchor = session.time;
            session
        });
        let now = session.read(cx).time;
        let time_picker = datetime_picker::DateTimePicker::new("reference-dt", now, false, cx);
        let date = time_picker.datetime.clone();
        let candidate_pickers = [
            datetime_picker::DateTimePicker::new("candidate-1-dt", now, false, cx),
            datetime_picker::DateTimePicker::new("candidate-2-dt", now, false, cx),
            datetime_picker::DateTimePicker::new("candidate-3-dt", now, false, cx),
        ];
        let candidate_inputs = [
            candidate_pickers[0].datetime.clone(),
            candidate_pickers[1].datetime.clone(),
            candidate_pickers[2].datetime.clone(),
        ];
        let search = text_input("", "Search by name or birthplace", cx);
        let tool_search = text_input("", "Search chart modes and advanced tools", cx);
        let palette_query = text_input("", "Jump to chart mode, tool, profile, or action…", cx);
        cx.observe(&tool_search, |_, _, cx| cx.notify()).detach();
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        cx.observe(&palette_query, |view, _, cx| {
            view.palette_index = 0;
            cx.notify();
        })
        .detach();
        Self {
            session,
            focus: cx.focus_handle(),
            sidebar_collapsed: false,
            inspector_open: true,
            palette_open: false,
            palette_query,
            palette_index: 0,
            date,
            time_picker,
            search,
            tool_search,
            draft: None,
            show_profile_editor: false,
            reference_location_draft: None,
            selected: None,
            delete_confirmation: None,
            deleted_profile: None,
            pending_import: None,
            reading_backup: false,
            show_modes: false,
            show_munp: false,
            circular_detail: false,
            split_comparison: false,
            compact: false,
            chart_palette: 0,
            show_birth: false,
            chart_zoom: 1.,
            chart_scroll: gpui::ScrollHandle::new(),
            chart_drag: None,
            chart_press_origin: None,
            chart_lightbox: None,
            lightbox_zoom: 1.,
            lightbox_scroll: gpui::ScrollHandle::new(),
            lightbox_drag: None,
            scrub_offset: 0,
            scrub_dragging: false,
            dasha_major: None,
            dasha_sub: None,
            all_strengths: false,
            rahu_28: false,
            rahu_reference: RahuReference::Node,
            rahu_manual_input: text_input("", "Longitude 0° to <360°", cx),
            rahu_manual_angle: None,
            sky_rotation: 0.,
            tara_from_ascendant: false,
            vedha_body: 2,
            vedha_transit: true,
            vedha_abhijit: true,
            selected_varga: varga::Varga::Navamsa,
            name_reference: text_input("", "Myanmar syllable", cx),
            name_matches: Vec::new(),
            name_sector: None,
            transit_report: None,
            transit_searching: false,
            transit_filter: TransitFilter::All,
            eclipse_lunar: false,
            eclipse_visible_only: false,
            eclipse_selection: None,
            strength_body: 2,
            strength_pending: false,
            strength_report: None,
            candidate_inputs,
            candidate_pickers,
            candidate_rows: None,
            interpretation: Interpretation::Period,
            interpretation_body: 2,
            interpretation_bhava: false,
            interpretation_transit: false,
            satapati_lagna: None,
            kp_horary: None,
            strength_age: None,
            ephemeris_month: None,
            ephemeris_progress: false,
            ephemeris_request_id: 0,
            towns: serde_json::from_str(include_str!("../design/source/locations.v31.1.json"))
                .expect("bundled MOTAA town directory"),
            tile_cache: std::collections::HashMap::new(),
        }
    }

    fn update(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut WorkspaceSession)) {
        self.session.update(cx, |session, cx| {
            f(session);
            cx.notify();
        });
    }

    fn navigate(&mut self, route: Workspace, cx: &mut Context<Self>) {
        let was_editing_profile = std::mem::take(&mut self.show_profile_editor);
        self.delete_confirmation = None;
        self.chart_lightbox = None;
        self.update(cx, |session| {
            if session.workspace != route || was_editing_profile {
                session.status.clear();
            }
            session.workspace = route;
        });
    }

    fn resume_profile_draft(&mut self, cx: &mut Context<Self>) {
        self.navigate(Workspace::Profiles, cx);
        self.show_profile_editor = true;
    }

    fn sync_datetime_pickers(&mut self, cx: &mut Context<Self>) {
        if let Some(draft) = &mut self.draft {
            draft.picker.sync_from_inputs(cx);
        }
        self.time_picker.sync_from_inputs(cx);
        for picker in &mut self.candidate_pickers {
            picker.sync_from_inputs(cx);
        }
    }

    fn open_draft(&mut self, request: ChartRequest, id: Option<i64>, cx: &mut Context<Self>) {
        if self.draft.is_some() {
            self.navigate(Workspace::Profiles, cx);
            self.set_status("Save or discard the open draft first.", cx);
            return;
        }
        let timestamp = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            request.year, request.month, request.day, request.hour, request.minute, request.second
        );
        let initial_dt = parse_time(&timestamp).unwrap_or_else(|_| {
            chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        });
        let picker = datetime_picker::DateTimePicker::new("profile-dt", initial_dt, true, cx);
        let datetime = picker.datetime.clone();
        self.draft = Some(ProfileDraft {
            copy_from: None,
            name: text_input(&request.name, "Name", cx),
            datetime,
            picker,
            place: text_input(&request.birthplace, "Birthplace", cx),
            latitude: text_input(&request.latitude.to_string(), "Latitude", cx),
            longitude: text_input(&request.longitude.to_string(), "Longitude", cx),
            timezone: text_input(
                &request
                    .timezone_id
                    .clone()
                    .unwrap_or_else(|| request.timezone_offset_minutes.to_string()),
                "IANA zone or offset minutes",
                cx,
            ),
            unknown: request.unknown_birth_time,
            town_search: None,
            location: location_picker::LocationPickerState::new(
                request.latitude,
                request.longitude,
                cx,
            ),
            original: request,
            id,
        });
        self.resume_profile_draft(cx);
    }

    fn set_status(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.update(cx, |session| session.status = message.into());
    }

    fn form_request(&self, cx: &App) -> Result<ChartRequest, String> {
        use chrono::{Datelike, Timelike};
        let draft = self.draft.as_ref().ok_or("Open a profile draft first.")?;
        draft.picker.validate()?;
        let mut request = draft.original.clone();
        request.name = draft.name.read(cx).value().trim().into();
        request.birthplace = draft.place.read(cx).value().trim().into();
        let timestamp = draft.datetime.read(cx).value().trim();
        let timestamp = if draft.unknown && timestamp.len() == 10 {
            format!("{timestamp} 00:00:00")
        } else {
            timestamp.to_owned()
        };
        let time = parse_time(&timestamp)?;
        request.year = time.year();
        request.month = time.month() as u8;
        request.day = time.day() as u8;
        request.hour = time.hour() as u8;
        request.minute = time.minute() as u8;
        request.second = time.second() as u8;
        request.latitude = parse_coordinate(draft.latitude.read(cx).value(), true)?;
        request.longitude = parse_coordinate(draft.longitude.read(cx).value(), false)?;
        set_timezone(&mut request, draft.timezone.read(cx).value());
        request.unknown_birth_time = draft.unknown;
        motaa_engine::validate(&request)?;
        motaa_engine::effective_utc_offset(&request)?;
        Ok(request)
    }

    fn save_draft(&mut self, cx: &mut Context<Self>) {
        let result = (|| {
            let request = self.form_request(cx)?;
            let draft = self.draft.as_ref().ok_or("Open a profile draft first.")?;
            let session = self.session.read(cx);
            let profile = save_profile_with_source(session, draft.id, &request, draft.copy_from)?;
            let reference = session.profile_reference_location(profile.id)?;
            let settings = session.profile_settings(profile.id)?;
            Ok::<_, String>((profile.id, request, reference, settings))
        })();
        match result {
            Ok((id, request, reference, settings)) => {
                self.draft = None;
                self.show_profile_editor = false;
                self.selected = Some(id);
                self.update(cx, move |s| {
                    if s.natal_id == Some(id) {
                        s.natal = request;
                    } else {
                        s.activate_natal(id, request, reference);
                        s.settings = settings;
                        s.settings_draft = settings;
                    }
                    s.status = "Profile saved.".into();
                });
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn select_mode(&mut self, mode: &'static str, cx: &mut Context<Self>) {
        self.show_modes = false;
        self.circular_detail = false;
        self.chart_lightbox = None;
        self.chart_zoom = 1.;
        self.chart_scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.chart_drag = None;
        self.chart_press_origin = None;
        self.update(cx, |session| {
            session.mode = mode;
            session.workspace = Workspace::Charts;
        });
    }

    fn set_time(&mut self, time: NaiveDateTime, reset_anchor: bool, cx: &mut Context<Self>) {
        self.time_picker.set_datetime(time, cx);
        if reset_anchor {
            self.scrub_offset = 0;
        }
        self.update(cx, |session| {
            session.time = time;
            if reset_anchor {
                session.anchor = time;
            }
            session.status.clear();
        });
    }

    fn step_time(&mut self, count: i32, scrub: bool, cx: &mut Context<Self>) {
        let s = self.session.read(cx);
        let base = if scrub { s.now() } else { s.time };
        match s.step.offset(base, count) {
            Some(time) => {
                if scrub {
                    self.scrub_offset = count;
                    self.update(cx, |session| session.anchor = base);
                }
                self.set_time(time, !scrub, cx);
            }
            None => self.set_status("The requested time is outside the supported calendar.", cx),
        }
    }

    fn mode_buttons(&self, cx: &mut Context<Self>) -> Div {
        let active = self.session.read(cx).mode;
        div().grid().grid_cols(3).gap_2().children(
            MODES
                .into_iter()
                .filter(|mode| {
                    !matches!(
                        *mode,
                        "Rasi"
                            | "Tricakka"
                            | "Circular chart"
                            | "Dasha periods"
                            | "Natal + transit"
                            | "Rahu wheel"
                    )
                })
                .map(|mode| {
                    button(mode, active == mode, cx, move |view, cx| {
                        view.select_mode(mode, cx)
                    })
                }),
        )
    }

    fn profile_edit_save_buttons(&self, cx: &mut Context<Self>) -> Div {
        row()
            .gap_2()
            .child(button("Edit", false, cx, |view, cx| {
                let s = view.session.read(cx);
                view.open_draft(s.natal.clone(), s.natal_id, cx);
            }))
            .child(button("Save", true, cx, |view, cx| {
                let s = view.session.read(cx);
                match save_profile(s, s.natal_id, &s.natal) {
                    Ok(profile) => view.update(cx, |s| {
                        s.natal_id = Some(profile.id);
                        s.status = "Profile saved.".into();
                    }),
                    Err(error) => view.set_status(error, cx),
                }
            }))
    }

    fn calculations_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let draft = self.session.read(cx).settings_draft;
        column()
            .child(heading("Calculation settings"))
            .child(caption(
                "Review options before applying them to this workspace.",
            ))
            .child(settings_strip(self.session.read(cx).settings))
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_6().pt_5()
                    .child(column().child("Zodiac").child(
                        row().children(["Sidereal", "Tropical"].into_iter().enumerate().map(
                            |(index, name)| {
                                button(name, draft.tropical == (index == 1), cx, move |view, cx| {
                                    view.update(cx, |s| s.settings_draft.tropical = index == 1)
                                })
                            },
                        )),
                    ))
                    .child(column().child("Lunar node").child(
                        row().children(["True Rahu", "Mean Rahu"].into_iter().enumerate().map(
                            |(index, name)| {
                                button(
                                    name,
                                    draft.mean_node == (index == 1),
                                    cx,
                                    move |view, cx| {
                                        view.update(cx, |s| s.settings_draft.mean_node = index == 1)
                                    },
                                )
                            },
                        )),
                    ))
                    .child(column().child("Ayanamsha").child(
                        div().grid().grid_cols(3).gap_2().children(
                            AYANAMSHAS.into_iter().enumerate().map(|(index, name)| {
                                button(name, draft.ayanamsha == index, cx, move |view, cx| {
                                    view.update(cx, |s| s.settings_draft.ayanamsha = index)
                                })
                            }),
                        ),
                    ))
                    .child(column().child("House system").child(
                        div().grid().grid_cols(2).gap_2().children(
                            HOUSES.into_iter().enumerate().map(|(index, name)| {
                                button(name, draft.houses == index, cx, move |view, cx| {
                                    view.update(cx, |s| s.settings_draft.houses = index)
                                })
                            }),
                        ),
                    )),
            )
            .child(div().grid().grid_cols(2).gap_4()
                .child(panel().child("Myanmar calendar convention")
                    .child(row().children(motaa_engine::myanmar_calendar::CivilCalendar::OPTIONS.map(|calendar| {
                        button(calendar.label(), draft.calendar == calendar, cx, move |view, cx| {
                            view.update(cx, |s| s.settings_draft.calendar = calendar);
                        })
                    }))))
                .child(panel().child("Original aspect display preference")
                    .child(caption("Stored with the profile. The recovered aspect formulas use fixed orbs."))
                    .child(row().children([("5D Aspect", false), ("15D Aspect", true)].map(|(label, choice)| {
                        button(label, draft.fifteen_degree_display == choice, cx, move |view, cx| {
                            view.update(cx, |s| s.settings_draft.fifteen_degree_display = choice);
                        })
                    })))))
            .child(
                row()
                    .child(button("Restore defaults", false, cx, |view, cx| {
                        view.update(cx, |s| s.settings_draft = Settings::default())
                    }))
                    .child(button("Apply to workspace", true, cx, |view, cx| {
                        view.update(cx, |s| {
                            s.settings = s.settings_draft;
                            s.status = "Workspace settings applied.".into();
                        })
                    })),
            )
            .when(self.session.read(cx).natal_id.is_some(), |view| {
                view.child(button("Save options to profile", false, cx, |view, cx| {
                    let session = view.session.read(cx);
                    let Some(id) = session.natal_id else {
                        return;
                    };
                    let draft = session.settings_draft;
                    let result = session.connection().and_then(|db| {
                        let value =
                            serde_json::to_string(&draft).map_err(|error| error.to_string())?;
                        motaa_store::put_setting(db, &format!("profile_calculations_{id}"), &value)
                    });
                    match result {
                        Ok(()) => view.update(cx, |s| {
                            s.settings = draft;
                            s.status = "Calculation options saved to profile.".into();
                        }),
                        Err(error) => view.set_status(error, cx),
                    }
                }))
            })
            .child(panel().child("Chart palette").child(row().children(
                PALETTES.into_iter().enumerate().map(|(index, (name, _))| {
                    button(name, self.chart_palette == index, cx, move |view, _| {
                        view.chart_palette = index
                    })
                }),
            )))
            .into_any_element()
    }
}

impl Render for DesktopRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session_ref = self.session.read(cx);
        let route = session_ref.workspace;
        let status = session_ref.status.clone();
        let active_natal = session_ref.natal.name.clone();
        let ref_time_label = session_ref.time.format("%d %b %Y · %H:%M").to_string();
        let settings = session_ref.settings;
        let ayanamsha_label = if settings.ayanamsha == 0 {
            "Lahiri"
        } else {
            AYANAMSHAS[settings.ayanamsha]
        };
        let houses_label = if settings.houses == 0 {
            "Equal"
        } else {
            HOUSES[settings.houses]
        };
        let zodiac_label = if settings.tropical {
            "Tropical"
        } else {
            "Sidereal"
        };
        let collapsed = self.sidebar_collapsed;
        let sidebar = column()
            .gap_1()
            .w(px(if collapsed { 64. } else { 208. }))
            .flex_shrink_0()
            .h_full()
            .bg(rgb(RAIL))
            .px(px(if collapsed { 8. } else { 12. }))
            .pb_3()
            .pt(px(if cfg!(target_os = "macos") { 56. } else { 16. }))
            .border_r_1()
            .border_color(rgb(SUBTLE_LINE))
            .child(
                div()
                    .flex()
                    .items_center()
                    .when(collapsed, |d| d.justify_center())
                    .gap_2()
                    .px_1()
                    .child(img(app_icon()).w(px(34.)).h(px(34.)).flex_shrink_0())
                    .when(!collapsed, |brand| {
                        brand.child(
                            column()
                                .gap_0()
                                .child(
                                    div()
                                        .text_size(px(22.))
                                        .line_height(px(24.))
                                        .font_family("Baskerville")
                                        .child("MOTAA"),
                                )
                                .child(caption("Myanmar astrology")),
                        )
                    }),
            )
            .child(div().h(px(14.)))
            .child(button(
                if collapsed { "+" } else { "New birth chart" },
                true,
                cx,
                |view, cx| view.open_draft(ChartRequest::default(), None, cx),
            ))
            .child(div().h(px(10.)))
            .children(Workspace::ALL.into_iter().enumerate().map(|(idx, entry)| {
                let selected = entry == route;
                let item = action_control(("workspace", entry as usize), cx, move |view, cx| {
                    view.navigate(entry, cx)
                })
                .relative()
                .px(px(if collapsed { 8. } else { 12. }))
                .h(px(38.))
                .gap_2()
                .rounded(px(RADIUS_NAV))
                .bg(rgb(if selected { SELECTED } else { RAIL }))
                .hover(|style| style.bg(rgb(if selected { SELECTED } else { SURFACE_HOVER })))
                .focus(|style| style.border_2().border_color(rgb(GOLD)))
                .when(selected && !collapsed, |row_el| {
                    row_el.child(
                        div()
                            .absolute()
                            .left(px(3.))
                            .w(px(3.))
                            .h(px(18.))
                            .rounded(px(2.))
                            .bg(rgb(GOLD)),
                    )
                })
                .when(collapsed, |row_el| row_el.justify_center())
                .child(rail_icon(entry as usize, selected));
                if collapsed {
                    item
                } else {
                    item.child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .whitespace_nowrap()
                            .text_size(px(13.5))
                            .font_weight(if selected {
                                gpui::FontWeight::MEDIUM
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .child(entry.label()),
                    )
                    .child(caption(shortcut_label(&(idx + 1).to_string())))
                }
            }))
            .child(div().flex_1())
            .child(
                div()
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        action_control("sidebar-collapse-btn", cx, |view, _| {
                            view.sidebar_collapsed = !view.sidebar_collapsed;
                        })
                        .w_full()
                        .h(px(32.))
                        .px_2()
                        .rounded(px(RADIUS_CONTROL))
                        .justify_center()
                        .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                        .child(caption(if collapsed {
                            "»".to_owned()
                        } else {
                            format!("« Collapse {}", shortcut_label("B"))
                        })),
                    ),
            );
        let lightbox_open = self.chart_lightbox.is_some();
        let viewport_w = f32::from(window.viewport_size().width);
        let show_inspector = self.inspector_open
            && !lightbox_open
            && viewport_w >= 1060.
            && matches!(
                route,
                Workspace::Charts | Workspace::Time | Workspace::Advanced
            );
        let footer = (!lightbox_open && route == Workspace::Charts).then(|| self.time_controls(cx));
        let content = if lightbox_open {
            div().into_any_element()
        } else {
            match route {
                Workspace::Charts => self.chart_view(window, cx),
                Workspace::Profiles => self.profiles_view(cx),
                Workspace::Calculations => self.calculations_view(cx),
                Workspace::Time => self.time_view(cx),
                Workspace::Advanced => self.advanced_view(cx),
            }
        };
        div()
            .relative()
            .flex()
            .size_full()
            .track_focus(&self.focus)
            .text_color(rgb(INK))
            .bg(rgb(IVORY))
            .text_size(px(14.))
            .font_family("Helvetica Neue")
            .on_key_down(cx.listener(|view, event: &gpui::KeyDownEvent, _, cx| {
                if view.chart_lightbox.is_none() {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "escape" => {
                        view.close_chart_lightbox(cx);
                        cx.stop_propagation();
                    }
                    "=" | "+" => {
                        view.set_lightbox_zoom(view.lightbox_zoom + 0.25, cx);
                        cx.stop_propagation();
                    }
                    "-" => {
                        view.set_lightbox_zoom(view.lightbox_zoom - 0.25, cx);
                        cx.stop_propagation();
                    }
                    "0" => {
                        view.set_lightbox_zoom(1., cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .on_action(cx.listener(|view, _: &Charts, _, cx| view.navigate(Workspace::Charts, cx)))
            .on_action(
                cx.listener(|view, _: &Profiles, _, cx| view.navigate(Workspace::Profiles, cx)),
            )
            .on_action(cx.listener(|view, _: &Calculations, _, cx| {
                view.navigate(Workspace::Calculations, cx)
            }))
            .on_action(
                cx.listener(|view, _: &TimeAnalysis, _, cx| view.navigate(Workspace::Time, cx)),
            )
            .on_action(
                cx.listener(|view, _: &Advanced, _, cx| view.navigate(Workspace::Advanced, cx)),
            )
            .on_action(cx.listener(|view, _: &TogglePalette, window, cx| {
                if view.palette_open {
                    view.close_palette(window, cx);
                } else {
                    view.open_palette(window, cx);
                }
            }))
            .on_action(cx.listener(|view, _: &ToggleSidebar, _, cx| {
                view.sidebar_collapsed = !view.sidebar_collapsed;
                cx.notify();
            }))
            .on_action(cx.listener(|view, _: &ToggleInspector, _, cx| {
                view.inspector_open = !view.inspector_open;
                cx.notify();
            }))
            .on_action(cx.listener(|view, _: &StepTimeBack, _, cx| {
                view.step_time(-1, false, cx);
            }))
            .on_action(cx.listener(|view, _: &StepTimeForward, _, cx| {
                view.step_time(1, false, cx);
            }))
            .on_action(cx.listener(|view, _: &JumpNow, _, cx| {
                view.set_time(view.session.read(cx).now(), true, cx);
            }))
            .on_action(cx.listener(|view, _: &NewProfile, _, cx| {
                view.open_draft(ChartRequest::default(), None, cx);
            }))
            .on_action(cx.listener(|view, _: &TabNext, window, _| {
                if !view.palette_open {
                    window.focus_next();
                }
            }))
            .on_action(cx.listener(|view, _: &TabPrevious, window, _| {
                if !view.palette_open {
                    window.focus_prev();
                }
            }))
            .on_action(|_: &Quit, _, cx| cx.quit())
            .child(sidebar)
            .child(
                column()
                    .gap_0()
                    .flex_1()
                    .min_w(px(0.))
                    .h_full()
                    .child(
                        row()
                            .h(px(52.))
                            .flex_shrink_0()
                            .px_6()
                            .gap_3()
                            .border_b_1()
                            .border_color(rgb(LINE))
                            .justify_between()
                            .bg(rgb(HEADER_BG))
                            .child(
                                row()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .h_full()
                                    .gap_2()
                                    .overflow_hidden()
                                    .window_control_area(gpui::WindowControlArea::Drag)
                                    .on_mouse_down(gpui::MouseButton::Left, |event, window, _| {
                                        if event.click_count == 1 {
                                            window.start_window_move();
                                        }
                                    })
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(INK))
                                            .child(route.label()),
                                    )
                                    .child(caption("·"))
                                    .child(
                                        div()
                                            .text_color(rgb(GOLD))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(active_natal),
                                    )
                                    .child(caption("·"))
                                    .child(
                                        div()
                                            .font_family("Menlo")
                                            .text_size(px(11.5))
                                            .text_color(rgb(MUTED))
                                            .child(ref_time_label),
                                    ),
                            )
                            .child(
                                row()
                                    .flex_shrink_0()
                                    .gap_2()
                                    .when(viewport_w >= 1120., |bar| {
                                        bar.child(chip(
                                            format!(
                                                "{zodiac_label} · {ayanamsha_label} · {houses_label}"
                                            ),
                                            false,
                                        ))
                                    })
                                    .child(
                                        action_control("header-quick-jump", cx, |_, _| {})
                                            .on_click(cx.listener(|view, _, window, cx| {
                                                view.open_palette(window, cx);
                                            }))
                                            .on_key_down(cx.listener(
                                                |view, event: &gpui::KeyDownEvent, window, cx| {
                                                    if matches!(
                                                        event.keystroke.key.as_str(),
                                                        "enter" | "space"
                                                    ) {
                                                        view.open_palette(window, cx);
                                                        cx.stop_propagation();
                                                    }
                                                },
                                            ))
                                            .h(px(30.))
                                            .px_3()
                                            .gap_2()
                                            .rounded(px(RADIUS_CONTROL))
                                            .border_1()
                                            .border_color(rgb(LINE))
                                            .bg(rgb(PAPER))
                                            .hover(|s| s.bg(rgb(SOFT)))
                                            .child(caption("Quick jump"))
                                            .child(badge(shortcut_label("K"), Tone::Neutral)),
                                    )
                                    .when(
                                        viewport_w >= 1060.
                                            && matches!(
                                                route,
                                                Workspace::Charts
                                                    | Workspace::Time
                                                    | Workspace::Advanced
                                            ),
                                        |bar| {
                                            bar.child(button(
                                                if self.inspector_open {
                                                    "Hide inspector".to_owned()
                                                } else {
                                                    format!("Inspector {}", shortcut_label("I"))
                                                },
                                                self.inspector_open,
                                                cx,
                                                |view, _| {
                                                    view.inspector_open = !view.inspector_open;
                                                },
                                            ))
                                        },
                                    )
                                    .when(route == Workspace::Profiles, |bar| {
                                        bar.child(button(
                                            if self.show_birth {
                                                "Hide birth details"
                                            } else {
                                                "Show birth details"
                                            },
                                            false,
                                            cx,
                                            |view, _| view.show_birth = !view.show_birth,
                                        ))
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.))
                            .w_full()
                            .flex()
                            .child(
                                div()
                                    .id(
                                        if route == Workspace::Profiles && self.show_profile_editor
                                        {
                                            ("profile-editor", 0)
                                        } else {
                                            ("workspace-content", route as usize)
                                        },
                                    )
                                    .overflow_y_scroll()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .px_6()
                                    .py_4()
                                    .child(content),
                            )
                            .when(show_inspector, |body| {
                                body.child(self.right_inspector(cx))
                            }),
                    )
                    .when(!status.is_empty() && !lightbox_open, |view| {
                        view.child(
                            div()
                                .flex_shrink_0()
                                .px_6()
                                .py_2()
                                .border_t_1()
                                .border_color(rgb(LINE))
                                .bg(rgb(SOFT))
                                .child(caption(status)),
                        )
                    })
                    .when_some(footer, |view, controls| {
                        view.child(
                            div()
                                .flex_shrink_0()
                                .bg(rgb(PAPER))
                                .px_6()
                                .h(px(64.))
                                .child(controls),
                        )
                    }),
            )
            .when_some(self.chart_lightbox, |root, target| {
                root.child(self.chart_lightbox_overlay(target, window, cx))
            })
            .when(self.palette_open, |root| {
                root.child(self.palette_overlay(cx))
            })
    }
}

fn mode_title(mode: &str) -> &'static str {
    if mode == "Circular chart" {
        "Circular chart"
    } else {
        "Rasi chart"
    }
}

fn data_row<const N: usize>(values: [String; N]) -> Div {
    row()
        .w_full()
        .gap_3()
        .px_2()
        .py(px(6.))
        .border_b_1()
        .border_color(rgb(LINE))
        .children(
            values
                .into_iter()
                .map(|value| div().flex_1().min_w(px(0.)).overflow_hidden().child(value)),
        )
}

fn settings_strip(settings: Settings) -> Div {
    row()
        .gap_12()
        .px_3()
        .py_2()
        .bg(rgb(0xf6f1e8))
        .rounded(px(5.))
        .children(
            [
                if settings.tropical {
                    "Tropical"
                } else {
                    "Sidereal"
                },
                if settings.ayanamsha == 0 {
                    "LAHIRI"
                } else {
                    AYANAMSHAS[settings.ayanamsha]
                },
                if settings.houses == 0 {
                    "Equal houses"
                } else {
                    HOUSES[settings.houses]
                },
                if settings.mean_node {
                    "Mean Rahu"
                } else {
                    "True Rahu"
                },
            ]
            .map(caption),
        )
}

fn format_time(time: NaiveDateTime) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
fn parse_time(text: &str) -> Result<NaiveDateTime, String> {
    datetime_picker::parse_flexible_datetime(text, false)
}

fn set_timezone(request: &mut ChartRequest, value: &str) {
    let value = value.trim();
    match value.parse() {
        Ok(minutes) => {
            request.timezone_offset_minutes = minutes;
            request.timezone_id = None;
        }
        Err(_) => request.timezone_id = Some(value.into()),
    }
}

fn parse_coordinate(input: &str, latitude: bool) -> Result<f64, String> {
    let label = if latitude { "Latitude" } else { "Longitude" };
    let max = if latitude { 90. } else { 180. };
    let value = if let Ok(value) = input.trim().parse::<f64>() {
        value
    } else {
        let parts = input.split_whitespace().collect::<Vec<_>>();
        let [degrees, minutes, seconds, hemisphere] = parts.as_slice() else {
            return Err(format!(
                "{label} needs decimal degrees or D M S and a hemisphere."
            ));
        };
        let degrees: u32 = degrees
            .parse()
            .map_err(|_| format!("Invalid {label} degrees."))?;
        let minutes: u32 = minutes
            .parse()
            .map_err(|_| format!("Invalid {label} minutes."))?;
        let seconds: f64 = seconds
            .parse()
            .map_err(|_| format!("Invalid {label} seconds."))?;
        let sign = match (latitude, hemisphere.to_ascii_uppercase().as_str()) {
            (true, "N") | (false, "E") => 1.,
            (true, "S") | (false, "W") => -1.,
            _ => return Err(format!("Invalid {label} hemisphere.")),
        };
        if minutes >= 60 || !(0. ..60.).contains(&seconds) {
            return Err(format!("Invalid {label} minutes or seconds."));
        }
        sign * (f64::from(degrees) + f64::from(minutes) / 60. + seconds / 3600.)
    };
    if !(-max..=max).contains(&value) {
        return Err(format!("{label} is outside ±{max}°."));
    }
    Ok(value)
}

fn save_profile(
    session: &WorkspaceSession,
    id: Option<i64>,
    request: &ChartRequest,
) -> Result<motaa_store::Profile, String> {
    save_profile_with_source(session, id, request, None)
}

fn save_profile_with_source(
    session: &WorkspaceSession,
    id: Option<i64>,
    request: &ChartRequest,
    copy_from: Option<i64>,
) -> Result<motaa_store::Profile, String> {
    let data = serde_json::to_string(request).map_err(|error| error.to_string())?;
    let transaction = session
        .connection()?
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let profile = motaa_store::save(&transaction, id, &request.name, &request.birthplace, &data)?;
    if id.is_none() {
        let settings = if let Some(source) = copy_from {
            session.profile_settings(source)?
        } else {
            session.settings
        };
        let value = serde_json::to_string(&settings).map_err(|error| error.to_string())?;
        motaa_store::put_setting(
            &transaction,
            &format!("profile_calculations_{}", profile.id),
            &value,
        )?;
        if let Some(source) = copy_from
            && let Some(reference) = session.profile_reference_location(source)?
        {
            let value = serde_json::to_string(&reference).map_err(|error| error.to_string())?;
            motaa_store::put_setting(
                &transaction,
                &format!("profile_reference_location_{}", profile.id),
                &value,
            )?;
        }
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(profile)
}

fn profile_database_path() -> Result<PathBuf, String> {
    let env_path = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| format!("{name} is unavailable"))
    };
    #[cfg(target_os = "macos")]
    let directory = env_path("HOME")?.join("Library/Application Support/MOTAA Desktop GPUI");
    #[cfg(target_os = "windows")]
    let directory = env_path("LOCALAPPDATA")?.join("MOTAA Desktop");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let directory = env_path("XDG_DATA_HOME")
        .ok()
        .filter(|path| path.is_absolute())
        .map_or_else(
            || env_path("HOME").map(|home| home.join(".local/share")),
            Ok,
        )?
        .join("motaa-desktop");
    Ok(directory.join("profiles.sqlite3"))
}

fn parse_store_screenshots_dir() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--store-screenshots" {
            return Some(
                args.next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
            );
        }
        if let Some(dir) = arg.strip_prefix("--store-screenshots=") {
            return Some(PathBuf::from(dir));
        }
    }
    None
}

fn main() {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let preview = std::env::args().any(|arg| arg == "--preview");
    let visual_capture = std::env::args().any(|arg| arg == "--visual-capture");
    let store_screenshots = parse_store_screenshots_dir();
    let store = if smoke || preview || visual_capture || store_screenshots.is_some() {
        rusqlite::Connection::open_in_memory()
            .map_err(|e| e.to_string())
            .and_then(|db| {
                motaa_store::migrate(&db)?;
                Ok(db)
            })
    } else {
        profile_database_path().and_then(|path| motaa_store::open(&path))
    };
    Application::new()
        .with_assets(DesktopAssets)
        .run(move |cx: &mut App| {
            input::bind_keys(cx);
            cx.bind_keys([
                KeyBinding::new(&primary_keybinding("1"), Charts, None),
                KeyBinding::new(&primary_keybinding("2"), Profiles, None),
                KeyBinding::new(&primary_keybinding("3"), Calculations, None),
                KeyBinding::new(&primary_keybinding("4"), TimeAnalysis, None),
                KeyBinding::new(&primary_keybinding("5"), Advanced, None),
                KeyBinding::new(&primary_keybinding("k"), TogglePalette, None),
                KeyBinding::new(&primary_keybinding("b"), ToggleSidebar, None),
                KeyBinding::new(&primary_keybinding("i"), ToggleInspector, None),
                KeyBinding::new(&primary_keybinding("["), StepTimeBack, None),
                KeyBinding::new(&primary_keybinding("]"), StepTimeForward, None),
                KeyBinding::new(&primary_keybinding("shift-n"), JumpNow, None),
                KeyBinding::new(&primary_keybinding("n"), NewProfile, None),
                KeyBinding::new("tab", TabNext, None),
                KeyBinding::new("shift-tab", TabPrevious, None),
                KeyBinding::new(&primary_keybinding("q"), Quit, None),
            ]);
            let window_size = if store_screenshots.is_some() {
                size(px(1440.), px(900.))
            } else {
                size(px(1280.), px(800.))
            };
            let bounds = Bounds::centered(None, window_size, cx);
            #[cfg(target_os = "macos")]
            let titlebar = Some(gpui::TitlebarOptions {
                title: Some("MOTAA Desktop".into()),
                appears_transparent: true,
                traffic_light_position: Some(gpui::point(px(20.), px(19.))),
            });
            #[cfg(not(target_os = "macos"))]
            let titlebar = Some(gpui::TitlebarOptions {
                title: Some("MOTAA Desktop".into()),
                appears_transparent: false,
                traffic_light_position: None,
            });
            let window = cx
                .open_window(
                    WindowOptions {
                        show: !smoke,
                        focus: !smoke,
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        window_min_size: Some(size(px(1024.), px(680.))),
                        titlebar,
                        ..Default::default()
                    },
                    |window, cx| {
                        let root = cx.new(|cx| DesktopRoot::new(store, cx));
                        window.focus(&root.read(cx).focus.clone());
                        root
                    },
                )
                .expect("open MOTAA Desktop");
            if !smoke {
                cx.activate(true);
            }
            #[cfg(target_os = "macos")]
            if !smoke
                && !visual_capture
                && let Some(mut rx) = mac_pinch::install()
            {
                cx.spawn(async move |cx| {
                    loop {
                        let (delta, next_rx) = cx
                            .background_executor()
                            .spawn(async move { (rx.recv().ok(), rx) })
                            .await;
                        rx = next_rx;
                        let Some(mut total) = delta else {
                            break;
                        };
                        while let Ok(more) = rx.try_recv() {
                            total += more;
                        }
                        if total.abs() <= 0.0001 {
                            continue;
                        }
                        let _ = window.update(cx, |root, _, cx| {
                            if root.chart_lightbox.is_some() {
                                let factor = (1. + total as f32).clamp(0.5, 2.0);
                                root.set_lightbox_zoom(root.lightbox_zoom * factor, cx);
                            }
                        });
                    }
                })
                .detach();
            }
            if smoke {
                cx.spawn(async move |cx| {
                    let native: gpui::AnyWindowHandle = window.into();
                    let draw = |cx: &mut gpui::AsyncApp| {
                        native
                            .update(cx, |_, window, app| {
                                window.refresh();
                                let _ = window.draw(app);
                            })
                            .expect("draw hidden native window");
                    };
                    draw(cx);
                    let steps = [
                        (Workspace::Profiles, primary_keybinding("2")),
                        (Workspace::Calculations, primary_keybinding("3")),
                        (Workspace::Time, primary_keybinding("4")),
                        (Workspace::Advanced, primary_keybinding("5")),
                        (Workspace::Charts, primary_keybinding("1")),
                    ];
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(300))
                        .await;
                    for (route, shortcut) in steps {
                        native
                            .update(cx, |_, window, app| {
                                let keystroke = gpui::Keystroke::parse(&shortcut)
                                    .expect("valid smoke shortcut");
                                assert!(window.dispatch_keystroke(keystroke, app));
                            })
                            .expect("dispatch native workspace shortcut");
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(250))
                            .await;
                        draw(cx);
                        window
                            .update(cx, |root, _, cx| {
                                assert!(
                                    root.session.read(cx).workspace == route,
                                    "expected {}",
                                    route.label()
                                );
                            })
                            .expect("verify native workspace");
                        println!("Rendered {}", route.label());
                    }
                    println!("Verified workspace keyboard shortcuts");
                    native
                        .update(cx, |_, window, app| {
                            let press = |window: &mut Window, app: &mut App, shortcut| {
                                let key =
                                    gpui::Keystroke::parse(shortcut).expect("valid focus shortcut");
                                assert!(window.dispatch_keystroke(key, app));
                            };
                            press(window, app, "tab");
                            let first = window.focused(app).expect("first tab stop");
                            press(window, app, "tab");
                            assert_ne!(window.focused(app), Some(first.clone()));
                            press(window, app, "shift-tab");
                            assert_eq!(window.focused(app), Some(first));
                        })
                        .expect("verify keyboard focus traversal");
                    println!("Verified forward and reverse keyboard focus");
                    let root_focus = window
                        .read_with(cx, |root, _| root.focus.clone())
                        .expect("read root focus");
                    native
                        .update(cx, |_, win, app| {
                            win.focus(&root_focus);
                            for shortcut in ["tab", "tab", "tab", "enter"] {
                                let key =
                                    gpui::Keystroke::parse(shortcut).expect("valid activation key");
                                assert!(win.dispatch_keystroke(key, app));
                            }
                        })
                        .expect("dispatch enter activation");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            assert!(root.session.read(cx).workspace == Workspace::Profiles);
                        })
                        .expect("verify enter switched to Profiles");
                    native
                        .update(cx, |_, win, app| {
                            for shortcut in ["shift-tab", "space"] {
                                let key =
                                    gpui::Keystroke::parse(shortcut).expect("valid activation key");
                                assert!(win.dispatch_keystroke(key, app));
                            }
                        })
                        .expect("dispatch space activation");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            assert!(root.session.read(cx).workspace == Workspace::Charts);
                        })
                        .expect("verify space returned to Charts");
                    println!("Verified keyboard control activation");
                    let smoke_profile_id = window
                        .update(cx, |root, _, cx| {
                            root.open_draft(ChartRequest::default(), None, cx);
                            let draft = root.draft.as_ref().expect("open profile draft");
                            draft.name.update(cx, |input, cx| {
                                input.set_value("Smoke Unknown".into(), cx)
                            });
                            draft
                                .datetime
                                .update(cx, |input, cx| input.set_value("invalid-date".into(), cx));
                            root.save_draft(cx);
                            assert!(root.draft.is_some());
                            assert!(!root.session.read(cx).status.is_empty());
                            root.draft
                                .as_ref()
                                .unwrap()
                                .datetime
                                .update(cx, |input, cx| {
                                    input.set_value("1992-08-14 00:00:00".into(), cx)
                                });
                            root.sync_datetime_pickers(cx);
                            assert_eq!(root.draft.as_ref().unwrap().picker.view_year, 1992);
                            assert_eq!(root.draft.as_ref().unwrap().picker.view_month, 8);
                            root.draft.as_mut().unwrap().picker.select_calendar_date(
                                chrono::NaiveDate::from_ymd_opt(1992, 8, 14).unwrap(),
                                cx,
                            );
                            root.draft.as_mut().unwrap().unknown = true;
                            root.save_draft(cx);
                            assert!(root.draft.is_none());
                            let id = root.selected.expect("saved profile id");
                            assert!(
                                root.session
                                    .read(cx)
                                    .calculate(false)
                                    .unwrap()
                                    .reference_ephemeris
                                    .is_none()
                            );
                            root.navigate(Workspace::Charts, cx);
                            id
                        })
                        .expect("verify profile error and unknown birth time");
                    window
                        .update(cx, |root, _, cx| {
                            root.open_draft(ChartRequest::default(), None, cx);
                            assert!(root.show_profile_editor);
                            root.draft.as_ref().unwrap().name.update(cx, |input, cx| {
                                input.set_value("Unfinished profile".into(), cx)
                            });
                        })
                        .expect("open a second profile draft");
                    draw(cx);
                    native
                        .update(cx, |_, win, app| {
                            let key = gpui::Keystroke::parse(&primary_keybinding("2")).unwrap();
                            assert!(win.dispatch_keystroke(key, app));
                        })
                        .expect("open saved profiles while editing a draft");
                    draw(cx);
                    window
                        .read_with(cx, |root, cx| {
                            assert!(root.session.read(cx).workspace == Workspace::Profiles);
                            assert!(!root.show_profile_editor);
                        })
                        .expect("verify Profiles returns to the library");
                    native
                        .update(cx, |_, win, app| {
                            win.focus(&root_focus);
                            let tab = gpui::Keystroke::parse("tab").unwrap();
                            // 6 sidebar controls + collapse toggle + quick jump + birth details + search + resume, then row.
                            for _ in 0..12 {
                                assert!(win.dispatch_keystroke(tab.clone(), app));
                            }
                            let enter = gpui::Keystroke::parse("enter").unwrap();
                            assert!(win.dispatch_keystroke(enter, app));
                        })
                        .expect("activate the saved profile row");
                    draw(cx);
                    native
                        .update(cx, |_, win, app| {
                            win.focus(&root_focus);
                            assert!(win.dispatch_keystroke(
                                gpui::Keystroke::parse(&primary_keybinding("k")).unwrap(),
                                app
                            ));
                        })
                        .expect("open command palette");
                    draw(cx);
                    window
                        .update(cx, |root, window, cx| {
                            assert!(root.palette_open);
                            root.close_palette(window, cx);
                            assert!(!root.palette_open);
                        })
                        .expect("verify command palette");
                    native
                        .update(cx, |_, win, app| {
                            assert!(win.dispatch_keystroke(
                                gpui::Keystroke::parse(&primary_keybinding("b")).unwrap(),
                                app
                            ));
                            assert!(win.dispatch_keystroke(
                                gpui::Keystroke::parse(&primary_keybinding("i")).unwrap(),
                                app
                            ));
                        })
                        .expect("toggle sidebar and inspector");
                    draw(cx);
                    window
                        .update(cx, |root, _, _| {
                            assert!(root.sidebar_collapsed);
                            assert!(!root.inspector_open);
                            root.sidebar_collapsed = false;
                            root.inspector_open = true;
                        })
                        .expect("restore sidebar and inspector");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            let session = root.session.read(cx);
                            assert!(session.workspace == Workspace::Charts);
                            assert_eq!(session.natal_id, Some(smoke_profile_id));
                            root.resume_profile_draft(cx);
                            assert!(root.show_profile_editor);
                            assert_eq!(
                                root.draft.as_ref().unwrap().name.read(cx).value(),
                                "Unfinished profile"
                            );
                        })
                        .expect("verify profile switching preserves the draft");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.navigate(Workspace::Profiles, cx);
                            root.search.update(cx, |input, cx| {
                                input.set_value("__no_such_profile__".into(), cx)
                            });
                        })
                        .expect("search non-matching profile");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.search
                                .update(cx, |input, cx| input.set_value(String::new(), cx));
                            root.delete_confirmation = Some(smoke_profile_id);
                        })
                        .expect("open delete confirmation");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.delete_profile(smoke_profile_id, cx);
                            assert!(root.deleted_profile.is_some());
                        })
                        .expect("delete smoke profile");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.undo_delete(cx);
                            assert_eq!(root.selected, Some(smoke_profile_id));
                            assert!(root.deleted_profile.is_none());
                        })
                        .expect("undo smoke profile deletion");
                    draw(cx);
                    println!("Verified profile errors, unknown time, and undo");
                    let bad_backup = std::env::temp_dir()
                        .join(format!("motaa-smoke-bad-{}.json", std::process::id()));
                    let good_backup = std::env::temp_dir()
                        .join(format!("motaa-smoke-good-{}.json", std::process::id()));
                    std::fs::write(&bad_backup, b"{not valid json").expect("write bad backup");
                    assert!(backup::read(&bad_backup).is_err());
                    let _ = std::fs::remove_file(&bad_backup);
                    window
                        .update(cx, |root, _, cx| {
                            let bytes =
                                backup::encode(root.session.read(cx)).expect("encode backup");
                            std::fs::write(&good_backup, bytes).expect("write good backup");
                            let records = backup::read(&good_backup).expect("read good backup");
                            root.pending_import = Some(records);
                        })
                        .expect("prepare backup review");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.apply_backup(cx);
                            assert!(root.pending_import.is_none());
                            assert!(
                                root.session
                                    .read(cx)
                                    .status
                                    .contains("Skipped 1 duplicates")
                            );
                            root.delete_profile(smoke_profile_id, cx);
                            root.deleted_profile = None;
                            root.navigate(Workspace::Charts, cx);
                        })
                        .expect("apply backup review and clean smoke profile");
                    let _ = std::fs::remove_file(&good_backup);
                    println!("Verified backup review and invalid-file handling");
                    for mode in [
                        "Rasi",
                        "Tricakka",
                        "LatLon",
                        "Natal + transit",
                        "Rasi aspects",
                    ] {
                        window
                            .update(cx, |root, _, cx| {
                                root.show_munp = true;
                                root.select_mode(mode, cx);
                            })
                            .expect("exercise MUNP mode");
                        draw(cx);
                    }
                    for palette in 0..PALETTES.len() {
                        window
                            .update(cx, |root, _, cx| {
                                root.chart_palette = palette;
                                root.select_mode("Rasi", cx);
                            })
                            .expect("exercise chart palette");
                        draw(cx);
                    }
                    window
                        .update(cx, |root, _, _| {
                            root.show_munp = false;
                            root.chart_palette = 0;
                        })
                        .expect("restore default chart flags");
                    println!("Verified MUNP modes and chart palettes");
                    window
                        .update(cx, |root, _, cx| {
                            root.navigate(Workspace::Time, cx);
                            let selected_time = parse_time("2024-07-01 12:00:00").unwrap();
                            root.set_time(selected_time, true, cx);
                            let before = root.session.read(cx).calculate(false).unwrap();
                            root.edit_reference_location(cx);
                            let draft = root.reference_location_draft.as_ref().unwrap();
                            draft
                                .place
                                .update(cx, |input, cx| input.set_value("New York".into(), cx));
                            draft
                                .latitude
                                .update(cx, |input, cx| input.set_value("40 42 46 N".into(), cx));
                            draft
                                .longitude
                                .update(cx, |input, cx| input.set_value("74 0 22 W".into(), cx));
                            let timezone = draft.timezone.clone();
                            timezone
                                .update(cx, |input, cx| input.set_value("Invalid/Zone".into(), cx));
                            root.apply_reference_location(cx);
                            assert!(root.reference_location_draft.is_some());
                            assert!(root.session.read(cx).transit.is_none());
                            timezone.update(cx, |input, cx| {
                                input.set_value("America/New_York".into(), cx)
                            });
                            root.apply_reference_location(cx);
                            assert!(root.reference_location_draft.is_none());
                            let session = root.session.read(cx);
                            assert_eq!(session.reference_profile().birthplace, "New York");
                            assert_eq!(session.time, selected_time);
                            assert_eq!(
                                session
                                    .calculate(true)
                                    .unwrap()
                                    .effective_utc_offset_minutes,
                                -240
                            );
                            assert_eq!(
                                session.calculate(false).unwrap().julian_day_utc,
                                before.julian_day_utc
                            );
                            root.use_birth_location(cx);
                            assert_eq!(
                                root.session.read(cx).reference_profile().birthplace,
                                root.session.read(cx).natal.birthplace
                            );
                        })
                        .expect("verify native reference location edit");
                    println!("Verified independent reference location");
                    window
                        .update(cx, |root, _, cx| {
                            root.open_draft(ChartRequest::default(), None, cx);
                            root.toggle_osm_search(false, cx);
                            root.toggle_map_picker(false, cx);
                            if let Some(draft) = &mut root.draft {
                                draft.location.tz_open = true;
                                draft
                                    .latitude
                                    .update(cx, |input, cx| input.set_value("42.2495".into(), cx));
                                draft
                                    .longitude
                                    .update(cx, |input, cx| input.set_value("-71.0662".into(), cx));
                            }
                            root.autodetect_timezone(false, cx);
                            assert_eq!(
                                root.draft.as_ref().unwrap().timezone.read(cx).value(),
                                "America/New_York"
                            );
                            root.select_timezone_name(false, "Europe/London", cx);
                            assert_eq!(
                                root.draft.as_ref().unwrap().timezone.read(cx).value(),
                                "Europe/London"
                            );
                            assert!(!root.draft.as_ref().unwrap().location.tz_open);
                        })
                        .expect("exercise location and timezone picker");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.draft = None;
                            root.show_profile_editor = false;
                            root.set_status("", cx);
                        })
                        .expect("discard smoke profile draft");
                    println!("Verified location and timezone picker");
                    for mode in MODES {
                        window
                            .update(cx, |root, _, cx| root.select_mode(mode, cx))
                            .expect("select chart mode");
                        draw(cx);
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(75))
                            .await;
                        println!("Exercised chart mode {mode}");
                    }
                    window
                        .update(cx, |root, _, cx| {
                            root.select_mode("Rahu wheel", cx);
                            root.rahu_28 = true;
                            root.rahu_reference = RahuReference::Manual;
                            root.rahu_manual_angle = Some(195.);
                            root.step_rahu_mansion(1, cx);
                            assert_eq!(root.rahu_manual_angle, Some(201.));
                        })
                        .expect("render 28-division Rahu with manual reference");
                    draw(cx);
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(125))
                        .await;
                    window
                        .update(cx, |root, _, _| {
                            root.rahu_28 = false;
                            root.rahu_reference = RahuReference::Node;
                        })
                        .expect("restore Rahu reference");
                    println!("Verified Rahu 28-division manual reference");
                    for tool in [
                        "Ephemeris calendar",
                        "Heliocentric model",
                        "KP subdivisions",
                        "Sky map · 2D",
                        "Sky map · 3D",
                        "Interpretations",
                        "Ten pillars / Ayu Moon",
                        "Satapati name wheel",
                        "Sarvatobhadra",
                        "Strength timeline",
                        "Yoga / candidate times",
                        "Eclipses",
                        "Transit events",
                        "Nakshatra groups",
                        "Daily Tatkalika",
                        "Electional timing",
                        "Ashtakavarga",
                        "Divisional charts",
                    ] {
                        window
                            .update(cx, |root, _, cx| {
                                root.update(cx, |session| {
                                    session.workspace = Workspace::Advanced;
                                    session.tool = Some(tool);
                                })
                            })
                            .expect("select implemented advanced result");
                        draw(cx);
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(75))
                            .await;
                        println!("Exercised advanced tool {tool}");
                    }
                    window
                        .update(cx, |root, _, cx| {
                            root.satapati_lagna = Some(9);
                            root.kp_horary = Some(1);
                            root.interpretation = Interpretation::Yoga;
                            root.strength_age = Some(10);
                            root.update(cx, |session| {
                                session.workspace = Workspace::Advanced;
                                session.tool = Some("Satapati name wheel");
                            });
                        })
                        .expect("exercise Satapati Lagna");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.update(cx, |session| session.tool = Some("KP subdivisions"));
                        })
                        .expect("exercise KP Horary shift");
                    draw(cx);
                    window
                        .update(cx, |root, _, cx| {
                            root.update(cx, |session| session.tool = Some("Interpretations"));
                        })
                        .expect("exercise Interpretations Yoga");
                    draw(cx);
                    window
                        .update(cx, |root, _, _| {
                            root.satapati_lagna = None;
                            root.kp_horary = None;
                            root.interpretation = Interpretation::Period;
                            root.strength_age = None;
                        })
                        .expect("reset advanced interactive states");
                    println!("Verified premium interactive states");
                    window
                        .update(cx, |root, _, cx| {
                            root.update(cx, |session| session.tool = Some("Ephemeris calendar"));
                            root.set_time(
                                parse_time("2024-02-01 12:00:00").expect("smoke date"),
                                true,
                                cx,
                            );
                            root.load_ephemeris_month(cx);
                        })
                        .expect("start monthly ephemeris");
                    let mut monthly_checked = false;
                    for _ in 0..50 {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(100))
                            .await;
                        monthly_checked = window
                            .update(cx, |root, _, _| match root.ephemeris_month.as_ref() {
                                Some(Ok(month)) => {
                                    assert_eq!(month.title, "February 2024");
                                    assert_eq!(month.rows.len(), 29);
                                    assert_eq!(month.planets.len(), 29);
                                    assert_eq!(month.planets[28][0], "2024-02-29");
                                    assert!(
                                        month.planets[28][1..]
                                            .iter()
                                            .all(|cell| cell.ends_with('°'))
                                    );
                                    assert_eq!(month.rows[28][0], "2024-02-29");
                                    true
                                }
                                Some(Err(error)) => panic!("monthly ephemeris failed: {error}"),
                                None => false,
                            })
                            .expect("check monthly ephemeris");
                        if monthly_checked {
                            break;
                        }
                    }
                    assert!(monthly_checked, "monthly ephemeris worker timed out");
                    println!("Verified leap-month ephemeris");
                    window
                        .update(cx, |root, _, cx| {
                            root.set_time(
                                parse_time("2024-03-15 12:00:00").expect("smoke date"),
                                true,
                                cx,
                            );
                            root.update(cx, |session| {
                                session.tool = Some("Transit events");
                                session.settings.tropical = true;
                            });
                            root.search_transits(cx);
                            root.update(cx, |session| session.settings.tropical = false);
                            root.load_strength_timeline(cx);
                        })
                        .expect("start native analysis workers");
                    let mut analyses_checked = false;
                    for _ in 0..80 {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(100))
                            .await;
                        analyses_checked = window
                            .update(cx, |root, _, _| {
                                match (&root.transit_report, &root.strength_report) {
                                    (Some(Ok(transits)), Some(Ok(strength))) => {
                                        assert!(transits.events.iter().any(|event| event.body
                                                == "Sun"
                                                && event.sign == 0));
                                        assert_eq!(strength.samples.len(), 96);
                                        true
                                    }
                                    (Some(Err(error)), _) | (_, Some(Err(error))) => {
                                        panic!("analysis failed: {error}")
                                    }
                                    _ => false,
                                }
                            })
                            .expect("verify analysis workers");
                        if analyses_checked {
                            break;
                        }
                    }
                    assert!(analyses_checked, "analysis workers timed out");
                    println!("Verified transit and strength workers");
                    window
                        .update(cx, |root, window, cx| {
                            root.select_mode("Circular chart", cx);
                            root.chart_zoom = 1.5;
                            window.resize(size(px(1024.), px(680.)));
                        })
                        .expect("render compact zoomed chart");
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(250))
                        .await;
                    window
                        .update(cx, |root, window, cx| {
                            let viewport = window.viewport_size();
                            assert!(f32::from(viewport.width) <= 1025.);
                            assert!(f32::from(viewport.height) >= 680.);
                            assert_eq!(root.session.read(cx).mode, "Circular chart");
                        })
                        .expect("verify compact chart");
                    println!("Verified compact chart resize");
                    window
                        .update(cx, |root, _, cx| root.toggle_circular_detail(cx))
                        .expect("open circular chart detail");
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(125))
                        .await;
                    window
                        .update(cx, |root, _, cx| {
                            assert!(root.circular_detail);
                            assert_eq!(root.chart_zoom, 1.);
                            root.toggle_circular_detail(cx);
                            assert!(!root.circular_detail);
                            root.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
                            assert_eq!(root.chart_lightbox, Some(LightboxTarget::ActiveChart));
                            root.set_lightbox_zoom(2.25, cx);
                            assert_eq!(root.lightbox_zoom, 2.25);
                            root.close_chart_lightbox(cx);
                            assert!(root.chart_lightbox.is_none());
                            assert_eq!(root.lightbox_zoom, 1.);
                        })
                        .expect("close circular chart detail");
                    println!("Verified circular chart detail navigation");

                    for (mode, target) in [
                        ("Tricakka", LightboxTarget::TricakkaSubchart(0)),
                        ("Tricakka", LightboxTarget::TricakkaSubchart(1)),
                        ("Tricakka", LightboxTarget::TricakkaSubchart(2)),
                        ("Natal + transit", LightboxTarget::ComparisonCombined),
                        ("Natal + transit", LightboxTarget::ComparisonSplit(1)),
                        ("Western aspects", LightboxTarget::RasiInset),
                        ("Rasi aspects", LightboxTarget::RasiInset),
                    ] {
                        window
                            .update(cx, |root, _, cx| {
                                root.select_mode(mode, cx);
                                root.open_chart_lightbox(target, cx);
                            })
                            .expect("open chart detail");
                        draw(cx);
                        window
                            .update(cx, |root, _, cx| {
                                root.close_chart_lightbox(cx);
                            })
                            .expect("close chart detail");
                    }
                    println!("Rendered compact Tricakka, comparison, and aspect details");
                    cx.update(|cx| cx.quit()).expect("quit smoke window");
                })
                .detach();
            } else if let Some(capture_dir) = store_screenshots {
                cx.spawn(async move |cx| {
                    let executor = cx.background_executor().clone();
                    let wait = |ms| executor.timer(std::time::Duration::from_millis(ms));
                    let wait_for_ack = |slug: &str| {
                        let done_path = capture_dir.join(format!("{slug}.done"));
                        let executor = executor.clone();
                        async move {
                            for _ in 0..150 {
                                if done_path.exists() {
                                    break;
                                }
                                executor.timer(std::time::Duration::from_millis(100)).await;
                            }
                        }
                    };
                    let emit = |name: &str| {
                        println!("STORE_CAPTURE_READY {name}");
                        let _ = std::io::stdout().flush();
                    };
                    wait(1200).await;
                    let chart_shots: &[(&str, &str, bool)] = &[
                        ("01-rasi-chart", "Rasi", true),
                        ("02-tricakka-chart", "Tricakka", true),
                        ("03-dasha-periods", "Dasha periods", false),
                    ];
                    for &(slug, mode, munp) in chart_shots {
                        window
                            .update(cx, |root, window, cx| {
                                root.navigate(Workspace::Charts, cx);
                                root.inspector_open = true;
                                root.sidebar_collapsed = false;
                                root.show_munp = munp;
                                root.select_mode(mode, cx);
                                window.refresh();
                            })
                            .expect("switch store screenshot chart mode");
                        wait(600).await;
                        emit(slug);
                        wait_for_ack(slug).await;
                    }
                    let workspace_shots: &[(&str, Workspace)] = &[
                        ("04-time-analysis", Workspace::Time),
                        ("05-profiles", Workspace::Profiles),
                        ("06-advanced-tools", Workspace::Advanced),
                    ];
                    for &(slug, route) in workspace_shots {
                        window
                            .update(cx, |root, window, cx| {
                                root.navigate(route, cx);
                                root.inspector_open = true;
                                root.sidebar_collapsed = false;
                                window.refresh();
                            })
                            .expect("switch store screenshot workspace");
                        wait(600).await;
                        emit(slug);
                        wait_for_ack(slug).await;
                    }
                    cx.update(|cx| cx.quit()).expect("quit store screenshots");
                })
                .detach();
            } else if visual_capture {
                cx.spawn(async move |cx| {
                    let executor = cx.background_executor().clone();
                    let wait = |ms| executor.timer(std::time::Duration::from_millis(ms));
                    let emit = |name: &str| {
                        println!("CAPTURE_READY {name}");
                        let _ = std::io::stdout().flush();
                    };
                    wait(900).await;
                    emit("01-circular-chart");
                    wait(700).await;
                    let modes: &[(&str, &str, bool)] = &[
                        ("02-rasi-munp", "Rasi", true),
                        ("03-tricakka", "Tricakka", true),
                        ("04-nakshatra-traversal", "Nakshatra traversal", false),
                        ("05-latlon", "LatLon", true),
                        ("06-western-aspects", "Western aspects", false),
                        ("07-natal-transit", "Natal + transit", true),
                        ("08-dasha-periods", "Dasha periods", false),
                        ("09-rahu-wheel", "Rahu wheel", false),
                        ("10-rasi-aspects", "Rasi aspects", true),
                    ];
                    for &(slug, mode, munp) in modes {
                        window
                            .update(cx, |root, window, cx| {
                                root.show_munp = munp;
                                root.all_strengths = mode == "Rasi aspects";
                                root.select_mode(mode, cx);
                                window.refresh();
                            })
                            .expect("switch capture mode");
                        wait(350).await;
                        emit(slug);
                        wait(700).await;
                    }
                    for (slug, route) in [
                        ("11-time-analysis", Workspace::Time),
                        ("12-profiles", Workspace::Profiles),
                        ("13-calculations", Workspace::Calculations),
                        ("14-advanced", Workspace::Advanced),
                    ] {
                        window
                            .update(cx, |root, window, cx| {
                                root.navigate(route, cx);
                                window.refresh();
                            })
                            .expect("switch capture workspace");
                        wait(350).await;
                        emit(slug);
                        wait(700).await;
                    }
                    window
                        .update(cx, |root, window, cx| {
                            root.navigate(Workspace::Charts, cx);
                            root.select_mode("Circular chart", cx);
                            window.resize(size(px(1024.), px(680.)));
                            window.refresh();
                        })
                        .expect("resize capture window");
                    wait(450).await;
                    emit("15-compact-1024x680");
                    wait(700).await;
                    window
                        .update(cx, |root, window, cx| {
                            root.select_mode("Rahu wheel", cx);
                            root.rahu_reference = RahuReference::Moon;
                            window.refresh();
                        })
                        .expect("capture compact Rahu with Moon reference");
                    wait(350).await;
                    emit("16-rahu-compact-moon");
                    wait(700).await;
                    window
                        .update(cx, |root, window, cx| {
                            root.rahu_28 = true;
                            root.rahu_reference = RahuReference::Manual;
                            root.step_rahu_mansion(1, cx);
                            root.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
                            window.refresh();
                        })
                        .expect("capture manual Rahu lightbox");
                    wait(350).await;
                    emit("17-rahu-manual-lightbox");
                    wait(700).await;
                    for (slug, mode, target) in [
                        ("18-compact-tricakka", "Tricakka", None),
                        (
                            "19-compact-bhava-detail",
                            "Tricakka",
                            Some(LightboxTarget::TricakkaSubchart(1)),
                        ),
                        ("20-compact-traversal", "Nakshatra traversal", None),
                        ("21-compact-split-transit", "Natal + transit", None),
                        ("22-compact-combined-transit", "Natal + transit", None),
                    ] {
                        window
                            .update(cx, |root, window, cx| {
                                root.select_mode(mode, cx);
                                root.show_munp = true;
                                root.split_comparison = slug == "21-compact-split-transit";
                                if let Some(target) = target {
                                    root.open_chart_lightbox(target, cx);
                                }
                                window.refresh();
                            })
                            .expect("switch compact chart capture");
                        wait(350).await;
                        emit(slug);
                        wait(700).await;
                    }
                    cx.update(|cx| cx.quit()).expect("quit visual capture");
                })
                .detach();
            }
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
        });
}

#[cfg(target_os = "macos")]
mod mac_pinch {
    use std::ffi::{CStr, c_char, c_void};
    use std::sync::{OnceLock, mpsc};

    static SENDER: OnceLock<mpsc::Sender<f64>> = OnceLock::new();

    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn class_addMethod(
            cls: *mut c_void,
            name: *mut c_void,
            imp: *const c_void,
            types: *const c_char,
        ) -> bool;
        fn objc_msgSend();
        #[cfg(target_arch = "x86_64")]
        fn objc_msgSend_fpret();
    }

    extern "C" fn magnify_with_event(_this: *mut c_void, _sel: *mut c_void, event: *mut c_void) {
        if event.is_null() {
            return;
        }
        let magnification: f64 = unsafe {
            let sel = sel_registerName(c"magnification".as_ptr());
            #[cfg(target_arch = "x86_64")]
            let msg_send = objc_msgSend_fpret as *const c_void;
            #[cfg(not(target_arch = "x86_64"))]
            let msg_send = objc_msgSend as *const c_void;
            let f: unsafe extern "C" fn(*mut c_void, *mut c_void) -> f64 =
                std::mem::transmute(msg_send);
            f(event, sel)
        };
        if magnification.is_finite()
            && magnification.abs() > 0.0001
            && let Some(tx) = SENDER.get()
        {
            let _ = tx.send(magnification);
        }
    }

    pub fn install() -> Option<mpsc::Receiver<f64>> {
        let (tx, rx) = mpsc::channel();
        if SENDER.set(tx).is_err() {
            return None;
        }
        unsafe {
            let cls = objc_getClass(c"GPUIView".as_ptr());
            if cls.is_null() {
                return None;
            }
            let sel = sel_registerName(c"magnifyWithEvent:".as_ptr());
            let types: &CStr = c"v@:@";
            class_addMethod(
                cls,
                sel,
                magnify_with_event as *const c_void,
                types.as_ptr(),
            );
        }
        Some(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> WorkspaceSession {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        motaa_store::migrate(&connection).unwrap();
        WorkspaceSession::new(Ok(connection), parse_time("2024-01-31 23:59:59").unwrap())
    }

    #[test]
    fn town_directory_matches_v31_1_arrays() {
        let towns: TownDirectory =
            serde_json::from_str(include_str!("../design/source/locations.v31.1.json")).unwrap();
        assert_eq!(towns.towns.len(), 534);
        let pakhangyi = towns.towns.iter().find(|t| t.name_my == "ပခန်းကြီး").unwrap();
        assert_eq!(
            (pakhangyi.latitude, pakhangyi.longitude),
            (21.53715, 95.2037)
        );
        assert!(towns.towns.iter().all(|t| {
            (9.0..=29.0).contains(&t.latitude) && (92.0..=102.0).contains(&t.longitude)
        }));
    }

    #[test]
    fn undo_restores_original_identity_and_history() {
        let session = session();
        let mut request = ChartRequest::default();
        let initial = save_profile(&session, None, &request).unwrap();
        request.name = "Edited profile".into();
        let edited = save_profile(&session, Some(initial.id), &request).unwrap();
        let db = session.connection().unwrap();
        motaa_store::put_setting(
            db,
            &format!("profile_calculations_{}", initial.id),
            r#"{"ayanamsha":3}"#,
        )
        .unwrap();
        let original_reference = ChartRequest {
            birthplace: "Separate transit place".into(),
            ..ChartRequest::default()
        };
        motaa_store::put_setting(
            db,
            &format!("profile_reference_location_{}", initial.id),
            &serde_json::to_string(&original_reference).unwrap(),
        )
        .unwrap();
        assert!(motaa_store::delete(db, edited.id).unwrap());
        motaa_store::restore_deleted(db, &edited).unwrap();
        assert_eq!(session.profiles().unwrap()[0].id, initial.id);
        assert_eq!(
            motaa_store::history(db, initial.id).unwrap()[0].name,
            initial.name
        );
        assert_eq!(session.profile_settings(initial.id).unwrap().ayanamsha, 3);
        assert_eq!(
            session
                .profile_reference_location(initial.id)
                .unwrap()
                .unwrap()
                .birthplace,
            "Separate transit place"
        );
        let copy = save_profile_with_source(&session, None, &request, Some(initial.id)).unwrap();
        assert_ne!(copy.id, initial.id);
        assert_eq!(session.profile_settings(copy.id).unwrap().ayanamsha, 3);
        assert_eq!(
            session
                .profile_reference_location(copy.id)
                .unwrap()
                .unwrap()
                .birthplace,
            "Separate transit place"
        );
    }

    #[test]
    fn calendar_steps_preserve_anchor_and_clamp_month_end() {
        let session = session();
        let anchor = session.anchor;
        assert_eq!(
            format_time(Step::Month.offset(anchor, 1).unwrap()),
            "2024-02-29 23:59:59"
        );
        assert_eq!(
            format_time(Step::Second.offset(anchor, 1).unwrap()),
            "2024-02-01 00:00:00"
        );
        assert_eq!(session.anchor, anchor);
    }

    #[test]
    fn coordinates_accept_dms_and_enforce_hemisphere_and_bounds() {
        assert_eq!(parse_coordinate("20 30 0 S", true).unwrap(), -20.5);
        assert_eq!(parse_coordinate("96.17", false).unwrap(), 96.17);
        assert!(parse_coordinate("90 0 1 N", true).is_err());
        assert!(parse_coordinate("180 0 0 N", false).is_err());
        assert!(parse_coordinate("NaN", false).is_err());
        assert!(parse_coordinate("1 60 0 W", false).is_err());
    }

    #[test]
    fn unknown_birth_time_has_no_reference_planet_positions() {
        let mut session = session();
        session.natal.unknown_birth_time = true;
        assert!(
            session
                .calculate(false)
                .unwrap()
                .reference_ephemeris
                .is_none()
        );
        assert!(
            session
                .calculate(true)
                .unwrap()
                .reference_ephemeris
                .is_some()
        );
    }
}
