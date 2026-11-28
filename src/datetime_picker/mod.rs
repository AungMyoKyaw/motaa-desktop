mod overlays;
mod parse;

use gpui::{Context, Div, Entity, SharedString, div, prelude::*, px, rgb};
use jiff::{
    ToSpan,
    civil::{Date, DateTime},
};

use crate::{
    DesktopRoot, ERROR, GOLD, INK, LINE, MUTED, PAPER, SELECTED, SOFT, SUBTLE_LINE, SURFACE_HOVER,
    TextInput, action_control, action_control_in, caption, column, format_time, panel,
    render_input, row, subscribe_input, text_input,
};

pub use parse::{MONTH_NAMES, MONTH_SHORT, days_in_month, parse_flexible_datetime};
use parse::{WEEKDAYS, parse_time_only, parse_time_part};

pub type PickerSelector = fn(&mut DesktopRoot) -> Option<&mut DateTimePicker>;
pub type PickerChangeCallback =
    fn(&mut DesktopRoot, DateTime, &mut gpui::Window, &mut Context<DesktopRoot>);
pub type ToggleCallback = fn(&mut DesktopRoot, &mut Context<DesktopRoot>);

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
    pub time_dropdown: Option<TimeSegment>,
    pub time_snapshot: Option<DateTime>,
    pub hour_input: Entity<TextInput>,
    pub minute_input: Entity<TextInput>,
    pub second_input: Entity<TextInput>,
    last_datetime_text: String,
    last_date_text: String,
    last_time_text: String,
    last_hour_text: String,
    last_minute_text: String,
    last_second_text: String,
    parts_error: Option<String>,
    pub modal_time_error: Option<String>,
}

impl DateTimePicker {
    pub fn new(
        id_prefix: &'static str,
        initial: DateTime,
        open: bool,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) -> Self {
        let formatted = format_time(initial);
        let (date_text, time_text) = Self::format_date_time(initial);
        let hour_text = format!("{:02}", initial.hour());
        let minute_text = format!("{:02}", initial.minute());
        let second_text = format!("{:02}", initial.second());
        let datetime = text_input(&formatted, "YYYY-MM-DD HH:MM:SS", window, cx);
        let date_input = text_input(&date_text, "YYYY-MM-DD", window, cx);
        let time_input = text_input(&time_text, "HH:MM:SS", window, cx);
        let hour_input = text_input(&hour_text, "HH", window, cx);
        let minute_input = text_input(&minute_text, "MM", window, cx);
        let second_input = text_input(&second_text, "SS", window, cx);

        for input in [
            &datetime,
            &date_input,
            &time_input,
            &hour_input,
            &minute_input,
            &second_input,
        ] {
            subscribe_input(input, window, cx, |root, window, cx| {
                root.sync_datetime_pickers(window, cx);
                cx.notify();
            });
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
            time_dropdown: None,
            time_snapshot: None,
            hour_input,
            minute_input,
            second_input,
            last_datetime_text: formatted,
            last_date_text: date_text,
            last_time_text: time_text,
            last_hour_text: hour_text,
            last_minute_text: minute_text,
            last_second_text: second_text,
            parts_error: None,
            modal_time_error: None,
        }
    }

    pub fn sync_modal_inputs(
        &mut self,
        dt: DateTime,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        let h_str = format!("{:02}", dt.hour());
        let m_str = format!("{:02}", dt.minute());
        let s_str = format!("{:02}", dt.second());
        self.last_hour_text = h_str.clone();
        self.last_minute_text = m_str.clone();
        self.last_second_text = s_str.clone();
        self.modal_time_error = None;

        if self.hour_input.read(cx).value() != h_str {
            self.hour_input
                .update(cx, |input, cx| input.set_value(h_str, window, cx));
        }
        if self.minute_input.read(cx).value() != m_str {
            self.minute_input
                .update(cx, |input, cx| input.set_value(m_str, window, cx));
        }
        if self.second_input.read(cx).value() != s_str {
            self.second_input
                .update(cx, |input, cx| input.set_value(s_str, window, cx));
        }
    }

