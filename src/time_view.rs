use super::*;

impl DesktopRoot {
    pub(super) fn time_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let s = self.session.read(cx);
        let display = s.time.format("%d %b %Y · %H:%M:%S").to_string();
        let unit = s.step.label();
        row()
            .h_full()
            .justify_between()
            .gap_3()
            .border_t_1()
            .border_color(rgb(LINE))
            .child(
                row()
                    .gap_2()
                    .child(
                        div()
                            .font_family("Menlo")
                            .text_size(px(12.5))
                            .text_color(rgb(INK))
                            .child(display),
                    )
                    .child(button(format!("Step: {unit}"), false, cx, |view, cx| {
                        view.update(cx, |s| {
                            let idx = Step::ALL
                                .iter()
                                .position(|step| *step == s.step)
                                .unwrap_or(2);
                            s.step = Step::ALL[(idx + 1) % Step::ALL.len()];
                        });
                    })),
            )
            .child(
                row()
                    .gap_1()
                    .child(button("‹", false, cx, |view, cx| {
                        view.step_time(-1, false, cx)
                    }))
                    .child(
                        div()
                            .w(px(180.))
                            .child(self.time_scrubber("chart-time-scrubber", cx)),
                    )
                    .child(button("›", false, cx, |view, cx| {
                        view.step_time(1, false, cx)
                    })),
            )
            .child(
                row()
                    .gap_2()
                    .child(button("Set date", false, cx, |view, cx| {
                        view.time_picker.open = true;
                        view.navigate(Workspace::Time, cx);
                    }))
                    .child(button("Now", true, cx, |view, cx| {
                        view.set_time(view.session.read(cx).now(), true, cx)
                    })),
            )
            .into_any_element()
    }

    fn time_scrubber(&self, id: &'static str, cx: &mut Context<Self>) -> AnyElement {
        use gpui::{
            MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, canvas, fill, point,
        };
        let view = cx.weak_entity();
        let offset = self.scrub_offset;
        let track = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let inner_width = bounds.size.width - px(16.);
                if inner_width <= px(0.) {
                    return;
                }
                let start = bounds.origin.x + px(8.);
                let y = bounds.origin.y + bounds.size.height / 2.;
                let fraction = (offset + 60) as f32 / 120.;
                let knob = start + inner_width * fraction;
                window.paint_quad(fill(
                    Bounds::new(point(start, y - px(1.)), size(inner_width, px(2.))),
                    rgb(LINE),
                ));
                window.paint_quad(fill(
                    Bounds::new(
                        point(start, y - px(1.)),
                        size(inner_width * fraction, px(2.)),
                    ),
                    rgb(GOLD),
                ));
                let center = start + inner_width * 0.5;
                window.paint_quad(fill(
                    Bounds::new(point(center - px(0.5), y - px(5.)), size(px(1.), px(10.))),
                    rgb(MUTED),
                ));
                window.paint_quad(gpui::quad(
                    Bounds::new(point(knob - px(5.), y - px(5.)), size(px(10.), px(10.))),
                    px(5.),
                    rgb(GOLD),
                    px(0.),
                    gpui::transparent_black(),
                    Default::default(),
                ));
                let to_offset = move |x| {
                    let width = f32::from(inner_width);
                    ((f32::from(x - start) / width).clamp(0., 1.) * 120.).round() as i32 - 60
                };

                let down = view.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, _, cx| {
                    if phase.bubble()
                        && event.button == MouseButton::Left
                        && bounds.contains(&event.position)
                    {
                        let _ = down.update(cx, |root, cx| {
                            root.scrub_dragging = true;
                            root.step_time(to_offset(event.position.x), true, cx);
                        });
                        cx.stop_propagation();
                    }
                });
                let drag = view.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                    if phase.bubble() && event.pressed_button == Some(MouseButton::Left) {
                        let _ = drag.update(cx, |root, cx| {
                            let next = to_offset(event.position.x);
                            if root.scrub_dragging && next != root.scrub_offset {
                                root.step_time(next, true, cx);
                            }
                        });
                    }
                });
                let release = view.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                    if phase.capture() && event.button == MouseButton::Left {
                        let _ = release.update(cx, |root, _| root.scrub_dragging = false);
                    }
                });
            },
        )
        .w_full()
        .h(px(34.));
        div()
            .id(id)
            .focusable()
            .tab_stop(true)
            .rounded(px(5.))
            .border_1()
            .border_color(gpui::transparent_black())
            .focus(|style| style.border_color(rgb(GOLD)))
            .on_key_down(cx.listener(|view, event: &gpui::KeyDownEvent, _, cx| {
                let offset = match event.keystroke.key.as_str() {
                    "left" => (view.scrub_offset - 1).max(-60),
                    "right" => (view.scrub_offset + 1).min(60),
                    "home" => -60,
                    "end" => 60,
                    _ => return,
                };
                view.step_time(offset, true, cx);
                cx.stop_propagation();
            }))
            .child(track)
            .into_any_element()
    }

    pub(super) fn edit_reference_location(&mut self, cx: &mut Context<Self>) {
        let profile = self.session.read(cx).reference_profile().clone();
        let zone = profile
            .timezone_id
            .unwrap_or_else(|| profile.timezone_offset_minutes.to_string());
        self.reference_location_draft = Some(ReferenceLocationDraft {
            town_search: None,
            place: text_input(&profile.birthplace, "Place name", cx),
            latitude: text_input(&profile.latitude.to_string(), "Latitude", cx),
            longitude: text_input(&profile.longitude.to_string(), "Longitude", cx),
            timezone: text_input(&zone, "IANA zone or offset minutes", cx),
            location: location_picker::LocationPickerState::new(
                profile.latitude,
                profile.longitude,
                cx,
            ),
        });
        self.set_status("", cx);
    }

    pub(super) fn apply_reference_location(&mut self, cx: &mut Context<Self>) {
        let result = (|| {
            let draft = self
                .reference_location_draft
                .as_ref()
                .ok_or("Open the location editor first.")?;
            let session = self.session.read(cx);
            let mut request = model::chart_request_at(session.reference_profile(), session.time);
            request.birthplace = draft.place.read(cx).value().trim().into();
            if request.birthplace.is_empty() {
                return Err("Enter a reference place name.".to_owned());
            }
            request.latitude = parse_coordinate(draft.latitude.read(cx).value(), true)?;
            request.longitude = parse_coordinate(draft.longitude.read(cx).value(), false)?;
            set_timezone(&mut request, draft.timezone.read(cx).value());
            motaa_engine::validate(&request)?;
            motaa_engine::effective_utc_offset(&request)?;
            session.store_reference_for_active_profile(Some(&request))?;
            Ok(request)
        })();
        match result {
            Ok(request) => {
                self.reference_location_draft = None;
                self.update(cx, |session| {
                    session.transit = Some(request);
                    session.reference_source =
                        session.natal_id.map(model::ReferenceSource::NatalProfile);
                    session.status = "Reference location updated.".into();
                });
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    pub(super) fn use_birth_location(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self
            .session
            .read(cx)
            .store_reference_for_active_profile(None)
        {
            self.set_status(error, cx);
            return;
        }
        self.update(cx, |session| {
            session.clear_reference_location();
            session.status.clear();
        });
    }

    fn reference_location_view(&self, cx: &mut Context<Self>) -> Div {
        let session = self.session.read(cx);
        let location = session.reference_profile();
        let zone = location.timezone_id.clone().unwrap_or_else(|| {
            let minutes = location.timezone_offset_minutes;
            let sign = if minutes < 0 { '−' } else { '+' };
            let value = minutes.unsigned_abs();
            format!("UTC{sign}{:02}:{:02}", value / 60, value % 60)
        });
        let summary = format!(
            "{} · {:.4}°, {:.4}° · {zone}",
            location.birthplace, location.latitude, location.longitude
        );
        let custom = session.transit.is_some();
        let ref_dt = session.time;
        if self.reference_location_draft.is_some() {
            return panel()
                .child("Reference location")
                .child(self.location_and_timezone_section(true, ref_dt, cx))
                .child(
                    row()
                        .child(button(
                            "Apply reference location",
                            true,
                            cx,
                            Self::apply_reference_location,
                        ))
                        .child(button("Cancel location edit", false, cx, |view, cx| {
                            view.reference_location_draft = None;
                            view.set_status("", cx);
                        })),
                );
        }
        row()
            .justify_between()
            .child(
                column()
                    .gap_1()
                    .child("Reference location")
                    .child(caption(summary)),
            )
            .child(
                row()
                    .child(button(
                        "Edit reference location",
                        false,
                        cx,
                        Self::edit_reference_location,
                    ))
                    .when(custom, |controls| {
                        controls.child(button("Use birth location", false, cx, |view, cx| {
                            view.use_birth_location(cx);
                        }))
                    }),
            )
    }

    pub(super) fn time_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let s = self.session.read(cx);
        let step = s.step;
        use chrono::Datelike;
        let calendar = s.settings.calendar;
        let day_number =
            calendar.numbered_day(s.time.year(), s.time.month() as u8, s.time.day() as u8);
        let today = motaa_engine::myanmar_calendar::from_julian_day_number(day_number);
        let weekday =
            chrono::Weekday::try_from(day_number.rem_euclid(7) as u8).expect("valid weekday");
        column()
            .child(heading("Explore time"))
            .child(caption(
                "Change transit time and location independently of the birth profile.",
            ))
            .child(settings_strip(s.settings))
            .child(
                column()
                    .pt_8()
                    .child(
                        column()
                            .gap_1()
                            .child("Step size")
                            .child(caption("Previous and next use the selected unit.")),
                    )
                    .child(row().children(Step::ALL.into_iter().map(|unit| {
                        button(unit.label(), unit == step, cx, move |view, cx| {
                            view.update(cx, |s| s.step = unit)
                        })
                    })))
                    .child(self.time_picker.render(
                        "Reference date and time",
                        false,
                        None,
                        Some(|view, dt, cx| view.set_time(dt, true, cx)),
                        |view| Some(&mut view.time_picker),
                        cx,
                    ))
                    .child(
                        row()
                            .child(button("Apply date", true, cx, |view, cx| {
                                if let Err(err) = view.time_picker.validate() {
                                    view.set_status(err, cx);
                                    return;
                                }
                                match parse_time(view.date.read(cx).value()) {
                                    Ok(time) => view.set_time(time, true, cx),
                                    Err(error) => view.set_status(error, cx),
                                }
                            }))
                            .child(button("View transit", false, cx, |view, cx| {
                                view.select_mode("Natal + transit", cx)
                            }))
                            .child(button("Use current time", false, cx, |view, cx| {
                                view.set_time(view.session.read(cx).now(), true, cx)
                            })),
                    ),
            )
            .child(
                column()
                    .py_6()
                    .child(column().gap_1().child("Scrub around now").child(caption(
                        "The midpoint is the current time. The step size sets the unit.",
                    )))
                    .child(self.time_scrubber("time-scrubber", cx))
                    .child(row().justify_between().children([
                        caption(format!("−60 {}", step.label())),
                        caption("Now"),
                        caption(format!("+60 {}", step.label())),
                    ])),
            )
            .child(
                row()
                    .child(button(
                        format!("Previous {}", step.label().to_lowercase()),
                        false,
                        cx,
                        |view, cx| {
                            view.step_time(-1, false, cx);
                        },
                    ))
                    .child(button(
                        format!("Next {}", step.label().to_lowercase()),
                        false,
                        cx,
                        |view, cx| {
                            view.step_time(1, false, cx);
                        },
                    )),
            )
            .child(self.reference_location_view(cx))
            .when_some(today, |view, date| {
                let observances = date.observances(weekday);
                view.child(
                    panel()
                        .child("Myanmar calendar")
                        .child(format!(
                            "Sasana {} · Myanmar {} ({}) · {} ({}) · day {} · {}",
                            date.sasana_year(),
                            date.year,
                            date.year_name_burmese(),
                            date.month_english,
                            date.month_burmese,
                            date.day,
                            date.phase
                        ))
                        .child(caption(format!(
                            "{} day-number convention · Birth lord {} · Dragon head {} · Nat class {}",
                            calendar.label(),
                            date.birth_lord(weekday),
                            date.dragon_head(),
                            date.nat_class()
                        )))
                        .child(if observances.is_empty() {
                            "No listed observance".into()
                        } else {
                            observances.join(" · ")
                        }),
                )
            })
            .into_any_element()
    }
}
