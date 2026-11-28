extern crate gpui_kit as gpui;

mod about;
mod advanced;
mod ashtakavarga;
mod backup;
mod chart;
mod chart_view;
mod datetime_picker;
mod harness;
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
    AnyElement, App, Bounds, ClickEvent, Context, Div, Entity, FocusHandle, Image, ImageFormat,
    KeyBinding, Window, WindowBounds, WindowOptions, actions, div, img, prelude::*, px, rgb, size,
};
use jiff::civil::DateTime;
use model::{AYANAMSHAS, HOUSES, Settings, Step, Workspace, WorkspaceSession};
use motaa_engine::ChartRequest;
pub(crate) use theme::*;

pub(crate) type TextInput = gpui::component::input::InputState;

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

struct LightboxState {
    target: Option<LightboxTarget>,
    zoom: f32,
    scroll: gpui::ScrollHandle,
    drag: Option<(gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>)>,
}

struct RahuState {
    divisions_28: bool,
    reference: RahuReference,
    manual_input: Entity<TextInput>,
    manual_angle: Option<f64>,
}

struct EphemerisState {
    month: Option<Result<EphemerisMonth, String>>,
    progress: bool,
    request_id: u64,
}

struct TransitState {
    report: Option<Result<transits::Report, String>>,
    searching: bool,
    filter: TransitFilter,
}

struct StrengthState {
    body: usize,
    pending: bool,
    report: Option<Result<strength_timeline::Timeline, String>>,
    age: Option<u8>,
}

struct EclipseState {
    lunar: bool,
    visible_only: bool,
    selection: Option<f64>,
}

struct VedhaState {
    body: usize,
    transit: bool,
    abhijit: bool,
}

struct InterpretationState {
    kind: Interpretation,
    body: usize,
    bhava: bool,
    transit: bool,
}

struct NameWheelState {
    reference: Entity<TextInput>,
    matches: Vec<usize>,
    sector: Option<usize>,
    satapati_lagna: Option<usize>,
}

struct DesktopRoot {
    session: Entity<WorkspaceSession>,
    focus: FocusHandle,
    sidebar_collapsed: bool,
    inspector_open: bool,
    palette_open: bool,
    workspace_scroll: gpui::ScrollHandle,
    inspector_scroll: gpui::ScrollHandle,
    palette_scroll: gpui::ScrollHandle,
    town_scroll: gpui::ScrollHandle,
    tz_scroll: gpui::ScrollHandle,
    map_modal_scroll: gpui::ScrollHandle,
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
    lightbox: LightboxState,
    scrub_offset: i32,
    scrub_dragging: bool,
    dasha_major: Option<usize>,
    dasha_sub: Option<usize>,
    all_strengths: bool,
    rahu: RahuState,
    sky_rotation: f64,
    tara_from_ascendant: bool,
    vedha: VedhaState,
    selected_varga: varga::Varga,
    name_wheel: NameWheelState,
    transits: TransitState,
    eclipses: EclipseState,
    strength: StrengthState,
    candidate_inputs: [Entity<TextInput>; 3],
    candidate_pickers: [datetime_picker::DateTimePicker; 3],
    candidate_rows: Option<Vec<[String; 5]>>,
    interpretations: InterpretationState,
    kp_horary: Option<usize>,
    ephemeris: EphemerisState,
    towns: TownDirectory,
    tile_cache: std::collections::HashMap<(u8, u32, u32), location_picker::TileState>,
}

// Navigation geometry exported from the master’s Lucide v0.468.0 components.
const RAIL_ICONS: [(&str, &[u8]); 6] = [
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
    ("info", include_bytes!("../assets/rail/info.svg")),
];

fn macos_chrome() -> bool {
    static MACOS_CHROME: OnceLock<bool> = OnceLock::new();
    *MACOS_CHROME.get_or_init(|| {
        cfg!(target_os = "macos") && !std::env::args().any(|arg| arg == "--windows-chrome")
    })
}

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

// Crisp Lucide-style UI chrome icons embedded directly into the binary.
const UI_ICONS: [(&str, &[u8]); 10] = [
    (
        "ui/panel-right",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M15 3v18"/></svg>"#,
    ),
    (
        "ui/panel-left",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/></svg>"#,
    ),
    (
        "ui/search",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"#,
    ),
    (
        "ui/ellipsis",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>"#,
    ),
    (
        "ui/eye",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/><circle cx="12" cy="12" r="3"/></svg>"#,
    ),
    (
        "ui/eye-off",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49"/><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242"/><path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143"/><path d="m2 2 20 20"/></svg>"#,
    ),
    (
        "ui/maximize",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" x2="14" y1="3" y2="10"/><line x1="3" x2="10" y1="21" y2="14"/></svg>"#,
    ),
    (
        "ui/plus",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>"#,
    ),
    (
        "ui/external-link",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>"#,
    ),
    (
        "ui/copy",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>"#,
    ),
];

struct DesktopAssets;