    fn format_date_time(dt: DateTime) -> (String, String) {
        (
            format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
            format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second()),
        )
    }

    pub fn current_datetime(&self, cx: &gpui::App) -> DateTime {
        parse_flexible_datetime(self.datetime.read(cx).value().as_ref(), true).unwrap_or_else(
            |_| {
                let date = Date::strptime("%Y-%m-%d", self.date_input.read(cx).value().trim())
                    .unwrap_or_else(|_| Date::constant(2000, 1, 1));
                let time = parse_time_only(self.time_input.read(cx).value().as_ref())
                    .unwrap_or_else(|_| jiff::civil::Time::constant(12, 0, 0, 0));
                date.to_datetime(time)
            },
        )
    }

    fn sync_date_time_inputs(
        &self,
        date_text: &str,
        time_text: &str,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        if self.date_input.read(cx).value() != date_text {
            let next = date_text.to_owned();
            self.date_input
                .update(cx, |input, cx| input.set_value(next, window, cx));
        }
        if self.time_input.read(cx).value() != time_text {
            let next = time_text.to_owned();
            self.time_input
                .update(cx, |input, cx| input.set_value(next, window, cx));
        }
    }

    pub fn set_datetime(
        &mut self,
        dt: DateTime,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
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
                .update(cx, |input, cx| input.set_value(formatted, window, cx));
        }
        self.sync_date_time_inputs(&date_text, &time_text, window, cx);
    }

    pub fn sync_from_inputs(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) -> Option<DateTime> {
        if self.time_open {
            let h_text = self.hour_input.read(cx).value().to_string();
            let m_text = self.minute_input.read(cx).value().to_string();
            let s_text = self.second_input.read(cx).value().to_string();
            if h_text != self.last_hour_text
                || m_text != self.last_minute_text
                || s_text != self.last_second_text
            {
                self.last_hour_text = h_text.clone();
                self.last_minute_text = m_text.clone();
                self.last_second_text = s_text.clone();
                if h_text.trim().is_empty() || m_text.trim().is_empty() || s_text.trim().is_empty()
                {
                    self.modal_time_error = None;
                    return None;
                }
                match (
                    parse_time_part(&h_text, 23),
                    parse_time_part(&m_text, 59),
                    parse_time_part(&s_text, 59),
                ) {
                    (Some(h), Some(m), Some(s)) => {
                        let cur = self.current_datetime(cx);
                        if let Ok(t) = jiff::civil::Time::new(h, m, s, 0) {
                            let dt = DateTime::from_parts(cur.date(), t);
                            self.modal_time_error = None;
                            self.parts_error = None;
                            let formatted = format_time(dt);
                            let (date_text, time_text) = Self::format_date_time(dt);
                            self.last_datetime_text = formatted.clone();
                            self.last_date_text = date_text.clone();
                            self.last_time_text = time_text.clone();
                            if self.datetime.read(cx).value() != formatted {
                                self.datetime
                                    .update(cx, |input, cx| input.set_value(formatted, window, cx));
                            }
                            self.sync_date_time_inputs(&date_text, &time_text, window, cx);
                            return Some(dt);
                        }
                    }
                    _ => {
                        self.modal_time_error =
                            Some("Hours must be 00–23, minutes and seconds 00–59.".into());
                        return None;
                    }
                }
            }
        }

        let dt_text = self.datetime.read(cx).value().to_string();
        if dt_text != self.last_datetime_text {
            self.last_datetime_text = dt_text.clone();
            if let Ok(dt) = parse_flexible_datetime(&dt_text, true) {
                self.view_year = i32::from(dt.year());
                self.view_month = u32::from(dt.month() as u8);
                self.parts_error = None;
                let (date_text, time_text) = Self::format_date_time(dt);
                self.last_date_text = date_text.clone();
                self.last_time_text = time_text.clone();
                self.sync_date_time_inputs(&date_text, &time_text, window, cx);
            } else {
                self.parts_error = None;
            }
            return None;
        }

        let date_text = self.date_input.read(cx).value().to_string();
        let time_text = self.time_input.read(cx).value().to_string();
        if date_text == self.last_date_text && time_text == self.last_time_text {
            return None;
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
                        .update(cx, |input, cx| input.set_value(formatted, window, cx));
                }
            }
            Err(error) => {
                self.parts_error = Some(error);
            }
        }
        None
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(error) = &self.parts_error {
            return Err(error.clone());
        }
        Ok(())
    }

    pub fn set_year_month(
        &mut self,
        year: i32,
        month: u32,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        let current = self.current_datetime(cx);
        let year = year.clamp(1, 9999);
        let month = month.clamp(1, 12);
        let day = u32::from(current.day() as u8).min(days_in_month(year, month));
        if let Ok(y) = i16::try_from(year)
            && let Ok(date) = Date::new(y, month as i8, day as i8)
        {
            self.set_datetime(
                date.at(current.hour(), current.minute(), current.second(), 0),
                window,
                cx,
            );
        }
    }

    pub fn shift_month(
        &mut self,
        delta_months: i32,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        let total = self.view_year * 12 + (self.view_month as i32 - 1) + delta_months;
        let year = total.div_euclid(12).clamp(1, 9999);
        let month = (total.rem_euclid(12) + 1) as u32;
        self.set_year_month(year, month, window, cx);
    }

    pub fn select_calendar_date(
        &mut self,
        date: Date,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        let current = self.current_datetime(cx);
        self.set_datetime(
            date.at(current.hour(), current.minute(), current.second(), 0),
            window,
            cx,
        );
    }

    pub fn set_hms(
        &mut self,
        hour: u32,
        minute: u32,
        second: u32,
        window: &mut gpui::Window,
        cx: &mut Context<DesktopRoot>,
    ) {
        let current = self.current_datetime(cx);
        let dt = current.date().at(
            hour.min(23) as i8,
            minute.min(59) as i8,
            second.min(59) as i8,
            0,
        );
        self.set_datetime(dt, window, cx);
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
        let is_pm = current.hour() >= 12;

        let mut header_actions = row().when(!unknown_time, |bar| {
            bar.child(small_action_button(
                (prefix, "now"),
                "Now",
                false,
                cx,
                move |view, window, cx| {
                    let now = view.session.read(cx).now();
                    if let Some(picker) = get_mut(view) {
                        picker.set_datetime(now, window, cx);
                        if let Some(cb) = on_change {
                            cb(view, now, window, cx);
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
                move |view, _, cx| toggle_unknown(view, cx),
            ));
        }

        let date_col = column().gap_1().child(caption("Date (YYYY-MM-DD)")).child(
            row()
                .w_full()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(render_input(&self.date_input)),
                )
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

        let open_time_modal = move |view: &mut DesktopRoot,
                                    window: &mut gpui::Window,
                                    cx: &mut Context<DesktopRoot>| {
            if let Some(picker) = get_mut(view) {
                let current = picker.current_datetime(cx);
                picker.time_snapshot = Some(current);
                picker.time_segment = TimeSegment::Hour;
                picker.time_dropdown = Some(TimeSegment::Hour);
                picker.time_open = true;
                picker.open = false;
                picker.sync_modal_inputs(current, window, cx);
                let focus = picker.time_focus.clone();
                window.focus(&focus, cx);
            }
            cx.notify();
        };

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
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .child(render_input(&self.time_input)),
                    )
                    .child(
                        row()
                            .gap_1()
                            .children([("am", "AM", false), ("pm", "PM", true)].map(
                                |(id_suffix, text, pm)| {
                                    small_action_button(
                                        (prefix, id_suffix),
                                        text,
                                        is_pm == pm,
                                        cx,
                                        move |view, window, cx| {
                                            if let Some(picker) = get_mut(view) {
                                                let cur = picker.current_datetime(cx);
                                                let target_hour = if pm && cur.hour() < 12 {
                                                    Some((cur.hour() + 12) as u32)
                                                } else if !pm && cur.hour() >= 12 {
                                                    Some((cur.hour() - 12) as u32)
                                                } else {
                                                    None
                                                };
                                                if let Some(nh) = target_hour {
                                                    picker.set_hms(
                                                        nh,
                                                        cur.minute() as u32,
                                                        cur.second() as u32,
                                                        window,
                                                        cx,
                                                    );
                                                    let dt = picker.current_datetime(cx);
                                                    if let Some(cb) = on_change {
                                                        cb(view, dt, window, cx);
                                                    }
                                                }
                                            }
                                        },
                                    )
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
                                open_time_modal(view, window, cx);
                            }))
                            .on_key_down(cx.listener(
                                move |view, event: &gpui::KeyDownEvent, window, cx| {
                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                        open_time_modal(view, window, cx);
                                        cx.stop_propagation();
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
                            .hover(|style| style.bg(rgb(SURFACE_HOVER)))
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
                view.child(div().text_size(px(12.)).text_color(rgb(ERROR)).child(err))
            })
    }

    fn render_calendar_popup_body(
        &self,
        current: DateTime,
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

                action_control_in(
                    (SharedString::from(format!("{prefix}-cal-day")), index),
                    cx,
                    move |view, window, cx| {
                        if let Some(picker) = get_mut(view) {
                            picker.select_calendar_date(cell_date, window, cx);
                            let dt = picker.current_datetime(cx);
                            if let Some(cb) = on_change {
                                cb(view, dt, window, cx);
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
                    SUBTLE_LINE
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
                    action_control_in(
                        (SharedString::from(format!("{prefix}-month-pill")), idx),
                        cx,
                        move |view, window, cx| {
                            if let Some(picker) = get_mut(view) {
                                let yr = picker.view_year;
                                picker.set_year_month(yr, m_num, window, cx);
                                let dt = picker.current_datetime(cx);
                                if let Some(cb) = on_change {
                                    cb(view, dt, window, cx);
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

        let shift_btn = |id_suffix: &'static str,
                         label: &'static str,
                         delta_months: i32,
                         cx: &mut Context<DesktopRoot>| {
            small_action_button(
                (prefix, id_suffix),
                label,
                false,
                cx,
                move |view, window, cx| {
                    if let Some(picker) = get_mut(view) {
                        picker.shift_month(delta_months, window, cx);
                        let dt = picker.current_datetime(cx);
                        if let Some(cb) = on_change {
                            cb(view, dt, window, cx);
                        }
                    }
                },
            )
        };

        panel()
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .child(
                        row()
                            .gap_1()
                            .child(shift_btn("cal-prev-10y", "« 10y", -120, cx))
                            .child(shift_btn("cal-prev-mo", "‹", -1, cx)),
                    )
                    .child(div().text_size(px(15.)).text_color(rgb(INK)).child(format!(
                        "{} {:04}",
                        MONTH_NAMES[(view_month - 1) as usize],
                        view_year
                    )))
                    .child(
                        row()
                            .gap_1()
                            .child(shift_btn("cal-next-mo", "›", 1, cx))
                            .child(shift_btn("cal-next-10y", "10y »", 120, cx)),
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
            .child(div().grid().grid_cols(7).gap_1().children(calendar_cells))
    }
}

fn small_action_button(
    id: (&'static str, &'static str),
    label: &'static str,
    active: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut gpui::Window, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control_in(SharedString::from(format!("{}-{}", id.0, id.1)), cx, action)
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
