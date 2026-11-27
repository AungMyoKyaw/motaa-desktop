use gpui::{Context, Div, Entity, SharedString, div, prelude::*, px, rgb};
use jiff::{
    ToSpan,
    civil::{Date, DateTime},
};

use crate::{
    DesktopRoot, GOLD, INK, LINE, MUTED, PAPER, SELECTED, action_control, button, caption, column,
    format_time, input::TextInput, panel, row, text_input,
};

const SOFT: u32 = 0xf6f1e8;

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

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub type PickerSelector = fn(&mut DesktopRoot) -> Option<&mut DateTimePicker>;
pub type PickerChangeCallback = fn(&mut DesktopRoot, DateTime, &mut Context<DesktopRoot>);
pub type ToggleCallback = fn(&mut DesktopRoot, &mut Context<DesktopRoot>);

fn parse_time_only(text: &str) -> Result<jiff::civil::Time, String> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSegment {
    Hour,
    Minute,
    Second,
}

pub struct DateTimePicker {
    pub id_prefix: &'static str,
    pub datetime: Entity<TextInput>,
    pub date_input: Entity<TextInput>,
    pub time_input: Entity<TextInput>,
    pub view_year: i32,
    pub view_month: u32,
    pub open: bool,
    pub time_open: bool,
    pub time_focus: gpui::FocusHandle,
    pub time_segment: TimeSegment,
    pub time_digit_buf: Option<u8>,
    pub time_snapshot: Option<DateTime>,
    pub hour_scroll: gpui::ScrollHandle,
    pub minute_scroll: gpui::ScrollHandle,
    pub second_scroll: gpui::ScrollHandle,
    last_datetime_text: String,
    last_date_text: String,
    last_time_text: String,
    parts_error: Option<String>,
}

impl DateTimePicker {
    pub fn new(
        id_prefix: &'static str,
        initial: DateTime,
        open: bool,
        cx: &mut Context<DesktopRoot>,
    ) -> Self {
        let formatted = format_time(initial);
        let (date_text, time_text) = Self::format_date_time(initial);
        let datetime = text_input(&formatted, "YYYY-MM-DD HH:MM:SS", cx);
        let date_input = text_input(&date_text, "YYYY-MM-DD", cx);
        let time_input = text_input(&time_text, "HH:MM:SS", cx);

        for input in [&datetime, &date_input, &time_input] {
            cx.observe(input, |root, _, cx| {
                root.sync_datetime_pickers(cx);
                cx.notify();
            })
            .detach();
        }

        Self {
            id_prefix,
            datetime,
            date_input,
            time_input,
            view_year: i32::from(initial.year()),
            view_month: u32::from(initial.month() as u8),
            open,
            time_open: false,
            time_focus: cx.focus_handle(),
            time_segment: TimeSegment::Hour,
            time_digit_buf: None,
            time_snapshot: None,
            hour_scroll: gpui::ScrollHandle::new(),
            minute_scroll: gpui::ScrollHandle::new(),
            second_scroll: gpui::ScrollHandle::new(),
            last_datetime_text: formatted,
            last_date_text: date_text,
            last_time_text: time_text,
            parts_error: None,
        }
    }

    pub fn sync_time_scrolls(&self, dt: DateTime) {
        self.hour_scroll.set_offset(gpui::point(
            px(0.),
            px(-time_scroll_offset_for(dt.hour(), 24)),
        ));
        self.minute_scroll.set_offset(gpui::point(
            px(0.),
            px(-time_scroll_offset_for(dt.minute(), 60)),
        ));
        self.second_scroll.set_offset(gpui::point(
            px(0.),
            px(-time_scroll_offset_for(dt.second(), 60)),
        ));
    }

