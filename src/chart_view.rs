use super::*;

impl DesktopRoot {
    pub(super) fn toggle_circular_detail(&mut self, cx: &mut Context<Self>) {
        self.circular_detail = !self.circular_detail;
        self.chart_zoom = 1.;
        self.chart_scroll.set_offset(gpui::point(px(0.), px(0.)));
        cx.notify();
    }

    pub(super) fn open_chart_lightbox(&mut self, target: LightboxTarget, cx: &mut Context<Self>) {
        self.lightbox.target = Some(target);
        self.lightbox.zoom = 1.;
        self.lightbox.scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.lightbox.drag = None;
        cx.notify();
    }

    pub(super) fn close_chart_lightbox(&mut self, cx: &mut Context<Self>) {
        self.lightbox.target = None;
        self.lightbox.zoom = 1.;
        self.lightbox.scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.lightbox.drag = None;
        cx.notify();
    }

    pub(super) fn set_lightbox_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        let old_zoom = self.lightbox.zoom;
        let new_zoom = zoom.clamp(0.75, 4.);
        if (new_zoom - old_zoom).abs() < 0.0001 {
            return;
        }
        self.lightbox.zoom = new_zoom;
        if new_zoom <= 1. {
            self.lightbox.scroll.set_offset(gpui::point(px(0.), px(0.)));
        } else {
            let ratio = new_zoom / old_zoom;
            let offset = self.lightbox.scroll.offset();
            let bounds = self.lightbox.scroll.bounds();
            let half_w = f32::from(bounds.size.width) * 0.5;
            let half_h = f32::from(bounds.size.height) * 0.5;
            let next_x = (f32::from(offset.x) - half_w) * ratio + half_w;
            let next_y = (f32::from(offset.y) - half_h) * ratio + half_h;
            self.lightbox
                .scroll
                .set_offset(gpui::point(px(next_x.min(0.)), px(next_y.min(0.))));
        }
        cx.notify();
    }

    pub(super) fn chart_lightbox_overlay(
        &self,
        target: LightboxTarget,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use gpui::{
            MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ScrollDelta, ScrollEvent,
            point, rgba,
        };

        let viewport = window.viewport_size();
        let avail_w = (f32::from(viewport.width) - 88.).max(320.);
        let avail_h = (f32::from(viewport.height) - 160.).max(320.);
        let base_side = avail_w.min(avail_h);
        let zoom = self.lightbox.zoom;
        let zoomed_side = base_side * zoom;
        let pad = 32.;
        let min_x = px(-(zoomed_side + pad - avail_w).max(0.));
        let min_y = px(-(zoomed_side + pad - avail_h).max(0.));

        let s = self.session.read(cx);
        let mode = s.mode;
        let profile_name = s.natal.name.clone();
        let settings = s.settings;
        let zodiac = if settings.tropical {
            "Tropical"
        } else {
            "Sidereal"
        };
        let ayanamsha = if settings.ayanamsha == 0 {
            "LAHIRI"
        } else {
            AYANAMSHAS[settings.ayanamsha]
        };
        let houses = if settings.houses == 0 {
            "Equal houses"
        } else {
            HOUSES[settings.houses]
        };
        let rahu = if settings.mean_node {
            "Mean Rahu"
        } else {
            "True Rahu"
        };
        let subtitle = format!(
            "{zodiac} · {ayanamsha} · {houses} · {rahu} · Use +/− or {} to zoom · Drag to pan · Esc to close",
            zoom_scroll_hint()
        );

        let (title, drawing) = self.lightbox_drawing(target, zoomed_side, cx);
        let header = if title == mode {
            format!("{title} — {profile_name}")
        } else {
            format!("{title} — {profile_name} ({mode})")
        };
        let scroll = self.lightbox.scroll.clone();

        div()
            .id("chart-lightbox-backdrop")
            .occlude()
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .absolute()
            .inset_0()
            .bg(rgba(0x28251f99))
            .flex()
            .items_center()
            .justify_center()
            .p_5()
            .on_click(cx.listener(|root, _, _, cx| root.close_chart_lightbox(cx)))
            .child(
                div()
                    .id("chart-lightbox-card")
                    .w_full()
                    .h_full()
                    .bg(rgb(IVORY))
                    .border_1()
                    .border_color(rgb(LINE))
                    .rounded(px(12.))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        row()
                            .px_5()
                            .py_3()
                            .border_b_1()
                            .border_color(rgb(LINE))
                            .bg(rgb(PAPER))
                            .justify_between()
                            .gap_4()
                            .child(
                                column()
                                    .gap_1()
                                    .child(div().text_size(px(18.)).child(header))
                                    .child(caption(subtitle)),
                            )
                            .child(
                                row()
                                    .gap_2()
                                    .child(button("−", false, cx, |root, cx| {
                                        root.set_lightbox_zoom(root.lightbox.zoom - 0.25, cx);
                                    }))
                                    .child(button(
                                        format!("{:.0}%", zoom * 100.),
                                        false,
                                        cx,
                                        |root, cx| root.set_lightbox_zoom(1., cx),
                                    ))
                                    .child(button("+", false, cx, |root, cx| {
                                        root.set_lightbox_zoom(root.lightbox.zoom + 0.25, cx);
                                    }))
                                    .child(button("Close", true, cx, |root, cx| {
                                        root.close_chart_lightbox(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("lightbox-chart-viewport")
                            .flex_1()
                            .min_h(px(0.))
                            .w_full()
                            .bg(rgb(PAPER))
                            .overflow_scroll()
                            .track_scroll(&scroll)
                            .cursor(
                                if zoomed_side + pad > avail_w || zoomed_side + pad > avail_h {
                                    gpui::CursorStyle::OpenHand
                                } else {
                                    gpui::CursorStyle::Arrow
                                },
                            )
                            .on_scroll(cx.listener(move |root, event: &ScrollEvent, window, cx| {
                                if event.modifiers.control || event.modifiers.platform {
                                    let dy = match event.delta {
                                        ScrollDelta::Lines(delta) => delta.y * 0.2,
                                        ScrollDelta::Pixels(delta) => f32::from(delta.y) / 240.,
                                    };
                                    if dy.abs() > 0.001 {
                                        let factor = (1. + dy).clamp(0.5, 2.0);
                                        root.set_lightbox_zoom(root.lightbox.zoom * factor, cx);
                                    }
                                } else {
                                    let delta = event.delta.pixel_delta(window.line_height());
                                    let current = root.lightbox.scroll.offset();
                                    let clamped = point(
                                        (current.x + delta.x).clamp(min_x, px(0.)),
                                        (current.y + delta.y).clamp(min_y, px(0.)),
                                    );
                                    if clamped != current {
                                        root.lightbox.scroll.set_offset(clamped);
                                        cx.notify();
                                    }
                                }
                                cx.stop_propagation();
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|root, event: &MouseDownEvent, _, cx| {
                                    root.lightbox.drag =
                                        Some((event.position, root.lightbox.scroll.offset()));
                                    cx.stop_propagation();
                                }),
                            )
                            .on_mouse_move(cx.listener(
                                move |root, event: &MouseMoveEvent, window, _| {
                                    if let Some((start, offset)) = root.lightbox.drag
                                        && event.pressed_button == Some(MouseButton::Left)
                                    {
                                        let clamped = point(
                                            (offset.x + (event.position.x - start.x))
                                                .clamp(min_x, px(0.)),
                                            (offset.y + (event.position.y - start.y))
                                                .clamp(min_y, px(0.)),
                                        );
                                        if clamped != root.lightbox.scroll.offset() {
                                            root.lightbox.scroll.set_offset(clamped);
                                            window.refresh();
                                        }
                                    }
                                },
                            ))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|root, _: &MouseUpEvent, _, _| {
                                    root.lightbox.drag = None;
                                }),
                            )
                            .on_mouse_up_out(
                                MouseButton::Left,
                                cx.listener(|root, _: &MouseUpEvent, _, _| {
                                    root.lightbox.drag = None;
                                }),
                            )
                            .child(
                                div()
                                    .min_w_full()
                                    .min_h_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .p_4()
                                    .child(drawing),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn lightbox_drawing(
        &self,
        target: LightboxTarget,
        side: f32,
        cx: &App,
    ) -> (&'static str, AnyElement) {
        let s = self.session.read(cx);
        let mode = s.mode;
        let settings = s.settings;
        let planet_transit = s.planet_transit;
        let ascendant_transit = s.ascendant_transit;
        let results = s.calculate(planet_transit && mode != "Natal + transit");
        let asc_result = s.calculate(ascendant_transit);
        let transit = (mode == "Natal + transit").then(|| s.calculate(true));

        let palette_color = INK;

        if matches!(
            target,
            LightboxTarget::ComparisonCombined | LightboxTarget::ComparisonSplit(_)
        ) {
            let (Ok(natal), Some(Ok(transit))) = (results, transit) else {
                return (
                    "Natal + transit",
                    panel().child("Chart unavailable.").into_any_element(),
                );
            };
            let (Some(natal), Some(transit)) =
                (natal.reference_ephemeris, transit.reference_ephemeris)
            else {
                return (
                    "Natal + transit",
                    panel()
                        .child("Both epochs need known, supported times.")
                        .into_any_element(),
                );
            };
            let (Some(natal), Some(transit)) = (
                chart::comparison::positions(&natal, settings, self.show_munp),
                chart::comparison::positions(&transit, settings, self.show_munp),
            ) else {
                return (
                    "Natal + transit",
                    panel()
                        .child("Planet positions unavailable.")
                        .into_any_element(),
                );
            };
            return match target {
                LightboxTarget::ComparisonSplit(1) => (
                    "Transit Rasi chart",
                    comparison_rasi_scene(&transit, palette_color).render(side),
                ),
                LightboxTarget::ComparisonSplit(_) => (
                    "Birth Rasi chart",
                    comparison_rasi_scene(&natal, palette_color).render(side),
                ),
                _ => (
                    "Birth + Transit chart",
                    super::chart::comparison::nested_chart(&natal, &transit, palette_color, side),
                ),
            };
        }

        let Ok(result) = results else {
            return (
                "Chart",
                panel().child("Chart unavailable.").into_any_element(),
            );
        };
        let ascendant = asc_result.ok().and_then(|value| value.reference_ephemeris);
        if mode == "Rahu wheel" {
            let scene = self.rahu_scene(
                result.reference_ephemeris.as_ref(),
                ascendant.as_ref(),
                settings,
            );
            return ("Rahu wheel", scene.render(side));
        }
        let Some(ephemeris) = result.reference_ephemeris else {
            return (
                "Chart",
                panel()
                    .child("Reference planetary positions unavailable.")
                    .into_any_element(),
            );
        };

        if let LightboxTarget::TricakkaSubchart(index) = target {
            return match self.tricakka_scenes(&ephemeris, ascendant.as_ref(), settings) {
                Ok((scenes, retrograde)) => {
                    let scene = &scenes[index];
                    let content = if index == 0 {
                        scene.render_with_retrograde(side, &retrograde)
                    } else {
                        scene.render(side)
                    };
                    (scene.title, content)
                }
                Err(error) => ("Tricakka", panel().child(error).into_any_element()),
            };
        }

        if target == LightboxTarget::RasiInset
            && let Some(asc_ref) = ascendant.as_ref()
            && let Some(positions) = recovered::positions(&ephemeris, asc_ref, settings)
        {
            let mut labels: Vec<_> = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"]
                .into_iter()
                .zip(positions)
                .map(|(code, angle)| (code.into(), angle))
                .collect();
            if self.show_munp && mode != "Western aspects" {
                labels.extend(munp_labels(&ephemeris, asc_ref, settings));
            }
            let scene = chart::ChartScene {
                labels,
                title: "Rasi chart",
                circular: false,
                rahu: None,
                houses: None,
                color: palette_color,
            };
            return ("Rasi chart", scene.render(side));
        }

        let cusps = ascendant.as_ref().and_then(|asc| {
            asc.cusps(
                motaa_engine::houses::System::OPTIONS[settings.houses],
                settings.tropical,
            )
            .ok()
        });
        let houses = if mode == "Circular chart" {
            cusps
                .zip(ascendant.as_ref().and_then(|asc| {
                    asc.house_midpoints(
                        motaa_engine::houses::System::OPTIONS[settings.houses],
                        settings.tropical,
                    )
                    .ok()
                }))
                .map(|(cusps, midpoints)| chart::HouseGeometry { cusps, midpoints })
        } else {
            None
        };
        let [rahu_deg, ketu_deg] = ephemeris.lunar_nodes(settings.mean_node, settings.tropical);
        let mut labels: Vec<_> = chart::TRADITIONAL_PLANETS
            .into_iter()
            .filter_map(|(name, code, _)| {
                let angle = match name {
                    "Rahu" => Some(rahu_deg),
                    "Ketu" => Some(ketu_deg),
                    _ => ephemeris.longitude(name, settings.tropical),
                }?;
                Some((code.into(), angle))
            })
            .collect();
        if mode == "Rasi"
            && self.show_munp
            && let Some(asc) = ascendant.as_ref()
        {
            for (_, code, angle) in munp_entries(&ephemeris, asc, settings) {
                labels.push((code.into(), angle));
            }
        }
        if let Some(asc) = ascendant.as_ref() {
            insert_ascendant_label(&mut labels, asc, settings.tropical);
        }
        let title = mode_title(mode);
        let scene = chart::ChartScene {
            labels,
            circular: mode == "Circular chart",
            rahu: None,
            houses,
            title,
            color: if mode == "Rasi" { INK } else { GOLD },
        };
        (title, self.render_scene(&scene, side, cx))
    }

    pub(super) fn chart_view(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let s = self.session.read(cx);
        let mode = s.mode;
        let name = s.natal.name.clone();
        let location = s.natal.birthplace.clone();
        let show_birth = self.show_birth;
        let settings = s.settings;
        let planet_transit = s.planet_transit;
        let ascendant_transit = s.ascendant_transit;
        let timezone = match motaa_engine::effective_utc_offset(&s.natal) {
            Ok(offset) => {
                let sign = if offset < 0 { '-' } else { '+' };
                let minutes = offset.unsigned_abs();
                format!("GMT{sign}{:02}:{:02}", minutes / 60, minutes % 60)
            }
            Err(_) => s
                .natal
                .timezone_id
                .clone()
                .unwrap_or_else(|| "Timezone unavailable".into()),
        };
        let birth_label = if show_birth {
            format!(
                "{:04}-{:02}-{:02}",
                s.natal.year, s.natal.month, s.natal.day
            )
        } else {
            "Hidden".into()
        };
        let latitude = s.natal.latitude;
        let longitude = s.natal.longitude;
        let place_label = format!(
            "{location} · {:.2}° {}, {:.2}° {}",
            latitude.abs(),
            if latitude < 0. { 'S' } else { 'N' },
            longitude.abs(),
            if longitude < 0. { 'W' } else { 'E' },
        );
        let results = s.calculate(planet_transit && mode != "Natal + transit");
        let asc_result = s.calculate(ascendant_transit);
        let transit = (mode == "Natal + transit").then(|| s.calculate(true));
        let viewport = window.viewport_size();
        let inspector_visible = self.inspector_open && f32::from(viewport.width) >= 1060.;
        let mut view = column().gap_2();
        if !inspector_visible {
            view = view.child(
                row()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .rounded(px(RADIUS_CONTROL))
                    .bg(rgb(SOFT))
                    .border_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        row()
                            .gap_2()
                            .min_w(px(0.))
                            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(
                                if mode == "Circular chart" && self.circular_detail {
                                    "Detailed circular chart".into()
                                } else {
                                    name
                                },
                            ))
                            .child(caption(format!(
                                "{birth_label} · {timezone} · {place_label}"
                            ))),
                    )
                    .child(self.profile_edit_save_buttons(cx)),
            );
        }

        let primary_modes = [
            ("Rasi", "Rasi"),
            ("Tricakka", "Tricakka"),
            ("Circular", "Circular chart"),
            ("Dasha", "Dasha periods"),
            ("Natal + transit", "Natal + transit"),
            ("Rahu wheel", "Rahu wheel"),
        ];
        let secondary_active = !primary_modes.iter().any(|(_, target)| *target == mode);
        let more_label = if secondary_active {
            format!("{mode} ▾")
        } else if self.show_modes {
            "More ▴".into()
        } else {
            "More ▾".into()
        };
        view = view.child(
            row()
                .justify_between()
                .flex_wrap()
                .gap_2()
                .pb_2()
                .child(
                    segmented_track()
                        .flex_wrap()
                        .children(primary_modes.into_iter().enumerate().map(
                            |(idx, (label, target))| {
                                mode_pill(
                                    ("primary-mode", idx),
                                    label,
                                    mode == target,
                                    cx,
                                    move |view, cx| view.select_mode(target, cx),
                                )
                            },
                        ))
                        .child(
                            div()
                                .relative()
                                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation()
                                })
                                .child(mode_pill(
                                    "more-modes-toggle",
                                    more_label,
                                    self.show_modes || secondary_active,
                                    cx,
                                    |view, _| view.show_modes = !view.show_modes,
                                ))
                                .when(self.show_modes, |anchor| {
                                    anchor.child(self.mode_buttons_popover(cx))
                                }),
                        ),
                )
                .child(
                    row()
                        .gap_2()
                        .when(mode == "Circular chart", |toolbar| {
                            toolbar.child(compact_button(
                                "chart-inspect-wheel",
                                if self.circular_detail {
                                    "Back to chart"
                                } else {
                                    "Inspect wheel"
                                },
                                self.circular_detail,
                                cx,
                                Self::toggle_circular_detail,
                            ))
                        })
                        .when(matches!(mode, "Rasi" | "Circular chart"), |toolbar| {
                            toolbar.child(icon_compact_button(
                                "chart-full-view",
                                "ui/maximize",
                                12.5,
                                "Full view",
                                cx,
                                |view, cx| {
                                    view.open_chart_lightbox(LightboxTarget::ActiveChart, cx)
                                },
                            ))
                        })
                        .child(compact_button(
                            "chart-compact-toggle",
                            if self.compact { "Standard" } else { "Compact" },
                            self.compact,
                            cx,
                            |view, _| view.compact = !view.compact,
                        )),
                ),
        );

        let max_side = if self.compact || viewport.width <= px(1320.) {
            436.
        } else {
            560.
        };
        let side = if mode == "Circular chart" && self.circular_detail {
            f32::from(viewport.width - px(500.)).clamp(380., 780.)
        } else if mode == "Rahu wheel" {
            let controls_height = if self.rahu.reference == RahuReference::Manual {
                44.
            } else {
                0.
            };
            f32::from(viewport.width - px(550.))
                .min(f32::from(viewport.height - px(390. + controls_height)))
                .clamp(210., max_side)
        } else if mode == "Natal + transit" {
            f32::from(viewport.width - px(550.))
                .min(f32::from(viewport.height - px(396.)))
                .clamp(290., max_side)
        } else {
            f32::from(viewport.width - px(550.))
                .min(f32::from(viewport.height - px(340.)))
                .clamp(296., max_side)
        };
        let content = if mode == "Dasha periods" {
            self.dasha_view(cx)
        } else {
            match (results, transit) {
                (Ok(natal), Some(Ok(transit))) => {
                    self.comparison(natal, transit, settings, side, cx)
                }
                (_, Some(Err(error))) => panel().child(error).into_any_element(),
                (Ok(result), _) => {
                    self.chart_output(mode, result, asc_result.ok(), settings, side, cx)
                }
                (Err(error), _) => panel().child(error).into_any_element(),
            }
        };
        view.child(content).into_any_element()
    }

    fn chart_surface(
        &self,
        scene: &chart::ChartScene,
        side: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use gpui::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, point};
        let zoom = self.chart_zoom;
        let scroll = self.chart_scroll.clone();
        let min_offset = px(side * (1. - zoom));
        let drawing = self.render_scene(scene, side * zoom, cx);
        let surface = div()
            .id("chart-surface")
            .size(px(side))
            .overflow_scroll()
            .track_scroll(&scroll)
            .cursor(if zoom > 1. {
                gpui::CursorStyle::OpenHand
            } else {
                gpui::CursorStyle::PointingHand
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |root, event: &MouseDownEvent, _, _| {
                    root.chart_press_origin = Some(event.position);
                    if root.chart_zoom > 1. {
                        root.chart_drag = Some((event.position, root.chart_scroll.offset()));
                    }
                }),
            )
            .on_mouse_move(cx.listener(move |root, event: &MouseMoveEvent, window, _| {
                if let Some((start, offset)) = root.chart_drag
                    && event.pressed_button == Some(MouseButton::Left)
                {
                    let next = offset + (event.position - start);
                    root.chart_scroll.set_offset(point(
                        next.x.clamp(min_offset, px(0.)),
                        next.y.clamp(min_offset, px(0.)),
                    ));
                    window.refresh();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|root, event: &MouseUpEvent, _, cx| {
                    let was_click = root.chart_press_origin.take().is_some_and(|origin| {
                        let delta = event.position - origin;
                        f32::from(delta.x).abs() < 5. && f32::from(delta.y).abs() < 5.
                    });
                    root.chart_drag = None;
                    if was_click {
                        root.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
                    }
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|root, _: &MouseUpEvent, _, _| {
                    root.chart_drag = None;
                    root.chart_press_origin = None;
                }),
            )
            .child(drawing);
        chart_plate()
            .items_center()
            .child(surface)
            .when(scene.rahu.is_none(), |plate| {
                plate.child(
                    row()
                        .w_full()
                        .pt_1()
                        .border_t_1()
                        .border_color(rgb(SUBTLE_LINE))
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(MUTED))
                                .child("Click chart for full view & zoom"),
                        )
                        .when(
                            !matches!(scene.title, "Rasi chart" | "Circular chart")
                                || self.show_modes,
                            |bar| {
                                bar.child(
                                    row()
                                        .gap_1()
                                        .child(ghost_button(
                                            "chart-zoom-out",
                                            "−",
                                            cx,
                                            |root, _| {
                                                root.chart_zoom = (root.chart_zoom - 0.25).max(1.);
                                                if root.chart_zoom == 1. {
                                                    root.chart_scroll
                                                        .set_offset(point(px(0.), px(0.)));
                                                }
                                            },
                                        ))
                                        .child(ghost_button(
                                            "chart-zoom-reset",
                                            format!("{:.0}%", zoom * 100.),
                                            cx,
                                            |root, _| {
                                                root.chart_zoom = 1.;
                                                root.chart_scroll.set_offset(point(px(0.), px(0.)));
                                            },
                                        ))
                                        .child(ghost_button(
                                            "chart-zoom-in",
                                            "+",
                                            cx,
                                            |root, _| {
                                                root.chart_zoom = (root.chart_zoom + 0.25).min(2.5)
                                            },
                                        )),
                                )
                            },
                        ),
                )
            })
    }

    fn render_scene(&self, scene: &chart::ChartScene, side: f32, cx: &App) -> AnyElement {
        if scene.title == "Circular chart" {
            let s = self.session.read(cx);
            let profile = &s.natal;
            let date = if self.show_birth {
                format!(
                    "{:04}-{:02}-{:02}",
                    profile.year, profile.month, profile.day
                )
            } else {
                "Hidden".into()
            };
            let time = if profile.unknown_birth_time {
                "Unknown time".into()
            } else if self.show_birth {
                format!(
                    "{:02}:{:02}:{:02}",
                    profile.hour, profile.minute, profile.second
                )
            } else {
                "Hidden".into()
            };
            let zodiac = if s.settings.tropical {
                "Tropical"
            } else {
                "Sidereal"
            };
            scene.render_mode9(
                side,
                &[
                    date,
                    time,
                    profile.birthplace.clone(),
                    format!("{zodiac} · {}", AYANAMSHAS[s.settings.ayanamsha]),
                    HOUSES[s.settings.houses].into(),
                ],
                None,
            )
        } else if scene.title == "Rasi chart" {
            let s = self.session.read(cx);
            let eph = s
                .calculate(s.planet_transit && s.mode != "Natal + transit")
                .ok()
                .and_then(|r| r.reference_ephemeris);
            let rates = eph
                .as_ref()
                .and_then(|eph| {
                    eph.longitude_rates(s.settings.tropical, s.settings.ayanamsha)
                        .ok()
                })
                .unwrap_or_default();
            let flags: Vec<bool> = scene
                .labels
                .iter()
                .map(|(code, _)| {
                    chart::TRADITIONAL_PLANETS
                        .iter()
                        .find(|(_, c, _)| *c == code.as_str())
                        .is_some_and(|(name, _, _)| {
                            is_retrograde(&rates, name, s.settings.mean_node)
                        })
                })
                .collect();
            scene.render_with_retrograde(side, &flags)
        } else {
            scene.render(side)
        }
    }

    fn dasha_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use motaa_engine::dasha::{calculate, day_number};
        let session = self.session.read(cx);
        let current = day_number(session.time);
        let timeline = session.calculate(false).and_then(|result| {
            let moon = result
                .reference_ephemeris
                .and_then(|ephemeris| ephemeris.longitude("Moon", session.settings.tropical))
                .ok_or("Dasha needs a supported natal date and a known birth time.")?;
            calculate(&session.natal, moon)
        });
        let timeline = match timeline {
            Ok(timeline) => timeline,
            Err(error) => return panel().child(error).into_any_element(),
        };
        let current_major = timeline
            .major
            .iter()
            .position(|period| period.contains(current));
        let major_index = self
            .dasha_major
            .filter(|&index| index < timeline.major.len())
            .or(current_major);
        let sub = major_index.map(|index| timeline.major[index].children());
        let sub_index = sub.as_ref().and_then(|rows| {
            self.dasha_sub
                .filter(|&index| index < rows.len())
                .or_else(|| rows.iter().position(|period| period.contains(current)))
        });
        let third = sub
            .zip(sub_index)
            .map(|(rows, index)| rows[index].children());
        let active_third = third
            .as_ref()
            .and_then(|rows| rows.iter().position(|period| period.contains(current)));
        let major_code = major_index
            .map(|index| planet_code(timeline.major[index].name()))
            .unwrap_or_default();
        let sub_code = sub
            .as_ref()
            .zip(sub_index)
            .map(|(rows, index)| planet_code(rows[index].name()))
            .unwrap_or_default();
        let major_end = major_index.map(|index| timeline.major[index].end);
        let sub_end = sub
            .as_ref()
            .zip(sub_index)
            .map(|(rows, index)| rows[index].end);
        column()
            .child(
                row()
                    .justify_between()
                    .child(caption("Choose a period to explore its subperiods."))
                    .when(
                        self.dasha_major.is_some() || self.dasha_sub.is_some(),
                        |row| {
                            row.child(button("Follow reference date", false, cx, |view, _| {
                                view.dasha_major = None;
                                view.dasha_sub = None;
                            }))
                        },
                    ),
            )
            .when(current_major.is_none(), |view| {
                view.child(caption(
                    "The reference date is outside the calculated Dasha periods.",
                ))
            })
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_4()
                    .child(Self::period_table(
                        "Major period",
                        "Mahadasha",
                        &timeline.major,
                        major_index,
                        0,
                        None,
                        None,
                        cx,
                    ))
                    .child(Self::period_table(
                        "Subperiod",
                        "Antardasha",
                        sub.as_ref().map_or(&[], |rows| rows.as_slice()),
                        sub_index,
                        1,
                        (!major_code.is_empty()).then(|| major_code.to_owned()),
                        major_end,
                        cx,
                    ))
                    .child(Self::period_table(
                        "Third level",
                        "Pratyantardasha",
                        third.as_ref().map_or(&[], |rows| rows.as_slice()),
                        active_third,
                        2,
                        (!major_code.is_empty() && !sub_code.is_empty())
                            .then(|| format!("{major_code}×{sub_code}")),
                        sub_end,
                        cx,
                    )),
            )
            .child(caption(format!(
                "Birth balance: {:.1} days · natal Moon position",
                timeline.balance_days
            )))
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn period_table(
        title: &'static str,
        subtitle: &'static str,
        periods: &[motaa_engine::dasha::Period],
        selected: Option<usize>,
        level: u8,
        code_prefix: Option<String>,
        footer_end: Option<f64>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        use motaa_engine::dasha::from_day_number;
        column()
            .gap_1()
            .id(title)
            .child(div().px_2().child(title))
            .child(div().px_2().child(caption(subtitle)))
            .when(periods.is_empty(), |col| {
                col.child(
                    div()
                        .p_2()
                        .child(caption("Choose a period in the previous column.")),
                )
            })
            .children(periods.iter().copied().enumerate().map(|(index, period)| {
                let start = from_day_number(period.start);
                let end = from_day_number(period.end);
                let label = start.map_or_else(
                    || "Unavailable".into(),
                    |value| value.strftime("%d %b %Y").to_string(),
                );
                let code = planet_code(period.name());
                let display_code = match code_prefix.as_deref() {
                    Some(prefix) => format!("{prefix}×{code}"),
                    None => code.to_owned(),
                };
                row()
                    .id(index)
                    .min_w(px(0.))
                    .border_b_1()
                    .border_color(rgb(0xeae5dc))
                    .child(
                        action_control_in("period", cx, move |view, window, cx| match level {
                            0 => {
                                view.dasha_major = Some(index);
                                view.dasha_sub = None;
                            }
                            1 => view.dasha_sub = Some(index),
                            _ => {
                                if let Some(start) = start {
                                    view.set_time(start, true, window, cx);
                                }
                            }
                        })
                        .flex_1()
                        .min_w(px(0.))
                        .justify_between()
                        .gap_1()
                        .px_2()
                        .h(px(36.))
                        .rounded(px(4.))
                        .text_size(px(12.))
                        .when(selected == Some(index), |it| it.bg(rgb(SELECTED)))
                        .hover(|it| it.bg(rgb(SELECTED)))
                        .child(label)
                        .child(div().text_size(px(15.)).child(display_code))
                        .child(caption(if level < 2 {
                            "→"
                        } else {
                            ""
                        })),
                    )
                    .when(selected == Some(index), |row| {
                        row.when_some(start.zip(end), |row, (start, end)| {
                            row.child(
                                action_control_in("period-start", cx, move |view, window, cx| {
                                    view.set_time(start, true, window, cx)
                                })
                                .text_size(px(11.))
                                .px_1()
                                .h(px(32.))
                                .child("Start"),
                            )
                            .child(
                                action_control_in("period-end", cx, move |view, window, cx| {
                                    view.set_time(end, true, window, cx)
                                })
                                .text_size(px(11.))
                                .px_1()
                                .h(px(32.))
                                .child("End"),
                            )
                        })
                    })
            }))
            .when_some(footer_end.and_then(from_day_number), |col, end_date| {
                col.child(
                    div()
                        .px_2()
                        .pt_1()
                        .child(caption(format!("Ends {}", end_date.strftime("%d %b %Y")))),
                )
            })
    }

    fn myanmar_birth_strip(&self, moon_longitude: Option<f64>, cx: &App) -> Option<AnyElement> {
        let s = self.session.read(cx);
        let jdn = s
            .settings
            .calendar
            .numbered_day(s.natal.year, s.natal.month, s.natal.day);
        let date = motaa_engine::myanmar_calendar::from_julian_day_number(jdn)?;
        let weekday =
            jiff::civil::Weekday::from_monday_zero_offset(jdn.rem_euclid(7) as i8).ok()?;
        let burmese = date.format_burmese_birth_line(weekday, moon_longitude);
        let mut english = format!(
            "ME {} {} {} {} · {} · Dragon {}",
            date.year,
            date.month_english,
            date.phase,
            date.fortnight_day,
            date.birth_lord(weekday),
            date.dragon_head(),
        );
        if let Some(lon) = moon_longitude {
            let idx = ((lon.rem_euclid(360.) / (360. / 27.)) as usize).min(26);
            english.push_str(&format!(
                " · {} ({})",
                motaa_engine::EN[idx],
                motaa_engine::MY[idx]
            ));
        }
        Some(
            panel()
                .gap_1()
                .child(div().text_size(px(13.)).child(burmese))
                .child(caption(english))
                .into_any_element(),
        )
    }

    fn chart_output(
        &self,
        mode: &str,
        result: motaa_engine::ChartResult,
        asc: Option<motaa_engine::ChartResult>,
        settings: Settings,
        side: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let ascendant = asc.and_then(|value| value.reference_ephemeris);
        if mode == "Rahu wheel" {
            let scene = self.rahu_scene(
                result.reference_ephemeris.as_ref(),
                ascendant.as_ref(),
                settings,
            );
            return self.rahu_view(&scene, side, cx);
        }
        let Some(ephemeris) = result.reference_ephemeris else {
            return panel()
                .child("Reference planetary positions are unavailable for these birth details.")
                .children(result.warnings.into_iter().map(caption))
                .into_any_element();
        };
        let geometry = ascendant.as_ref().map(|asc| {
            asc.cusps(
                motaa_engine::houses::System::OPTIONS[settings.houses],
                settings.tropical,
            )
        });
        let cusps = geometry
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .copied();
        let houses = if matches!(mode, "Circular chart" | "Nakshatra traversal") {
            cusps
                .zip(ascendant.as_ref().and_then(|ascendant| {
                    ascendant
                        .house_midpoints(
                            motaa_engine::houses::System::OPTIONS[settings.houses],
                            settings.tropical,
                        )
                        .ok()
                }))
                .map(|(cusps, midpoints)| chart::HouseGeometry { cusps, midpoints })
        } else {
            None
        };
        let palette_color = INK;
        if mode == "Western aspects" {
            return match ascendant.as_ref() {
                Some(ascendant) => super::chart::western::render(
                    &ephemeris,
                    ascendant,
                    settings,
                    palette_color,
                    cx,
                ),
                None => panel()
                    .child("W-Style requires a supported reference Ascendant.")
                    .into_any_element(),
            };
        }
        if mode == "Nakshatra traversal" {
            return match ascendant
                .as_ref()
                .and_then(|asc| recovered::positions(&ephemeris, asc, settings))
            {
                Some(positions) => {
                    let rates = ephemeris
                        .longitude_rates(settings.tropical, settings.ayanamsha)
                        .unwrap_or_default();
                    let retrograde = recovered::BODIES
                        .map(|body| is_retrograde(&rates, body, settings.mean_node));
                    super::chart::traversal::render(
                        positions,
                        retrograde,
                        houses,
                        palette_color,
                        side / 2.,
                    )
                }
                None => panel()
                    .child("Nakshatra traversal requires a supported reference Ascendant.")
                    .into_any_element(),
            };
        }
        if mode == "LatLon" {
            let session = self.session.read(cx);
            let longitude = if session.ascendant_transit {
                session.reference_profile().longitude
            } else {
                session.natal.longitude
            };
            return super::chart::latlon::render(
                ephemeris,
                ascendant.as_ref(),
                cusps,
                longitude,
                settings,
                self.show_munp,
            );
        }
        if matches!(
            mode,
            "Traditional aspects" | "Rasi strengths" | "Rasi aspects"
        ) {
            return self.relationships(mode, &ephemeris, ascendant.as_ref(), settings, cx);
        }
        if mode == "Tricakka" {
            let (scenes, retrograde) =
                match self.tricakka_scenes(&ephemeris, ascendant.as_ref(), settings) {
                    Ok(result) => result,
                    Err(error) => return panel().child(error).into_any_element(),
                };
            let chart_side = (side * 0.56).max(180.);
            return div()
                .grid()
                .grid_cols(3)
                .gap_2()
                .children(scenes.into_iter().enumerate().map(|(index, scene)| {
                    let rendered = if index == 0 {
                        scene.render_with_retrograde(chart_side, &retrograde)
                    } else {
                        scene.render(chart_side)
                    };
                    action_control(("tricakka-subchart", index), cx, move |root, cx| {
                        root.open_chart_lightbox(LightboxTarget::TricakkaSubchart(index), cx);
                    })
                    .w(px(chart_side))
                    .child(rendered)
                }))
                .into_any_element();
        }
        if !matches!(mode, "Rasi" | "Circular chart") {
            return panel()
                .child(mode.to_owned())
                .child(caption(
                    "A verified calculation for this mode is unavailable in this build.",
                ))
                .into_any_element();
        }
        let [rahu, ketu] = ephemeris.lunar_nodes(settings.mean_node, settings.tropical);
        let mut positions: Vec<_> = chart::TRADITIONAL_PLANETS
            .into_iter()
            .filter_map(|(name, code, color)| {
                let angle = match name {
                    "Rahu" => Some(rahu),
                    "Ketu" => Some(ketu),
                    _ => ephemeris.longitude(name, settings.tropical),
                }?;
                Some((name, code, color, angle))
            })
            .collect();
        if mode == "Rasi" && self.show_munp {
            for (name, code, angle) in ascendant
                .as_ref()
                .map_or_else(Vec::new, |asc| munp_entries(&ephemeris, asc, settings))
            {
                positions.push((name, code, INK, angle));
            }
        }
        let mut labels: Vec<_> = positions
            .iter()
            .map(|(_, code, _, angle)| ((*code).into(), *angle))
            .collect();
        if let Some(asc) = ascendant.as_ref() {
            insert_ascendant_label(&mut labels, asc, settings.tropical);
        }
        if matches!(mode, "Rasi" | "Circular chart")
            && let Some((_, angle)) = labels.first().filter(|(label, _)| label == "Asc")
        {
            positions.insert(0, ("Ascendant", "Asc", INK, *angle));
        }
        let scene = chart::ChartScene {
            labels,
            circular: mode == "Circular chart",
            rahu: None,
            houses,
            title: mode_title(mode),
            color: if mode == "Rasi" { INK } else { GOLD },
        };
        if mode == "Circular chart" && self.circular_detail {
            let moon = ephemeris
                .longitude("Moon", settings.tropical)
                .map(|longitude| {
                    motaa_engine::original_positions::original_x0(longitude)[0] as usize
                });
            let legend = column()
                .gap_1()
                .flex_1()
                .min_w(px(190.))
                .child("Nakshatras")
                .child(caption("English names"))
                .children((0..27).filter_map(|index| {
                    name_wheel::mansion_name(index).map(|name| {
                        div()
                            .px_2()
                            .rounded(px(3.))
                            .when(moon == Some(index), |row| row.bg(rgb(SELECTED)))
                            .child(caption(format!("{} · {name}", index + 1)))
                    })
                }))
                .child(caption(
                    "Black dots mark house cusps. Colored dots mark house midpoints.",
                ));
            return column()
                .child(caption("108 divisions · 27 nakshatras"))
                .child(
                    row()
                        .items_start()
                        .gap_6()
                        .child(self.chart_surface(&scene, side, cx))
                        .child(legend),
                )
                .into_any_element();
        }
        let rates = ephemeris
            .longitude_rates(settings.tropical, settings.ayanamsha)
            .unwrap_or_default();
        let show_kp = mode == "Rasi";
        let top_row = row()
            .items_start()
            .gap_8()
            .child(self.chart_surface(&scene, side, cx))
            .child(
                column()
                    .gap_0()
                    .w(px(if show_kp { 420. } else { 382. }))
                    .max_w_full()
                    .flex_shrink_1()
                    .child(div().text_size(px(16.)).child("Planet positions"))
                    .child(caption(if self.session.read(cx).planet_transit {
                        "Transit positions"
                    } else {
                        "Natal positions"
                    }))
                    .when_some(
                        geometry.as_ref().and_then(|result| result.as_ref().err()),
                        |view, error| view.child(caption(error.clone())),
                    )
                    .child(
                        row()
                            .h(px(30.))
                            .px_2()
                            .border_t_1()
                            .border_color(rgb(LINE))
                            .child(div().flex_1().child(caption("Planet")))
                            .child(div().flex_1().child(caption("Longitude")))
                            .when(show_kp, |row| {
                                row.child(div().w(px(34.)).text_center().child(caption("NL")))
                                    .child(div().w(px(34.)).text_center().child(caption("SL")))
                            }),
                    )
                    .children(positions.into_iter().map(|(name, code, color, angle)| {
                        let longitude = chart_longitude(angle);
                        let retrograde = is_retrograde(&rates, name, settings.mean_node);
                        let (nl, sl) = motaa_engine::dasha::kp_lords(angle).map_or(
                            ("—".to_string(), "—".to_string()),
                            |[star, sub, _]| {
                                (
                                    format!("({})", planet_code(star)),
                                    format!("({})", planet_code(sub)),
                                )
                            },
                        );
                        row()
                            .h(px(33.))
                            .px_2()
                            .border_b_1()
                            .border_color(rgb(0xeae5dc))
                            .text_size(px(12.))
                            .child(
                                row()
                                    .flex_1()
                                    .child(
                                        div()
                                            .w(px(38.))
                                            .flex_shrink_0()
                                            .whitespace_nowrap()
                                            .text_size(px(if code.len() > 1 { 13. } else { 18. }))
                                            .text_color(rgb(color))
                                            .child(code),
                                    )
                                    .child(if retrograde {
                                        format!("{name} · R")
                                    } else {
                                        name.to_owned()
                                    }),
                            )
                            .child(div().flex_1().child(longitude))
                            .when(show_kp, |row| {
                                row.child(div().w(px(34.)).text_center().child(nl))
                                    .child(div().w(px(34.)).text_center().child(sl))
                            })
                    }))
                    .when_some(houses, |view, houses| {
                        view.child(
                            row().gap_4().px_2().pt_2().children(
                                [(10, "10th house center"), (4, "4th house center")]
                                    .into_iter()
                                    .map(|(house, label)| {
                                        let angle = houses.midpoints[house - 1];
                                        caption(format!("{label} · {}", chart_longitude(angle)))
                                    }),
                            ),
                        )
                    }),
            );
        if mode == "Rasi" {
            let natal_moon = self
                .session
                .read(cx)
                .calculate(false)
                .ok()
                .and_then(|r| r.reference_ephemeris)
                .and_then(|eph| eph.longitude("Moon", settings.tropical));
            column()
                .gap_4()
                .child(top_row)
                .when_some(self.myanmar_birth_strip(natal_moon, cx), |col, strip| {
                    col.child(strip)
                })
                .child(self.dasha_view(cx))
                .into_any_element()
        } else {
            top_row.into_any_element()
        }
    }

    fn tricakka_scenes(
        &self,
        planets: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
        ascendant: Option<&motaa_engine::reference_ephemeris::ReferenceEphemeris>,
        settings: Settings,
    ) -> Result<([chart::ChartScene; 3], Vec<bool>), String> {
        let ascendant = ascendant.ok_or("Tricakka requires a supported reference Ascendant.")?;
        let positions = recovered::positions(planets, ascendant, settings)
            .ok_or("Tricakka requires supported planetary positions.")?;
        let cusps = ascendant.cusps(
            motaa_engine::houses::System::OPTIONS[settings.houses],
            settings.tropical,
        )?;
        let rates = planets
            .longitude_rates(settings.tropical, settings.ayanamsha)
            .unwrap_or_default();
        let mut rasi = Vec::new();
        let mut bhava = Vec::new();
        let mut navamsa = Vec::new();
        let mut retrograde = Vec::new();
        for ((name, code), longitude) in recovered::BODIES
            .into_iter()
            .zip(chart::BODY_CODES)
            .zip(positions)
        {
            rasi.push((code.into(), longitude));
            navamsa.push((code.into(), recovered::navamsa(longitude)));
            retrograde.push(is_retrograde(&rates, name, settings.mean_node));
            if let Some(house) = motaa_engine::houses::house_at(cusps, longitude) {
                let sign = (recovered::sign(positions[0]) + usize::from(house - 1)) % 12;
                bhava.push((code.into(), sign as f64 * 30.));
            }
        }
        if self.show_munp {
            rasi.extend(munp_labels(planets, ascendant, settings));
        }
        let color = INK;
        Ok((
            [("Rasi", rasi), ("Bhava", bhava), ("Navamsa", navamsa)].map(|(title, labels)| {
                chart::ChartScene {
                    labels,
                    title,
                    circular: false,
                    rahu: None,
                    houses: None,
                    color,
                }
            }),
            retrograde,
        ))
    }

    pub(super) fn step_rahu_mansion(
        &mut self,
        step: i32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mansion = self.rahu.manual_angle.map_or_else(
            || if step < 0 { 26 } else { 0 },
            |angle| (crate::tara::mansion(angle) as i32 + step).rem_euclid(27),
        );
        let angle = f64::from(mansion) * (360. / 27.) + 1.;
        self.rahu.manual_angle = Some(angle);
        self.rahu.manual_input.update(cx, |input, cx| {
            input.set_value(format!("{angle:.9}"), window, cx);
        });
        self.set_status("", cx);
    }

    fn rahu_scene(
        &self,
        ephemeris: Option<&motaa_engine::reference_ephemeris::ReferenceEphemeris>,
        ascendant: Option<&motaa_engine::reference_ephemeris::ReferenceEphemeris>,
        settings: Settings,
    ) -> chart::ChartScene {
        let node =
            ephemeris.map(|value| value.lunar_nodes(settings.mean_node, settings.tropical)[0]);
        let (reference_name, reference) = match self.rahu.reference {
            RahuReference::Node => ("Rahu", node),
            RahuReference::Moon => (
                "Moon",
                ephemeris.and_then(|value| value.longitude("Moon", settings.tropical)),
            ),
            RahuReference::Manual => ("Manual", self.rahu.manual_angle),
        };
        let labels = ephemeris
            .and_then(|planets| {
                recovered::positions(planets, ascendant.unwrap_or(planets), settings)
            })
            .map(|positions| {
                ["A", "1", "2", "3", "4", "5", "6", "0", "8", "9"]
                    .into_iter()
                    .zip(positions)
                    .skip(usize::from(ascendant.is_none()))
                    .map(|(code, angle)| (code.into(), angle))
                    .collect()
            })
            .unwrap_or_default();
        chart::ChartScene {
            labels,
            circular: true,
            color: GOLD,
            rahu: Some(chart::RahuWheel {
                use_28: self.rahu.divisions_28,
                node,
                sun: ephemeris.and_then(|value| value.longitude("Sun", settings.tropical)),
                reference,
                reference_name,
            }),
            houses: None,
            title: "Rahu wheel",
        }
    }

    fn rahu_view(
        &self,
        scene: &chart::ChartScene,
        side: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let wheel = scene.rahu.expect("Rahu scene includes wheel data");
        let mut inspector = panel().gap_2().flex_1().min_w(px(0.));
        inspector =
            if let Some(angle) = wheel.reference {
                inspector
                    .child("Jamma grouping")
                    .child(caption(format!(
                        "{} reference · {:.3}° longitude",
                        wheel.reference_name, angle
                    )))
                    .child(column().gap_0().children(
                        chart::GROUP_NAMES.into_iter().zip(wheel.groups()).map(
                            |(group, member)| {
                                data_row([
                                    group.into(),
                                    member
                                        .and_then(crate::name_wheel::mansion_name)
                                        .unwrap_or("—")
                                        .into(),
                                ])
                                .py(px(2.))
                            },
                        ),
                    ))
            } else {
                inspector.child(caption(match self.rahu.reference {
                RahuReference::Manual => "Choose a mansion or apply a manual longitude.",
                _ => "This chart has no reference position. Choose Manual to explore Jamma groups.",
            }))
            };
        inspector = inspector.when_some(wheel.zones(), |view, bounds| {
            view.child(caption("Rahu zones · boundary longitudes"))
                .child(
                    div().grid().grid_cols(2).gap_1().children(
                        chart::ZONE_NAMES
                            .into_iter()
                            .zip(chart::ZONE_COLORS)
                            .zip(bounds)
                            .map(|((zone, color), angle)| {
                                row()
                                    .child(
                                        div()
                                            .size(px(9.))
                                            .flex_shrink_0()
                                            .rounded_sm()
                                            .bg(rgb(color))
                                            .border_1()
                                            .border_color(rgb(LINE)),
                                    )
                                    .child(caption(format!(
                                        "{zone} · {:.2}°",
                                        angle.rem_euclid(360.)
                                    )))
                            }),
                    ),
                )
        });
        column()
            .child(
                row()
                    .child(button(
                        "27 divisions",
                        !self.rahu.divisions_28,
                        cx,
                        |view, _| view.rahu.divisions_28 = false,
                    ))
                    .child(button(
                        "28 divisions",
                        self.rahu.divisions_28,
                        cx,
                        |view, _| view.rahu.divisions_28 = true,
                    ))
                    .children(
                        [
                            ("Rahu reference", RahuReference::Node),
                            ("Moon reference", RahuReference::Moon),
                            ("Manual reference", RahuReference::Manual),
                        ]
                        .map(|(name, reference)| {
                            button(
                                name,
                                self.rahu.reference == reference,
                                cx,
                                move |view, _| view.rahu.reference = reference,
                            )
                        }),
                    )
                    .child(div().flex_1())
                    .child(button("Full view", false, cx, |view, cx| {
                        view.open_chart_lightbox(LightboxTarget::ActiveChart, cx)
                    })),
            )
            .when(self.rahu.reference == RahuReference::Manual, |view| {
                let mansion = self
                    .rahu
                    .manual_angle
                    .and_then(|angle| crate::name_wheel::mansion_name(crate::tara::mansion(angle)))
                    .unwrap_or("Choose a mansion");
                view.child(
                    row()
                        .child(button_in(
                            "Previous mansion",
                            false,
                            cx,
                            |view, window, cx| view.step_rahu_mansion(-1, window, cx),
                        ))
                        .child(div().min_w(px(120.)).child(mansion))
                        .child(button_in("Next mansion", false, cx, |view, window, cx| {
                            view.step_rahu_mansion(1, window, cx)
                        }))
                        .child(div().flex_1())
                        .child(
                            div()
                                .w(px(130.))
                                .child(render_input(&self.rahu.manual_input)),
                        )
                        .child(button("Apply longitude", true, cx, |view, cx| {
                            match view
                                .rahu
                                .manual_input
                                .read(cx)
                                .value()
                                .trim()
                                .parse::<f64>()
                            {
                                Ok(angle) if (0. ..360.).contains(&angle) => {
                                    view.rahu.manual_angle = Some(angle);
                                    view.set_status("", cx);
                                }
                                _ => view.set_status(
                                    "Enter a longitude from 0° up to 360°, exclusive.",
                                    cx,
                                ),
                            }
                        })),
                )
            })
            .child(
                row()
                    .items_start()
                    .gap_6()
                    .child(self.chart_surface(scene, side, cx))
                    .child(inspector),
            )
            .into_any_element()
    }

    fn relationships(
        &self,
        mode: &str,
        ephemeris: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
        ascendant: Option<&motaa_engine::reference_ephemeris::ReferenceEphemeris>,
        settings: Settings,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use motaa_engine::{
            original_colors::original_c0,
            original_strengths::{original_d0, original_u0},
        };
        use recovered::{
            BODIES, house_aspect, positions, rasi_aspect, rasi_house_aspect, whole_sign_aspect,
        };
        let Some(positions) = ascendant.and_then(|asc| positions(ephemeris, asc, settings)) else {
            return panel()
                .child("An Ascendant is required for this view.")
                .into_any_element();
        };
        if mode == "Rasi strengths" {
            return panel()
                .child(data_row([
                    "Body".into(),
                    "Longitude".into(),
                    "Sign class".into(),
                    "Mansion class".into(),
                ]))
                .children((1..10).map(|index| {
                    data_row([
                        BODIES[index].into(),
                        format!("{:.3}°", positions[index]),
                        original_c0(index as u8, positions[index]).to_string(),
                        original_d0(index as u8, positions[index])
                            .expect("validated longitude")
                            .to_string(),
                    ])
                }))
                .into_any_element();
        }
        let show_scores = mode == "Rasi aspects";
        let all = self.all_strengths;
        let mut view = column();
        if matches!(mode, "Traditional aspects" | "Rasi aspects") {
            let mut labels: Vec<_> = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"]
                .into_iter()
                .zip(positions)
                .map(|(code, angle)| (code.into(), angle))
                .collect();
            if self.show_munp
                && let Some(asc) = ascendant
            {
                labels.extend(munp_labels(ephemeris, asc, settings));
            }
            let inset = action_control("rasi-aspect-inset", cx, |root, cx| {
                root.open_chart_lightbox(LightboxTarget::RasiInset, cx);
            })
            .w(px(218.))
            .child(
                chart::ChartScene {
                    labels,
                    title: "Rasi chart",
                    circular: false,
                    rahu: None,
                    houses: None,
                    color: INK,
                }
                .render(218.),
            );
            if show_scores {
                view = view.child(
                    row().items_center().gap_4().child(inset).child(
                        column()
                            .gap_2()
                            .child(
                                row()
                                    .child(button("Aspected only", !all, cx, |view, _| {
                                        view.all_strengths = false
                                    }))
                                    .child(button("All strengths", all, cx, |view, _| {
                                        view.all_strengths = true
                                    })),
                            )
                            .child(
                                row()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(3.))
                                            .bg(rgb(0xfdf3a7))
                                            .text_size(px(11.))
                                            .child("Rasi aspect · ရာသီအမြင်"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded(px(3.))
                                            .border_1()
                                            .border_color(rgb(INK))
                                            .text_size(px(11.))
                                            .child("Planetary aspect · ဂြိုဟ်အမြင်"),
                                    ),
                            ),
                    ),
                );
            } else {
                view = view.child(inset);
            }
        }
        let header = [
            "From", "Asc", "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu",
            "Ketu",
        ];
        let styled_aspect_row = |lead: String, cells: Vec<(String, bool, bool)>| {
            row()
                .gap_1()
                .py_1()
                .border_b_1()
                .border_color(rgb(LINE))
                .child(
                    div()
                        .w(px(64.))
                        .flex_shrink_0()
                        .text_size(px(12.))
                        .child(lead),
                )
                .children(cells.into_iter().map(|(text, is_rasi, is_graha)| {
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(3.))
                        .text_size(px(12.))
                        .when(is_rasi, |cell| cell.bg(rgb(0xfdf3a7)))
                        .when(is_graha, |cell| cell.border_1().border_color(rgb(INK)))
                        .child(text)
                }))
        };
        let table = if show_scores {
            panel()
                .child("Planet-to-planet and Ascendant")
                .child(styled_aspect_row(
                    header[0].into(),
                    header[1..]
                        .iter()
                        .map(|label| ((*label).into(), false, false))
                        .collect(),
                ))
                .children((1..10).map(|source| {
                    let cells = (0..10)
                        .map(|target| {
                            if source == target {
                                return ("—".into(), false, false);
                            }
                            let is_rasi = rasi_aspect(positions[source], positions[target]);
                            let is_graha =
                                whole_sign_aspect(source, positions[source], positions[target]);
                            let text = if all || is_graha {
                                let score = original_u0(&positions, source, target)
                                    .expect("validated positions");
                                if score != 0 {
                                    score.to_string()
                                } else if is_graha {
                                    "●".into()
                                } else {
                                    "—".into()
                                }
                            } else {
                                "—".into()
                            };
                            (text, is_rasi, is_graha)
                        })
                        .collect();
                    styled_aspect_row(BODIES[source].into(), cells)
                }))
        } else {
            panel()
                .child("Planet-to-planet and Ascendant")
                .child(data_row(header.map(str::to_owned)))
                .children((1..10).map(|source| {
                    let cells = std::array::from_fn::<_, 11, _>(|column| {
                        if column == 0 {
                            return BODIES[source].into();
                        }
                        let target = column - 1;
                        if source == target {
                            return "—".into();
                        }
                        if whole_sign_aspect(source, positions[source], positions[target]) {
                            "●".into()
                        } else {
                            "—".into()
                        }
                    });
                    data_row(cells)
                }))
        };
        let asc_ref = ascendant.expect("available from recovered positions");
        let system = motaa_engine::houses::System::OPTIONS[settings.houses];
        let cusps = match asc_ref.cusps(system, settings.tropical) {
            Ok(cusps) => cusps,
            Err(error) => {
                return view
                    .child(table)
                    .child(panel().child(error))
                    .into_any_element();
            }
        };
        let midpoints = asc_ref.house_midpoints(system, settings.tropical).ok();
        let house_table = if show_scores {
            panel()
                .child("Planet-to-house · selected cusps")
                .when_some(midpoints, |p, midpoints| {
                    p.child(styled_aspect_row(
                        "Center".into(),
                        midpoints
                            .into_iter()
                            .map(|angle| (format!("{}°", angle as u32), false, false))
                            .collect(),
                    ))
                })
                .child(styled_aspect_row(
                    "Body".into(),
                    (1..=12)
                        .map(|house| (format!("H{house}"), false, false))
                        .collect(),
                ))
                .children((1..10).map(|source| {
                    let planet_house = motaa_engine::houses::house_at(cusps, positions[source])
                        .map(|h| usize::from(h - 1));
                    let cells = (0..12)
                        .map(|house_index| {
                            let cusp = cusps[house_index];
                            let is_rasi =
                                rasi_house_aspect(positions[source], positions[0], house_index);
                            let is_graha =
                                planet_house.is_some_and(|h| house_aspect(source, h, house_index));
                            let text = if all || is_graha {
                                let mut inputs = positions;
                                inputs[0] = midpoints.map_or(cusp, |m| m[house_index]);
                                let score =
                                    original_u0(&inputs, source, 0).expect("validated positions");
                                if score != 0 {
                                    score.to_string()
                                } else if is_graha {
                                    "●".into()
                                } else {
                                    "—".into()
                                }
                            } else {
                                "—".into()
                            };
                            (text, is_rasi, is_graha)
                        })
                        .collect();
                    styled_aspect_row(BODIES[source].into(), cells)
                }))
                .child(styled_aspect_row(
                    "Cusp".into(),
                    cusps
                        .into_iter()
                        .map(|angle| (format!("{}°", angle as u32), false, false))
                        .collect(),
                ))
        } else {
            panel()
                .child("Planet-to-house · selected cusps")
                .child(data_row(std::array::from_fn::<_, 13, _>(|index| {
                    if index == 0 {
                        "Body".into()
                    } else {
                        format!("H{index}")
                    }
                })))
                .children((1..10).map(|source| {
                    let planet_house = motaa_engine::houses::house_at(cusps, positions[source])
                        .map(|h| usize::from(h - 1));
                    data_row(std::array::from_fn::<_, 13, _>(|column| {
                        if column == 0 {
                            return BODIES[source].into();
                        }
                        if planet_house.is_some_and(|h| house_aspect(source, h, column - 1)) {
                            "●".into()
                        } else {
                            "—".into()
                        }
                    }))
                }))
        };
        view.child(table).child(house_table).into_any_element()
    }
}

