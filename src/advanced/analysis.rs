use super::*;

impl DesktopRoot {
    pub(super) fn traditional_charts_view(&self, tool: &str, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let settings = session.settings;
        let positions_at = |transit| {
            session.calculate(transit).and_then(|result| {
                result
                    .reference_ephemeris
                    .and_then(|ephemeris| recovered::positions(&ephemeris, &ephemeris, settings))
                    .ok_or_else(|| "A supported date and known time are required.".to_owned())
            })
        };
        let positions = positions_at(false);
        let mut view = column()
            .child(heading(tool.to_owned()))
            .child(caption("Traditional natal calculations."));
        if tool == "Ashtakavarga" {
            view = view.child(caption("Brihat Jataka convention · unreduced points"));
            match positions {
                Ok(positions) => {
                    let signs =
                        std::array::from_fn(|index| recovered::sign(positions[(index + 1) % 8]));
                    let (individual, total) = ashtakavarga::calculate(signs);
                    let score_row = |label: &str, points: [u8; 12]| {
                        data_row(std::array::from_fn::<_, 14, _>(|index| match index {
                            0 => label.into(),
                            13 => points
                                .iter()
                                .map(|&point| u16::from(point))
                                .sum::<u16>()
                                .to_string(),
                            _ => points[index - 1].to_string(),
                        }))
                    };
                    view = view.child(
                        panel()
                            .child(data_row(
                                [
                                    "Body", "Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco",
                                    "Sag", "Cap", "Aqu", "Pis", "Total",
                                ]
                                .map(str::to_owned),
                            ))
                            .children(individual.into_iter().enumerate().map(|(index, points)| {
                                score_row(recovered::BODIES[index + 1], points)
                            }))
                            .child(score_row("All", total)),
                    );
                    view = view
                        .child(match positions_at(true) {
                            Ok(transit) => panel()
                                .child("Transit points from the natal table")
                                .child(data_row(
                                    [
                                        "Body",
                                        "Natal sign",
                                        "Transit sign",
                                        "Body points",
                                        "Sign total",
                                    ]
                                    .map(str::to_owned),
                                ))
                                .children(individual.iter().enumerate().map(|(index, points)| {
                                    let body = index + 1;
                                    let birth_sign = recovered::sign(positions[body]);
                                    let transit_sign = recovered::sign(transit[body]);
                                    data_row([
                                        recovered::BODIES[body].into(),
                                        varga::SIGNS[birth_sign].into(),
                                        varga::SIGNS[transit_sign].into(),
                                        points[transit_sign].to_string(),
                                        total[transit_sign].to_string(),
                                    ])
                                })),
                            Err(error) => panel().child(error),
                        })
                        .child(self.time_controls(cx));
                }
                Err(error) => view = view.child(panel().child(error)),
            }
        } else {
            let varga = self.selected_varga;
            view = view
                .child(caption(
                    "Parashara divisions with Narasimha Rao's D5 and D6 conventions.",
                ))
                .child(
                    div()
                        .grid()
                        .grid_cols(3)
                        .gap_2()
                        .children(varga::Varga::ALL.map(|item| {
                            button(
                                format!("D{} · {}", item as u8, item.name()),
                                item == varga,
                                cx,
                                move |view, _| view.selected_varga = item,
                            )
                        })),
                );
            match positions {
                Ok(positions) => {
                    let mapped = positions.map(|longitude| varga.longitude(longitude));
                    let scene = chart::ChartScene {
                        labels: chart::BODY_CODES
                            .into_iter()
                            .zip(mapped)
                            .map(|(code, value)| (code.into(), value))
                            .collect(),
                        title: varga.name(),
                        circular: false,
                        rahu: None,
                        houses: None,
                        color: GOLD,
                    };
                    view = view.child(
                        row().items_start().gap_4().child(scene.render(350.)).child(
                            panel()
                                .flex_1()
                                .child(data_row(["Body".into(), "Sign".into(), "Degree".into()]))
                                .children(
                                    chart::BODY_CODES
                                        .into_iter()
                                        .zip(recovered::BODIES)
                                        .zip(mapped)
                                        .map(|((code, name), longitude)| {
                                            data_row([
                                                format!("{code} · {name}"),
                                                varga::SIGNS[recovered::sign(longitude)].into(),
                                                format!("{:.3}°", longitude.rem_euclid(30.)),
                                            ])
                                        }),
                                ),
                        ),
                    );
                }
                Err(error) => view = view.child(panel().child(error)),
            }
        }
        view.child(button("All tools", false, cx, |view, cx| {
            view.update(cx, |s| s.tool = None)
        }))
        .into_any_element()
    }

