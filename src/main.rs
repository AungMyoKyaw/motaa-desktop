mod advanced;
mod ashtakavarga;
mod backup;
mod chart;
mod chart_view;
mod datetime_picker;
mod harness;
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

use gpui::{
    AnyElement, App, Application, Bounds, ClickEvent, Context, Div, Entity, FocusHandle, Image,
    ImageFormat, KeyBinding, Window, WindowBounds, WindowOptions, actions, div, img, prelude::*,
    px, rgb, size,
};
use input::TextInput;
use jiff::civil::DateTime;
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
        About,
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
    about_open: bool,
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
            let mut session = WorkspaceSession::new(
                store,
                jiff::Timestamp::now()
                    .in_tz("UTC")
                    .map(|z| z.datetime())
                    .unwrap_or_else(|_| DateTime::constant(2000, 1, 1, 0, 0, 0, 0)),
            );
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
            about_open: false,
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
        let initial_dt =
            parse_time(&timestamp).unwrap_or_else(|_| DateTime::constant(2000, 1, 1, 12, 0, 0, 0));
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
        request.year = i32::from(time.year());
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

    fn set_time(&mut self, time: DateTime, reset_anchor: bool, cx: &mut Context<Self>) {
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
            .into_any_element()
    }
}

impl Render for DesktopRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session_ref = self.session.read(cx);
        let route = session_ref.workspace;
        let status = session_ref.status.clone();
        let active_natal = session_ref.natal.name.clone();
        let ref_time_label = session_ref.time.strftime("%d %b %Y · %H:%M").to_string();
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
            .children(Workspace::ALL.into_iter().map(|entry| {
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
                            "»"
                        } else {
                            "« Collapse"
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
                                            .child(caption("Quick jump")),
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
                                                    "Hide inspector"
                                                } else {
                                                    "Inspector"
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
                                        bar.child(self.birth_details_button(cx))
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
            .when_some(self.active_map_popup(), |root, reference| {
                root.child(self.location_map_overlay(reference, cx))
            })
            .when(self.palette_open, |root| {
                root.child(self.palette_overlay(cx))
            })
            .when(self.about_open, |root| {
                root.child(self.about_overlay(cx))
            })
    }
}

impl DesktopRoot {
    pub(crate) fn about_overlay(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("about-backdrop")
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x28251f70))
            .flex()
            .items_center()
            .justify_center()
            .p_6()
            .on_click(cx.listener(|root, _, _, cx| {
                root.about_open = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("about-dialog")
                    .w(px(420.))
                    .max_w_full()
                    .bg(rgb(PAPER))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(rgb(LINE))
                    .shadow_lg()
                    .p_6()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .on_key_down(cx.listener(|root, event: &gpui::KeyDownEvent, _, cx| {
                        if event.keystroke.key == "escape" {
                            root.about_open = false;
                            cx.stop_propagation();
                            cx.notify();
                        }
                    }))
                    .child(img(app_icon()).w(px(56.)).h(px(56.)).rounded(px(12.)))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(INK))
                            .child("MOTAA Desktop"),
                    )
                    .child(caption(format!("Version {}", env!("CARGO_PKG_VERSION"))))
                    .child(div().text_size(px(13.)).text_color(rgb(INK)).child(
                        "Offline-first Myanmar astrology workspace built with Rust and GPUI.",
                    ))
                    .child(caption(
                        "License: GNU Affero General Public License v3.0 (AGPL-3.0)",
                    ))
                    .child(caption(
                        "Local-first SQLite storage · No accounts or telemetry",
                    ))
                    .child(div().pt_2().child(button("Close", true, cx, |root, cx| {
                        root.about_open = false;
                        cx.notify();
                    }))),
            )
            .into_any_element()
    }

    pub(crate) fn birth_details_button(&self, cx: &mut Context<Self>) -> AnyElement {
        button(
            if self.show_birth {
                "Hide birth details"
            } else {
                "Show birth details"
            },
            false,
            cx,
            |view, _| view.show_birth = !view.show_birth,
        )
        .into_any_element()
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

fn format_time(time: DateTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second()
    )
}
fn parse_time(text: &str) -> Result<DateTime, String> {
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
            cx.set_menus(vec![gpui::Menu {
                name: "MOTAA Desktop".into(),
                items: vec![
                    gpui::MenuItem::action("About MOTAA Desktop", About),
                    gpui::MenuItem::separator(),
                    gpui::MenuItem::action("Quit MOTAA Desktop", Quit),
                ],
            }]);
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
            cx.on_action(|_: &Quit, cx| cx.quit());
            let about_window = window;
            cx.on_action(move |_: &About, cx| {
                let _ = about_window.update(cx, |root, _, cx| {
                    root.about_open = true;
                    cx.notify();
                });
            });
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
            harness::run(smoke, store_screenshots, visual_capture, window, cx);
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