pub(crate) fn munp_entries(
    planets: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
    ascendant: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
    settings: Settings,
) -> Vec<(&'static str, &'static str, f64)> {
    let mut entries: Vec<(&'static str, &'static str, f64)> =
        [("Uranus", "U"), ("Neptune", "N"), ("Pluto", "P")]
            .into_iter()
            .filter_map(|(name, code)| {
                planets
                    .longitude(name, settings.tropical)
                    .map(|angle| (name, code, angle))
            })
            .collect();
    if let Some(angle) = ascendant.midheaven(settings.tropical) {
        entries.push(("Midheaven", "M", angle));
    }
    entries
}

pub(crate) fn munp_labels(
    planets: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
    ascendant: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
    settings: Settings,
) -> Vec<(String, f64)> {
    munp_entries(planets, ascendant, settings)
        .into_iter()
        .map(|(_, code, angle)| (code.into(), angle))
        .collect()
}

pub(crate) fn planet_code(name: &str) -> &'static str {
    match name {
        "Sun" => "1",
        "Moon" => "2",
        "Mars" => "3",
        "Mercury" => "4",
        "Jupiter" => "5",
        "Venus" => "6",
        "Saturn" => "0",
        "Rahu" => "8",
        "Ketu" => "9",
        _ => "—",
    }
}