impl gpui::AssetSource for DesktopAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = RAIL_ICONS
            .iter()
            .chain(UI_ICONS.iter())
            .find(|(name, _)| *name == path)
        {
            return Ok(Some(std::borrow::Cow::Borrowed(*bytes)));
        }
        gpui::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let mut items: Vec<gpui::SharedString> = RAIL_ICONS
            .iter()
            .chain(UI_ICONS.iter())
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| (*name).into())
            .collect();
        items.extend(gpui::assets::Assets.list(path)?);
        Ok(items)
    }
}

fn rail_icon(index: usize, selected: bool) -> impl IntoElement {
    gpui::svg()
        .path(RAIL_ICONS[index].0)
        .size(px(16.))
        .text_color(rgb(if selected { GOLD } else { MUTED }))
}

pub(crate) fn ui_icon(path: &'static str, size_px: f32, color: u32) -> impl IntoElement {
    gpui::svg()
        .path(path)
        .size(px(size_px))
        .flex_shrink_0()
        .text_color(rgb(color))
}

pub(crate) fn render_input(state: &Entity<TextInput>) -> gpui::component::input::Input {
    gpui::component::input::Input::new(state)
}

fn field(label: &'static str, input: Entity<TextInput>) -> Div {
    column()
        .gap_1()
        .child(caption(label))
        .child(render_input(&input))
}

fn text_input(
    value: &str,
    placeholder: &str,
    window: &mut Window,
    cx: &mut Context<DesktopRoot>,
) -> Entity<TextInput> {
    let value = value.to_owned();
    let placeholder = placeholder.to_owned();
    cx.new(|cx| {
        TextInput::new(window, cx)
            .placeholder(placeholder)
            .default_value(value)
    })
}

pub(crate) fn subscribe_input(
    input: &Entity<TextInput>,
    window: &mut Window,
    cx: &mut Context<DesktopRoot>,
    mut on_change: impl FnMut(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
) {
    cx.subscribe_in(
        input,
        window,
        move |root, _, event: &gpui::component::input::InputEvent, window, cx| {
            if matches!(event, gpui::component::input::InputEvent::Change) {
                on_change(root, window, cx);
            }
        },
    )
    .detach();
}

fn button(
    label: impl Into<gpui::SharedString>,
    primary: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    button_in(label, primary, cx, move |view, _, cx| action(view, cx))
}

fn button_in(
    label: impl Into<gpui::SharedString>,
    primary: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    use gpui::component::{Sizable as _, button::ButtonVariants as _};
    let label = label.into();
    let btn = gpui::component::button::Button::new(label.clone())
        .small()
        .label(label)
        .on_click(cx.listener(move |view, _, window, cx| {
            action(view, window, cx);
            cx.notify();
        }));
    if primary {
        btn.primary().into_any_element()
    } else {
        btn.outline().into_any_element()
    }
}

fn pill_control(
    id: impl Into<gpui::ElementId>,
    radius: f32,
    text_size: f32,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control(id, cx, action)
        .px(px(10.))
        .h(px(26.))
        .justify_center()
        .rounded(px(radius))
        .text_size(px(text_size))
}

fn compact_button(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    active: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    compact_button_in(id, label, active, cx, move |view, _, cx| action(view, cx))
}

fn compact_button_in(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    active: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    use gpui::component::{Sizable as _, button::ButtonVariants as _};
    let btn = gpui::component::button::Button::new(id)
        .xsmall()
        .label(label)
        .on_click(cx.listener(move |view, _, window, cx| {
            action(view, window, cx);
            cx.notify();
        }));
    if active {
        btn.primary().into_any_element()
    } else {
        btn.outline().into_any_element()
    }
}

pub(crate) fn icon_compact_button(
    id: impl Into<gpui::ElementId>,
    icon_path: &'static str,
    icon_size: f32,
    label: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    use gpui::component::Sizable as _;
    gpui::component::button::Button::new(id)
        .outline()
        .xsmall()
        .child(ui_icon(icon_path, icon_size, MUTED))
        .child(label.into())
        .on_click(cx.listener(move |view, _, _, cx| {
            action(view, cx);
            cx.notify();
        }))
        .into_any_element()
}

pub(crate) fn anchored_popover(
    id: impl Into<gpui::ElementId>,
    anchor: gpui::Anchor,
    width: f32,
    content: impl FnOnce(gpui::Stateful<Div>) -> gpui::Stateful<Div>,
) -> AnyElement {
    let container = div()
        .id(id)
        .occlude()
        .mt(px(4.))
        .w(px(width))
        .p(px(5.))
        .rounded(px(10.))
        .bg(rgb(PAPER))
        .border_1()
        .border_color(rgb(LINE))
        .shadow(popover_shadow())
        .flex()
        .flex_col()
        .gap(px(1.))
        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(|_, _, cx| cx.stop_propagation());
    gpui::deferred(
        gpui::anchored()
            .anchor(anchor)
            .snap_to_window_with_margin(px(12.))
            .child(content(container)),
    )
    .with_priority(1)
    .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn modal_dialog(
    backdrop_id: &'static str,
    dialog_id: &'static str,
    width: f32,
    max_height: Option<f32>,
    scroll_handle: Option<&gpui::ScrollHandle>,
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
    close_label: &'static str,
    body: AnyElement,
    cx: &mut Context<DesktopRoot>,
    on_close: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    let on_close = Rc::new(on_close);
    let on_escape = on_close.clone();
    let header = row()
        .justify_between()
        .items_center()
        .child(
            column()
                .gap_0()
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(INK))
                        .child(title.into()),
                )
                .child(caption(subtitle.into())),
        )
        .child(button(close_label, true, cx, move |root, cx| {
            on_close(root, cx);
            cx.notify();
        }));
    let mut dialog = div()
        .id(dialog_id)
        .w(px(width))
        .max_w_full()
        .bg(rgb(PAPER))
        .rounded(px(12.))
        .border_1()
        .border_color(rgb(LINE))
        .shadow_lg()
        .p_5()
        .flex()
        .flex_col()
        .gap_3()
        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(|_, _, cx| cx.stop_propagation())
        .on_key_down(cx.listener(move |root, event: &gpui::KeyDownEvent, _, cx| {
            if event.keystroke.key == "escape" {
                on_escape(root, cx);
                cx.stop_propagation();
                cx.notify();
            }
        }));
    if let (Some(mh), Some(handle)) = (max_height, scroll_handle) {
        use gpui::component::scroll::ScrollableElement as _;
        let scroll_inner = div()
            .id((dialog_id, 1usize))
            .w_full()
            .max_h(px(mh - 40.))
            .overflow_y_scroll()
            .track_scroll(handle)
            .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
            .flex()
            .flex_col()
            .gap_3()
            .child(header)
            .child(body);
        dialog = dialog.child(
            div()
                .relative()
                .w_full()
                .max_h(px(mh - 40.))
                .child(scroll_inner)
                .vertical_scrollbar(handle),
        );
    } else {
        dialog = dialog
            .when_some(max_height, |el, mh| el.max_h(px(mh)).overflow_y_scroll())
            .child(header)
            .child(body);
    }
    div()
        .id(backdrop_id)
        .occlude()
        .on_scroll(|_, _, cx| cx.stop_propagation())
        .absolute()
        .inset_0()
        .bg(gpui::rgba(BACKDROP))
        .flex()
        .items_center()
        .justify_center()
        .p_6()
        .child(dialog)
        .into_any_element()
}