    pub(super) fn nakshatra_groups_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use motaa_engine::EN;
        let session = self.session.read(cx);
        let settings = session.settings;
        let from_ascendant = self.tara_from_ascendant;
        let result = (|| {
            let natal = session
                .calculate(false)?
                .reference_ephemeris
                .ok_or("A known natal time is required.")?;
            let transit = session
                .calculate(true)?
                .reference_ephemeris
                .ok_or("Transit positions are outside the reference range.")?;
            let reference = if from_ascendant {
                natal.ascendant_sidereal_deg
            } else {
                natal.longitude("Moon", false).expect("reference Moon")
            };
            let reference = tara::mansion(reference);
            let mut settings = settings;
            settings.tropical = false;
            let positions = recovered::positions(&transit, &transit, settings)
                .ok_or("Planetary positions unavailable.")?;
            let transits = |matches: &dyn Fn(usize) -> bool| {
                let bodies = recovered::BODIES[1..]
                    .iter()
                    .zip(&positions[1..])
                    .filter_map(|(&body, &longitude)| {
                        matches(tara::mansion(longitude)).then_some(body)
                    })
                    .collect::<Vec<_>>();
                if bodies.is_empty() {
                    "—".into()
                } else {
                    bodies.join(", ")
                }
            };
            let tara = panel()
                .child("Nine Tara groups")
                .child(data_row([
                    "Group".into(),
                    "Mansions".into(),
                    "Transit bodies".into(),
                ]))
                .children(tara::GROUPS.into_iter().enumerate().map(|(index, group)| {
                    let members = [1, 10, 19]
                        .map(|count| EN[tara::from_count(reference, count + index)])
                        .join(" · ");
                    data_row([
                        group.into(),
                        members,
                        transits(&|mansion| tara::group(reference, mansion) == index),
                    ])
                }));
            let special = panel()
                .child("Six reference groups")
                .child(data_row([
                    "Group".into(),
                    "Mansion".into(),
                    "Transit bodies".into(),
                ]))
                .children(tara::SPECIAL.map(|(group, count)| {
                    let mansion = tara::from_count(reference, count);
                    data_row([
                        group.into(),
                        EN[mansion].into(),
                        transits(&|candidate| candidate == mansion),
                    ])
                }));
            Ok::<_, String>(
                column()
                    .child(caption(format!("Natal reference · {}", EN[reference])))
                    .child(tara)
                    .child(special),
            )
        })();
        let mut view = column()
            .child(heading("Nakshatra groups"))
            .child(caption(format!(
                "27 mansions · {}",
                model::AYANAMSHAS[settings.ayanamsha]
            )))
            .child(
                row()
                    .child(button(
                        "Natal Moon reference",
                        !from_ascendant,
                        cx,
                        |view, _| view.tara_from_ascendant = false,
                    ))
                    .child(button(
                        "Natal Ascendant reference",
                        from_ascendant,
                        cx,
                        |view, _| view.tara_from_ascendant = true,
                    )),
            );
        view = view.child(match result {
            Ok(data) => data,
            Err(error) => panel().child(error),
        });
        view.child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(crate) fn load_strength_timeline(&mut self, cx: &mut Context<Self>) {
        if self.strength.pending {
            return;
        }
        let session = self.session.read(cx);
        let birth = session.natal.clone();
        let settings = session.settings;
        let body = self.strength.body;
        self.strength.pending = true;
        spawn_named_worker(
            "motaa-strength",
            "Strength timeline",
            cx,
            move || strength_timeline::calculate(&birth, settings, body),
            |view, result, _| {
                view.strength.pending = false;
                view.strength.report = Some(result);
            },
        );
    }

