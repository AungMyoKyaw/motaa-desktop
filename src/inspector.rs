use super::*;

impl DesktopRoot {
    pub(super) fn right_inspector(&self, cx: &mut Context<Self>) -> AnyElement {
        let (natal, settings, mode, workspace, planet_transit, ascendant_transit) = {
            let s = self.session.read(cx);
            (
                s.natal.clone(),
                s.settings,
                s.mode,
                s.workspace,
                s.planet_transit,
                s.ascendant_transit,
            )
        };
        let timezone = match motaa_engine::effective_utc_offset(&natal) {
            Ok(offset) => {
                let sign = if offset < 0 { '-' } else { '+' };
                let minutes = offset.unsigned_abs();
                format!("GMT{sign}{:02}:{:02}", minutes / 60, minutes % 60)
            }
            Err(_) => natal
                .timezone_id
                .clone()
                .unwrap_or_else(|| "Timezone unavailable".into()),
        };
        let birth_label = if self.show_birth {
            if natal.unknown_birth_time {
                format!(
                    "{:04}-{:02}-{:02} · Unknown time",
                    natal.year, natal.month, natal.day
                )
            } else {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    natal.year, natal.month, natal.day, natal.hour, natal.minute, natal.second
                )
            }
        } else {
            "Hidden (privacy)".into()
        };
        let place_label = format!(
            "{} · {:.2}° {}, {:.2}° {}",
            natal.birthplace,
            natal.latitude.abs(),
            if natal.latitude < 0. { 'S' } else { 'N' },
            natal.longitude.abs(),
            if natal.longitude < 0. { 'W' } else { 'E' },
        );
        let ayanamsha = if settings.ayanamsha == 0 {
            "Lahiri"
        } else {
            AYANAMSHAS[settings.ayanamsha]
        };
        let houses = if settings.houses == 0 {
            "Equal"
        } else {
            HOUSES[settings.houses]
        };

        div()
            .id("right-inspector")
            .w(px(284.))
            .flex_shrink_0()
            .h_full()
            .bg(rgb(PAPER))
            .border_l_1()
            .border_color(rgb(LINE))
            .overflow_y_scroll()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                column()
                    .gap_2()
                    .pb_3()
                    .border_b_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        row()
                            .justify_between()
                            .child(section_label("BIRTH PROFILE"))
                            .child(button(
                                if self.show_birth { "Hide" } else { "Show" },
                                false,
                                cx,
                                |view, _| view.show_birth = !view.show_birth,
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(INK))
                            .child(natal.name.clone()),
                    )
                    .child(caption(format!("{birth_label} · {timezone}")))
                    .child(caption(place_label))
                    .child(div().pt_1().child(self.profile_edit_save_buttons(cx))),
            )
            .child(
                column()
                    .gap_2()
                    .pb_3()
                    .border_b_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        row()
                            .justify_between()
                            .child(section_label("CALCULATION"))
                            .child(button(
                                format!("Setup {}", shortcut_label("3")),
                                false,
                                cx,
                                |view, cx| {
                                    view.navigate(Workspace::Calculations, cx);
                                },
                            )),
                    )
                    .child(
                        row()
                            .flex_wrap()
                            .gap_1()
                            .child(chip(
                                if settings.tropical {
                                    "Tropical"
                                } else {
                                    "Sidereal"
                                },
                                !settings.tropical,
                            ))
                            .child(chip(ayanamsha, false))
                            .child(chip(houses, false)),
                    )
                    .child(
                        row()
                            .gap_2()
                            .child(button(
                                if settings.tropical {
                                    "Tropical"
                                } else {
                                    "Sidereal"
                                },
                                !settings.tropical,
                                cx,
                                |view, cx| {
                                    view.update(cx, |s| {
                                        s.settings.tropical = !s.settings.tropical;
                                        s.settings_draft.tropical = s.settings.tropical;
                                    });
                                },
                            ))
                            .child(button(
                                if settings.mean_node {
                                    "Mean Rahu"
                                } else {
                                    "True Rahu"
                                },
                                settings.mean_node,
                                cx,
                                |view, cx| {
                                    view.update(cx, |s| {
                                        s.settings.mean_node = !s.settings.mean_node;
                                        s.settings_draft.mean_node = s.settings.mean_node;
                                    });
                                },
                            )),
                    ),
            )
            .when(
                workspace == Workspace::Charts
                    && !matches!(mode, "Natal + transit" | "Dasha periods"),
                |insp| {
                    insp.child(
                        column()
                            .gap_2()
                            .pb_3()
                            .border_b_1()
                            .border_color(rgb(SUBTLE_LINE))
                            .child(section_label("EPOCH OVERLAYS"))
                            .child(button(
                                if planet_transit {
                                    "Planets: Transit"
                                } else {
                                    "Planets: Natal"
                                },
                                planet_transit,
                                cx,
                                |view, cx| {
                                    view.update(cx, |s| s.planet_transit = !s.planet_transit)
                                },
                            ))
                            .child(button(
                                if ascendant_transit {
                                    "Ascendant: Transit"
                                } else {
                                    "Ascendant: Natal"
                                },
                                ascendant_transit,
                                cx,
                                |view, cx| {
                                    view.update(cx, |s| s.ascendant_transit = !s.ascendant_transit)
                                },
                            ))
                            .when(
                                matches!(
                                    mode,
                                    "Rasi"
                                        | "Tricakka"
                                        | "Traditional aspects"
                                        | "LatLon"
                                        | "Rasi aspects"
                                ),
                                |col| {
                                    col.child(button(
                                        "MUNP overlay",
                                        self.show_munp,
                                        cx,
                                        |view, _| {
                                            view.show_munp = !view.show_munp;
                                        },
                                    ))
                                },
                            ),
                    )
                },
            )
            .into_any_element()
    }
}
