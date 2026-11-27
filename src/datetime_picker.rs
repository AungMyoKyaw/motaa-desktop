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

pub struct DateTimePicker {
    pub id_prefix: &'static str,
    pub datetime: Entity<TextInput>,
    pub date_input: Entity<TextInput>,
    pub time_input: Entity<TextInput>,
    pub view_year: i32,
    pub view_month: u32,
    pub open: bool,
    pub time_open: bool,
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
            hour_scroll: gpui::ScrollHandle::new(),
            minute_scroll: gpui::ScrollHandle::new(),
            second_scroll: gpui::ScrollHandle::new(),
            last_datetime_text: formatted,
            last_date_text: date_text,
            last_time_text: time_text,
            parts_error: None,
        }
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
                        action_control(
                            SharedString::from(format!("{prefix}-pick-time")),
                            cx,
                            move |view, cx| {
                                if let Some(picker) = get_mut(view) {
                                    picker.time_open = true;
                                    picker.open = false;
                                    picker.hour_scroll.set_offset(gpui::point(px(0.), px(0.)));
                                    picker.minute_scroll.set_offset(gpui::point(px(0.), px(0.)));
                                    picker.second_scroll.set_offset(gpui::point(px(0.), px(0.)));
                                }
                                cx.notify();
                            },
                        )
                        .h(px(36.))
                        .px_3()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.))
                        .bg(rgb(SOFT))
                        .border_1()
                        .border_color(rgb(LINE))
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

    pub(crate) fn close_time_popup(&mut self, target: ActiveDateTimePicker) {
        match target {
            ActiveDateTimePicker::Profile => {
                if let Some(d) = self.draft.as_mut() {
                    d.picker.time_open = false;
                }
            }
            ActiveDateTimePicker::ReferenceTime => {
                self.time_picker.time_open = false;
            }
            ActiveDateTimePicker::Candidate(idx) => {
                if let Some(p) = self.candidate_pickers.get_mut(idx) {
                    p.time_open = false;
                }
            }
        }
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
        let summary = format!(
            "{:02}:{:02}:{:02}",
            current.hour(),
            current.minute(),
            current.second()
        );
        let prefix = picker.id_prefix;

        let mut build_col = |col_id: &'static str,
                             col_title: &'static str,
                             count: i8,
                             selected: i8,
                             scroll_handle: gpui::ScrollHandle,
                             make_dt: fn(DateTime, i8) -> DateTime| {
            let items: Vec<_> = (0..count)
                .map(|val| {
                    let is_sel = val == selected;
                    action_control(
                        (
                            SharedString::from(format!("{prefix}-tp-{col_id}")),
                            usize::try_from(val).unwrap_or(0),
                        ),
                        cx,
                        move |view, cx| {
                            if let Some(picker) = get_mut(view) {
                                let cur = picker.current_datetime(cx);
                                let next = make_dt(cur, val);
                                picker.set_datetime(next, cx);
                                if let Some(cb) = on_change {
                                    cb(view, next, cx);
                                }
                            }
                            cx.notify();
                        },
                    )
                    .w_full()
                    .h(px(28.))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(5.))
                    .bg(if is_sel {
                        rgb(GOLD)
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .text_size(px(12.))
                    .font_family("Menlo")
                    .text_color(if is_sel { rgb(0xffffff) } else { rgb(INK) })
                    .child(format!("{val:02}"))
                })
                .collect();

            column()
                .flex_1()
                .min_w(px(0.))
                .gap_1()
                .child(div().w_full().text_center().child(caption(col_title)))
                .child(
                    div()
                        .relative()
                        .w_full()
                        .h(px(260.))
                        .rounded(px(8.))
                        .bg(rgb(SOFT))
                        .border_1()
                        .border_color(rgb(crate::SUBTLE_LINE))
                        .child(
                            div()
                                .id((
                                    SharedString::from(format!("{prefix}-tp-scroll-{col_id}")),
                                    0,
                                ))
                                .size_full()
                                .overflow_y_scroll()
                                .track_scroll(&scroll_handle)
                                .on_scroll_wheel(cx.listener(|_, _, _, cx| {
                                    cx.stop_propagation();
                                    cx.notify();
                                }))
                                .p_1()
                                .pr(px(14.))
                                .flex()
                                .flex_col()
                                .gap(px(2.))
                                .children(items),
                        )
                        .child(crate::vertical_scrollbar(&scroll_handle, 260.0)),
                )
        };

        let hour_col = build_col(
            "hour",
            "Hour",
            24,
            current.hour(),
            picker.hour_scroll.clone(),
            |dt, h| {
                jiff::civil::Time::new(h, dt.minute(), dt.second(), 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );
        let minute_col = build_col(
            "min",
            "Minute",
            60,
            current.minute(),
            picker.minute_scroll.clone(),
            |dt, m| {
                jiff::civil::Time::new(dt.hour(), m, dt.second(), 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );
        let second_col = build_col(
            "sec",
            "Second",
            60,
            current.second(),
            picker.second_scroll.clone(),
            |dt, s| {
                jiff::civil::Time::new(dt.hour(), dt.minute(), s, 0)
                    .map(|t| DateTime::from_parts(dt.date(), t))
                    .unwrap_or(dt)
            },
        );

        let presets = row()
            .gap_2()
            .flex_wrap()
            .children(
                [
                    ("00:00:00", 0i8),
                    ("06:00:00", 6),
                    ("12:00:00", 12),
                    ("18:00:00", 18),
                ]
                .map(|(label, hour)| {
                    small_action_button((prefix, label), label, false, cx, move |view, cx| {
                        if let Some(picker) = get_mut(view) {
                            let cur = picker.current_datetime(cx);
                            let next = jiff::civil::Time::new(hour, 0, 0, 0)
                                .map(|t| DateTime::from_parts(cur.date(), t))
                                .unwrap_or(cur);
                            picker.set_datetime(next, cx);
                            if let Some(cb) = on_change {
                                cb(view, next, cx);
                            }
                        }
                        cx.notify();
                    })
                }),
            )
            .child(small_action_button(
                (prefix, "Now"),
                "Now",
                false,
                cx,
                move |view, cx| {
                    if let Some(picker) = get_mut(view) {
                        let now = jiff::tz::TimeZone::system().to_datetime(jiff::Timestamp::now());
                        let cur = picker.current_datetime(cx);
                        let next =
                            jiff::civil::Time::new(now.hour(), now.minute(), now.second(), 0)
                                .map(|t| DateTime::from_parts(cur.date(), t))
                                .unwrap_or(cur);
                        picker.set_datetime(next, cx);
                        if let Some(cb) = on_change {
                            cb(view, next, cx);
                        }
                    }
                    cx.notify();
                },
            ));

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
                    .w(px(480.))
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
                    .on_key_down(cx.listener(move |root, event: &gpui::KeyDownEvent, _, cx| {
                        if event.keystroke.key == "escape" {
                            root.close_time_popup(target);
                            cx.notify();
                        }
                    }))
                    .child(
                        row()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .child(
                                column()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(15.))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(INK))
                                            .child(title),
                                    )
                                    .child(caption(summary)),
                            )
                            .child(button("Done", true, cx, move |root, cx| {
                                root.close_time_popup(target);
                                cx.notify();
                            })),
                    )
                    .child(
                        row()
                            .gap_3()
                            .child(hour_col)
                            .child(minute_col)
                            .child(second_col),
                    )
                    .child(presets),
            )
            .into_any_element()
    }
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
}
