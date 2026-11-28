mod analysis;
mod timing;

use super::*;

pub(super) fn spawn_named_worker<T: Send + 'static>(
    thread_name: &str,
    label: &'static str,
    cx: &mut Context<DesktopRoot>,
    compute: impl FnOnce() -> Result<T, String> + Send + 'static,
    apply: impl FnOnce(&mut DesktopRoot, Result<T, String>, &mut Context<DesktopRoot>) + 'static,
) {
    let worker = std::thread::Builder::new()
        .name(thread_name.into())
        .stack_size(8 * 1024 * 1024)
        .spawn(compute);
    cx.spawn(async move |view, cx| {
        let result = match worker {
            Ok(worker) => {
                cx.background_executor()
                    .spawn(async move {
                        worker
                            .join()
                            .map_err(|_| format!("{label} worker failed."))?
                    })
                    .await
            }
            Err(error) => Err(format!("Could not start {label}: {error}")),
        };
        let _ = view.update(cx, |view, cx| {
            apply(view, result, cx);
            cx.notify();
        });
    })
    .detach();
}

impl DesktopRoot {
    pub(super) fn advanced_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let active_tool = self.session.read(cx).tool;
        if active_tool == Some("Interpretations") {
            return self.interpretations_view(cx);
        }
        if active_tool == Some("Ten pillars / Ayu Moon") {
            return self.ten_pillars_view(cx);
        }
        if active_tool == Some("Satapati name wheel") {
            return self.satapati_view(cx);
        }
        if active_tool == Some("Sarvatobhadra") {
            return self.sarvatobhadra_view(cx);
        }
        if active_tool == Some("Strength timeline") {
            return self.strength_view(cx);
        }
        if active_tool == Some("Yoga / candidate times") {
            return self.candidate_view(cx);
        }
        if active_tool == Some("Eclipses") {
            return self.eclipses_view(cx);
        }
        if active_tool == Some("Transit events") {
            return self.transits_view(cx);
        }
        if active_tool == Some("Nakshatra groups") {
            return self.nakshatra_groups_view(cx);
        }
        if active_tool == Some("Daily Tatkalika") {
            return self.daily_tatkalika_view(cx);
        }
        if active_tool == Some("Electional timing") {
            return self.electional_view(cx);
        }
        if let Some(tool @ ("Ashtakavarga" | "Divisional charts")) = active_tool {
            return self.traditional_charts_view(tool, cx);
        }
        if matches!(active_tool, Some("Sky map · 2D" | "Sky map · 3D")) {
            let session = self.session.read(cx);
            let tilted = active_tool == Some("Sky map · 3D");
            let projection = sky::Projection {
                azimuth: self.sky_rotation,
                tilted,
            };
            let output = match session.calculate(true) {
                Ok(result) => match result.reference_ephemeris {
                    Some(ephemeris) => row()
                        .items_start()
                        .gap_4()
                        .child(projection.render(&ephemeris.bodies))
                        .child(
                            panel()
                                .flex_1()
                                .child(data_row([
                                    "Body".into(),
                                    "Longitude".into(),
                                    "Latitude".into(),
                                ]))
                                .children(ephemeris.bodies.into_iter().map(|body| {
                                    data_row([
                                        body.name,
                                        format!("{:.2}°", body.tropical_longitude_deg),
                                        format!("{:.2}°", body.ecliptic_latitude_deg),
                                    ])
                                })),
                        ),
                    None => panel().child("Planetary positions unavailable for this date."),
                },
                Err(error) => panel().child(error),
            };
            return column()
                .child(tool_header(
                    "Sky map",
                    "Geocentric ecliptic projection · unit sphere",
                    cx,
                ))
                .child(
                    row()
                        .child(button("2D projection", !tilted, cx, |view, cx| {
                            view.update(cx, |s| s.tool = Some("Sky map · 2D"))
                        }))
                        .child(button("Tilted 3D projection", tilted, cx, |view, cx| {
                            view.update(cx, |s| s.tool = Some("Sky map · 3D"))
                        }))
                        .child(button("Rotate left", false, cx, |view, _| {
                            view.sky_rotation = (view.sky_rotation - 30.).rem_euclid(360.)
                        }))
                        .child(button("Rotate right", false, cx, |view, _| {
                            view.sky_rotation = (view.sky_rotation + 30.).rem_euclid(360.)
                        })),
                )
                .child(output)
                .child(self.time_controls(cx))
                .into_any_element();
        }
        if self.session.read(cx).tool == Some("KP subdivisions") {
            let horary = self.kp_horary;
            let shifted_asc = horary.and_then(motaa_engine::dasha::kp_249_longitude);
            let (natal_panel, transit_panel) = {
                let session = self.session.read(cx);
                let render_epoch = |transit, title| {
                    let result = match session.calculate(transit) {
                        Ok(result) => result,
                        Err(error) => return panel().child(title).child(error),
                    };
                    let mut settings = session.settings;
                    settings.tropical = false;
                    let Some(ephemeris) = result.reference_ephemeris else {
                        return panel()
                            .child(title)
                            .child("This chart needs a known time within the reference range.");
                    };
                    let Some(mut positions) =
                        recovered::positions(&ephemeris, &ephemeris, settings)
                    else {
                        return panel()
                            .child(title)
                            .child("Planetary positions are unavailable for this chart.");
                    };
                    if let Some((asc_deg, _)) = shifted_asc {
                        positions[0] = asc_deg;
                    }
                    let house_centers = if let Some((asc_deg, _)) = shifted_asc {
                        Ok(std::array::from_fn::<_, 12, _>(|i| {
                            (asc_deg + i as f64 * 30. + 15.).rem_euclid(360.)
                        }))
                    } else {
                        ephemeris.house_midpoints(
                            motaa_engine::houses::System::OPTIONS[settings.houses],
                            settings.tropical,
                        )
                    };
                    let rows = recovered::BODIES
                        .into_iter()
                        .map(str::to_owned)
                        .zip(positions)
                        .chain(
                            house_centers
                                .as_ref()
                                .ok()
                                .into_iter()
                                .flatten()
                                .enumerate()
                                .map(|(index, &longitude)| {
                                    (format!("House {} center", index + 1), longitude)
                                }),
                        );
                    panel()
                        .child(title)
                        .child(data_row(
                            ["Point", "Longitude", "Star", "Sub", "Sub-sub"].map(str::to_owned),
                        ))
                        .children(rows.filter_map(|(name, longitude)| {
                            let [star, sub, sub_sub] = motaa_engine::dasha::kp_lords(longitude)?;
                            Some(data_row([
                                name,
                                format!("{longitude:.4}°"),
                                star.into(),
                                sub.into(),
                                sub_sub.into(),
                            ]))
                        }))
                        .when(house_centers.is_err(), |view| {
                            view.child(caption(
                                "House centers are unavailable for this location and house system.",
                            ))
                        })
                };
                (render_epoch(false, "Natal"), render_epoch(true, "Transit"))
            };
            let current_slot = horary.unwrap_or(1);
            let horary_controls = panel()
                .child(match shifted_asc {
                    Some((deg, [star, sub])) => format!(
                        "KP Horary / Bhava Shift #{} · {:.4}° ({} · Star {star} / Sub {sub})",
                        current_slot,
                        deg,
                        varga::SIGNS[recovered::sign(deg)]
                    ),
                    None => "KP Bhava Shift: Birth / Transit Ascendant (unshifted)".into(),
                })
                .child(
                    row()
                        .child(button("Shift #1", horary == Some(1), cx, |view, _| {
                            view.kp_horary = Some(1)
                        }))
                        .child(button("-1", false, cx, move |view, _| {
                            view.kp_horary = Some(current_slot.saturating_sub(1).max(1))
                        }))
                        .child(button("+1", false, cx, move |view, _| {
                            view.kp_horary = Some((current_slot + 1).min(249))
                        }))
                        .child(button("+27", false, cx, move |view, _| {
                            view.kp_horary = Some((current_slot + 27).min(249))
                        }))
                        .child(button(
                            "Reset Ascendant",
                            horary.is_none(),
                            cx,
                            |view, _| view.kp_horary = None,
                        )),
                );
            return column()
                .child(heading("KP subdivisions"))
                .child(caption(
                    "Vimshottari subdivisions for sidereal positions, house centers, and 1–249 KP Horary Bhava shift",
                ))
                .child(horary_controls)
                .child(natal_panel)
                .child(transit_panel)
                .child(self.time_controls(cx))
                .child(button("All tools", false, cx, |view, cx| {
                    view.update(cx, |s| s.tool = None)
                }))
                .into_any_element();
        }
        if self.session.read(cx).tool == Some("Heliocentric model") {
            let session = self.session.read(cx);
            let output = match session.calculate(true) {
                Ok(result) => match result.reference_ephemeris {
                    Some(ephemeris) => {
                        let header = || {
                            data_row(
                                [
                                    "Body",
                                    "Longitude",
                                    "Latitude",
                                    "Distance (AU)",
                                    "Motion (°/day)",
                                ]
                                .map(str::to_owned),
                            )
                        };
                        let solar_rates = ephemeris.heliocentric_longitude_rates();
                        let earth_rates = ephemeris.longitude_rates(true, 0);
                        column()
                            .child(crate::orbit::render(&ephemeris))
                            .child(
                                panel().child("Sun-centred").child(header()).children(
                                    ephemeris
                                        .heliocentric_positions()
                                        .into_iter()
                                        .zip(solar_rates)
                                        .map(|((name, lon, lat, distance), (_, rate))| {
                                            data_row([
                                                name.into(),
                                                format!("{lon:.3}°"),
                                                format!("{lat:.3}°"),
                                                format!("{distance:.6}"),
                                                format!("{rate:+.5}"),
                                            ])
                                        }),
                                ),
                            )
                            .child(panel().child("Earth-centred").child(header()).children(
                                ephemeris.bodies.iter().map(|body| {
                                    let rate = earth_rates.as_ref().ok().and_then(|rates| {
                                        rates
                                            .iter()
                                            .find(|(name, _)| *name == body.name)
                                            .map(|(_, rate)| *rate)
                                    });
                                    data_row([
                                        body.name.clone(),
                                        format!("{:.3}°", body.tropical_longitude_deg),
                                        format!("{:.3}°", body.ecliptic_latitude_deg),
                                        body.distance_au().map_or_else(
                                            || "—".into(),
                                            |distance| format!("{distance:.6}"),
                                        ),
                                        rate.map_or_else(
                                            || "—".into(),
                                            |rate| format!("{rate:+.5}"),
                                        ),
                                    ])
                                }),
                            ))
                    }
                    None => panel().child("Planetary positions unavailable for this date."),
                },
                Err(error) => panel().child(error),
            };
            return column()
                .child(tool_header(
                    "Heliocentric model",
                    "Tropical ecliptic coordinates at the reference time",
                    cx,
                ))
                .child(output)
                .child(self.time_controls(cx))
                .into_any_element();
        }
        if self.session.read(cx).tool == Some("Ephemeris calendar") {
            let session = self.session.read(cx);
            let date = format_time(session.time);
            let positions = session.calculate(true);
            let output = match positions {
                Ok(result) => match result.reference_ephemeris {
                    Some(ephemeris) => panel()
                        .gap_0()
                        .child(section_label("TROPICAL GEOCENTRIC POSITIONS"))
                        .child(div().h(px(8.)))
                        .child(table_header_row(["Body", "Longitude", "Latitude"]))
                        .children(ephemeris.bodies.into_iter().enumerate().map(|(i, body)| {
                            tabular_row(
                                [
                                    body.name,
                                    format!("{:.3}°", body.tropical_longitude_deg),
                                    format!("{:.3}°", body.ecliptic_latitude_deg),
                                ],
                                i % 2 == 1,
                            )
                        })),
                    None => panel().child("Planetary positions unavailable for this date."),
                },
                Err(error) => error_banner(error),
            };
            return column()
                .child(tool_header("Ephemeris calendar", date, cx))
                .child(output)
                .child(self.time_controls(cx))
                .child(button(
                    "Calculate selected month",
                    false,
                    cx,
                    DesktopRoot::load_ephemeris_month,
                ))
                .when(self.ephemeris.progress, |view| {
                    view.child(caption("Calculating selected month…"))
                })
                .when_some(self.ephemeris.month.as_ref(), |view, result| match result {
                    Ok(month) => view.child(
                        panel()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(month.title.clone()),
                            )
                            .child(caption(format!(
                                "Local midnight · {} · {} · {}",
                                month.birthplace,
                                if month.settings.tropical {
                                    "Tropical"
                                } else {
                                    model::AYANAMSHAS[month.settings.ayanamsha]
                                },
                                month.settings.calendar.label(),
                            )))
                            .child(caption(format!(
                                "Tithi and Nakshatra transitions · Sidereal {}",
                                model::AYANAMSHAS[month.settings.ayanamsha]
                            )))
                            .child(table_header_row([
                                "Date",
                                "Sun",
                                "Moon",
                                "Myanmar date",
                                "Tithi transition",
                                "Nakshatra transition",
                                "Moon enters",
                            ]))
                            .children(
                                month
                                    .rows
                                    .iter()
                                    .cloned()
                                    .enumerate()
                                    .map(|(i, r)| tabular_row(r, i % 2 == 1)),
                            )
                            .child(
                                div()
                                    .pt_3()
                                    .child(section_label("ADDITIONAL MIDNIGHT POSITIONS")),
                            )
                            .child(data_row(
                                [
                                    "Date",
                                    "Mars",
                                    "Mercury",
                                    "Jupiter",
                                    "Venus",
                                    "Saturn",
                                    if month.settings.mean_node {
                                        "Mean Rahu"
                                    } else {
                                        "True Rahu"
                                    },
                                    "Ketu",
                                ]
                                .map(str::to_owned),
                            ))
                            .children(
                                month
                                    .planets
                                    .iter()
                                    .cloned()
                                    .enumerate()
                                    .map(|(i, r)| tabular_row(r, i % 2 == 1)),
                            ),
                    ),
                    Err(error) => view.child(error_banner(error.clone())),
                })
                .into_any_element();
        }
        if let Some(tool) = self.session.read(cx).tool {
            return column()
                .child(tool_header(tool, "Advanced analysis", cx))
                .child(empty_state(
                    "Calculation unavailable",
                    "A validated implementation for this tool is unavailable in this build.",
                    Some(button("Calculation options", false, cx, |view, cx| {
                        view.navigate(Workspace::Calculations, cx)
                    })),
                ))
                .into_any_element();
        }
        let query = self.tool_search.read(cx).value().trim().to_lowercase();
        let matches = |label: &str, desc: &str| {
            query.is_empty()
                || label.to_lowercase().contains(&query)
                || desc.to_lowercase().contains(&query)
        };
        let tool_groups: [(&str, &[(&str, &str)]); 4] = [
            (
                "STRENGTH & TIMING",
                &[
                    (
                        "Strength timeline",
                        "Annual planetary strength curve over age",
                    ),
                    (
                        "Ashtakavarga",
                        "Sarvashtakavarga & Bhinnashtakavarga bindus",
                    ),
                    ("Electional timing", "Panchanga & auspicious moment checks"),
                    (
                        "Yoga / candidate times",
                        "Evaluate 3 candidate epochs side-by-side",
                    ),
                ],
            ),
            (
                "TRADITIONAL & DIVISIONAL CHARTS",
                &[
                    (
                        "Divisional charts",
                        "Varga charts D2–D60 with sign placements",
                    ),
                    ("KP subdivisions", "249-sub KP star & sub-lord tables"),
                    (
                        "Satapati name wheel",
                        "Myanmar syllable & nakshatra sector wheel",
                    ),
                    ("Sarvatobhadra", "28-nakshatra vedha & ray grid"),
                ],
            ),
            (
                "SKY & ECLIPSES",
                &[
                    (
                        "Ephemeris calendar",
                        "Monthly midnight longitudes & transitions",
                    ),
                    ("Eclipses", "Solar & lunar eclipse visibility & contacts"),
                    ("Transit events", "Sign ingresses & planetary stations"),
                    (
                        "Heliocentric model",
                        "Sun-centred & Earth-centred ecliptic rates",
                    ),
                    ("Sky map · 2D", "Geocentric ecliptic unit-sphere projection"),
                    ("Sky map · 3D", "Tilted 3D ecliptic sphere projection"),
                ],
            ),
            (
                "INTERPRETATIONS & REFERENCE",
                &[
                    (
                        "Interpretations",
                        "Period, planet, house, nakshatra & yoga notes",
                    ),
                    ("Daily Tatkalika", "Temporary planetary friendship matrix"),
                    (
                        "Ten pillars / Ayu Moon",
                        "Mahabote & Panchadha Maitri tables",
                    ),
                    ("Nakshatra groups", "Tara & 27-mansion group classification"),
                ],
            ),
        ];

        let filtered_modes: Vec<_> = MODES
            .into_iter()
            .filter(|m| matches(m, "Chart mode"))
            .collect();
        let any_modes = !filtered_modes.is_empty();
        let any_tools = tool_groups
            .iter()
            .any(|(_, items)| items.iter().any(|(name, desc)| matches(name, desc)));

        column()
            .gap_4()
            .child(
                row()
                    .justify_between()
                    .gap_4()
                    .child(column().gap_0().child(heading("All tools")).child(caption(
                        "Choose a chart view or open an advanced analysis workspace.",
                    )))
                    .child(div().w(px(340.)).child(render_input(&self.tool_search))),
            )
            .when(any_modes, |col| {
                col.child(
                    column().gap_2().child(section_label("CHART MODES")).child(
                        div().grid().grid_cols(3).gap_x_4().children(
                            filtered_modes.into_iter().map(|mode| {
                                directory_link(mode, cx, move |view, cx| view.select_mode(mode, cx))
                            }),
                        ),
                    ),
                )
            })
            .children(tool_groups.into_iter().filter_map(|(group_title, items)| {
                let matching: Vec<_> = items
                    .iter()
                    .copied()
                    .filter(|(name, desc)| matches(name, desc))
                    .collect();
                if matching.is_empty() {
                    return None;
                }
                Some(
                    column().gap_2().child(section_label(group_title)).child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_3()
                            .children(matching.into_iter().map(|(tool, desc)| {
                                action_control(
                                    gpui::SharedString::from(format!("tool-card-{tool}")),
                                    cx,
                                    move |view, cx| {
                                        view.update(cx, |s| s.tool = Some(tool));
                                    },
                                )
                                .p_3()
                                .rounded(px(RADIUS_CONTROL))
                                .border_1()
                                .border_color(rgb(LINE))
                                .bg(rgb(PAPER))
                                .hover(|s| s.bg(rgb(SOFT)))
                                .justify_between()
                                .child(
                                    column()
                                        .gap_1()
                                        .child(
                                            div()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(rgb(INK))
                                                .child(tool),
                                        )
                                        .child(caption(desc)),
                                )
                                .child(caption("→"))
                            })),
                    ),
                )
            }))
            .when(!any_tools && !any_modes, |view| {
                view.child(empty_state(
                    "No matching modes or tools",
                    "Try another keyword or clear the search filter.",
                    None,
                ))
            })
            .child(
                row()
                    .pt_1()
                    .child(button("Myanmar calendar", false, cx, |view, cx| {
                        view.navigate(Workspace::Time, cx)
                    })),
            )
            .into_any_element()
    }
}

fn tool_header(
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
    cx: &mut Context<DesktopRoot>,
) -> Div {
    row()
        .justify_between()
        .pb_3()
        .border_b_1()
        .border_color(rgb(SUBTLE_LINE))
        .child(
            column()
                .gap_0()
                .child(heading(title))
                .child(caption(subtitle)),
        )
        .child(button("← All tools", false, cx, |view, cx| {
            view.update(cx, |s| s.tool = None);
        }))
}