pub(crate) fn is_retrograde(rates: &[(&str, f64)], name: &str, mean_node: bool) -> bool {
    let lookup = match name {
        "Ascendant" | "Midheaven" => return false,
        "Rahu" | "Ketu" => {
            if mean_node {
                "Mean node"
            } else {
                "True node"
            }
        }
        other => other,
    };
    rates
        .iter()
        .find(|(body, _)| *body == lookup)
        .is_some_and(|(_, speed)| *speed <= 0.)
}

pub(crate) fn chart_longitude(angle: f64) -> String {
    let angle = angle.rem_euclid(360.);
    let sign = crate::varga::SIGNS[crate::recovered::sign(angle)];
    let within_sign = angle.rem_euclid(30.);
    let degrees = within_sign as u32;
    let minutes = ((within_sign - f64::from(degrees)) * 60.) as u32;
    format!("{sign} {degrees}°{minutes:02}′")
}

fn comparison_rasi_scene(
    positions: &[(&'static str, &'static str, f64)],
    color: u32,
) -> chart::ChartScene {
    chart::ChartScene {
        labels: positions
            .iter()
            .map(|(c, _, a)| ((*c).into(), *a))
            .collect(),
        title: "Rasi chart",
        circular: false,
        rahu: None,
        houses: None,
        color,
    }
}