fn ghost_button(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    ghost_button_in(id, label, cx, move |view, _, cx| action(view, cx))
}

fn ghost_button_in(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    use gpui::component::{Sizable as _, button::ButtonVariants as _};
    gpui::component::button::Button::new(id)
        .ghost()
        .xsmall()
        .label(label)
        .on_click(cx.listener(move |view, _, window, cx| {
            action(view, window, cx);
            cx.notify();
        }))
        .into_any_element()
}

fn mode_pill(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    active: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    pill_control(id, 6., 12.5, cx, action)
        .font_weight(if active {
            gpui::FontWeight::SEMIBOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .bg(if active {
            rgb(PAPER).into()
        } else {
            gpui::transparent_black()
        })
        .text_color(rgb(if active { GOLD } else { MUTED }))
        .when(active, |el| el.shadow(pill_shadow()))
        .hover(|style| {
            if active {
                style.bg(rgb(PAPER))
            } else {
                style.bg(rgb(SURFACE_HOVER)).text_color(rgb(INK))
            }
        })
        .child(label.into())
        .into_any_element()
}

fn segmented_pair(
    id_prefix: &'static str,
    labels: (&'static str, &'static str),
    second_active: bool,
    cx: &mut Context<DesktopRoot>,
    on_select: impl Fn(&mut DesktopRoot, bool, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    use gpui::component::{
        Sizable as _,
        tab::{Tab, TabBar},
    };
    TabBar::new(id_prefix)
        .segmented()
        .xsmall()
        .w_full()
        .selected_index(usize::from(second_active))
        .on_click(cx.listener(move |view, idx: &usize, _, cx| {
            on_select(view, *idx == 1, cx);
            cx.notify();
        }))
        .children([
            Tab::new().label(labels.0).flex_1(),
            Tab::new().label(labels.1).flex_1(),
        ])
        .into_any_element()
}

fn menu_item(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    checked: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    menu_item_in(id, label, checked, cx, move |view, _, cx| action(view, cx))
}

fn menu_item_in(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    checked: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    action_control_in(id, cx, action)
        .w_full()
        .h(px(30.))
        .px(px(10.))
        .justify_between()
        .rounded(px(6.))
        .text_size(px(12.5))
        .font_weight(if checked {
            gpui::FontWeight::SEMIBOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .text_color(rgb(if checked { GOLD } else { INK }))
        .hover(|style| style.bg(rgb(SOFT)))
        .active(|style| style.bg(rgb(SELECTED)))
        .child(label.into())
        .when(checked, |item| {
            item.child(div().text_size(px(11.5)).text_color(rgb(GOLD)).child("✓"))
        })
        .into_any_element()
}

fn menu_item_danger(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    action_control(id, cx, action)
        .w_full()
        .h(px(30.))
        .px(px(10.))
        .justify_start()
        .rounded(px(6.))
        .text_size(px(12.5))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(ERROR))
        .hover(|style| style.bg(rgb(ERROR_BG)))
        .child(label.into())
        .into_any_element()
}

fn menu_divider() -> impl IntoElement {
    gpui::component::separator::Separator::horizontal()
        .my_1()
        .color(gpui::Hsla::from(rgb(SUBTLE_LINE)))
}

fn action_control(
    id: impl Into<gpui::ElementId>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control_in(id, cx, move |view, _, cx| action(view, cx))
}

fn action_control_in(
    id: impl Into<gpui::ElementId>,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Window, &mut Context<DesktopRoot>) + 'static,
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
        .on_click(cx.listener(move |view, _: &ClickEvent, window, cx| {
            action(view, window, cx);
            cx.notify();
        }))
        .on_key_down(
            cx.listener(move |view, event: &gpui::KeyDownEvent, window, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    keyboard_action(view, window, cx);
                    cx.stop_propagation();
                    cx.notify();
                }
            }),
        )
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
        .border_color(rgb(SUBTLE_LINE))
        .hover(|item| item.bg(rgb(SELECTED)))
        .child(label)
        .child(caption("→"))
}

impl DesktopRoot {
    fn new(
        store: Result<rusqlite::Connection, String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
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
        let time_picker =
            datetime_picker::DateTimePicker::new("reference-dt", now, false, window, cx);
        let date = time_picker.datetime.clone();
        let candidate_pickers = [
            datetime_picker::DateTimePicker::new("candidate-1-dt", now, false, window, cx),
            datetime_picker::DateTimePicker::new("candidate-2-dt", now, false, window, cx),
            datetime_picker::DateTimePicker::new("candidate-3-dt", now, false, window, cx),
        ];
        let candidate_inputs = [
            candidate_pickers[0].datetime.clone(),
            candidate_pickers[1].datetime.clone(),
            candidate_pickers[2].datetime.clone(),
        ];
        let search = text_input("", "Search by name or birthplace", window, cx);
        let tool_search = text_input("", "Search chart modes and advanced tools", window, cx);
        let palette_query = text_input(
            "",
            "Jump to chart mode, tool, profile, or action…",
            window,
            cx,
        );
        subscribe_input(&tool_search, window, cx, |_, _, cx| cx.notify());
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        subscribe_input(&search, window, cx, |_, _, cx| cx.notify());
        subscribe_input(&palette_query, window, cx, |view, _, cx| {
            view.palette_index = 0;
            cx.notify();
        });
        Self {
            session,
            focus: cx.focus_handle(),
            sidebar_collapsed: false,
            inspector_open: false,
            palette_open: false,
            workspace_scroll: gpui::ScrollHandle::new(),
            inspector_scroll: gpui::ScrollHandle::new(),
            palette_scroll: gpui::ScrollHandle::new(),
            town_scroll: gpui::ScrollHandle::new(),
            tz_scroll: gpui::ScrollHandle::new(),
            map_modal_scroll: gpui::ScrollHandle::new(),
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
            lightbox: LightboxState {
                target: None,
                zoom: 1.,
                scroll: gpui::ScrollHandle::new(),
                drag: None,
            },
            scrub_offset: 0,
            scrub_dragging: false,
            dasha_major: None,
            dasha_sub: None,
            all_strengths: false,
            rahu: RahuState {
                divisions_28: false,
                reference: RahuReference::Node,
                manual_input: text_input("", "Longitude 0° to <360°", window, cx),
                manual_angle: None,
            },
            sky_rotation: 0.,
            tara_from_ascendant: false,
            vedha: VedhaState {
                body: 2,
                transit: true,
                abhijit: true,
            },
            selected_varga: varga::Varga::Navamsa,
            name_wheel: NameWheelState {
                reference: text_input("", "Myanmar syllable", window, cx),
                matches: Vec::new(),
                sector: None,
                satapati_lagna: None,
            },
            transits: TransitState {
                report: None,
                searching: false,
                filter: TransitFilter::All,
            },
            eclipses: EclipseState {
                lunar: false,
                visible_only: false,
                selection: None,
            },
            strength: StrengthState {
                body: 2,
                pending: false,
                report: None,
                age: None,
            },
            candidate_inputs,
            candidate_pickers,
            candidate_rows: None,
            interpretations: InterpretationState {
                kind: Interpretation::Period,
                body: 2,
                bhava: false,
                transit: false,
            },
            kp_horary: None,
            ephemeris: EphemerisState {
                month: None,
                progress: false,
                request_id: 0,
            },
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
        self.workspace_scroll
            .set_offset(gpui::point(px(0.), px(0.)));
        self.delete_confirmation = None;
        self.lightbox.target = None;
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
        self.workspace_scroll
            .set_offset(gpui::point(px(0.), px(0.)));
    }

    fn sync_datetime_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(draft) = &mut self.draft {
            draft.picker.sync_from_inputs(window, cx);
        }
        if let Some(dt) = self.time_picker.sync_from_inputs(window, cx)
            && self.time_picker.time_open
        {
            self.update(cx, |session| {
                session.time = dt;
                session.status.clear();
            });
        }
        for picker in &mut self.candidate_pickers {
            picker.sync_from_inputs(window, cx);
        }
    }

    fn open_draft(
        &mut self,
        request: ChartRequest,
        id: Option<i64>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        let picker =
            datetime_picker::DateTimePicker::new("profile-dt", initial_dt, false, window, cx);
        let datetime = picker.datetime.clone();
        self.draft = Some(ProfileDraft {
            copy_from: None,
            name: text_input(&request.name, "Name", window, cx),
            datetime,
            picker,
            place: text_input(&request.birthplace, "Birthplace", window, cx),
            latitude: text_input(&request.latitude.to_string(), "Latitude", window, cx),
            longitude: text_input(&request.longitude.to_string(), "Longitude", window, cx),
            timezone: text_input(
                &request
                    .timezone_id
                    .clone()
                    .unwrap_or_else(|| request.timezone_offset_minutes.to_string()),
                "IANA zone or offset minutes",
                window,
                cx,
            ),
            unknown: request.unknown_birth_time,
            town_search: None,
            location: location_picker::LocationPickerState::new(
                request.latitude,
                request.longitude,
                window,
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
        let timestamp_raw = draft.datetime.read(cx).value();
        let timestamp = timestamp_raw.trim();
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
        request.latitude = parse_coordinate(draft.latitude.read(cx).value().as_ref(), true)?;
        request.longitude = parse_coordinate(draft.longitude.read(cx).value().as_ref(), false)?;
        set_timezone(&mut request, draft.timezone.read(cx).value().as_ref());
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
        self.lightbox.target = None;
        self.chart_zoom = 1.;
        self.chart_scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.chart_drag = None;
        self.chart_press_origin = None;
        self.update(cx, |session| {
            session.mode = mode;
            session.workspace = Workspace::Charts;
        });
    }

    fn set_time(
        &mut self,
        time: DateTime,
        reset_anchor: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.time_picker.set_datetime(time, window, cx);
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

    fn step_time(&mut self, count: i32, scrub: bool, window: &mut Window, cx: &mut Context<Self>) {
        let s = self.session.read(cx);
        let base = if scrub { s.now() } else { s.time };
        match s.step.offset(base, count) {
            Some(time) => {
                if scrub {
                    self.scrub_offset = count;
                    self.update(cx, |session| session.anchor = base);
                }
                self.set_time(time, !scrub, window, cx);
            }
            None => self.set_status("The requested time is outside the supported calendar.", cx),
        }
    }

    fn mode_buttons_popover(&self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.session.read(cx).mode;
        anchored_popover("more-modes-popover", gpui::Anchor::TopLeft, 216., |menu| {
            menu.children(
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
                    .enumerate()
                    .map(|(idx, mode)| {
                        menu_item(
                            ("extra-mode", idx),
                            mode,
                            active == mode,
                            cx,
                            move |view, cx| {
                                view.show_modes = false;
                                view.select_mode(mode, cx);
                            },
                        )
                    }),
            )
        })
    }

    fn profile_edit_save_buttons(&self, cx: &mut Context<Self>) -> Div {
        row()
            .gap_2()
            .child(compact_button_in(
                "profile-edit-btn",
                "Edit",
                false,
                cx,
                |view, window, cx| {
                    let s = view.session.read(cx);
                    view.open_draft(s.natal.clone(), s.natal_id, window, cx);
                },
            ))
            .child(compact_button(
                "profile-save-btn",
                "Save",
                true,
                cx,
                |view, cx| {
                    let s = view.session.read(cx);
                    match save_profile(s, s.natal_id, &s.natal) {
                        Ok(profile) => view.update(cx, |s| {
                            s.natal_id = Some(profile.id);
                            s.status = "Profile saved.".into();
                        }),
                        Err(error) => view.set_status(error, cx),
                    }
                },
            ))
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
                    .gap_6()
                    .pt_5()
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
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .child(
                        panel()
                            .child("Myanmar calendar convention")
                            .child(row().children(
                                motaa_engine::myanmar_calendar::CivilCalendar::OPTIONS.map(
                                    |calendar| {
                                        button(
                                            calendar.label(),
                                            draft.calendar == calendar,
                                            cx,
                                            move |view, cx| {
                                                view.update(cx, |s| {
                                                    s.settings_draft.calendar = calendar
                                                });
                                            },
                                        )
                                    },
                                ),
                            )),
                    )
                    .child(
                        panel()
                            .child("Aspect display preference")
                            .child(caption(
                                "Stored with the profile. Aspect formulas use fixed orbs.",
                            ))
                            .child(row().children(
                                [("5D Aspect", false), ("15D Aspect", true)].map(
                                    |(label, choice)| {
                                        button(
                                            label,
                                            draft.fifteen_degree_display == choice,
                                            cx,
                                            move |view, cx| {
                                                view.update(cx, |s| {
                                                    s.settings_draft.fifteen_degree_display = choice
                                                });
                                            },
                                        )
                                    },
                                ),
                            )),
                    ),
            )
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
            .pt(px(14.))
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
            .child({
                use gpui::component::{Sizable as _, button::ButtonVariants as _};
                gpui::component::button::Button::new("sidebar-new-chart")
                    .primary()
                    .small()
                    .w_full()
                    .child(ui_icon("ui/plus", 14., PAPER))
                    .when(!collapsed, |btn| btn.child("New birth chart"))
                    .when(collapsed, |btn| btn.tooltip("New birth chart"))
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.open_draft(ChartRequest::default(), None, window, cx);
                        cx.notify();
                    }))
            })
            .child(div().h(px(10.)))
            .children(Workspace::ALL.into_iter().enumerate().map(|(idx, entry)| {
                let selected = entry == route;
                let shortcut_hint = if macos_chrome() {
                    format!("⌘{}", idx + 1)
                } else {
                    format!("^{}", idx + 1)
                };
                let item = action_control(("workspace", entry as usize), cx, move |view, cx| {
                    view.navigate(entry, cx)
                })
                .relative()
                .px(px(if collapsed { 8. } else { 12. }))
                .h(px(36.))
                .gap_2()
                .rounded(px(RADIUS_NAV))
                .bg(rgb(if selected { PAPER } else { RAIL }))
                .when(selected, |el| {
                    el.border_1()
                        .border_color(rgb(SUBTLE_LINE))
                        .shadow(pill_shadow())
                })
                .hover(|style| style.bg(rgb(if selected { PAPER } else { SURFACE_HOVER })))
                .focus(|style| style.border_2().border_color(rgb(GOLD)))
                .when(selected && !collapsed, |row_el| {
                    row_el.child(
                        div()
                            .absolute()
                            .left(px(3.))
                            .w(px(3.))
                            .h(px(16.))
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
                            .text_size(px(13.))
                            .text_color(rgb(if selected { INK } else { MUTED }))
                            .font_weight(if selected {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::MEDIUM
                            })
                            .child(entry.label()),
                    )
                    .child(
                        div()
                            .font_family("Menlo")
                            .text_size(px(10.))
                            .text_color(rgb(if selected { GOLD } else { MUTED }))
                            .child(shortcut_hint),
                    )
                }
            }))
            .child(div().flex_1())
            .child(
                div()
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child({
                        use gpui::component::{Sizable as _, button::ButtonVariants as _};
                        gpui::component::button::Button::new("sidebar-collapse-btn")
                            .ghost()
                            .small()
                            .w_full()
                            .tooltip(if collapsed {
                                "Expand sidebar"
                            } else {
                                "Collapse sidebar"
                            })
                            .child(ui_icon("ui/panel-left", 14., MUTED))
                            .when(!collapsed, |btn| btn.child(caption("Collapse")))
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.sidebar_collapsed = !view.sidebar_collapsed;
                                cx.notify();
                            }))
                    }),
            );
        let lightbox_open = self.lightbox.target.is_some();
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
                Workspace::About => self.about_view(cx),
            }
        };
        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .track_focus(&self.focus)
            .text_color(rgb(INK))
            .bg(rgb(IVORY))
            .text_size(px(14.))
            .font_family("Helvetica Neue")
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|view, _, _, cx| {
                    if view.selected.is_some() || view.show_modes {
                        view.selected = None;
                        view.delete_confirmation = None;
                        view.show_modes = false;
                        cx.notify();
                    }
                }),
            )
            .on_key_down(cx.listener(|view, event: &gpui::KeyDownEvent, _, cx| {
                let key = event.keystroke.key.as_str();
                if matches!(key, "pageup" | "pagedown") {
                    let dir = if key == "pageup" { 1.0 } else { -1.0 };
                    let scrolled = if view.active_town_popup().is_some() {
                        scroll_handle_page(&view.town_scroll, dir)
                    } else if view.active_timezone_popup().is_some() {
                        scroll_handle_page(&view.tz_scroll, dir)
                    } else if view.active_map_popup().is_some() {
                        scroll_handle_page(&view.map_modal_scroll, dir)
                    } else if view.palette_open {
                        scroll_handle_page(&view.palette_scroll, dir)
                    } else if view.lightbox.target.is_none()
                        && view.active_datetime_popup().is_none()
                        && view.active_time_popup().is_none()
                    {
                        scroll_handle_page(&view.workspace_scroll, dir)
                    } else {
                        false
                    };
                    if scrolled {
                        cx.stop_propagation();
                        cx.notify();
                    }
                    return;
                }
                if key == "escape"
                    && view.lightbox.target.is_none()
                    && (view.selected.is_some() || view.show_modes)
                {
                    view.selected = None;
                    view.delete_confirmation = None;
                    view.show_modes = false;
                    cx.stop_propagation();
                    cx.notify();
                    return;
                }
                if view.lightbox.target.is_none() {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "escape" => {
                        view.close_chart_lightbox(cx);
                        cx.stop_propagation();
                    }
                    "=" | "+" => {
                        view.set_lightbox_zoom(view.lightbox.zoom + 0.25, cx);
                        cx.stop_propagation();
                    }
                    "-" => {
                        view.set_lightbox_zoom(view.lightbox.zoom - 0.25, cx);
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
            .on_action(
                cx.listener(|view, _: &About, _, cx| view.navigate(Workspace::About, cx)),
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
            .on_action(cx.listener(|view, _: &StepTimeBack, window, cx| {
                view.step_time(-1, false, window, cx);
            }))
            .on_action(cx.listener(|view, _: &StepTimeForward, window, cx| {
                view.step_time(1, false, window, cx);
            }))
            .on_action(cx.listener(|view, _: &JumpNow, window, cx| {
                view.set_time(view.session.read(cx).now(), true, window, cx);
            }))
            .on_action(cx.listener(|view, _: &NewProfile, window, cx| {
                view.open_draft(ChartRequest::default(), None, window, cx);
            }))
            .on_action(cx.listener(|view, _: &TabNext, window, cx| {
                if !view.palette_open && view.active_time_popup().is_none() {
                    window.focus_next(cx);
                }
            }))
            .on_action(cx.listener(|view, _: &TabPrevious, window, cx| {
                if !view.palette_open && view.active_time_popup().is_none() {
                    window.focus_prev(cx);
                }
            }))
            .on_action(|_: &Quit, _, cx| cx.quit())
            .child(
                gpui::component::TitleBar::new().child(
                    row()
                        .w_full()
                        .pr_2()
                        .justify_between()
                        .items_center()
                        .child(
                            row()
                                .gap_2()
                                .items_center()
                                .child(img(app_icon()).w(px(18.)).h(px(18.)).flex_shrink_0())
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(INK))
                                        .child("MOTAA Desktop"),
                                ),
                        )
                        .child(caption(format!("{} · {}", route.label(), active_natal))),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .w_full()
                    .flex()
                    .child(sidebar)
                    .child(
                        column()
                            .gap_0()
                            .flex_1()
                            .min_w(px(0.))
                            .h_full()
                            .child(
                                row()
                                    .h(px(48.))
                                    .flex_shrink_0()
                                    .px_6()
                                    .gap_3()
                                    .border_b_1()
                                    .border_color(rgb(SUBTLE_LINE))
                                    .justify_between()
                                    .bg(rgb(HEADER_BG))
                                    .child(
                                        row()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .h_full()
                                            .gap_2()
                                            .overflow_hidden()
                                            .child(
                                                div()
                                                    .text_size(px(14.))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(INK))
                                            .child(route.label()),
                                    )
                                    .child(caption("·"))
                                    .child(
                                        action_control("header-active-profile", cx, |view, cx| {
                                            view.navigate(Workspace::Profiles, cx);
                                        })
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(5.))
                                        .text_size(px(13.))
                                        .text_color(rgb(GOLD))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .hover(|s| s.bg(rgb(SOFT)))
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
                                    .when(viewport_w >= 1120. && !show_inspector, |bar| {
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
                                            .h(px(26.))
                                            .px(px(10.))
                                            .gap_2()
                                            .rounded(px(RADIUS_CONTROL))
                                            .border_1()
                                            .border_color(rgb(LINE))
                                            .bg(rgb(PAPER))
                                            .shadow(pill_shadow())
                                            .hover(|s| s.bg(rgb(SOFT)))
                                            .child(ui_icon("ui/search", 12.5, MUTED))
                                            .child(caption("Quick jump"))
                                            .child(shortcut_keycap(if macos_chrome() {
                                                "⌘K"
                                            } else {
                                                "^K"
                                            })),
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
                                            use gpui::component::{
                                                Sizable as _, button::ButtonVariants as _,
                                            };
                                            let btn = gpui::component::button::Button::new(
                                                "header-inspector-toggle",
                                            )
                                            .xsmall()
                                            .tooltip(if self.inspector_open {
                                                "Hide inspector"
                                            } else {
                                                "Show inspector"
                                            })
                                            .child(ui_icon(
                                                "ui/panel-right",
                                                14.5,
                                                if self.inspector_open { PAPER } else { INK },
                                            ))
                                            .on_click(cx.listener(|view, _, _, cx| {
                                                view.inspector_open = !view.inspector_open;
                                                cx.notify();
                                            }));
                                            bar.child(if self.inspector_open {
                                                btn.primary()
                                            } else {
                                                btn.outline()
                                            })
                                        },
                                    )
                                    .when(route == Workspace::Profiles, |bar| {
                                        bar.child(self.birth_details_button(cx))
                                    }),
                            ),
                    )
                    .child(
                        {
                            use gpui::component::scroll::ScrollableElement as _;
                            div()
                                .flex_1()
                                .min_h(px(0.))
                                .w_full()
                                .flex()
                                .child(
                                    div()
                                        .relative()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .h_full()
                                        .min_h(px(0.))
                                        .child(
                                            div()
                                                .id(
                                                    if route == Workspace::Profiles
                                                        && self.show_profile_editor
                                                    {
                                                        ("profile-editor", 0)
                                                    } else {
                                                        ("workspace-content", route as usize)
                                                    },
                                                )
                                                .size_full()
                                                .overflow_y_scroll()
                                                .track_scroll(&self.workspace_scroll)
                                                .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                                                .px_6()
                                                .py_4()
                                                .child(content),
                                        )
                                        .vertical_scrollbar(&self.workspace_scroll),
                                )
                                .when(show_inspector, |body| {
                                    body.child(self.right_inspector(cx))
                                })
                        },
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
                                .bg(rgb(HEADER_BG))
                                .border_t_1()
                                .border_color(rgb(LINE))
                                .px_6()
                                .h(px(48.))
                                .child(controls),
                        )
                    }),
            ),
            )
            .when_some(self.lightbox.target, |root, target| {
                root.child(self.chart_lightbox_overlay(target, window, cx))
            })
            .when_some(self.active_map_popup(), |root, reference| {
                root.child(self.location_map_overlay(reference, cx))
            })
            .when_some(self.active_timezone_popup(), |root, reference| {
                root.child(self.timezone_overlay(reference, cx))
            })
            .when_some(self.active_town_popup(), |root, reference| {
                root.child(self.town_picker_overlay(reference, cx))
            })
            .when_some(self.active_datetime_popup(), |root, target| {
                root.child(self.datetime_picker_overlay(target, cx))
            })
            .when_some(self.active_time_popup(), |root, target| {
                root.child(self.time_picker_overlay(target, cx))
            })
            .when(self.palette_open, |root| {
                root.child(self.palette_overlay(cx))
            })
    }
}