    fn format_date_time(dt: DateTime) -> (String, String) {
        (
            format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
            format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second()),
        )
    }

    pub fn current_datetime(&self, cx: &gpui::App) -> DateTime {
        parse_flexible_datetime(self.datetime.read(cx).value(), true).unwrap_or_else(|_| {
            let date = Date::strptime("%Y-%m-%d", self.date_input.read(cx).value().trim())
                .unwrap_or_else(|_| Date::constant(2000, 1, 1));
            let time = parse_time_only(self.time_input.read(cx).value())
                .unwrap_or_else(|_| jiff::civil::Time::constant(12, 0, 0, 0));
            date.to_datetime(time)
        })
    }

    fn sync_date_time_inputs(
        &self,
        date_text: &str,
        time_text: &str,
        cx: &mut Context<DesktopRoot>,
    ) {
        if self.date_input.read(cx).value() != date_text {
            let next = date_text.to_owned();
            self.date_input
                .update(cx, |input, cx| input.set_value(next, cx));
        }
        if self.time_input.read(cx).value() != time_text {
            let next = time_text.to_owned();
            self.time_input
                .update(cx, |input, cx| input.set_value(next, cx));
        }
    }

    pub fn set_datetime(&mut self, dt: DateTime, cx: &mut Context<DesktopRoot>) {
        self.view_year = i32::from(dt.year());
        self.view_month = u32::from(dt.month() as u8);
        self.parts_error = None;
        let formatted = format_time(dt);
        let (date_text, time_text) = Self::format_date_time(dt);
        self.last_datetime_text = formatted.clone();
        self.last_date_text = date_text.clone();
        self.last_time_text = time_text.clone();

        if self.datetime.read(cx).value() != formatted {
            self.datetime
                .update(cx, |input, cx| input.set_value(formatted, cx));
        }
        self.sync_date_time_inputs(&date_text, &time_text, cx);
    }

    pub fn sync_from_inputs(&mut self, cx: &mut Context<DesktopRoot>) {
        let dt_text = self.datetime.read(cx).value().to_owned();
        if dt_text != self.last_datetime_text {
            self.last_datetime_text = dt_text.clone();
            if let Ok(dt) = parse_flexible_datetime(&dt_text, true) {
                self.view_year = i32::from(dt.year());
                self.view_month = u32::from(dt.month() as u8);
                self.parts_error = None;
                let (date_text, time_text) = Self::format_date_time(dt);
                self.last_date_text = date_text.clone();
                self.last_time_text = time_text.clone();
                self.sync_date_time_inputs(&date_text, &time_text, cx);
            } else {
                self.parts_error = None;
            }
            return;
        }

        let date_text = self.date_input.read(cx).value().to_owned();
        let time_text = self.time_input.read(cx).value().to_owned();
        if date_text == self.last_date_text && time_text == self.last_time_text {
            return;
        }
        self.last_date_text = date_text.clone();
        self.last_time_text = time_text.clone();

        let parsed = (|| -> Result<DateTime, String> {
            let date = Date::strptime("%Y-%m-%d", date_text.trim())
                .map_err(|_| "Enter date as YYYY-MM-DD.".to_owned())?;
            let time = parse_time_only(&time_text)?;
            Ok(date.to_datetime(time))
        })();

        match parsed {
            Ok(dt) => {
                self.view_year = i32::from(dt.year());
                self.view_month = u32::from(dt.month() as u8);
                self.parts_error = None;
                let formatted = format_time(dt);
                self.last_datetime_text = formatted.clone();
                if self.datetime.read(cx).value() != formatted {
                    self.datetime
                        .update(cx, |input, cx| input.set_value(formatted, cx));
                }
            }
            Err(error) => {
                self.parts_error = Some(error);
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(error) = &self.parts_error {
            return Err(error.clone());
        }
        Ok(())
    }

    pub fn set_year_month(&mut self, year: i32, month: u32, cx: &mut Context<DesktopRoot>) {
        let current = self.current_datetime(cx);
        let year = year.clamp(1, 9999);
        let month = month.clamp(1, 12);
        let day = u32::from(current.day() as u8).min(days_in_month(year, month));
        if let Ok(y) = i16::try_from(year)
            && let Ok(date) = Date::new(y, month as i8, day as i8)
        {
            self.set_datetime(
                date.at(current.hour(), current.minute(), current.second(), 0),
                cx,
            );
        }
    }

    pub fn shift_month(&mut self, delta_months: i32, cx: &mut Context<DesktopRoot>) {
        let total = self.view_year * 12 + (self.view_month as i32 - 1) + delta_months;
        let year = total.div_euclid(12).clamp(1, 9999);
        let month = (total.rem_euclid(12) + 1) as u32;
        self.set_year_month(year, month, cx);
    }

    pub fn select_calendar_date(&mut self, date: Date, cx: &mut Context<DesktopRoot>) {
        let current = self.current_datetime(cx);
        self.set_datetime(
            date.at(current.hour(), current.minute(), current.second(), 0),
            cx,
        );
    }

    pub fn set_hms(&mut self, hour: u32, minute: u32, second: u32, cx: &mut Context<DesktopRoot>) {
        let current = self.current_datetime(cx);
        let dt = current.date().at(
            hour.min(23) as i8,
            minute.min(59) as i8,
            second.min(59) as i8,
            0,
        );
        self.set_datetime(dt, cx);
    }

    pub fn render(
        &self,
        label: &'static str,
        unknown_time: bool,
        on_toggle_unknown: Option<ToggleCallback>,
        on_change: Option<PickerChangeCallback>,
        get_mut: PickerSelector,
        cx: &mut Context<DesktopRoot>,
    ) -> Div {
        let prefix = self.id_prefix;
        let current = self.current_datetime(cx);
        let hour24 = current.hour();
        let is_pm = hour24 >= 12;

        let mut header_actions = row().when(!unknown_time, |bar| {
            bar.child(small_action_button(
                (prefix, "now"),
                "Now",
                false,
                cx,
                move |view, cx| {
                    let now = view.session.read(cx).now();
                    if let Some(picker) = get_mut(view) {
                        picker.set_datetime(now, cx);
                        if let Some(cb) = on_change {
                            cb(view, now, cx);
                        }
                    }
                },
            ))
        });

        if let Some(toggle_unknown) = on_toggle_unknown {
            header_actions = header_actions.child(small_action_button(
                (prefix, "unknown-time"),
                "Unknown birth time",
                unknown_time,
                cx,
                toggle_unknown,
            ));
        }

        let date_col = column().gap_1().child(caption("Date (YYYY-MM-DD)")).child(
            row()
                .w_full()
                .gap_2()
                .child(div().flex_1().min_w(px(0.)).child(self.date_input.clone()))
                .child(
                    action_control(
                        SharedString::from(format!("{prefix}-toggle-open")),
                        cx,
                        move |view, _| {
                            if let Some(picker) = get_mut(view) {
                                picker.open = !picker.open;
                            }
                        },
                    )
                    .px_3()
                    .h(px(36.))
                    .justify_center()
                    .rounded(px(5.))
                    .border_1()
                    .border_color(rgb(if self.open { GOLD } else { LINE }))
                    .bg(rgb(if self.open { SELECTED } else { PAPER }))
                    .text_color(rgb(INK))
                    .text_size(px(12.5))
                    .hover(|style| style.bg(rgb(SELECTED)))
                    .child("Calendar"),
                ),
        );

        let time_col = column()
            .gap_1()
            .child(caption("Time (HH:MM:SS)"))
            .child(if unknown_time {
                div()
                    .h(px(36.))
                    .px_3()
                    .flex()
                    .items_center()
                    .rounded(px(5.))
                    .border_1()
                    .border_color(rgb(LINE))
                    .bg(rgb(SOFT))
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child("Birth time unknown")
                    .into_any_element()
            } else {
                row()
                    .w_full()
                    .gap_2()
                    .child(div().flex_1().min_w(px(0.)).child(self.time_input.clone()))
                    .child(
                        row()
                            .gap_1()
                            .child(small_action_button(
                                (prefix, "am"),
                                "AM",
                                !is_pm,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        let cur = picker.current_datetime(cx);
                                        if cur.hour() >= 12 {
                                            picker.set_hms(
                                                (cur.hour() - 12) as u32,
                                                cur.minute() as u32,
                                                cur.second() as u32,
                                                cx,
                                            );
                                            let dt = picker.current_datetime(cx);
                                            if let Some(cb) = on_change {
                                                cb(view, dt, cx);
                                            }
                                        }
                                    }
                                },
                            ))
                            .child(small_action_button(
                                (prefix, "pm"),
                                "PM",
                                is_pm,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        let cur = picker.current_datetime(cx);
                                        if cur.hour() < 12 {
                                            picker.set_hms(
                                                (cur.hour() + 12) as u32,
                                                cur.minute() as u32,
                                                cur.second() as u32,
                                                cx,
                                            );
                                            let dt = picker.current_datetime(cx);
                                            if let Some(cb) = on_change {
                                                cb(view, dt, cx);
                                            }
                                        }
                                    }
                                },
                            )),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("{prefix}-pick-time")))
                            .focusable()
                            .tab_stop(true)
                            .cursor_pointer()
                            .on_click(cx.listener(move |view, _, window, cx| {
                                if let Some(picker) = get_mut(view) {
                                    let current = picker.current_datetime(cx);
                                    picker.time_snapshot = Some(current);
                                    picker.time_segment = TimeSegment::Hour;
                                    picker.time_digit_buf = None;
                                    picker.time_open = true;
                                    picker.open = false;
                                    picker.sync_time_scrolls(current);
                                    let focus = picker.time_focus.clone();
                                    window.focus(&focus);
                                }
                                cx.notify();
                            }))
                            .on_key_down(cx.listener(
                                move |view, event: &gpui::KeyDownEvent, window, cx| {
                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                        if let Some(picker) = get_mut(view) {
                                            let current = picker.current_datetime(cx);
                                            picker.time_snapshot = Some(current);
                                            picker.time_segment = TimeSegment::Hour;
                                            picker.time_digit_buf = None;
                                            picker.time_open = true;
                                            picker.open = false;
                                            picker.sync_time_scrolls(current);
                                            let focus = picker.time_focus.clone();
                                            window.focus(&focus);
                                        }
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                },
                            ))
                            .h(px(36.))
                            .px_3()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(6.))
                            .bg(rgb(SOFT))
                            .border_1()
                            .border_color(rgb(LINE))
                            .focus(|style| style.border_color(rgb(GOLD)))
                            .hover(|style| style.bg(rgb(crate::SURFACE_HOVER)))
                            .text_size(px(12.))
                            .text_color(rgb(INK))
                            .child("Pick time"),
                    )
                    .into_any_element()
            });

        column()
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .flex_wrap()
                    .gap_2()
                    .child(caption(label))
                    .child(header_actions),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_3()
                    .child(date_col)
                    .child(time_col),
            )
            .when_some(self.parts_error.clone(), |view, err| {
                view.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0xb42318))
                        .child(err),
                )
            })
    }

    fn render_calendar_popup_body(
        &self,
        current: DateTime,
        unknown_time: bool,
        on_change: Option<PickerChangeCallback>,
        get_mut: PickerSelector,
        cx: &mut Context<DesktopRoot>,
    ) -> Div {
        let prefix = self.id_prefix;
        let view_year = self.view_year;
        let view_month = self.view_month;

        // Build 42-day calendar grid (6 rows x 7 cols) starting on Sunday
        let first_of_month = i16::try_from(view_year)
            .ok()
            .and_then(|y| Date::new(y, view_month as i8, 1).ok())
            .unwrap_or_else(|| Date::constant(2000, 1, 1));
        let start_offset = first_of_month.weekday().to_sunday_zero_offset();
        let grid_start = first_of_month
            .checked_sub(i32::from(start_offset).days())
            .unwrap_or(first_of_month);

        let calendar_cells = (0..42)
            .map(|index| {
                let cell_date = grid_start
                    .checked_add((index as i32).days())
                    .unwrap_or(first_of_month);
                let in_month = u32::from(cell_date.month() as u8) == view_month
                    && i32::from(cell_date.year()) == view_year;
                let is_selected = cell_date == current.date();
                let day_label = format!("{}", cell_date.day());

                action_control(
                    (SharedString::from(format!("{prefix}-cal-day")), index),
                    cx,
                    move |view, cx| {
                        if let Some(picker) = get_mut(view) {
                            picker.select_calendar_date(cell_date, cx);
                            let dt = picker.current_datetime(cx);
                            if let Some(cb) = on_change {
                                cb(view, dt, cx);
                            }
                        }
                    },
                )
                .h(px(30.))
                .justify_center()
                .rounded(px(5.))
                .border_1()
                .border_color(rgb(if is_selected {
                    GOLD
                } else if in_month {
                    LINE
                } else {
                    0xeae5dc
                }))
                .bg(rgb(if is_selected {
                    GOLD
                } else if in_month {
                    PAPER
                } else {
                    SOFT
                }))
                .text_color(rgb(if is_selected {
                    PAPER
                } else if in_month {
                    INK
                } else {
                    MUTED
                }))
                .text_size(px(13.))
                .hover(|style| style.bg(rgb(if is_selected { GOLD } else { SELECTED })))
                .child(day_label)
            })
            .collect::<Vec<_>>();

        let month_grid =
            div()
                .grid()
                .grid_cols(6)
                .gap_1()
                .children(MONTH_SHORT.into_iter().enumerate().map(|(idx, m_name)| {
                    let m_num = (idx + 1) as u32;
                    let active = m_num == view_month;
                    action_control(
                        (SharedString::from(format!("{prefix}-month-pill")), idx),
                        cx,
                        move |view, cx| {
                            if let Some(picker) = get_mut(view) {
                                let yr = picker.view_year;
                                picker.set_year_month(yr, m_num, cx);
                                let dt = picker.current_datetime(cx);
                                if let Some(cb) = on_change {
                                    cb(view, dt, cx);
                                }
                            }
                        },
                    )
                    .h(px(26.))
                    .justify_center()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rgb(if active { GOLD } else { LINE }))
                    .bg(rgb(if active { SELECTED } else { PAPER }))
                    .text_color(rgb(INK))
                    .text_size(px(12.))
                    .hover(|style| style.bg(rgb(SELECTED)))
                    .child(m_name)
                }));

        let calendar_column = panel()
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .child(
                        row()
                            .gap_1()
                            .child(small_action_button(
                                (prefix, "cal-prev-10y"),
                                "« 10y",
                                false,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        let yr = picker.view_year - 10;
                                        let mo = picker.view_month;
                                        picker.set_year_month(yr, mo, cx);
                                        let dt = picker.current_datetime(cx);
                                        if let Some(cb) = on_change {
                                            cb(view, dt, cx);
                                        }
                                    }
                                },
                            ))
                            .child(small_action_button(
                                (prefix, "cal-prev-mo"),
                                "‹",
                                false,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        picker.shift_month(-1, cx);
                                        let dt = picker.current_datetime(cx);
                                        if let Some(cb) = on_change {
                                            cb(view, dt, cx);
                                        }
                                    }
                                },
                            )),
                    )
                    .child(div().text_size(px(15.)).text_color(rgb(INK)).child(format!(
                        "{} {:04}",
                        MONTH_NAMES[(view_month - 1) as usize],
                        view_year
                    )))
                    .child(
                        row()
                            .gap_1()
                            .child(small_action_button(
                                (prefix, "cal-next-mo"),
                                "›",
                                false,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        picker.shift_month(1, cx);
                                        let dt = picker.current_datetime(cx);
                                        if let Some(cb) = on_change {
                                            cb(view, dt, cx);
                                        }
                                    }
                                },
                            ))
                            .child(small_action_button(
                                (prefix, "cal-next-10y"),
                                "10y »",
                                false,
                                cx,
                                move |view, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        let yr = picker.view_year + 10;
                                        let mo = picker.view_month;
                                        picker.set_year_month(yr, mo, cx);
                                        let dt = picker.current_datetime(cx);
                                        if let Some(cb) = on_change {
                                            cb(view, dt, cx);
                                        }
                                    }
                                },
                            )),
                    ),
            )
            .child(month_grid)
            .child(
                div()
                    .grid()
                    .grid_cols(7)
                    .gap_1()
                    .pt_1()
                    .children(WEEKDAYS.into_iter().map(|wd| {
                        div()
                            .flex()
                            .justify_center()
                            .text_size(px(11.))
                            .text_color(rgb(MUTED))
                            .child(wd)
                    })),
            )
            .child(div().grid().grid_cols(7).gap_1().children(calendar_cells));

        let presets = div().grid().grid_cols(4).gap_1().children(
            [
                ("Midnight", 0u32, 0u32),
                ("06:00 AM", 6, 0),
                ("12:00 PM", 12, 0),
                ("06:00 PM", 18, 0),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, (p_label, h, m))| {
                let active = !unknown_time
                    && current.hour() == h as i8
                    && current.minute() == m as i8
                    && current.second() == 0;
                action_control(
                    (SharedString::from(format!("{prefix}-preset")), idx),
                    cx,
                    move |view, cx| {
                        if let Some(picker) = get_mut(view) {
                            picker.set_hms(h, m, 0, cx);
                            let dt = picker.current_datetime(cx);
                            if let Some(cb) = on_change {
                                cb(view, dt, cx);
                            }
                        }
                    },
                )
                .h(px(28.))
                .justify_center()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(if active { GOLD } else { LINE }))
                .bg(rgb(if active { SELECTED } else { PAPER }))
                .text_color(rgb(INK))
                .text_size(px(12.))
                .hover(|style| style.bg(rgb(SELECTED)))
                .child(p_label)
            }),
        );

        let time_column = panel()
            .gap_2()
            .when(!unknown_time, |p| {
                p.child(caption("Time presets")).child(presets)
            })
            .child(
                column()
                    .gap_1()
                    .child(caption("Direct timestamp (YYYY-MM-DD HH:MM:SS)"))
                    .child(self.datetime.clone()),
            );

        column().gap_3().child(calendar_column).child(time_column)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveDateTimePicker {
    Profile,
    ReferenceTime,
    Candidate(usize),
}

