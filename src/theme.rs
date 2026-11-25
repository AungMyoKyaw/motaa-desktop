use gpui::{AnyElement, Div, SharedString, div, prelude::*, px, rgb};

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

pub fn section_label(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(MUTED))
        .child(value.into())
}

pub fn heading(value: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(22.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(INK))
        .child(value.into())
}

pub fn panel() -> Div {
    column()
        .p_4()
        .border_1()
        .border_color(rgb(LINE))
        .rounded(px(RADIUS_PANEL))
        .bg(rgb(PAPER))
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
        .min_h(px(26.))
        .px_2()
        .py(px(4.))
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
