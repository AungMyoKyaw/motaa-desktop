use gpui::{Context, SharedString, div, prelude::*, px, rgb, rgba};
use jiff::civil::DateTime;

use crate::{
    ACTION_HOVER, BACKDROP_DARK, DesktopRoot, GOLD, HEADER_BG, INK, IVORY, LINE, MUTED, PAPER,
    SELECTED, SOFT, SUBTLE_LINE, SURFACE_HOVER, TRANSPARENT, WHITE, button, modal_dialog,
    slim_vertical_scrollbar,
};

use super::parse::{apply_time_digit, nudge_time_seconds, step_time_segment, time_of_day_label};
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
        picker.sync_time_scrolls(next);
        picker.time_digit_buf = None;
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
                    .bg(rgba(TRANSPARENT))
                    .border_1()
                    .border_color(rgba(TRANSPARENT))
                    .hover(|s| s.bg(rgb(SURFACE_HOVER)));
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

        let am_pm_capsule = crate::row()
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
            );

        let digital_bar = crate::row()
            .w_full()
            .px_4()
            .py_3()
            .rounded(px(10.))
            .bg(rgb(SOFT))
            .border_1()
            .border_color(rgb(SUBTLE_LINE))
            .justify_between()
            .items_center()
            .gap_3()
            .child(
                crate::column()
                    .gap(px(4.))
                    .child(
                        crate::row()
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
                .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                .on_click(cx.listener(move |view, _, _, cx| {
                    if let Some(picker) = get_mut(view) {
                        picker.time_segment = seg;
                        picker.time_digit_buf = None;
                    }
                    cx.notify();
                }))
                .child(label);
            if with_right_border {
                hdr = hdr.border_r_1().border_color(rgb(SUBTLE_LINE));
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
                            .text_color(rgb(WHITE))
                            .font_weight(gpui::FontWeight::SEMIBOLD);
                    } else {
                        item = item
                            .bg(rgba(TRANSPARENT))
                            .text_color(rgb(INK))
                            .hover(|s| s.bg(rgb(SURFACE_HOVER)));
                    }
                    item
                })
                .collect();

            let mut col_box = crate::row()
                .flex_1()
                .min_w(px(0.))
                .h_full()
                .p_1()
                .gap(px(2.))
                .bg(if is_active_col {
                    rgb(IVORY)
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
                col_box = col_box.border_r_1().border_color(rgb(SUBTLE_LINE));
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

        let columns_frame = crate::column()
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
                    .bg(rgb(HEADER_BG))
                    .border_b_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(make_col_header(TimeSegment::Hour, "hour", "HOUR", true))
                    .child(make_col_header(TimeSegment::Minute, "min", "MIN", true))
                    .child(make_col_header(TimeSegment::Second, "sec", "SEC", false)),
            )
            .child(
                crate::row()
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
                root.close_time_popup(target, window);
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
                                        apply_picker_time(root, get_mut, on_change, next, cx);
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
                                        apply_picker_time(root, get_mut, on_change, next, cx);
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                    _ => {}
                                }
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