impl DesktopRoot {
    pub(crate) fn active_datetime_popup(&self) -> Option<ActiveDateTimePicker> {
        if self.draft.as_ref().is_some_and(|d| d.picker.open) {
            Some(ActiveDateTimePicker::Profile)
        } else if self.time_picker.open {
            Some(ActiveDateTimePicker::ReferenceTime)
        } else {
            self.candidate_pickers
                .iter()
                .position(|p| p.open)
                .map(ActiveDateTimePicker::Candidate)
        }
    }

    pub(crate) fn close_datetime_popup(&mut self, target: ActiveDateTimePicker) {
        match target {
            ActiveDateTimePicker::Profile => {
                if let Some(d) = &mut self.draft {
                    d.picker.open = false;
                }
            }
            ActiveDateTimePicker::ReferenceTime => self.time_picker.open = false,
            ActiveDateTimePicker::Candidate(idx) => {
                if let Some(p) = self.candidate_pickers.get_mut(idx) {
                    p.open = false;
                }
            }
        }
    }

    pub(crate) fn datetime_picker_overlay(
        &self,
        target: ActiveDateTimePicker,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let (picker, unknown_time, on_change, get_mut, title): (
            &DateTimePicker,
            bool,
            Option<PickerChangeCallback>,
            PickerSelector,
            &'static str,
        ) = match target {
            ActiveDateTimePicker::Profile => {
                let Some(d) = self.draft.as_ref() else {
                    return div().into_any_element();
                };
                (
                    &d.picker,
                    d.unknown,
                    None,
                    |view| view.draft.as_mut().map(|d| &mut d.picker),
                    "Birth Date & Time",
                )
            }
            ActiveDateTimePicker::ReferenceTime => (
                &self.time_picker,
                false,
                Some(|view, dt, cx| view.set_time(dt, true, cx)),
                |view| Some(&mut view.time_picker),
                "Reference Date & Time",
            ),
            ActiveDateTimePicker::Candidate(idx) => {
                let Some(p) = self.candidate_pickers.get(idx) else {
                    return div().into_any_element();
                };
                let get_mut: PickerSelector = match idx {
                    0 => |view| view.candidate_pickers.get_mut(0),
                    1 => |view| view.candidate_pickers.get_mut(1),
                    _ => |view| view.candidate_pickers.get_mut(2),
                };
                (p, false, None, get_mut, "Candidate Date & Time")
            }
        };

        let current = picker.current_datetime(cx);
        let summary = format_time(current);

        div()
            .id("datetime-backdrop")
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x28251f70))
            .flex()
            .items_center()
            .justify_center()
            .p_6()
            .child(
                div()
                    .id("datetime-dialog")
                    .w(px(560.))
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
                            root.close_datetime_popup(target);
                            cx.stop_propagation();
                            cx.notify();
                        }
                    }))
                    .child(
                        row()
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
                                            .child(title),
                                    )
                                    .child(caption(summary)),
                            )
                            .child(button("Done", true, cx, move |root, cx| {
                                root.close_datetime_popup(target);
                                cx.notify();
                            })),
                    )
                    .child(picker.render_calendar_popup_body(
                        current,
                        unknown_time,
                        on_change,
                        get_mut,
                        cx,
                    )),
            )
            .into_any_element()
    }

    pub(crate) fn active_time_popup(&self) -> Option<ActiveDateTimePicker> {
        if self.draft.as_ref().is_some_and(|d| d.picker.time_open) {
            return Some(ActiveDateTimePicker::Profile);
        }
        if self.time_picker.time_open {
            return Some(ActiveDateTimePicker::ReferenceTime);
        }
        self.candidate_pickers
            .iter()
            .position(|p| p.time_open)
            .map(ActiveDateTimePicker::Candidate)
    }

    pub(crate) fn close_time_popup(
        &mut self,
        target: ActiveDateTimePicker,
        window: &mut gpui::Window,
    ) {
        let picker = match target {
            ActiveDateTimePicker::Profile => self.draft.as_mut().map(|d| &mut d.picker),
            ActiveDateTimePicker::ReferenceTime => Some(&mut self.time_picker),
            ActiveDateTimePicker::Candidate(idx) => self.candidate_pickers.get_mut(idx),
        };
        if let Some(picker) = picker {
            picker.time_open = false;
            picker.time_snapshot = None;
            picker.time_digit_buf = None;
        }
        window.focus(&self.focus);
    }

    pub(crate) fn cancel_time_popup(
        &mut self,
        target: ActiveDateTimePicker,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let orig = match target {
            ActiveDateTimePicker::Profile => self.draft.as_mut().and_then(|d| {
                d.picker.time_open = false;
                d.picker.time_digit_buf = None;
                d.picker.time_snapshot.take()
            }),
            ActiveDateTimePicker::ReferenceTime => {
                self.time_picker.time_open = false;
                self.time_picker.time_digit_buf = None;
                self.time_picker.time_snapshot.take()
            }
            ActiveDateTimePicker::Candidate(idx) => {
                self.candidate_pickers.get_mut(idx).and_then(|p| {
                    p.time_open = false;
                    p.time_digit_buf = None;
                    p.time_snapshot.take()
                })
            }
        };
        if let Some(orig) = orig {
            match target {
                ActiveDateTimePicker::Profile => {
                    if let Some(d) = self.draft.as_mut() {
                        d.picker.set_datetime(orig, cx);
                    }
                }
                ActiveDateTimePicker::ReferenceTime => {
                    self.time_picker.set_datetime(orig, cx);
                    self.set_time(orig, true, cx);
                }
                ActiveDateTimePicker::Candidate(idx) => {
                    if let Some(p) = self.candidate_pickers.get_mut(idx) {
                        p.set_datetime(orig, cx);
                    }
                }
            }
        }
        window.focus(&self.focus);
    }

    pub(crate) fn time_picker_overlay(
        &self,
        target: ActiveDateTimePicker,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let (picker, on_change, get_mut, title): (
            &DateTimePicker,
            Option<PickerChangeCallback>,
            PickerSelector,
            &'static str,
        ) = match target {
            ActiveDateTimePicker::Profile => {
                let Some(d) = self.draft.as_ref() else {
                    return div().into_any_element();
                };
                (
                    &d.picker,
                    None,
                    |view| view.draft.as_mut().map(|d| &mut d.picker),
                    "Pick Birth Time",
                )
            }
            ActiveDateTimePicker::ReferenceTime => (
                &self.time_picker,
                Some(|view, dt, cx| view.set_time(dt, true, cx)),
                |view| Some(&mut view.time_picker),
                "Pick Reference Time",
            ),
            ActiveDateTimePicker::Candidate(idx) => {
                let Some(p) = self.candidate_pickers.get(idx) else {
                    return div().into_any_element();
                };
                let get_mut: PickerSelector = match idx {
                    0 => |view| view.candidate_pickers.get_mut(0),
                    1 => |view| view.candidate_pickers.get_mut(1),
                    _ => |view| view.candidate_pickers.get_mut(2),
                };
                (p, None, get_mut, "Pick Candidate Time")
            }
        };

        let current = picker.current_datetime(cx);
        let h = current.hour();
        let m = current.minute();
        let s = current.second();
        let is_pm = h >= 12;
        let h12 = match h % 12 {
            0 => 12,
            v => v,
        };
        let twelve_hour_hint = format!(
            "{h12:02}:{m:02}:{s:02} {} · {}",
            if is_pm { "PM" } else { "AM" },
            time_of_day_label(h)
        );
        let prefix = picker.id_prefix;
        let active_seg = picker.time_segment;

        let make_seg_pill = |seg: TimeSegment, id_suffix: &'static str, val: i8| {
            let is_active = active_seg == seg;
            let mut pill = div()
                .id(SharedString::from(format!("{prefix}-tp-pill-{id_suffix}")))
                .cursor_pointer()
                .px_2()
                .py(px(2.))
                .rounded(px(6.))
                .font_family("Menlo")
                .text_size(px(22.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(if is_active { rgb(GOLD) } else { rgb(INK) })
                .on_click(cx.listener(move |view, _, _, cx| {
                    if let Some(picker) = get_mut(view) {
                        picker.time_segment = seg;
                        picker.time_digit_buf = None;
                    }
                    cx.notify();
                }))
                .child(format!("{val:02}"));
            if is_active {
                pill = pill.bg(rgb(PAPER)).border_1().border_color(rgb(GOLD));
            } else {
                pill = pill
                    .bg(gpui::rgba(0x00000000))
                    .border_1()
                    .border_color(gpui::rgba(0x00000000))
                    .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)));
            }
            pill
        };

        let readout_colon = || {
            div()
                .text_size(px(18.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(MUTED))
                .child(":")
        };

        let make_period_btn =
            |id_suffix: &'static str, label: &'static str, active: bool, target_pm: bool| {
                let mut btn = div()
                    .id(SharedString::from(format!("{prefix}-tp-{id_suffix}")))
                    .cursor_pointer()
                    .px_2()
                    .py(px(4.))
                    .rounded(px(5.))
                    .text_size(px(11.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if let Some(picker) = get_mut(view) {
                            let cur = picker.current_datetime(cx);
                            let next_h = if target_pm && cur.hour() < 12 {
                                Some(cur.hour() + 12)
                            } else if !target_pm && cur.hour() >= 12 {
                                Some(cur.hour() - 12)
                            } else {
                                None
                            };
                            if let Some(nh) = next_h {
                                let next =
                                    jiff::civil::Time::new(nh, cur.minute(), cur.second(), 0)
                                        .map(|t| DateTime::from_parts(cur.date(), t))
                                        .unwrap_or(cur);
                                picker.set_datetime(next, cx);
                                picker.sync_time_scrolls(next);
                                picker.time_digit_buf = None;
                                if let Some(cb) = on_change {
                                    cb(view, next, cx);
                                }
                            }
                        }
                        cx.notify();
                    }))
                    .child(label);
                if active {
                    btn = btn.bg(rgb(GOLD)).text_color(rgb(0xffffff));
                } else {
                    btn = btn
                        .bg(gpui::rgba(0x00000000))
                        .text_color(rgb(INK))
                        .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)));
                }
                btn
            };

        let am_pm_capsule = row()
            .gap(px(2.))
            .p(px(2.))
            .rounded(px(7.))
            .bg(rgb(PAPER))
            .border_1()
            .border_color(rgb(crate::SUBTLE_LINE))
            .child(make_period_btn("am", "AM", !is_pm, false))
            .child(make_period_btn("pm", "PM", is_pm, true));

        let digital_bar = row()
            .w_full()
            .px_4()
            .py_3()
            .rounded(px(10.))
            .bg(rgb(SOFT))
            .border_1()
            .border_color(rgb(crate::SUBTLE_LINE))
            .justify_between()
            .items_center()
            .gap_3()
            .child(
                column()
                    .gap(px(4.))
                    .child(
                        row()
                            .items_center()
                            .gap_1()
                            .child(make_seg_pill(TimeSegment::Hour, "h", h))
                            .child(readout_colon())
                            .child(make_seg_pill(TimeSegment::Minute, "m", m))
                            .child(readout_colon())
                            .child(make_seg_pill(TimeSegment::Second, "s", s)),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(rgb(MUTED))
                            .child(twelve_hour_hint),
                    ),
            )
            .child(am_pm_capsule);

        let make_col_header = |seg: TimeSegment,
                               id_suffix: &'static str,
                               label: &'static str,
                               with_right_border: bool| {
            let is_active = active_seg == seg;
            let mut hdr = div()
                .id(SharedString::from(format!("{prefix}-tp-hdr-{id_suffix}")))
                .cursor_pointer()
                .py(px(7.))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(if is_active { rgb(GOLD) } else { rgb(MUTED) })
                .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)))
                .on_click(cx.listener(move |view, _, _, cx| {
                    if let Some(picker) = get_mut(view) {
                        picker.time_segment = seg;
                        picker.time_digit_buf = None;
                    }
                    cx.notify();
                }))
                .child(label);
            if with_right_border {
                hdr = hdr.border_r_1().border_color(rgb(crate::SUBTLE_LINE));
            }
            hdr
        };

        let build_col = |col_seg: TimeSegment,
                         col_id: &'static str,
                         count: i8,
                         selected: i8,
                         scroll_handle: gpui::ScrollHandle,
                         with_right_border: bool,
                         make_dt: fn(DateTime, i8) -> DateTime| {
            let is_active_col = active_seg == col_seg;
            let items: Vec<_> = (0..count)
                .map(|val| {
                    let is_sel = val == selected;
                    let mut item = div()
                        .id((
                            SharedString::from(format!("{prefix}-tp-{col_id}")),
                            usize::try_from(val).unwrap_or(0),
                        ))
                        .cursor_pointer()
                        .flex_none()
                        .w_full()
                        .h(px(30.))
                        .rounded(px(6.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family("Menlo")
                        .text_size(px(13.))
                        .on_click(cx.listener(move |view, _, _, cx| {
                            if let Some(picker) = get_mut(view) {
                                let cur = picker.current_datetime(cx);
                                let next = make_dt(cur, val);
                                picker.set_datetime(next, cx);
                                picker.time_segment = col_seg;
                                picker.time_digit_buf = None;
                                if let Some(cb) = on_change {
                                    cb(view, next, cx);
                                }
                            }
                            cx.notify();
                        }))
                        .child(format!("{val:02}"));

                    if is_sel {
                        item = item
                            .bg(rgb(GOLD))
                            .text_color(rgb(0xffffff))
                            .font_weight(gpui::FontWeight::SEMIBOLD);
                    } else {
                        item = item
                            .bg(gpui::rgba(0x00000000))
                            .text_color(rgb(INK))
                            .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)));
                    }
                    item
                })
                .collect();

            let mut col_box = row()
                .flex_1()
                .min_w(px(0.))
                .h_full()
                .p_1()
                .gap(px(2.))
                .bg(if is_active_col {
                    rgb(crate::IVORY)
                } else {
                    rgb(PAPER)
                })
                .child(
                    div()
                        .id((
                            SharedString::from(format!("{prefix}-tp-scroll-{col_id}")),
                            0,
                        ))
                        .flex_1()
                        .min_w(px(0.))
                        .h_full()
                        .overflow_y_scroll()
                        .track_scroll(&scroll_handle)
                        .on_scroll_wheel(cx.listener(|_, _, _, cx| {
                            cx.stop_propagation();
                            cx.notify();
                        }))
                        .child(div().w_full().flex().flex_col().gap(px(2.)).children(items)),
                )
                .child(slim_vertical_scrollbar(&scroll_handle, 210.0));
            if with_right_border {
                col_box = col_box.border_r_1().border_color(rgb(crate::SUBTLE_LINE));
            }
            col_box
        };

        let hour_col = build_col(
            TimeSegment::Hour,
            "hour",
            24,
            h,
            picker.hour_scroll.clone(),
            true,
            |dt, next_h| {
                jiff::civil::Time::new(next_h, dt.minute(), dt.second(), 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );
        let minute_col = build_col(
            TimeSegment::Minute,
            "min",
            60,
            m,
            picker.minute_scroll.clone(),
            true,
            |dt, next_m| {
                jiff::civil::Time::new(dt.hour(), next_m, dt.second(), 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );
        let second_col = build_col(
            TimeSegment::Second,
            "sec",
            60,
            s,
            picker.second_scroll.clone(),
            false,
            |dt, next_s| {
                jiff::civil::Time::new(dt.hour(), dt.minute(), next_s, 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );

        let columns_frame = column()
            .w_full()
            .gap_0()
            .rounded(px(10.))
            .bg(rgb(PAPER))
            .border_1()
            .border_color(rgb(LINE))
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .grid()
                    .grid_cols(3)
                    .bg(rgb(crate::HEADER_BG))
                    .border_b_1()
                    .border_color(rgb(crate::SUBTLE_LINE))
                    .child(make_col_header(TimeSegment::Hour, "hour", "HOUR", true))
                    .child(make_col_header(TimeSegment::Minute, "min", "MIN", true))
                    .child(make_col_header(TimeSegment::Second, "sec", "SEC", false)),
            )
            .child(
                row()
                    .w_full()
                    .gap_0()
                    .h(px(218.))
                    .child(hour_col)
                    .child(minute_col)
                    .child(second_col),
            );

        let nudges = div().w_full().grid().grid_cols(6).gap_1().children(
            [
                ("-15m", -900i32),
                ("-5m", -300),
                ("-1m", -60),
                ("+1m", 60),
                ("+5m", 300),
                ("+15m", 900),
            ]
            .map(|(label, delta)| {
                div()
                    .id(SharedString::from(format!("{prefix}-tp-nudge-{label}")))
                    .cursor_pointer()
                    .h(px(26.))
                    .rounded(px(5.))
                    .bg(rgb(SOFT))
                    .border_1()
                    .border_color(rgb(crate::SUBTLE_LINE))
                    .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family("Menlo")
                    .text_size(px(11.))
                    .text_color(rgb(INK))
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if let Some(picker) = get_mut(view) {
                            let cur = picker.current_datetime(cx);
                            let next = nudge_time_seconds(cur, delta);
                            picker.set_datetime(next, cx);
                            picker.sync_time_scrolls(next);
                            picker.time_digit_buf = None;
                            if let Some(cb) = on_change {
                                cb(view, next, cx);
                            }
                        }
                        cx.notify();
                    }))
                    .child(label)
            }),
        );

        let presets = div().w_full().grid().grid_cols(4).gap_2().children(
            [("Midnight", 0i8), ("06:00", 6), ("Noon", 12), ("18:00", 18)].map(|(label, hour)| {
                let active = h == hour && m == 0 && s == 0;
                let mut btn = div()
                    .id(SharedString::from(format!("{prefix}-tp-preset-{label}")))
                    .cursor_pointer()
                    .h(px(30.))
                    .rounded(px(6.))
                    .border_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(12.))
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if let Some(picker) = get_mut(view) {
                            let cur = picker.current_datetime(cx);
                            let next = jiff::civil::Time::new(hour, 0, 0, 0)
                                .map(|t| DateTime::from_parts(cur.date(), t))
                                .unwrap_or(cur);
                            picker.set_datetime(next, cx);
                            picker.sync_time_scrolls(next);
                            picker.time_digit_buf = None;
                            if let Some(cb) = on_change {
                                cb(view, next, cx);
                            }
                        }
                        cx.notify();
                    }))
                    .child(label);
                if active {
                    btn = btn
                        .bg(rgb(SELECTED))
                        .border_color(rgb(GOLD))
                        .text_color(rgb(GOLD))
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else {
                    btn = btn
                        .bg(rgb(PAPER))
                        .border_color(rgb(LINE))
                        .text_color(rgb(INK))
                        .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)));
                }
                btn
            }),
        );

        let cancel_btn = div()
            .id(SharedString::from(format!("{prefix}-tp-cancel")))
            .cursor_pointer()
            .px_3()
            .py(px(6.))
            .rounded(px(6.))
            .bg(rgb(PAPER))
            .border_1()
            .border_color(rgb(LINE))
            .hover(|s| s.bg(rgb(crate::SURFACE_HOVER)))
            .text_size(px(12.5))
            .text_color(rgb(INK))
            .on_click(cx.listener(move |root, _, window, cx| {
                root.cancel_time_popup(target, window, cx);
                cx.notify();
            }))
            .child("Cancel");

        let done_btn = div()
            .id(SharedString::from(format!("{prefix}-tp-done")))
            .cursor_pointer()
            .px_4()
            .py(px(6.))
            .rounded(px(6.))
            .bg(rgb(GOLD))
            .border_1()
            .border_color(rgb(GOLD))
            .hover(|s| s.bg(rgb(crate::ACTION_HOVER)))
            .text_size(px(12.5))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(0xffffff))
            .on_click(cx.listener(move |root, _, window, cx| {
                root.close_time_popup(target, window);
                cx.notify();
            }))
            .child("Done");

        let footer = row()
            .w_full()
            .pt_3()
            .border_t_1()
            .border_color(rgb(crate::SUBTLE_LINE))
            .justify_between()
            .items_center()
            .child(button("Now", false, cx, move |view, cx| {
                if let Some(picker) = get_mut(view) {
                    let now = jiff::tz::TimeZone::system().to_datetime(jiff::Timestamp::now());
                    let cur = picker.current_datetime(cx);
                    let next = jiff::civil::Time::new(now.hour(), now.minute(), now.second(), 0)
                        .map(|t| DateTime::from_parts(cur.date(), t))
                        .unwrap_or(cur);
                    picker.set_datetime(next, cx);
                    picker.sync_time_scrolls(next);
                    picker.time_digit_buf = None;
                    if let Some(cb) = on_change {
                        cb(view, next, cx);
                    }
                }
                cx.notify();
            }))
            .child(row().gap_2().child(cancel_btn).child(done_btn));

        div()
            .id("time-backdrop")
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x1f1d1988))
            .flex()
            .items_center()
            .justify_center()
            .p_4()
            .child(
                div()
                    .id("time-dialog")
                    .track_focus(&picker.time_focus)
                    .w(px(412.))
                    .max_w_full()
                    .rounded(px(14.))
                    .bg(rgb(PAPER))
                    .border_1()
                    .border_color(rgb(LINE))
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .on_key_down(cx.listener(
                        move |root, event: &gpui::KeyDownEvent, window, cx| {
                            let key = event.keystroke.key.as_str();
                            let mods = &event.keystroke.modifiers;
                            if key == "escape" {
                                root.cancel_time_popup(target, window, cx);
                                cx.stop_propagation();
                                cx.notify();
                                return;
                            }
                            if key == "enter" {
                                root.close_time_popup(target, window);
                                cx.stop_propagation();
                                cx.notify();
                                return;
                            }
                            if let Some(picker) = get_mut(root) {
                                match key {
                                    "up" | "down" => {
                                        let delta = if key == "up" { 1 } else { -1 };
                                        let cur = picker.current_datetime(cx);
                                        let next =
                                            step_time_segment(cur, picker.time_segment, delta);
                                        picker.set_datetime(next, cx);
                                        picker.sync_time_scrolls(next);
                                        picker.time_digit_buf = None;
                                        if let Some(cb) = on_change {
                                            cb(root, next, cx);
                                        }
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    "left" => {
                                        picker.time_segment = match picker.time_segment {
                                            TimeSegment::Hour => TimeSegment::Second,
                                            TimeSegment::Minute => TimeSegment::Hour,
                                            TimeSegment::Second => TimeSegment::Minute,
                                        };
                                        picker.time_digit_buf = None;
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    "right" => {
                                        picker.time_segment = match picker.time_segment {
                                            TimeSegment::Hour => TimeSegment::Minute,
                                            TimeSegment::Minute => TimeSegment::Second,
                                            TimeSegment::Second => TimeSegment::Hour,
                                        };
                                        picker.time_digit_buf = None;
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    "tab" => {
                                        picker.time_segment = if mods.shift {
                                            match picker.time_segment {
                                                TimeSegment::Hour => TimeSegment::Second,
                                                TimeSegment::Minute => TimeSegment::Hour,
                                                TimeSegment::Second => TimeSegment::Minute,
                                            }
                                        } else {
                                            match picker.time_segment {
                                                TimeSegment::Hour => TimeSegment::Minute,
                                                TimeSegment::Minute => TimeSegment::Second,
                                                TimeSegment::Second => TimeSegment::Hour,
                                            }
                                        };
                                        picker.time_digit_buf = None;
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    _ if !mods.control
                                        && !mods.platform
                                        && !mods.alt
                                        && key.len() == 1
                                        && key.as_bytes()[0].is_ascii_digit() =>
                                    {
                                        let digit = key.as_bytes()[0] - b'0';
                                        let cur = picker.current_datetime(cx);
                                        let (next, next_buf, next_seg) = apply_time_digit(
                                            cur,
                                            picker.time_segment,
                                            picker.time_digit_buf,
                                            digit,
                                        );
                                        picker.time_digit_buf = next_buf;
                                        picker.time_segment = next_seg;
                                        picker.set_datetime(next, cx);
                                        picker.sync_time_scrolls(next);
                                        if let Some(cb) = on_change {
                                            cb(root, next, cx);
                                        }
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    _ => {}
                                }
                            }
                        },
                    ))
                    .child(
                        row()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(INK))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(rgb(MUTED))
                                    .child("↑↓ step · 0-9 type"),
                            ),
                    )
                    .child(digital_bar)
                    .child(columns_frame)
                    .child(nudges)
                    .child(presets)
                    .child(footer),
            )
            .into_any_element()
    }
}

fn time_scroll_offset_for(val: i8, count: i8) -> f32 {
    const ITEM_H: f32 = 30.0;
    const GAP: f32 = 2.0;
    const STRIDE: f32 = ITEM_H + GAP;
    const VIEWPORT_H: f32 = 210.0;
    let total_h = (f32::from(count) * STRIDE) - GAP;
    let max_offset = (total_h - VIEWPORT_H).max(0.0);
    let centered = (f32::from(val) * STRIDE) - ((VIEWPORT_H - ITEM_H) / 2.0);
    centered.clamp(0.0, max_offset)
}

fn time_of_day_label(hour: i8) -> &'static str {
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

fn nudge_time_seconds(dt: DateTime, delta_secs: i32) -> DateTime {
    let total = i32::from(dt.hour()) * 3600 + i32::from(dt.minute()) * 60 + i32::from(dt.second());
    let wrapped = (total + delta_secs).rem_euclid(86_400);
    let nh = (wrapped / 3600) as i8;
    let nm = ((wrapped % 3600) / 60) as i8;
    let ns = (wrapped % 60) as i8;
    jiff::civil::Time::new(nh, nm, ns, 0)
        .map(|t| DateTime::from_parts(dt.date(), t))
        .unwrap_or(dt)
}

fn step_time_segment(dt: DateTime, seg: TimeSegment, delta: i8) -> DateTime {
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

fn apply_time_digit(
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

fn slim_vertical_scrollbar(handle: &gpui::ScrollHandle, track_height: f32) -> Div {
    let max_y = f32::from(handle.max_offset().height).max(0.0);
    let viewport_h = f32::from(handle.bounds().size.height).max(track_height);
    let scroll_y = (-f32::from(handle.offset().y)).clamp(0.0, max_y);
    let thumb_h = ((viewport_h / (viewport_h + max_y)) * track_height).clamp(28.0, track_height);
    let thumb_y = if max_y > 0.5 {
        (scroll_y / max_y) * (track_height - thumb_h)
    } else {
        0.0
    };
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
                .bg(gpui::rgba(0x68605366)),
        );
    }
    track
}

fn small_action_button(
    id: (&'static str, &'static str),
    label: &'static str,
    active: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control(SharedString::from(format!("{}-{}", id.0, id.1)), cx, action)
        .px_2()
        .h(px(30.))
        .justify_center()
        .rounded(px(5.))
        .border_1()
        .border_color(rgb(if active { GOLD } else { LINE }))
        .bg(rgb(if active { GOLD } else { PAPER }))
        .text_color(rgb(if active { PAPER } else { INK }))
        .text_size(px(12.))
        .hover(|style| style.opacity(0.85))
        .child(label)
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