fn insert_ascendant_label(
    labels: &mut Vec<(String, f64)>,
    asc: &motaa_engine::reference_ephemeris::ReferenceEphemeris,
    tropical: bool,
) {
    labels.insert(
        0,
        (
            "Asc".into(),
            if tropical {
                asc.ascendant_tropical_deg
            } else {
                asc.ascendant_sidereal_deg
            },
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn munp_labels_track_independent_planet_and_ascendant_epochs() {
        let natal = ChartRequest {
            year: 1984,
            month: 1,
            day: 3,
            hour: 5,
            minute: 45,
            timezone_offset_minutes: 390,
            latitude: 21.535556,
            longitude: 95.201667,
            ..Default::default()
        };
        let transit = ChartRequest {
            year: 2024,
            month: 7,
            day: 1,
            hour: 12,
            minute: 0,
            timezone_offset_minutes: -240,
            latitude: 40.7128,
            longitude: -74.006,
            ..Default::default()
        };
        let natal_eph = motaa_engine::calculate(&natal)
            .unwrap()
            .reference_ephemeris
            .unwrap();
        let transit_eph = motaa_engine::calculate(&transit)
            .unwrap()
            .reference_ephemeris
            .unwrap();
        let settings = Settings::default();

        for (planets, ascendant) in [
            (&natal_eph, &natal_eph),
            (&natal_eph, &transit_eph),
            (&transit_eph, &natal_eph),
            (&transit_eph, &transit_eph),
        ] {
            let labels = munp_labels(planets, ascendant, settings);
            assert_eq!(labels.len(), 4);
            assert_eq!(
                labels[0],
                ("U".into(), planets.longitude("Uranus", false).unwrap())
            );
            assert_eq!(
                labels[1],
                ("N".into(), planets.longitude("Neptune", false).unwrap())
            );
            assert_eq!(
                labels[2],
                ("P".into(), planets.longitude("Pluto", false).unwrap())
            );
            assert_eq!(labels[3], ("M".into(), ascendant.midheaven(false).unwrap()));
        }
    }

    #[test]
    fn retrograde_markers_match_signed_motion_and_copy_rahu_to_ketu() {
        // 2024-04-08 18:17:18 UTC: Mercury is retrograde; Sun and Moon are direct; mean node is always retrograde.
        let request = ChartRequest {
            year: 2024,
            month: 4,
            day: 8,
            hour: 18,
            minute: 17,
            second: 18,
            timezone_offset_minutes: 0,
            latitude: 25.287,
            longitude: -104.138,
            ..Default::default()
        };
        let eph = motaa_engine::calculate(&request)
            .unwrap()
            .reference_ephemeris
            .unwrap();
        let rates = eph.longitude_rates(false, 0).unwrap();
        assert!(!is_retrograde(&rates, "Ascendant", false));
        assert!(!is_retrograde(&rates, "Midheaven", false));
        assert!(!is_retrograde(&rates, "Sun", false));
        assert!(!is_retrograde(&rates, "Moon", false));
        assert!(is_retrograde(&rates, "Mercury", false));
        assert_eq!(
            is_retrograde(&rates, "Rahu", false),
            is_retrograde(&rates, "Ketu", false)
        );
        assert!(is_retrograde(&rates, "Rahu", true));
        assert!(is_retrograde(&rates, "Ketu", true));
    }
}