    pub(super) fn strength_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view = column()
            .child(heading("Strength timeline"))
            .child(caption(
                "95-year Laghu Parashari Dasha + Maitri + Ashtakavarga strength curve with Antardasha/Pratyantardasha drilldown.",
            ))
            .child(row().children((1..=7).map(|body| {
                button(
                    recovered::BODIES[body],
                    self.strength.body == body,
                    cx,
                    move |view, _| view.strength.body = body,
                )
            })))
            .child(button(
                "Calculate timeline",
                true,
                cx,
                DesktopRoot::load_strength_timeline,
            ));
        if self.strength.pending {
            view = view.child(caption("Calculating annual samples…"));
        }
        if let Some(result) = self.strength.report.as_ref() {
            view = view.child(match result {
                Ok(report) => {
                    let scores: Vec<(u8, i32)> = report
                        .samples
                        .iter()
                        .map(|s| (s.age, s.combined_score))
                        .collect();
                    let min_score = scores.iter().map(|&(_, v)| v).min().unwrap_or(0);
                    let max_score = scores
                        .iter()
                        .map(|&(_, v)| v)
                        .max()
                        .unwrap_or(16)
                        .max(min_score + 1);
                    let selected_age = self.strength.age.unwrap_or(0).min(report.last_age);
                    let selected_sample = report
                        .samples
                        .iter()
                        .find(|s| s.age == selected_age)
                        .or_else(|| report.samples.first());
                    let curve = panel()
                        .child(format!(
                            "{} · {} · age 0–{} · Sidereal {}",
                            report.name,
                            report.body,
                            report.last_age,
                            model::AYANAMSHAS[report.ayanamsha]
                        ))
                        .child(
                            gpui::canvas(
                                |_, _, _| (),
                                move |bounds, _, window, _| {
                                    if scores.len() < 2 {
                                        return;
                                    }
                                    let pad = px(12.);
                                    let w = bounds.size.width - pad * 2.;
                                    let h = bounds.size.height - pad * 2.;
                                    let mut path = gpui::PathBuilder::stroke(px(2.));
                                    for (idx, &(_, val)) in scores.iter().enumerate() {
                                        let x = bounds.origin.x
                                            + pad
                                            + w * (idx as f32 / (scores.len() - 1) as f32);
                                        let norm = (val - min_score) as f32
                                            / (max_score - min_score) as f32;
                                        let y = bounds.origin.y + pad + h * (1. - norm);
                                        let pt = gpui::point(x, y);
                                        if idx == 0 {
                                            path.move_to(pt);
                                        } else {
                                            path.line_to(pt);
                                        }
                                    }
                                    if let Ok(built) = path.build() {
                                        window.paint_path(built, rgb(GOLD));
                                    }
                                },
                            )
                            .w_full()
                            .h(px(120.)),
                        );

                    let drilldown = if let Some(sample) = selected_sample {
                        let when_jd = motaa_engine::dasha::day_number(sample.when);
                        let md = sample
                            .major_index
                            .and_then(|i| report.dasha.major.get(i))
                            .copied();
                        let ad_rows: Vec<AnyElement> = md.map_or_else(Vec::new, |major| {
                            major
                                .children()
                                .into_iter()
                                .map(|ad| {
                                    let active_pd = ad
                                        .children()
                                        .into_iter()
                                        .find(|pd| pd.contains(when_jd))
                                        .map_or("—", |pd| pd.name());
                                    let start_s = motaa_engine::dasha::from_day_number(ad.start)
                                        .map_or_else(|| "—".into(), |d| d.date().to_string());
                                    let end_s = motaa_engine::dasha::from_day_number(ad.end)
                                        .map_or_else(|| "—".into(), |d| d.date().to_string());
                                    data_row([
                                        format!("{} → {}", major.name(), ad.name()),
                                        active_pd.into(),
                                        start_s,
                                        end_s,
                                    ])
                                    .into_any_element()
                                })
                                .collect()
                        });
                        panel()
                            .child(format!(
                                "Age {} ({}) · Score {} · Maitri: {} · Mahadasha: {}",
                                sample.age,
                                sample.when.date(),
                                sample.combined_score,
                                sample.maitri,
                                sample.lord
                            ))
                            .child(
                                row()
                                    .child(button("Prev age", false, cx, move |view, _| {
                                        view.strength.age = Some(selected_age.saturating_sub(1));
                                    }))
                                    .child(button("Next age", false, cx, move |view, _| {
                                        view.strength.age = Some(selected_age.saturating_add(1));
                                    }))
                                    .child(button("+10 yrs", false, cx, move |view, _| {
                                        view.strength.age = Some(selected_age.saturating_add(10));
                                    })),
                            )
                            .child(data_row([
                                "MD → AD".into(),
                                "PD at anniversary".into(),
                                "Start".into(),
                                "End".into(),
                            ]))
                            .children(ad_rows)
                    } else {
                        panel()
                    };

                    column()
                        .child(curve)
                        .child(drilldown)
                        .child(
                            panel()
                                .child(data_row([
                                    "Age".into(),
                                    "Sign (0–5)".into(),
                                    "Mansion (0–3)".into(),
                                    "Bindus (0–8)".into(),
                                    "Maitri".into(),
                                    "Score".into(),
                                    "Major lord".into(),
                                ]))
                                .children(report.samples.iter().map(|sample| {
                                    let date = if self.show_birth {
                                        format!("{} · {}", sample.age, sample.when.date())
                                    } else {
                                        sample.age.to_string()
                                    };
                                    data_row([
                                        date,
                                        sample.sign_class.to_string(),
                                        sample.mansion_class.to_string(),
                                        sample.natal_points.to_string(),
                                        sample.maitri.into(),
                                        sample.combined_score.to_string(),
                                        sample.lord.into(),
                                    ])
                                })),
                        )
                        .into_any_element()
                }
                Err(error) => panel().child(error.clone()).into_any_element(),
            });
        }
        view.child(self.birth_details_button(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn sarvatobhadra_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use sarvatobhadra::{GRID, cell, rays};
        let body = self.vedha.body;
        let (positions, natal_pos) = {
            let session = self.session.read(cx);
            let mut settings = session.settings;
            settings.tropical = false;
            let positions = session.calculate(self.vedha.transit).and_then(|result| {
                result
                    .reference_ephemeris
                    .and_then(|ephemeris| recovered::positions(&ephemeris, &ephemeris, settings))
                    .ok_or_else(|| "A supported date and known time are needed.".into())
            });
            let natal_pos = session
                .calculate(false)
                .ok()
                .and_then(|r| r.reference_ephemeris)
                .and_then(|e| recovered::positions(&e, &e, settings));
            (positions, natal_pos)
        };
        let mut view = column()
            .child(heading("Sarvatobhadra"))
            .child(caption("All-three vedha convention · sidereal positions"))
            .child(
                row()
                    .child(button("27 mansions", !self.vedha.abhijit, cx, |view, _| {
                        view.vedha.abhijit = false
                    }))
                    .child(button("28 mansions", self.vedha.abhijit, cx, |view, _| {
                        view.vedha.abhijit = true
                    }))
                    .child(button("Natal", !self.vedha.transit, cx, |view, _| {
                        view.vedha.transit = false
                    }))
                    .child(button("Transit", self.vedha.transit, cx, |view, _| {
                        view.vedha.transit = true
                    })),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_2()
                    .children((1..10).map(|index| {
                        button(
                            recovered::BODIES[index],
                            index == body,
                            cx,
                            move |view, _| view.vedha.body = index,
                        )
                    })),
            );
        match positions {
            Ok(positions) => {
                let origin = cell(positions[body], self.vedha.abhijit);
                let lines = rays(origin);
                let occupants = sarvatobhadra::cell_occupants(&positions, self.vedha.abhijit);
                let sp = natal_pos
                    .map(|np| sarvatobhadra::special_points(np[2], &positions, self.vedha.abhijit));
                let board = div()
                    .grid()
                    .grid_cols(9)
                    .border_1()
                    .border_color(rgb(LINE))
                    .children(GRID.into_iter().enumerate().flat_map(|(y, values)| {
                        let lines = &lines;
                        let occupants = &occupants;
                        values.into_iter().enumerate().map(move |(x, value)| {
                            let location = (y, x);
                            let color = if location == origin {
                                GOLD
                            } else if lines.iter().any(|line| line.contains(&location)) {
                                SELECTED
                            } else {
                                PAPER
                            };
                            let label = if !self.vedha.abhijit && value == "Abhijit" {
                                "—".to_owned()
                            } else if let Some((_, bodies)) =
                                occupants.iter().find(|(pos, _)| *pos == location)
                            {
                                format!("{value}\n[{}]", bodies.join(" "))
                            } else {
                                value.to_owned()
                            };
                            div()
                                .min_h(px(44.))
                                .p_1()
                                .flex()
                                .justify_center()
                                .items_center()
                                .text_size(px(11.))
                                .border_b_1()
                                .border_r_1()
                                .border_color(rgb(LINE))
                                .bg(rgb(color))
                                .text_color(rgb(if location == origin { PAPER } else { INK }))
                                .child(label)
                        })
                    }));
                let cell_name = |(r, c): (usize, usize)| GRID[r][c];
                let mut summary = panel()
                    .child(format!(
                        "{} · {}",
                        recovered::BODIES[body],
                        GRID[origin.0][origin.1]
                    ))
                    .children(
                        ["Across", "Fore", "Hind"]
                            .into_iter()
                            .zip(lines.iter())
                            .map(|(name, line)| {
                                let cells = line
                                    .iter()
                                    .map(|&(y, x)| GRID[y][x])
                                    .filter(|label| self.vedha.abhijit || *label != "Abhijit")
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                data_row([name.into(), cells])
                            }),
                    );
                if let Some(sp) = sp {
                    summary = summary
                        .child("Special Sarvatobhadra & Latta points (from Natal Moon)")
                        .child(data_row([
                            format!("Janma: {}", cell_name(sp.janma_cell)),
                            format!("Vajira (19th): {}", cell_name(sp.vajira_cell)),
                            format!("Nighata (22nd): {}", cell_name(sp.nighata_cell)),
                            format!("Bhumi (23rd): {}", cell_name(sp.bhumi_cell)),
                        ]))
                        .child(data_row([
                            format!("Sun Latta: {}", cell_name(sp.latta_sun)),
                            format!("Mars Latta: {}", cell_name(sp.latta_mars)),
                            format!("Jupiter Latta: {}", cell_name(sp.latta_jupiter)),
                            format!("Saturn Latta: {}", cell_name(sp.latta_saturn)),
                        ]));
                }
                view = view.child(board).child(summary);
            }
            Err(error) => view = view.child(panel().child(error)),
        }
        view.child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn satapati_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let settings = session.settings;
        let active_sector = self.name_wheel.sector;
        let scene = chart::ChartScene {
            labels: Vec::new(),
            title: "Name reference",
            circular: true,
            rahu: None,
            houses: None,
            color: GOLD,
        };
        let epochs = [
            ("Natal", session.calculate(false)),
            ("Transit", session.calculate(true)),
        ];
        let inspected = panel()
            .flex_1()
            .child("Bodies on the MOTAA syllable ring")
            .child(caption("Satapati syllable ring · select any syllable to set it as Lagna and inspect 12-house natal/transit placements and aspects."))
            .child(data_row(["Body".into(), "Epoch".into(), "Syllable".into(), "Longitude".into()]))
            .children(epochs.into_iter().flat_map(|(epoch, chart)| {
                let Ok(result) = chart else {
                    return vec![caption(format!("{epoch} positions unavailable."))];
                };
                let Some(ephemeris) = result.reference_ephemeris else {
                    return vec![caption(format!("{epoch} positions unavailable."))];
                };
                let mut rows = ephemeris.bodies.into_iter().filter_map(|body| {
                    let degree = if settings.tropical { body.tropical_longitude_deg } else { body.sidereal_longitude_deg };
                    let index = name_wheel::sector(degree);
                    if active_sector.is_some_and(|selected| selected != index) {
                        return None;
                    }
                    Some(data_row([body.name, epoch.into(), name_wheel::syllables()[index].clone(), format!("{degree:.3}°")]))
                }).collect::<Vec<_>>();
                if rows.is_empty() {
                    rows.push(caption(format!("No {epoch} bodies in this sector.")));
                }
                rows
            }));
        let lagna_sector = self.name_wheel.satapati_lagna;
        let natal_pos = session
            .calculate(false)
            .ok()
            .and_then(|r| r.reference_ephemeris)
            .and_then(|e| recovered::positions(&e, &e, settings));
        let transit_pos = session
            .calculate(true)
            .ok()
            .and_then(|r| r.reference_ephemeris)
            .and_then(|e| recovered::positions(&e, &e, settings));
        let satapati_panel = lagna_sector.map(|sec| {
            let houses = name_wheel::satapati_houses(sec, natal_pos, transit_pos);
            panel()
                .child(format!(
                    "Satapati Lagna · {} (Sector {} · {:.2}°)",
                    name_wheel::syllables()[sec],
                    sec + 1,
                    name_wheel::center(sec)
                ))
                .child(data_row([
                    "House".into(),
                    "Sign".into(),
                    "Natal".into(),
                    "Transit".into(),
                    "Aspect to Lagna".into(),
                ]))
                .children(houses.into_iter().map(|h| {
                    data_row([
                        format!("H{}", h.house),
                        h.sign.into(),
                        h.natal_bodies,
                        h.transit_bodies,
                        h.aspects_to_lagna,
                    ])
                }))
        });
        let matches = self.name_wheel.matches.clone();
        column()
            .child(heading("Satapati name wheel"))
            .child(
                row()
                    .child(div().w(px(230.)).child(self.name_wheel.reference.clone()))
                    .child(button("Find matching syllables", true, cx, |view, cx| {
                        let query = view.name_wheel.reference.read(cx).value().trim();
                        view.name_wheel.matches = name_wheel::syllables()
                            .iter()
                            .enumerate()
                            .filter_map(|(index, text)| (text == query).then_some(index))
                            .collect();
                        view.name_wheel.sector = view.name_wheel.matches.first().copied();
                        if view.name_wheel.sector.is_none() {
                            view.set_status("No exact matching syllable in the name wheel.", cx);
                        } else {
                            view.set_status("", cx);
                        }
                    }))
                    .when_some(active_sector, |r, sec| {
                        r.child(button(
                            "Set as Satapati Lagna",
                            lagna_sector == Some(sec),
                            cx,
                            move |view, _| view.name_wheel.satapati_lagna = Some(sec),
                        ))
                    })
                    .when(lagna_sector.is_some(), |r| {
                        r.child(button("Clear Lagna", false, cx, |view, _| {
                            view.name_wheel.satapati_lagna = None;
                        }))
                    }),
            )
            .when(active_sector.is_some(), |view| {
                view.child(button("Show all sectors", false, cx, |view, cx| {
                    view.name_wheel.sector = None;
                    view.name_wheel.matches.clear();
                    view.name_wheel
                        .reference
                        .update(cx, |input, cx| input.set_value(String::new(), cx));
                    view.set_status("", cx);
                }))
            })
            .when(!matches.is_empty(), |view| {
                view.child(row().flex_wrap().children(matches.into_iter().map(|index| {
                    button(
                        format!("{} · sector {}", name_wheel::syllables()[index], index + 1),
                        active_sector == Some(index),
                        cx,
                        move |view, _| view.name_wheel.sector = Some(index),
                    )
                })))
            })
            .child(
                row()
                    .items_start()
                    .gap_4()
                    .child(scene.render_mode9(400., &[], active_sector))
                    .child(inspected),
            )
            .when_some(satapati_panel, |view, panel| view.child(panel))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn interpretations_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let planet = self.interpretations.body;
        let category = self.interpretations.kind;
        let use_bhava = self.interpretations.bhava;
        let use_transit = self.interpretations.transit;
        let mut settings = session.settings;
        settings.tropical = false;
        let data = session.calculate(use_transit).and_then(|result| {
            let ephemeris = result
                .reference_ephemeris
                .ok_or("A supported date and known time are needed.")?;
            let positions = recovered::positions(&ephemeris, &ephemeris, settings)
                .ok_or("Chart positions unavailable.")?;
            let natal_ephemeris = session
                .calculate(false)?
                .reference_ephemeris
                .ok_or("A supported natal date and known time are needed.")?;
            let natal_positions = recovered::positions(&natal_ephemeris, &natal_ephemeris, settings)
                .ok_or("Natal positions unavailable.")?;
            let name = recovered::BODIES[planet];
            let current_jd = motaa_engine::dasha::day_number(session.time);
            let epoch_label = if use_transit { "Transit" } else { "Natal" };
            let text = match category {
                Interpretation::Period => {
                    let timeline = motaa_engine::dasha::calculate(&session.natal, natal_positions[2])?;
                    let period = timeline
                        .major
                        .iter()
                        .find(|period| period.contains(current_jd))
                        .copied();
                    if let Some(md) = period {
                        let ad = md.children().into_iter().find(|c| c.contains(current_jd));
                        let pd = ad.and_then(|c| c.children().into_iter().find(|p| p.contains(current_jd)));
                        let (chain, commentary) =
                            interpretations::dasha_pd_summary(md, ad, pd, &natal_positions);
                        panel()
                            .child("Vimshottari Mahadasha / Antardasha / Pratyantardasha")
                            .child(chain)
                            .child(commentary)
                            .child(caption(
                                "Active at selected reference time · Vimshottari Laghu Parashari synthesis",
                            ))
                    } else {
                        panel()
                            .child("Vimshottari period")
                            .child("Selected date is outside the timeline.")
                    }
                }
                Interpretation::Planet => {
                    let (title, nature, house_text) =
                        interpretations::planet_summary(planet, &positions, use_bhava, epoch_label);
                    let rows = (1..8).filter(|&target| target != planet).map(|target| {
                        let sign_offset =
                            (12 + recovered::sign(positions[target]) - recovered::sign(positions[planet])) % 12;
                        let maitri = ten_pillars::panchadha_maitri(planet, target, &positions);
                        data_row([
                            recovered::BODIES[target].into(),
                            format!("Sign {}", sign_offset + 1),
                            maitri.into(),
                        ])
                    });
                    panel()
                        .child(title)
                        .child(nature)
                        .child(house_text)
                        .child(data_row([
                            "Planet".into(),
                            "Offset".into(),
                            "Panchadha Maitri".into(),
                        ]))
                        .children(rows)
                }
                Interpretation::House => {
                    let sign = recovered::sign(positions[planet]);
                    let house = if use_bhava {
                        recovered::equal_bhava(positions[planet], positions[0])
                    } else {
                        ((12 + sign - recovered::sign(positions[0])) % 12) as u8 + 1
                    };
                    let signs =
                        std::array::from_fn(|index| recovered::sign(natal_positions[(index + 1) % 8]));
                    let (points, total) = ashtakavarga::calculate(signs);
                    let sign_lord = recovered::BODIES[ten_pillars::sign_lord(sign)];
                    panel()
                        .child(format!(
                            "{name} ({epoch_label}) · {} · House {house} ({}) · Lord: {sign_lord}",
                            varga::SIGNS[sign],
                            if use_bhava { "Bhava" } else { "Rasi" }
                        ))
                        .child(interpretations::house_domain(house))
                        .child(format!(
                            "{} individual bindus · {} combined Sarvashtakavarga bindus in {}",
                            points[planet.min(7) - 1][sign],
                            total[sign],
                            varga::SIGNS[sign]
                        ))
                }
                Interpretation::Nakshatra => {
                    let birth = tara::mansion(natal_positions[2]);
                    let selected = tara::mansion(positions[planet]);
                    let pada = ((positions[planet].rem_euclid(360.) % (360. / 27.)) / (360. / 108.)) as usize + 1;
                    let nav_sign = recovered::sign(recovered::navamsa(positions[planet]));
                    let group = tara::GROUPS[tara::group(birth, selected)];
                    panel()
                        .child(format!(
                            "{name} ({epoch_label}) · {} Pada {pada} · Navamsa {} · {group}",
                            motaa_engine::EN[selected],
                            varga::SIGNS[nav_sign]
                        ))
                        .child(interpretations::pada_focus(pada))
                        .child(format!(
                            "Natal Moon reference: {} · Nakshatra lord: {}",
                            motaa_engine::EN[birth],
                            recovered::BODIES[ten_pillars::nakshatra_lord(selected)]
                        ))
                }
                Interpretation::Yoga => {
                    let yogas = interpretations::detect_yogas(&positions);
                    let mut p = panel()
                        .child(format!(
                            "{epoch_label} Yogas & Sambandha ({} detected)",
                            yogas.len()
                        ))
                        .child(data_row([
                            "Category".into(),
                            "Yoga".into(),
                            "Participants".into(),
                            "Interpretation".into(),
                        ]));
                    if yogas.is_empty() {
                        p = p.child(caption("No classical Yoga triggers matched in this chart."));
                    } else {
                        p = p.children(yogas.into_iter().map(|y| {
                            data_row([y.category.into(), y.name.into(), y.participants, y.meaning.into()])
                        }));
                    }
                    p
                }
            };
            Ok(text)
        });
        column()
            .child(heading("Interpretations"))
            .child(caption(
                "Pratyantardasha, Planet/Sign/House (Rasi or Bhava), Nakshatra Pada, and Yoga/Sambandha analysis.",
            ))
            .child(
                row().children(
                    [
                        ("Dasha (MD/AD/PD)", Interpretation::Period),
                        ("Planet & Maitri", Interpretation::Planet),
                        ("Sign / House", Interpretation::House),
                        ("Nakshatra Pada", Interpretation::Nakshatra),
                        ("Yogas & Sambandha", Interpretation::Yoga),
                    ]
                    .map(|(label, choice)| {
                        button(label, category == choice, cx, move |view, _| {
                            view.interpretations.kind = choice
                        })
                    }),
                ),
            )
            .child(
                row()
                    .child(button("Natal", !use_transit, cx, |view, _| {
                        view.interpretations.transit = false
                    }))
                    .child(button("Transit", use_transit, cx, |view, _| {
                        view.interpretations.transit = true
                    }))
                    .when(matches!(category, Interpretation::Planet | Interpretation::House), |r| {
                        r.child(button("Rasi house", !use_bhava, cx, |view, _| {
                            view.interpretations.bhava = false
                        }))
                        .child(button("Bhava house", use_bhava, cx, |view, _| {
                            view.interpretations.bhava = true
                        }))
                    }),
            )
            .when(!matches!(category, Interpretation::Period | Interpretation::Yoga), |view| {
                view.child(row().children((1..8).map(|choice| {
                    button(
                        recovered::BODIES[choice],
                        choice == planet,
                        cx,
                        move |view, _| view.interpretations.body = choice,
                    )
                })))
            })
            .child(match data {
                Ok(view) => view,
                Err(error) => panel().child(error),
            })
            .when(category == Interpretation::Period, |view| view.child(self.time_controls(cx)))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn ten_pillars_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let mut settings = session.settings;
        settings.tropical = false;
        let output = (|| {
            let natal = session
                .calculate(false)?
                .reference_ephemeris
                .ok_or("A supported natal date and known time are required.")?;
            let transit = session
                .calculate(true)
                .ok()
                .and_then(|r| r.reference_ephemeris);
            let birth = recovered::positions(&natal, &natal, settings)
                .ok_or("Natal positions unavailable.")?;
            let current = transit.and_then(|t| recovered::positions(&t, &t, settings));
            let myanmar_year = ten_pillars::myanmar_year_for_date(
                session.natal.year,
                session.natal.month,
                session.natal.day,
            )
            .unwrap_or(session.natal.year - 638);
            let weekday_iso = i16::try_from(session.natal.year)
                .ok()
                .and_then(|y| {
                    jiff::civil::Date::new(y, session.natal.month as i8, session.natal.day as i8)
                        .ok()
                })
                .map_or(0, |d| d.weekday().to_sunday_zero_offset() as usize);
            let report =
                ten_pillars::calculate(myanmar_year, weekday_iso, birth, current, settings);
            Ok::<_, String>(
                column()
                    .child(
                        panel()
                            .child(format!(
                                "Mahabote & Ten Pillars · Myanmar Year {} (Seed {}) · {} ({})",
                                report.myanmar_year,
                                report.mahabote_seed,
                                report.birth_weekday,
                                report.birth_house
                            ))
                            .child(data_row([
                                "Adipati Pillar".into(),
                                report.adipati_planet.into(),
                                "Ratana Pillar".into(),
                                report.ratana_pillar,
                            ]))
                            .child(data_row([
                                "Suvanna (Gold)".into(),
                                report.suvanna_pillar,
                                "Rajita (Silver)".into(),
                                report.rajita_pillar,
                            ]))
                            .child(data_row([
                                "Ayu Moon".into(),
                                format!(
                                    "{} Pada {}",
                                    report.ayu_moon_mansion, report.ayu_moon_pada
                                ),
                                "Navamsa / Status".into(),
                                format!("{} · {}", report.ayu_moon_navamsa, report.ayu_moon_status),
                            ])),
                    )
                    .child(
                        panel()
                            .child("Seven Mahabote Houses")
                            .child(data_row([
                                "House".into(),
                                "Planet".into(),
                                "Weekday".into(),
                                "Pillar Role".into(),
                            ]))
                            .children(report.pillars.into_iter().map(|p| {
                                data_row([
                                    p.house_name.into(),
                                    p.planet.into(),
                                    p.weekday.into(),
                                    p.pillar_role.into(),
                                ])
                            })),
                    )
                    .child(
                        panel()
                            .child("Ayu Moon · Navamsa & Nakshatra-Lord Maitri (Natal & Transit)")
                            .child(data_row([
                                "Body".into(),
                                "Epoch".into(),
                                "Nakshatra (Pada)".into(),
                                "Navamsa (Lord)".into(),
                                "Maitri".into(),
                                "Sote / Upad".into(),
                            ]))
                            .children(report.rows.into_iter().map(|r| {
                                data_row([
                                    r.body.into(),
                                    r.epoch.into(),
                                    format!("{} P{}", r.mansion, r.pada),
                                    format!("{} ({})", r.navamsa_sign, r.navamsa_lord),
                                    r.maitri_to_moon.into(),
                                    r.padakhike_status.into(),
                                ])
                            })),
                    ),
            )
        })();
        column()
            .child(heading("Ten pillars / Ayu Moon"))
            .child(caption(
                "Traditional Burmese Mahabote Ten Pillars, Sote/Upad/Padakhike, and Ayu Moon Navamsa-Maitri analysis.",
            ))
            .child(match output {
                Ok(view) => view.into_any_element(),
                Err(error) => panel().child(error).into_any_element(),
            })
            .child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }
}
