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
            .w(px(268.))
            .flex_shrink_0()
            .h_full()
            .bg(rgb(HEADER_BG))
            .border_l_1()
            .border_color(rgb(SUBTLE_LINE))
            .overflow_y_scroll()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                column()
                    .gap_2()
                    .pb_4()
                    .border_b_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        row()
                            .justify_between()
                            .child(section_label("BIRTH PROFILE"))
                            .child(ghost_button(
                                "inspector-privacy",
                                if self.show_birth { "Hide" } else { "Show" },
                                cx,
                                |view, _| view.show_birth = !view.show_birth,
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(15.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(INK))
                            .child(natal.name.clone()),
                    )
                    .child(
                        div()
                            .font_family("Menlo")
                            .text_size(px(11.5))
                            .text_color(rgb(MUTED))
                            .child(format!("{birth_label} · {timezone}")),
                    )
                    .child(caption(place_label))
                    .child(div().pt_1().child(self.profile_edit_save_buttons(cx))),
            )
            .child(
                column()
                    .gap(px(10.))
                    .pb_4()
                    .border_b_1()
                    .border_color(rgb(SUBTLE_LINE))
                    .child(
                        row()
                            .justify_between()
                            .child(section_label("CALCULATION"))
                            .child(ghost_button(
                                "inspector-setup",
                                "Setup ›",
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
                            .child(chip(ayanamsha, false))
                            .child(chip(format!("{houses} houses"), false)),
                    )
                    .child(
                        column()
                            .gap_1()
                            .child(caption("Zodiac"))
                            .child(segmented_pair(
                                "insp-zodiac",
                                ("Sidereal", "Tropical"),
                                settings.tropical,
                                cx,
                                |view, is_tropical, cx| {
                                    view.update(cx, |s| {
                                        s.settings.tropical = is_tropical;
                                        s.settings_draft.tropical = is_tropical;
                                    });
                                },
                            )),
                    )
                    .child(column().gap_1().child(caption("Lunar node (Rahu)")).child(
                        segmented_pair(
                            "insp-node",
                            ("True", "Mean"),
                            settings.mean_node,
                            cx,
                            |view, is_mean, cx| {
                                view.update(cx, |s| {
                                    s.settings.mean_node = is_mean;
                                    s.settings_draft.mean_node = is_mean;
                                });
                            },
                        ),
                    )),
            )
            .when(
                workspace == Workspace::Charts
                    && !matches!(mode, "Natal + transit" | "Dasha periods"),
                |insp| {
                    insp.child(
                        column()
                            .gap(px(10.))
                            .pb_3()
                            .child(section_label("EPOCH OVERLAYS"))
                            .child(column().gap_1().child(caption("Planets")).child(
                                segmented_pair(
                                    "insp-planets-epoch",
                                    ("Natal", "Transit"),
                                    planet_transit,
                                    cx,
                                    |view, is_transit, cx| {
                                        view.update(cx, |s| s.planet_transit = is_transit);
                                    },
                                ),
                            ))
                            .child(column().gap_1().child(caption("Ascendant")).child(
                                segmented_pair(
                                    "insp-asc-epoch",
                                    ("Natal", "Transit"),
                                    ascendant_transit,
                                    cx,
                                    |view, is_transit, cx| {
                                        view.update(cx, |s| s.ascendant_transit = is_transit);
                                    },
                                ),
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
                                    col.child(
                                        row()
                                            .pt_1()
                                            .justify_between()
                                            .child(caption("MUNP overlay"))
                                            .child(compact_button(
                                                "insp-munp-toggle",
                                                if self.show_munp { "On" } else { "Off" },
                                                self.show_munp,
                                                cx,
                                                |view, _| {
                                                    view.show_munp = !view.show_munp;
                                                },
                                            )),
                                    )
                                },
                            ),
                    )
                },
            )
            .into_any_element()
    }
}
