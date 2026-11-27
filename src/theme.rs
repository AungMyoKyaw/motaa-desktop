use gpui::{AnyElement, BoxShadow, Div, SharedString, div, point, prelude::*, px, rgb, rgba};

// Observatory Manuscript (Light-only Heritage Gold) tokens.
pub const GOLD: u32 = 0x7c5b21;
pub const ACTION_HOVER: u32 = 0x694a17;
pub const ACTION_PRESSED: u32 = 0x523a13;
pub const SOFT: u32 = 0xf6f1e8;
pub const IVORY: u32 = 0xf8f5ee;
pub const RAIL: u32 = 0xf1ecdf;
pub const PAPER: u32 = 0xfffefc;
pub const HEADER_BG: u32 = 0xf9f7f2;
pub const SURFACE_HOVER: u32 = 0xf3ede0;
pub const INK: u32 = 0x28251f;
pub const MUTED: u32 = 0x686053;
pub const LINE: u32 = 0xd9cdbb;
pub const SUBTLE_LINE: u32 = 0xeae5dc;
pub const SELECTED: u32 = 0xede0bf;

pub const ERROR: u32 = 0xb42318;
pub const ERROR_BG: u32 = 0xfef3f2;
pub const BACKDROP: u32 = 0x28251f70;
pub const BACKDROP_LIGHT: u32 = 0x28251f66;
pub const BACKDROP_DARK: u32 = 0x1f1d1988;
pub const WHITE: u32 = 0xffffff;
pub const SCROLL_THUMB_MUTED: u32 = 0x68605366;
pub const SELECTION_BG: u32 = 0x8b6b2e45;
pub const TRANSPARENT: u32 = 0x00000000;
pub const ORBIT_ACCENT: u32 = 0xaa6500;

pub const RADIUS_CONTROL: f32 = 6.;
pub const RADIUS_FIELD: f32 = 5.;
pub const RADIUS_NAV: f32 = 7.;
pub const RADIUS_PANEL: f32 = 12.;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Gold,
    Error,
}

impl Tone {
    pub fn colors(self) -> (u32, u32, u32) {
        match self {
            Self::Neutral => (SOFT, MUTED, LINE),
            Self::Gold => (SELECTED, GOLD, GOLD),
            Self::Error => (ERROR_BG, ERROR, 0xf5c2be),
        }
    }
}

pub fn primary_keybinding(key: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("cmd-{key}")
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!("ctrl-{key}")
    }
}

pub fn zoom_scroll_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "pinch/Cmd+scroll"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+scroll"
    }
}

pub fn row() -> Div {
    div().flex().items_center().gap_2()
}

pub fn column() -> Div {
    div().flex().flex_col().gap_3()
}

pub fn caption(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(12.))
        .text_color(rgb(MUTED))
        .child(value.into())
}

pub fn card_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: rgba(0x28251f0a).into(),
            offset: point(px(0.), px(1.)),
            blur_radius: px(2.),
            spread_radius: px(0.),
        },
        BoxShadow {
            color: rgba(0x28251f08).into(),
            offset: point(px(0.), px(4.)),
            blur_radius: px(12.),
            spread_radius: px(-2.),
        },
    ]
}

pub fn pill_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x28251f14).into(),
        offset: point(px(0.), px(1.)),
        blur_radius: px(2.),
        spread_radius: px(0.),
    }]
}

pub fn popover_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: rgba(0x28251f14).into(),
            offset: point(px(0.), px(2.)),
            blur_radius: px(6.),
            spread_radius: px(0.),
        },
        BoxShadow {
            color: rgba(0x28251f1c).into(),
            offset: point(px(0.), px(12.)),
            blur_radius: px(28.),
            spread_radius: px(-4.),
        },
    ]
}

pub fn plate_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: rgba(0x28251f0c).into(),
            offset: point(px(0.), px(2.)),
            blur_radius: px(4.),
            spread_radius: px(0.),
        },
        BoxShadow {
            color: rgba(0x28251f10).into(),
            offset: point(px(0.), px(10.)),
            blur_radius: px(24.),
            spread_radius: px(-4.),
        },
    ]
}

pub fn section_label(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(10.5))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(MUTED))
        .child(value.into())
}

pub fn heading(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(21.))
        .line_height(px(26.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(INK))
        .child(value.into())
}

pub fn panel() -> Div {
    column()
        .p_4()
        .border_1()
        .border_color(rgb(SUBTLE_LINE))
        .rounded(px(RADIUS_PANEL))
        .bg(rgb(PAPER))
        .shadow(card_shadow())
}

pub fn chart_plate() -> Div {
    column()
        .gap_3()
        .p_4()
        .border_1()
        .border_color(rgb(LINE))
        .rounded(px(RADIUS_PANEL))
        .bg(rgb(PAPER))
        .shadow(plate_shadow())
}