impl DesktopRoot {
    pub(crate) fn birth_details_button(&self, cx: &mut Context<Self>) -> AnyElement {
        icon_compact_button(
            "header-birth-details",
            if self.show_birth {
                "ui/eye-off"
            } else {
                "ui/eye"
            },
            13.5,
            if self.show_birth {
                "Hide birth details"
            } else {
                "Show birth details"
            },
            cx,
            |view, _| view.show_birth = !view.show_birth,
        )
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
        .bg(rgb(SOFT))
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
    gpui::application()
        .with_assets(DesktopAssets)
        .run(move |cx: &mut App| {
            gpui::init(cx);
            apply_gold_heritage_theme(cx);
            cx.set_menus(vec![gpui::Menu {
                name: "MOTAA Desktop".into(),
                disabled: false,
                items: vec![
                    gpui::MenuItem::action("About MOTAA Desktop", About),
                    gpui::MenuItem::separator(),
                    gpui::MenuItem::action("Quit MOTAA Desktop", Quit),
                ],
            }]);
            let key = |suffix: &str| -> &'static str {
                Box::leak(primary_keybinding(suffix).into_boxed_str())
            };
            cx.bind_keys([
                KeyBinding::new(key("1"), Charts, None),
                KeyBinding::new(key("2"), Profiles, None),
                KeyBinding::new(key("3"), Calculations, None),
                KeyBinding::new(key("4"), TimeAnalysis, None),
                KeyBinding::new(key("5"), Advanced, None),
                KeyBinding::new(key("6"), About, None),
                KeyBinding::new(key("k"), TogglePalette, None),
                KeyBinding::new(key("b"), ToggleSidebar, None),
                KeyBinding::new(key("i"), ToggleInspector, None),
                KeyBinding::new(key("["), StepTimeBack, None),
                KeyBinding::new(key("]"), StepTimeForward, None),
                KeyBinding::new(key("shift-n"), JumpNow, None),
                KeyBinding::new(key("n"), NewProfile, None),
                KeyBinding::new("tab", TabNext, None),
                KeyBinding::new("shift-tab", TabPrevious, None),
                KeyBinding::new(key("q"), Quit, None),
            ]);
            let window_size = if store_screenshots.is_some() {
                if macos_chrome() {
                    size(px(1440.), px(900.))
                } else {
                    size(px(1440.), px(868.))
                }
            } else {
                size(px(1280.), px(800.))
            };
            let bounds = Bounds::centered(None, window_size, cx);
            let window_options = WindowOptions {
                show: !smoke,
                focus: !smoke,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1024.), px(680.))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("MOTAA Desktop".into()),
                    ..gpui::component::TitleBar::title_bar_options()
                }),
                ..gpui::component::TitleBar::window_options()
            };
            let (window, root_entity) = gpui::open_window(window_options, cx, |window, cx| {
                let root = cx.new(|cx| DesktopRoot::new(store, window, cx));
                window.focus(&root.read(cx).focus.clone(), cx);
                root
            })
            .expect("open MOTAA Desktop");
            if !smoke {
                cx.activate(true);
            }
            cx.on_action(|_: &Quit, cx| cx.quit());
            let about_root = root_entity.clone();
            cx.on_action(move |_: &About, cx| {
                about_root.update(cx, |root, cx| {
                    root.navigate(Workspace::About, cx);
                });
            });
            #[cfg(target_os = "macos")]
            if !smoke
                && !visual_capture
                && let Some(mut rx) = mac_pinch::install()
            {
                let pinch_root = root_entity.clone();
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
                        pinch_root.update(cx, |root, cx| {
                            if root.lightbox.target.is_some() {
                                let factor = (1. + total as f32).clamp(0.5, 2.0);
                                root.set_lightbox_zoom(root.lightbox.zoom * factor, cx);
                            }
                        });
                    }
                })
                .detach();
            }
            harness::run(
                smoke,
                store_screenshots,
                visual_capture,
                window,
                root_entity,
                cx,
            );
            cx.on_window_closed(|cx, _| {
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
