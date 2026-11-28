use gpui::{Context, SharedString, div, prelude::*, px, rgb, rgba};
use jiff::civil::DateTime;

use crate::{
    ACTION_HOVER, BACKDROP_DARK, DesktopRoot, ERROR, GOLD, HEADER_BG, INK, LINE, MUTED, PAPER,
    SELECTED, SOFT, SUBTLE_LINE, SURFACE_HOVER, TRANSPARENT, WHITE, button, modal_dialog,
};

use super::parse::{nudge_time_seconds, step_time_segment, time_of_day_label};
use super::{DateTimePicker, PickerChangeCallback, PickerSelector, TimeSegment};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveDateTimePicker {
    Profile,
    ReferenceTime,
    Candidate(usize),
}

fn apply_picker_time(
    view: &mut DesktopRoot,
    get_mut: PickerSelector,
    on_change: Option<PickerChangeCallback>,
    next: DateTime,
    cx: &mut Context<DesktopRoot>,
) {
    if let Some(picker) = get_mut(view) {
        picker.set_datetime(next, cx);
        picker.sync_modal_inputs(next, cx);
        if let Some(cb) = on_change {
            cb(view, next, cx);
        }
    }
}

impl DesktopRoot {
    fn resolve_datetime_target(
        &self,
        target: ActiveDateTimePicker,
        is_time: bool,
    ) -> Option<(
        &DateTimePicker,
        Option<PickerChangeCallback>,
        PickerSelector,
        &'static str,
    )> {
        match target {
            ActiveDateTimePicker::Profile => self.draft.as_ref().map(|d| {
                (
                    &d.picker,
                    None,
                    (|view: &mut DesktopRoot| view.draft.as_mut().map(|d| &mut d.picker))
                        as PickerSelector,
                    if is_time {
                        "Pick Birth Time"
                    } else {
                        "Pick Birth Date"
                    },
                )
            }),
            ActiveDateTimePicker::ReferenceTime => Some((
                &self.time_picker,
                Some(|view, dt, cx| view.set_time(dt, true, cx)),
                |view| Some(&mut view.time_picker),
                if is_time {
                    "Pick Reference Time"
                } else {
                    "Pick Reference Date"
                },
            )),
            ActiveDateTimePicker::Candidate(idx) => self.candidate_pickers.get(idx).map(|p| {
                let get_mut: PickerSelector = match idx {
                    0 => |view| view.candidate_pickers.get_mut(0),
                    1 => |view| view.candidate_pickers.get_mut(1),
                    _ => |view| view.candidate_pickers.get_mut(2),
                };
                (
                    p,
                    None,
                    get_mut,
                    if is_time {
                        "Pick Candidate Time"
                    } else {
                        "Pick Candidate Date"
                    },
                )
            }),
        }
    }

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
        let Some((picker, on_change, get_mut, title)) = self.resolve_datetime_target(target, false)
        else {
            return div().into_any_element();
        };
        let current = picker.current_datetime(cx);
        let summary = format!(
            "{:04}-{:02}-{:02}",
            current.year(),
            current.month(),
            current.day()
        );
        modal_dialog(
            "datetime-backdrop",
            "datetime-dialog",
            560.,
            None,
            title,
            summary,
            "Done",
            picker
                .render_calendar_popup_body(current, on_change, get_mut, cx)
                .into_any_element(),
            cx,
            move |root, _| root.close_datetime_popup(target),
        )
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
        cx: &mut Context<Self>,
    ) {
        let current = match target {
            ActiveDateTimePicker::Profile => {
                self.draft.as_ref().map(|d| d.picker.current_datetime(cx))
            }
            ActiveDateTimePicker::ReferenceTime => Some(self.time_picker.current_datetime(cx)),
            ActiveDateTimePicker::Candidate(idx) => self
                .candidate_pickers
                .get(idx)
                .map(|p| p.current_datetime(cx)),
        };
        let picker = match target {
            ActiveDateTimePicker::Profile => self.draft.as_mut().map(|d| &mut d.picker),
            ActiveDateTimePicker::ReferenceTime => Some(&mut self.time_picker),
            ActiveDateTimePicker::Candidate(idx) => self.candidate_pickers.get_mut(idx),
        };
        if let Some(picker) = picker {
            if let Some(dt) = current {
                picker.sync_modal_inputs(dt, cx);
            }
            picker.time_open = false;
            picker.time_dropdown = None;
            picker.time_snapshot = None;
            picker.modal_time_error = None;
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
                d.picker.time_dropdown = None;
                d.picker.modal_time_error = None;
                d.picker.time_snapshot.take()
            }),
            ActiveDateTimePicker::ReferenceTime => {
                self.time_picker.time_open = false;
                self.time_picker.time_dropdown = None;
                self.time_picker.modal_time_error = None;
                self.time_picker.time_snapshot.take()
            }
            ActiveDateTimePicker::Candidate(idx) => {
                self.candidate_pickers.get_mut(idx).and_then(|p| {
                    p.time_open = false;
                    p.time_dropdown = None;
                    p.modal_time_error = None;
                    p.time_snapshot.take()
                })
            }
        };
        if let Some(orig) = orig {
            match target {
                ActiveDateTimePicker::Profile => {
                    if let Some(d) = self.draft.as_mut() {
                        d.picker.set_datetime(orig, cx);
                        d.picker.sync_modal_inputs(orig, cx);
                    }
                }
                ActiveDateTimePicker::ReferenceTime => {
                    self.time_picker.set_datetime(orig, cx);
                    self.time_picker.sync_modal_inputs(orig, cx);
                    self.set_time(orig, true, cx);
                }
                ActiveDateTimePicker::Candidate(idx) => {
                    if let Some(p) = self.candidate_pickers.get_mut(idx) {
                        p.set_datetime(orig, cx);
                        p.sync_modal_inputs(orig, cx);
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
        let Some((picker, on_change, get_mut, title)) = self.resolve_datetime_target(target, true)
        else {
            return div().into_any_element();
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
        let open_dropdown = picker.time_dropdown;

        let make_combo = |seg: TimeSegment,
                          id_suffix: &'static str,
                          label: &'static str,
                          input: gpui::Entity<crate::TextInput>| {
            let is_open = open_dropdown == Some(seg);
            let is_active = active_seg == seg || is_open;
            crate::column()
                .gap(px(3.))
                .child(
                    div()
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(if is_active { rgb(GOLD) } else { rgb(MUTED) })
                        .child(label),
                )
                .child(
                    crate::row()
                        .gap_0()
                        .h(px(36.))
                        .rounded(px(7.))
                        .bg(rgb(PAPER))
                        .border_1()
                        .border_color(rgb(if is_active { GOLD } else { LINE }))
                        .overflow_hidden()
                        .child(div().w(px(52.)).child(input))
                        .child(
                            div()
                                .id(SharedString::from(format!("{prefix}-tp-dd-{id_suffix}")))
                                .cursor_pointer()
                                .w(px(24.))
                                .h_full()
                                .border_l_1()
                                .border_color(rgb(SUBTLE_LINE))
                                .bg(if is_open { rgb(SELECTED) } else { rgb(SOFT) })
                                .hover(|st| st.bg(rgb(SURFACE_HOVER)))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(10.))
                                .text_color(if is_open { rgb(GOLD) } else { rgb(MUTED) })
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    if let Some(picker) = get_mut(view) {
                                        picker.time_segment = seg;
                                        picker.time_dropdown = if picker.time_dropdown == Some(seg)
                                        {
                                            None
                                        } else {
                                            Some(seg)
                                        };
                                    }
                                    cx.notify();
                                }))
                                .child(if is_open { "▴" } else { "▾" }),
                        ),
                )
        };

        let readout_colon = || {
            div()
                .pt(px(14.))
                .text_size(px(16.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(MUTED))
                .child(":")
        };

        let am_pm_capsule = crate::row().pt(px(14.)).child(
            crate::row()
                .gap(px(2.))
                .p(px(2.))
                .rounded(px(7.))
                .bg(rgb(PAPER))
                .border_1()
                .border_color(rgb(SUBTLE_LINE))
                .children(
                    [("am", "AM", !is_pm, false), ("pm", "PM", is_pm, true)].map(
                        |(id_suffix, label, active, target_pm)| {
                            let mut btn = div()
                                .id(SharedString::from(format!("{prefix}-tp-{id_suffix}")))
                                .cursor_pointer()
                                .px_2()
                                .py(px(5.))
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
                                            let next = jiff::civil::Time::new(
                                                nh,
                                                cur.minute(),
                                                cur.second(),
                                                0,
                                            )
                                            .map(|t| DateTime::from_parts(cur.date(), t))
                                            .unwrap_or(cur);
                                            apply_picker_time(view, get_mut, on_change, next, cx);
                                        }
                                    }
                                    cx.notify();
                                }))
                                .child(label);
                            if active {
                                btn = btn.bg(rgb(GOLD)).text_color(rgb(WHITE));
                            } else {
                                btn = btn
                                    .bg(rgba(TRANSPARENT))
                                    .text_color(rgb(INK))
                                    .hover(|s| s.bg(rgb(SURFACE_HOVER)));
                            }
                            btn
                        },
                    ),
                ),
        );

        let digital_bar = crate::column()
            .w_full()
            .px_4()
            .py_3()
            .rounded(px(10.))
            .bg(rgb(SOFT))
            .border_1()
            .border_color(rgb(SUBTLE_LINE))
            .gap_2()
            .child(
                crate::row()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .gap_2()
                    .child(
                        crate::row()
                            .items_center()
                            .gap_1()
                            .child(make_combo(
                                TimeSegment::Hour,
                                "h",
                                "HOUR",
                                picker.hour_input.clone(),
                            ))
                            .child(readout_colon())
                            .child(make_combo(
                                TimeSegment::Minute,
                                "m",
                                "MIN",
                                picker.minute_input.clone(),
                            ))
                            .child(readout_colon())
                            .child(make_combo(
                                TimeSegment::Second,
                                "s",
                                "SEC",
                                picker.second_input.clone(),
                            )),
                    )
                    .child(am_pm_capsule),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(rgb(MUTED))
                    .child(twelve_hour_hint),
            )
            .when_some(picker.modal_time_error.clone(), |bar, err| {
                bar.child(div().text_size(px(11.5)).text_color(rgb(ERROR)).child(err))
            });

        let dropdown_panel = open_dropdown.map(|open_seg| {
            let tabs = crate::row()
                .w_full()
                .bg(rgb(HEADER_BG))
                .border_b_1()
                .border_color(rgb(SUBTLE_LINE))
                .children(
                    [
                        (TimeSegment::Hour, "hour", "Hour · 00–23"),
                        (TimeSegment::Minute, "min", "Minute · 00–59"),
                        (TimeSegment::Second, "sec", "Second · 00–59"),
                    ]
                    .map(|(seg, id_suffix, label)| {
                        let active = open_seg == seg;
                        div()
                            .id(SharedString::from(format!("{prefix}-tp-tab-{id_suffix}")))
                            .flex_1()
                            .cursor_pointer()
                            .py(px(7.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.))
                            .font_weight(if active {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::MEDIUM
                            })
                            .bg(if active { rgb(PAPER) } else { rgb(HEADER_BG) })
                            .text_color(if active { rgb(GOLD) } else { rgb(MUTED) })
                            .border_r_1()
                            .border_color(rgb(SUBTLE_LINE))
                            .hover(|st| st.bg(rgb(SURFACE_HOVER)))
                            .on_click(cx.listener(move |view, _, _, cx| {
                                if let Some(picker) = get_mut(view) {
                                    picker.time_segment = seg;
                                    picker.time_dropdown = Some(seg);
                                }
                                cx.notify();
                            }))
                            .child(label)
                    }),
                );

            let make_cell = |seg: TimeSegment,
                             col_id: &'static str,
                             val: i8,
                             selected: bool,
                             major_mark: bool,
                             next_dropdown: Option<TimeSegment>,
                             make_dt: fn(DateTime, i8) -> DateTime| {
                let mut cell = div()
                    .id((
                        SharedString::from(format!("{prefix}-tp-{col_id}")),
                        usize::try_from(val).unwrap_or(0),
                    ))
                    .cursor_pointer()
                    .h(px(26.))
                    .rounded(px(5.))
                    .border_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family("Menlo")
                    .text_size(px(12.))
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if let Some(picker) = get_mut(view) {
                            let cur = picker.current_datetime(cx);
                            let next = make_dt(cur, val);
                            picker.time_segment = seg;
                            if let Some(next_dd) = next_dropdown {
                                picker.time_dropdown = Some(next_dd);
                                picker.time_segment = next_dd;
                            }
                            apply_picker_time(view, get_mut, on_change, next, cx);
                        }
                        cx.notify();
                    }))
                    .child(format!("{val:02}"));

                if selected {
                    cell = cell
                        .bg(rgb(GOLD))
                        .border_color(rgb(GOLD))
                        .text_color(rgb(WHITE))
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else if major_mark {
                    cell = cell
                        .bg(rgb(SOFT))
                        .border_color(rgb(LINE))
                        .text_color(rgb(INK))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .hover(|st| st.bg(rgb(SELECTED)));
                } else {
                    cell = cell
                        .bg(rgb(PAPER))
                        .border_color(rgb(SUBTLE_LINE))
                        .text_color(rgb(INK))
                        .hover(|st| st.bg(rgb(SURFACE_HOVER)));
                }
                cell
            };

            let body =
                match open_seg {
                    TimeSegment::Hour => {
                        let hour_dt: fn(DateTime, i8) -> DateTime = |dt, next_h| {
                            jiff::civil::Time::new(next_h, dt.minute(), dt.second(), 0)
                                .map(|t| DateTime::from_parts(dt.date(), t))
                                .unwrap_or(dt)
                        };
                        crate::column()
                            .gap_2()
                            .p_3()
                            .child(
                                div()
                                    .text_size(px(10.5))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(MUTED))
                                    .child("AM (00–11)"),
                            )
                            .child(div().w_full().grid().grid_cols(6).gap_1().children(
                                (0..12i8).map(|val| {
                                    make_cell(
                                        TimeSegment::Hour,
                                        "hour",
                                        val,
                                        val == h,
                                        val % 3 == 0,
                                        Some(TimeSegment::Minute),
                                        hour_dt,
                                    )
                                }),
                            ))
                            .child(
                                div()
                                    .pt_1()
                                    .text_size(px(10.5))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(MUTED))
                                    .child("PM (12–23)"),
                            )
                            .child(div().w_full().grid().grid_cols(6).gap_1().children(
                                (12..24i8).map(|val| {
                                    make_cell(
                                        TimeSegment::Hour,
                                        "hour",
                                        val,
                                        val == h,
                                        val % 3 == 0,
                                        Some(TimeSegment::Minute),
                                        hour_dt,
                                    )
                                }),
                            ))
                            .into_any_element()
                    }
                    TimeSegment::Minute => {
                        let min_dt: fn(DateTime, i8) -> DateTime = |dt, next_m| {
                            jiff::civil::Time::new(dt.hour(), next_m, dt.second(), 0)
                                .map(|t| DateTime::from_parts(dt.date(), t))
                                .unwrap_or(dt)
                        };
                        div()
                            .p_3()
                            .child(div().w_full().grid().grid_cols(10).gap_1().children(
                                (0..60i8).map(|val| {
                                    make_cell(
                                        TimeSegment::Minute,
                                        "min",
                                        val,
                                        val == m,
                                        val % 5 == 0,
                                        None,
                                        min_dt,
                                    )
                                }),
                            ))
                            .into_any_element()
                    }
                    TimeSegment::Second => {
                        let sec_dt: fn(DateTime, i8) -> DateTime = |dt, next_s| {
                            jiff::civil::Time::new(dt.hour(), dt.minute(), next_s, 0)
                                .map(|t| DateTime::from_parts(dt.date(), t))
                                .unwrap_or(dt)
                        };
                        let quick_secs = div().w_full().grid().grid_cols(4).gap_1().children(
                            [0i8, 15, 30, 45].map(|val| {
                                let active = s == val;
                                div()
                                    .id((
                                        SharedString::from(format!("{prefix}-tp-sec-q")),
                                        usize::try_from(val).unwrap_or(0),
                                    ))
                                    .cursor_pointer()
                                    .h(px(24.))
                                    .rounded(px(5.))
                                    .border_1()
                                    .border_color(rgb(if active { GOLD } else { LINE }))
                                    .bg(rgb(if active { SELECTED } else { SOFT }))
                                    .text_color(rgb(if active { GOLD } else { INK }))
                                    .font_family("Menlo")
                                    .text_size(px(11.5))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .hover(|st| st.bg(rgb(SELECTED)))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .on_click(cx.listener(move |view, _, _, cx| {
                                        if let Some(picker) = get_mut(view) {
                                            let cur = picker.current_datetime(cx);
                                            let next = sec_dt(cur, val);
                                            apply_picker_time(view, get_mut, on_change, next, cx);
                                        }
                                        cx.notify();
                                    }))
                                    .child(format!(":{val:02}"))
                            }),
                        );
                        crate::column()
                            .gap_2()
                            .p_3()
                            .child(quick_secs)
                            .child(div().w_full().grid().grid_cols(10).gap_1().children(
                                (0..60i8).map(|val| {
                                    make_cell(
                                        TimeSegment::Second,
                                        "sec",
                                        val,
                                        val == s,
                                        val % 5 == 0,
                                        None,
                                        sec_dt,
                                    )
                                }),
                            ))
                            .into_any_element()
                    }
                };

            crate::column()
                .w_full()
                .gap_0()
                .rounded(px(10.))
                .bg(rgb(PAPER))
                .border_1()
                .border_color(rgb(LINE))
                .overflow_hidden()
                .child(tabs)
                .child(body)
        });

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
                    .border_color(rgb(SUBTLE_LINE))
                    .hover(|s| s.bg(rgb(SURFACE_HOVER)))
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
                            apply_picker_time(view, get_mut, on_change, next, cx);
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
                            apply_picker_time(view, get_mut, on_change, next, cx);
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
                        .hover(|s| s.bg(rgb(SURFACE_HOVER)));
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
            .hover(|s| s.bg(rgb(SURFACE_HOVER)))
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
            .hover(|s| s.bg(rgb(ACTION_HOVER)))
            .text_size(px(12.5))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(WHITE))
            .on_click(cx.listener(move |root, _, window, cx| {
                root.close_time_popup(target, window, cx);
                cx.notify();
            }))
            .child("Done");

        let footer = crate::row()
            .w_full()
            .pt_3()
            .border_t_1()
            .border_color(rgb(SUBTLE_LINE))
            .justify_between()
            .items_center()
            .child(button("Now", false, cx, move |view, cx| {
                if let Some(picker) = get_mut(view) {
                    let now = jiff::tz::TimeZone::system().to_datetime(jiff::Timestamp::now());
                    let cur = picker.current_datetime(cx);
                    let next = jiff::civil::Time::new(now.hour(), now.minute(), now.second(), 0)
                        .map(|t| DateTime::from_parts(cur.date(), t))
                        .unwrap_or(cur);
                    apply_picker_time(view, get_mut, on_change, next, cx);
                }
                cx.notify();
            }))
            .child(crate::row().gap_2().child(cancel_btn).child(done_btn));

        div()
            .id("time-backdrop")
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .absolute()
            .inset_0()
            .bg(rgba(BACKDROP_DARK))
            .flex()
            .items_center()
            .justify_center()
            .p_4()
            .child(
                div()
                    .id("time-dialog")
                    .track_focus(&picker.time_focus)
                    .w(px(460.))
                    .max_w_full()
                    .rounded(px(14.))
                    .bg(rgb(PAPER))
                    .border_1()
                    .border_color(rgb(LINE))
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
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
                                root.close_time_popup(target, window, cx);
                                cx.stop_propagation();
                                cx.notify();
                                return;
                            }
                            if let Some(picker) = get_mut(root)
                                && matches!(key, "up" | "down")
                            {
                                let step: i8 = if mods.shift { 5 } else { 1 };
                                let delta = if key == "up" { step } else { -step };
                                let cur = picker.current_datetime(cx);
                                let next = step_time_segment(cur, picker.time_segment, delta);
                                apply_picker_time(root, get_mut, on_change, next, cx);
                                cx.stop_propagation();
                                cx.notify();
                            }
                        },
                    ))
                    .child(
                        crate::row()
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
                                    .child("Type HH/MM/SS or pick from dropdown"),
                            ),
                    )
                    .child(digital_bar)
                    .when_some(dropdown_panel, |dlg, panel| dlg.child(panel))
                    .child(nudges)
                    .child(presets)
                    .child(footer),
            )
            .into_any_element()
    }
}