pub fn segmented_track() -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .p(px(3.))
        .rounded(px(8.))
        .bg(rgb(SOFT))
        .border_1()
        .border_color(rgb(SUBTLE_LINE))
}

pub fn shortcut_keycap(label: impl Into<SharedString>) -> Div {
    div()
        .px(px(5.))
        .py(px(1.))
        .rounded(px(4.))
        .bg(rgb(SOFT))
        .border_1()
        .border_color(rgb(LINE))
        .font_family("Menlo")
        .text_size(px(10.))
        .text_color(rgb(MUTED))
        .child(label.into())
}

fn scrollbar_metrics(handle: &gpui::ScrollHandle, track_height: f32) -> (f32, f32, f32) {
    let max_y = f32::from(handle.max_offset().height).max(0.0);
    let viewport_h = f32::from(handle.bounds().size.height).max(track_height);
    let scroll_y = (-f32::from(handle.offset().y)).clamp(0.0, max_y);
    let thumb_h = ((viewport_h / (viewport_h + max_y)) * track_height).clamp(28.0, track_height);
    let thumb_y = if max_y > 0.5 {
        (scroll_y / max_y) * (track_height - thumb_h)
    } else {
        0.0
    };
    (max_y, thumb_h, thumb_y)
}

pub fn vertical_scrollbar(handle: &gpui::ScrollHandle, track_height: f32) -> Div {
    let (max_y, thumb_h, thumb_y) = scrollbar_metrics(handle, track_height);
    let mut track = div()
        .relative()
        .w(px(8.))
        .h(px(track_height))
        .rounded(px(4.))
        .bg(rgb(SOFT))
        .border_1()
        .border_color(rgb(SUBTLE_LINE));
    if max_y > 0.5 {
        track = track.child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(thumb_y))
                .w(px(6.))
                .h(px(thumb_h))
                .rounded(px(3.))
                .bg(rgb(MUTED)),
        );
    }
    track
}

pub fn slim_vertical_scrollbar(handle: &gpui::ScrollHandle, track_height: f32) -> Div {
    let (max_y, thumb_h, thumb_y) = scrollbar_metrics(handle, track_height);
    let mut track = div().relative().w(px(6.)).h(px(track_height)).flex_none();
    if max_y > 0.5 {
        track = track.child(
            div()
                .absolute()
                .left(px(1.))
                .top(px(thumb_y))
                .w(px(4.))
                .h(px(thumb_h))
                .rounded(px(2.))
                .bg(rgba(SCROLL_THUMB_MUTED)),
        );
    }
    track
}

pub fn badge(label: impl Into<SharedString>, tone: Tone) -> Div {
    let (bg, fg, border) = tone.colors();
    div()
        .flex()
        .items_center()
        .px_2()
        .py(px(2.))
        .rounded(px(RADIUS_FIELD))
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(bg))
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(fg))
        .child(label.into())
}

pub fn chip(label: impl Into<SharedString>, active: bool) -> Div {
    badge(label, if active { Tone::Gold } else { Tone::Neutral })
}

pub fn table_header_row<const N: usize>(cols: [&'static str; N]) -> Div {
    row()
        .w_full()
        .gap_3()
        .min_h(px(26.))
        .px_2()
        .py_1()
        .bg(rgb(SOFT))
        .border_b_1()
        .border_color(rgb(LINE))
        .children(cols.into_iter().map(|col| {
            div()
                .flex_1()
                .min_w(px(0.))
                .overflow_hidden()
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(MUTED))
                .child(col)
        }))
}

pub fn tabular_row<const N: usize>(values: [String; N], striped: bool) -> Div {
    row()
        .w_full()
        .gap_3()
        .min_h(px(28.))
        .px_2()
        .py(px(5.))
        .bg(rgb(if striped { SOFT } else { PAPER }))
        .border_b_1()
        .border_color(rgb(SUBTLE_LINE))
        .children(values.into_iter().enumerate().map(|(index, value)| {
            let cell = div()
                .flex_1()
                .min_w(px(0.))
                .overflow_hidden()
                .text_size(px(12.5))
                .text_color(rgb(INK));
            if index > 0 {
                cell.font_family("Menlo").child(value)
            } else {
                cell.child(value)
            }
        }))
}

pub fn empty_state(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    action: Option<AnyElement>,
) -> Div {
    let mut box_el = panel()
        .items_center()
        .justify_center()
        .py_8()
        .gap_2()
        .child(
            div()
                .text_size(px(15.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(INK))
                .child(title.into()),
        )
        .child(caption(detail.into()));
    if let Some(action) = action {
        box_el = box_el.child(div().pt_2().child(action));
    }
    box_el
}

pub fn error_banner(message: impl Into<SharedString>) -> Div {
    let (bg, fg, border) = Tone::Error.colors();
    row()
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(RADIUS_CONTROL))
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(bg))
        .text_size(px(12.5))
        .text_color(rgb(fg))
        .child(message.into())
}
