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
    pub year_input: Entity<TextInput>,
    pub day_input: Entity<TextInput>,
    pub hour_input: Entity<TextInput>,
    pub minute_input: Entity<TextInput>,
    pub second_input: Entity<TextInput>,
    pub view_year: i32,
    pub view_month: u32,
    pub open: bool,
    last_datetime_text: String,
    last_parts_text: [String; 5],
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
        let parts = Self::format_parts(initial);
        let datetime = text_input(&formatted, "YYYY-MM-DD HH:MM:SS", cx);
        let year_input = text_input(&parts[0], "YYYY", cx);
        let day_input = text_input(&parts[1], "DD", cx);
        let hour_input = text_input(&parts[2], "HH", cx);
        let minute_input = text_input(&parts[3], "MM", cx);
        let second_input = text_input(&parts[4], "SS", cx);

        for input in [
            &datetime,
            &year_input,
            &day_input,
            &hour_input,
            &minute_input,
            &second_input,
        ] {
            cx.observe(input, |root, _, cx| {
                root.sync_datetime_pickers(cx);
                cx.notify();
            })
            .detach();
        }

        Self {
            id_prefix,
            datetime,
            year_input,
            day_input,
            hour_input,
            minute_input,
            second_input,
            view_year: i32::from(initial.year()),
            view_month: u32::from(initial.month() as u8),
            open,
            last_datetime_text: formatted,
            last_parts_text: parts,
            parts_error: None,
        }
    }

    fn format_parts(dt: DateTime) -> [String; 5] {
        [
            format!("{:04}", dt.year()),
            format!("{:02}", dt.day()),
            format!("{:02}", dt.hour()),
            format!("{:02}", dt.minute()),
            format!("{:02}", dt.second()),
        ]
    }

    pub fn current_datetime(&self, cx: &gpui::App) -> DateTime {
        parse_flexible_datetime(self.datetime.read(cx).value(), true).unwrap_or_else(|_| {
            let year = self
                .year_input
                .read(cx)
                .value()
                .trim()
                .parse::<i32>()
                .unwrap_or(self.view_year)
                .clamp(1, 9999);
            let max_day = days_in_month(year, self.view_month);
            let day = self
                .day_input
                .read(cx)
                .value()
                .trim()
                .parse::<u32>()
                .unwrap_or(1)
                .clamp(1, max_day);
            let hour = self
                .hour_input
                .read(cx)
                .value()
                .trim()
                .parse::<u32>()
                .unwrap_or(12)
                .min(23);
            let minute = self
                .minute_input
                .read(cx)
                .value()
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
                .min(59);
            let second = self
                .second_input
                .read(cx)
                .value()
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
                .min(59);
            i16::try_from(year)
                .ok()
                .and_then(|y| Date::new(y, self.view_month as i8, day as i8).ok())
                .map(|d| d.at(hour as i8, minute as i8, second as i8, 0))
                .unwrap_or_else(|| DateTime::constant(2000, 1, 1, 12, 0, 0, 0))
        })
    }

    fn sync_part_inputs(&self, parts: &[String; 5], cx: &mut Context<DesktopRoot>) {
        for (entity, value) in [
            (&self.year_input, &parts[0]),
            (&self.day_input, &parts[1]),
            (&self.hour_input, &parts[2]),
            (&self.minute_input, &parts[3]),
            (&self.second_input, &parts[4]),
        ] {
            if entity.read(cx).value() != value {
                let next = value.clone();
                entity.update(cx, |input, cx| input.set_value(next, cx));
            }
        }
    }

    pub fn set_datetime(&mut self, dt: DateTime, cx: &mut Context<DesktopRoot>) {
        self.view_year = i32::from(dt.year());
        self.view_month = u32::from(dt.month() as u8);
        self.parts_error = None;
        let formatted = format_time(dt);
        let parts = Self::format_parts(dt);
        self.last_datetime_text = formatted.clone();
        self.last_parts_text = parts.clone();

        if self.datetime.read(cx).value() != formatted {
            self.datetime
                .update(cx, |input, cx| input.set_value(formatted, cx));
        }
        self.sync_part_inputs(&parts, cx);
    }

    pub fn sync_from_inputs(&mut self, cx: &mut Context<DesktopRoot>) {
        let dt_text = self.datetime.read(cx).value().to_owned();
        if dt_text != self.last_datetime_text {
            self.last_datetime_text = dt_text.clone();
            if let Ok(dt) = parse_flexible_datetime(&dt_text, true) {
                self.view_year = i32::from(dt.year());
                self.view_month = u32::from(dt.month() as u8);
                self.parts_error = None;
                let parts = Self::format_parts(dt);
                self.last_parts_text = parts.clone();
                self.sync_part_inputs(&parts, cx);
            } else {
                self.parts_error = None;
            }
            return;
        }

        let parts = [
            self.year_input.read(cx).value().to_owned(),
            self.day_input.read(cx).value().to_owned(),
            self.hour_input.read(cx).value().to_owned(),
            self.minute_input.read(cx).value().to_owned(),
            self.second_input.read(cx).value().to_owned(),
        ];
        if parts == self.last_parts_text {
            return;
        }
        self.last_parts_text = parts.clone();

        let parsed = (|| -> Result<DateTime, String> {
            let year: i32 = parts[0]
                .trim()
                .parse()
                .map_err(|_| "Enter a valid 4-digit year.".to_owned())?;
            if !(1..=9999).contains(&year) {
                return Err("Year must be between 0001 and 9999.".into());
            }
            let max_day = days_in_month(year, self.view_month);
            let day: u32 = parts[1]
                .trim()
                .parse()
                .map_err(|_| "Enter a valid day of the month.".to_owned())?;
            if !(1..=max_day).contains(&day) {
                return Err(format!(
                    "{} {year} has {max_day} days.",
                    MONTH_NAMES[(self.view_month - 1) as usize]
                ));
            }
            let hour: u32 = parts[2]
                .trim()
                .parse()
                .map_err(|_| "Enter an hour from 00 to 23.".to_owned())?;
            if hour > 23 {
                return Err("Hour must be between 00 and 23.".into());
            }
            let minute: u32 = parts[3]
                .trim()
                .parse()
                .map_err(|_| "Enter minutes from 00 to 59.".to_owned())?;
            if minute > 59 {
                return Err("Minute must be between 00 and 59.".into());
            }
            let second: u32 = parts[4]
                .trim()
                .parse()
                .map_err(|_| "Enter seconds from 00 to 59.".to_owned())?;
            if second > 59 {
                return Err("Second must be between 00 and 59.".into());
            }
            i16::try_from(year)
                .ok()
                .and_then(|y| Date::new(y, self.view_month as i8, day as i8).ok())
                .map(|d| d.at(hour as i8, minute as i8, second as i8, 0))
                .ok_or_else(|| "Enter a valid calendar date and time.".into())
        })();

        match parsed {
            Ok(dt) => {
                self.view_year = i32::from(dt.year());
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

    pub fn step_field(
        &mut self,
        unit: crate::model::Step,
        delta: i32,
        cx: &mut Context<DesktopRoot>,
    ) {
        let current = self.current_datetime(cx);
        if let Some(next) = unit.offset(current, delta) {
            self.set_datetime(next, cx);
        }
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
        let hour12 = match hour24 % 12 {
            0 => 12,
            h => h,
        };
        let am_pm = if is_pm { "PM" } else { "AM" };
        let summary = if unknown_time {
            format!(
                "{} · {:02} {} {:04} · Birth time unknown",
                current.strftime("%a"),
                current.day(),
                MONTH_SHORT[(current.month() - 1) as usize],
                current.year()
            )
        } else {
            format!(
                "{} · {:02} {} {:04} · {:02}:{:02}:{:02} {} ({:02}:{:02}:{:02})",
                current.strftime("%a"),
                current.day(),
                MONTH_SHORT[(current.month() - 1) as usize],
                current.year(),
                hour12,
                current.minute(),
                current.second(),
                am_pm,
                hour24,
                current.minute(),
                current.second()
            )
        };

        let mut header_actions = row()
            .child(small_action_button(
                (prefix, "today"),
                "Today",
                false,
                cx,
                move |view, cx| {
                    let now = view.session.read(cx).now();
                    if let Some(picker) = get_mut(view) {
                        let cur = picker.current_datetime(cx);
                        let dt = now.date().at(cur.hour(), cur.minute(), cur.second(), 0);
                        picker.set_datetime(dt, cx);
                        if let Some(cb) = on_change {
                            cb(view, dt, cx);
                        }
                    }
                },
            ))
            .when(!unknown_time, |bar| {
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

        header_actions = header_actions.child(small_action_button(
            (prefix, "toggle-open"),
            "Calendar",
            self.open,
            cx,
            move |view, _| {
                if let Some(picker) = get_mut(view) {
                    picker.open = !picker.open;
                }
            },
        ));

        let date_segments = row()
            .flex_wrap()
            .gap_3()
            .child(stepper_input_group(
                (prefix, "year"),
                "Year",
                self.year_input.clone(),
                px(72.),
                cx,
                step_cb(get_mut, on_change, crate::model::Step::Year, -1),
                step_cb(get_mut, on_change, crate::model::Step::Year, 1),
            ))
            .child(month_stepper_group(
                (prefix, "month"),
                self.view_month,
                cx,
                move |view, cx| shift_month_and_notify(view, -1, get_mut, on_change, cx),
                move |view, _| {
                    if let Some(picker) = get_mut(view) {
                        picker.open = true;
                    }
                },
                move |view, cx| shift_month_and_notify(view, 1, get_mut, on_change, cx),
            ))
            .child(stepper_input_group(
                (prefix, "day"),
                "Day",
                self.day_input.clone(),
                px(52.),
                cx,
                step_cb(get_mut, on_change, crate::model::Step::Day, -1),
                step_cb(get_mut, on_change, crate::model::Step::Day, 1),
            ))
            .when(!unknown_time, |bar| {
                bar.child(stepper_input_group(
                    (prefix, "hour"),
                    "Hour (0–23)",
                    self.hour_input.clone(),
                    px(52.),
                    cx,
                    step_cb(get_mut, on_change, crate::model::Step::Hour, -1),
                    step_cb(get_mut, on_change, crate::model::Step::Hour, 1),
                ))
                .child(stepper_input_group(
                    (prefix, "minute"),
                    "Minute",
                    self.minute_input.clone(),
                    px(52.),
                    cx,
                    step_cb(get_mut, on_change, crate::model::Step::Minute, -1),
                    step_cb(get_mut, on_change, crate::model::Step::Minute, 1),
                ))
                .child(stepper_input_group(
                    (prefix, "second"),
                    "Second",
                    self.second_input.clone(),
                    px(52.),
                    cx,
                    step_cb(get_mut, on_change, crate::model::Step::Second, -1),
                    step_cb(get_mut, on_change, crate::model::Step::Second, 1),
                ))
                .child(
                    column().gap_1().child(caption("Period")).child(
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
                    ),
                )
            });

        column()
            .gap_2()
            .child(
                row()
                    .justify_between()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        row().gap_3().child(caption(label)).child(
                            div()
                                .px_2()
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(rgb(SOFT))
                                .border_1()
                                .border_color(rgb(LINE))
                                .text_size(px(12.))
                                .text_color(rgb(INK))
                                .child(summary),
                        ),
                    )
                    .child(header_actions),
            )
            .child(
                div()
                    .p_3()
                    .rounded(px(8.))
                    .bg(rgb(SOFT))
                    .border_1()
                    .border_color(rgb(LINE))
                    .child(date_segments),
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
            .on_click(cx.listener(move |root, _, _, cx| {
                root.close_datetime_popup(target);
                cx.notify();
            }))
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

fn step_cb(
    get_mut: fn(&mut DesktopRoot) -> Option<&mut DateTimePicker>,
    on_change: Option<fn(&mut DesktopRoot, DateTime, &mut Context<DesktopRoot>)>,
    step: crate::model::Step,
    delta: i32,
) -> impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static {
    move |view, cx| {
        if let Some(picker) = get_mut(view) {
            picker.step_field(step, delta, cx);
            let dt = picker.current_datetime(cx);
            if let Some(cb) = on_change {
                cb(view, dt, cx);
            }
        }
    }
}

fn shift_month_and_notify(
    view: &mut DesktopRoot,
    delta: i32,
    get_mut: fn(&mut DesktopRoot) -> Option<&mut DateTimePicker>,
    on_change: Option<fn(&mut DesktopRoot, DateTime, &mut Context<DesktopRoot>)>,
    cx: &mut Context<DesktopRoot>,
) {
    if let Some(picker) = get_mut(view) {
        picker.shift_month(delta, cx);
        let dt = picker.current_datetime(cx);
        if let Some(cb) = on_change {
            cb(view, dt, cx);
        }
    }
}

fn stepper_button(
    id: SharedString,
    width: gpui::Pixels,
    label: &'static str,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> gpui::Stateful<Div> {
    action_control(id, cx, action)
        .w(width)
        .h(px(34.))
        .justify_center()
        .rounded(px(5.))
        .border_1()
        .border_color(rgb(LINE))
        .bg(rgb(PAPER))
        .hover(|style| style.bg(rgb(SELECTED)))
        .child(label)
}

fn stepper_input_group(
    id: (&'static str, &'static str),
    label: &'static str,
    input: Entity<TextInput>,
    width: gpui::Pixels,
    cx: &mut Context<DesktopRoot>,
    on_prev: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
    on_next: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> Div {
    column().gap_1().child(caption(label)).child(
        row()
            .gap_1()
            .child(stepper_button(
                format!("{}-{}-prev", id.0, id.1).into(),
                px(28.),
                "‹",
                cx,
                on_prev,
            ))
            .child(div().w(width).child(input))
            .child(stepper_button(
                format!("{}-{}-next", id.0, id.1).into(),
                px(28.),
                "›",
                cx,
                on_next,
            )),
    )
}

fn month_stepper_group(
    id: (&'static str, &'static str),
    view_month: u32,
    cx: &mut Context<DesktopRoot>,
    on_prev: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
    on_open: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
    on_next: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> Div {
    let month_idx = (view_month.clamp(1, 12) - 1) as usize;
    let label = format!("{:02} · {}", view_month, MONTH_SHORT[month_idx]);
    column().gap_1().child(caption("Month")).child(
        row()
            .gap_1()
            .child(stepper_button(
                format!("{}-{}-prev", id.0, id.1).into(),
                px(28.),
                "‹",
                cx,
                on_prev,
            ))
            .child(
                action_control(
                    SharedString::from(format!("{}-{}-display", id.0, id.1)),
                    cx,
                    on_open,
                )
                .w(px(96.))
                .h(px(34.))
                .justify_center()
                .rounded(px(5.))
                .border_1()
                .border_color(rgb(LINE))
                .bg(rgb(PAPER))
                .hover(|style| style.bg(rgb(SELECTED)))
                .child(label),
            )
            .child(stepper_button(
                format!("{}-{}-next", id.0, id.1).into(),
                px(28.),
                "›",
                cx,
                on_next,
            )),
    )
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
