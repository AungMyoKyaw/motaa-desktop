use super::*;

impl DesktopRoot {
    pub(super) fn load_ephemeris_month(&mut self, cx: &mut Context<Self>) {
        if self.ephemeris_progress {
            return;
        }
        let session = self.session.read(cx);
        let date = session.time.date();
        let profile = session.reference_profile().clone();
        let settings = session.settings;
        self.ephemeris_request_id = self.ephemeris_request_id.wrapping_add(1);
        let request_id = self.ephemeris_request_id;
        self.ephemeris_progress = true;
        // astro's VSOP routines use large stack frames in debug builds. The
        // macOS dispatch pool has a much smaller stack than the window thread.
        let worker = std::thread::Builder::new()
            .name("motaa-ephemeris".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                if !(1800..=2100).contains(&date.year()) {
                    return Err("Monthly reference ephemerides support 1800–2100.".into());
                }
                let mut rows = Vec::new();
                let mut planets = Vec::new();
                for day in 1..=31 {
                    let Some(midnight) =
                        jiff::civil::Date::new(date.year(), date.month(), day as i8)
                            .ok()
                            .map(|d| d.at(0, 0, 0, 0))
                    else {
                        break;
                    };
                    let civil_date = midnight.date().to_string();
                    let result = (|| {
                        let mut positions = model::calculate_at(&profile, midnight)?
                            .reference_ephemeris
                            .ok_or("Reference positions unavailable.")?;
                        positions.select_ayanamsha(settings.ayanamsha)?;
                        let longitude = |name| {
                            positions
                                .longitude(name, settings.tropical)
                                .map_or("—".into(), |angle| format!("{angle:.3}°"))
                        };
                        let jdn = settings.calendar.numbered_day(
                            i32::from(date.year()),
                            date.month() as u8,
                            day as u8,
                        );
                        let lunar = motaa_engine::myanmar_calendar::from_julian_day_number(jdn)
                            .map_or("—".into(), |date| {
                                format!("{} {} · {}", date.month_english, date.day, date.phase)
                            });
                        let (bounds, timing) =
                            model::daily_timing(&profile, midnight, settings.ayanamsha)?;
                        let transition = |index: usize| {
                            let entries = timing.remaining_day_transitions[index]
                                .iter()
                                .filter_map(|(instant, label)| {
                                    model::reference_datetime(&profile, *instant)
                                        .map(|time| format!("{} · {label}", time.strftime("%H:%M")))
                                })
                                .collect::<Vec<_>>();
                            if entries.is_empty() {
                                "—".into()
                            } else {
                                entries.join("\n")
                            }
                        };
                        let moon_change = motaa_engine::panchanga::moon_sign_change(
                            bounds,
                            &model::chart_request_at(&profile, midnight),
                            settings.tropical,
                            settings.ayanamsha,
                        )?
                        .and_then(|(instant, sign)| {
                            model::reference_datetime(&profile, instant).map(|time| {
                                format!("{} · {}", time.strftime("%H:%M"), varga::SIGNS[sign])
                            })
                        })
                        .unwrap_or_else(|| "—".into());
                        let [rahu, ketu] =
                            positions.lunar_nodes(settings.mean_node, settings.tropical);
                        planets.push([
                            civil_date.clone(),
                            longitude("Mars"),
                            longitude("Mercury"),
                            longitude("Jupiter"),
                            longitude("Venus"),
                            longitude("Saturn"),
                            format!("{rahu:.3}°"),
                            format!("{ketu:.3}°"),
                        ]);
                        Ok::<_, String>([
                            civil_date.clone(),
                            longitude("Sun"),
                            longitude("Moon"),
                            lunar,
                            transition(0),
                            transition(1),
                            moon_change,
                        ])
                    })();
                    rows.push(result.unwrap_or_else(|error| {
                        planets.push(std::array::from_fn(|index| {
                            if index == 0 {
                                civil_date.clone()
                            } else {
                                "—".into()
                            }
                        }));
                        [
                            civil_date,
                            "—".into(),
                            "—".into(),
                            error,
                            "—".into(),
                            "—".into(),
                            "—".into(),
                        ]
                    }));
                }

                Ok::<_, String>(EphemerisMonth {
                    title: date.strftime("%B %Y").to_string(),
                    rows,
                    planets,
                    settings,
                    birthplace: profile.birthplace,
                })
            });
        cx.spawn(async move |view, cx| {
            let result = match worker {
                Ok(worker) => {
                    cx.background_executor()
                        .spawn(async move {
                            worker
                                .join()
                                .map_err(|_| "Monthly ephemeris worker failed.".to_owned())?
                        })
                        .await
                }
                Err(error) => Err(format!("Could not start monthly ephemeris: {error}")),
            };
            let _ = view.update(cx, |view, cx| {
                if request_id == view.ephemeris_request_id {
                    view.ephemeris_progress = false;
                    view.ephemeris_month = Some(result);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn traditional_charts_view(&self, tool: &str, cx: &mut Context<Self>) -> AnyElement {
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
        let mut view = column().child(heading(tool.to_owned())).child(caption(
            "Traditional calculations using independent natal positions.",
        ));
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
                        color: PALETTES[self.chart_palette].1,
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

    fn electional_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let profile = session.reference_profile();
        let selection = session.time;
        let ayanamsha = session.settings.ayanamsha;
        let display = |jd: Option<f64>| {
            jd.and_then(|instant| model::display_reference_time(profile, instant))
                .unwrap_or_else(|| "No crossing in range".into())
        };
        let result = match model::daily_timing(profile, selection, ayanamsha) {
            Ok((_, timing)) => panel()
                .child(
                    row()
                        .gap_6()
                        .child(caption(format!("Sunrise · {}", display(timing.sunrise))))
                        .child(caption(format!("Sunset · {}", display(timing.sunset)))),
                )
                .child(
                    row()
                        .gap_6()
                        .child(caption(format!("Moonrise · {}", display(timing.moonrise))))
                        .child(caption(format!("Moonset · {}", display(timing.moonset)))),
                )
                .child(data_row([
                    "Limb".into(),
                    "Current".into(),
                    "Next".into(),
                    "Estimated local transition".into(),
                ]))
                .children(timing.limbs.into_iter().map(|limb| {
                    data_row([
                        limb.kind.into(),
                        limb.active,
                        limb.next,
                        display(limb.next_jdut),
                    ])
                })),
            Err(error) => panel().child(error),
        };
        column()
            .child(heading("Electional timing"))
            .child(caption(format!(
                "{} · {}",
                profile.birthplace,
                format_time(selection)
            )))
            .child(caption(format!(
                "Panchanga · Sidereal {}",
                model::AYANAMSHAS[ayanamsha]
            )))
            .child(result)
            .child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn search_transits(&mut self, cx: &mut Context<Self>) {
        if self.transit_searching {
            return;
        }
        let session = self.session.read(cx);
        let profile = session.reference_profile().clone();
        let start = session.time;
        let settings = session.settings;
        self.transit_searching = true;
        let worker = std::thread::Builder::new()
            .name("motaa-transits".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                transits::calculate(profile, start, settings.tropical, settings.ayanamsha)
            });
        cx.spawn(async move |view, cx| {
            let result = match worker {
                Ok(worker) => {
                    cx.background_executor()
                        .spawn(async move {
                            worker
                                .join()
                                .map_err(|_| "Transit search worker failed.".to_owned())?
                        })
                        .await
                }
                Err(error) => Err(format!("Could not start transit search: {error}")),
            };
            let _ = view.update(cx, |view, cx| {
                view.transit_searching = false;
                view.transit_report = Some(result);
                cx.notify();
            });
        })
        .detach();
    }

    fn transits_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view = column()
            .child(heading("Transit events"))
            .child(caption(
                "Reference sign ingresses and planetary stations over the following 30 days.",
            ))
            .child(
                row()
                    .child(button(
                        "Search from selected time",
                        true,
                        cx,
                        DesktopRoot::search_transits,
                    ))
                    .children(
                        [
                            ("All events", TransitFilter::All),
                            ("Ingresses", TransitFilter::Ingress),
                            ("Stations", TransitFilter::Station),
                        ]
                        .map(|(label, filter)| {
                            button(label, self.transit_filter == filter, cx, move |view, _| {
                                view.transit_filter = filter
                            })
                        }),
                    ),
            );
        if self.transit_searching {
            view = view.child(caption("Calculating transit events…"));
        }
        match self.transit_report.as_ref() {
            Some(Ok(report)) => {
                let display = |jd| {
                    model::display_reference_time(&report.profile, jd).unwrap_or_else(|| "—".into())
                };
                let rows: Vec<_> = report
                    .events
                    .iter()
                    .filter(|event| match self.transit_filter {
                        TransitFilter::All => true,
                        TransitFilter::Ingress => event.change == transits::Change::Ingress,
                        TransitFilter::Station => event.change != transits::Change::Ingress,
                    })
                    .collect();
                view = view.child(
                    panel()
                        .child(caption(format!(
                            "{} to {} · {}",
                            display(report.start),
                            display(report.end),
                            if report.tropical {
                                "Tropical"
                            } else {
                                model::AYANAMSHAS[report.ayanamsha]
                            }
                        )))
                        .child(data_row([
                            "Local date and time".into(),
                            "Body".into(),
                            "Event".into(),
                            "Sign".into(),
                        ]))
                        .when(rows.is_empty(), |panel| {
                            panel.child(caption("No matching events in this window."))
                        })
                        .children(rows.into_iter().enumerate().map(|(index, event)| {
                            let datetime = display(event.jdut);
                            let julian = event.jdut;
                            let target = model::reference_datetime(
                                self.session.read(cx).reference_profile(),
                                julian,
                            );
                            row()
                                .id(index)
                                .child(
                                    data_row([
                                        datetime,
                                        event.body.into(),
                                        event.change.label().into(),
                                        varga::SIGNS[event.sign].into(),
                                    ])
                                    .flex_1(),
                                )
                                .when_some(target, |entry, time| {
                                    entry.child(button(
                                        "Go to event",
                                        false,
                                        cx,
                                        move |view, cx| {
                                            view.set_time(time, true, cx);
                                            view.select_mode("Natal + transit", cx);
                                        },
                                    ))
                                })
                        })),
                );
            }
            Some(Err(error)) => view = view.child(panel().child(error.clone())),
            None => {}
        }
        view.child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    fn daily_tatkalika_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let profile = session.reference_profile();
        let selection = session.time;
        let settings = session.settings;
        let result = (|| {
            let (bounds, timing) = model::daily_timing(profile, selection, settings.ayanamsha)?;
            let noon = selection.date().at(12, 0, 0, 0);
            let noon = model::julian_at(profile, noon)?;
            let anchors = [
                Some(bounds[0]),
                timing.sunrise,
                Some(noon),
                timing.sunset,
                Some(bounds[1]),
            ];
            let charts = anchors.map(|anchor| {
                anchor
                    .map(|jd| {
                        let mut chart = motaa_engine::reference_ephemeris::calculate(
                            jd,
                            i32::from(selection.year()),
                            selection.month() as u8,
                            profile.latitude,
                            profile.longitude,
                        )?;
                        chart.select_ayanamsha(settings.ayanamsha)?;
                        Ok::<_, String>(chart)
                    })
                    .transpose()
            });
            let charts = charts.into_iter().collect::<Result<Vec<_>, _>>()?;
            let mut anchors_table = panel().child("Daily Moon path");
            for ((label, instant), chart) in
                ["Start of day", "Sunrise", "Noon", "Sunset", "End of day"]
                    .into_iter()
                    .zip(anchors)
                    .zip(&charts)
            {
                let date = instant
                    .and_then(|jd| model::display_reference_time(profile, jd))
                    .unwrap_or_else(|| "No horizon crossing".into());
                let mansion = chart
                    .as_ref()
                    .and_then(|chart| chart.longitude("Moon", false))
                    .map(|angle| {
                        let sector = angle / (360. / 27.);
                        let name = motaa_engine::EN[sector as usize];
                        format!("{name} · Pada {}", (sector.fract() * 4.) as usize + 1)
                    })
                    .unwrap_or_else(|| "—".into());
                anchors_table = anchors_table.child(data_row([label.into(), date, mansion]));
            }
            let positions = panel()
                .child(data_row(
                    ["Body", "Start", "Sunrise", "Noon", "Sunset", "End"].map(str::to_owned),
                ))
                .children(recovered::BODIES[1..].iter().map(|&body| {
                    let values = std::array::from_fn::<_, 6, _>(|index| {
                        if index == 0 {
                            return body.into();
                        }
                        charts[index - 1]
                            .as_ref()
                            .and_then(|chart| match body {
                                "Rahu" => Some(chart.lunar_nodes(settings.mean_node, false)[0]),
                                "Ketu" => Some(chart.lunar_nodes(settings.mean_node, false)[1]),
                                _ => chart.longitude(body, false),
                            })
                            .map_or_else(|| "—".into(), |longitude| format!("{longitude:.3}°"))
                    });
                    data_row(values)
                }));
            Ok::<_, String>(column().child(anchors_table).child(positions))
        })();
        column()
            .child(heading("Daily Tatkalika"))
            .child(caption(format!(
                "{} · {}",
                profile.birthplace,
                selection.date()
            )))
            .child(caption(format!(
                "Sidereal {} · {} Rahu",
                model::AYANAMSHAS[settings.ayanamsha],
                if settings.mean_node { "Mean" } else { "True" }
            )))
            .child(match result {
                Ok(view) => view,
                Err(error) => panel().child(error),
            })
            .child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    fn nakshatra_groups_view(&self, cx: &mut Context<Self>) -> AnyElement {
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
                .ok_or("Reference positions unavailable.")?;
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

    fn eclipses_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use motaa_engine::eclipses::{catalog, local};
        let session = self.session.read(cx);
        let profile = session.reference_profile().clone();
        let year = session.time.year();
        let range = year..=(year + 4).min(2100);
        let lunar = self.eclipse_lunar;
        let visible_only = self.eclipse_visible_only;
        let events: Vec<_> = catalog()
            .filter(|event| event.lunar == lunar && range.contains(&(event.year as i16)))
            .filter(|&event| {
                !visible_only || local(event, profile.latitude, profile.longitude).is_some()
            })
            .collect();
        let selected = events
            .iter()
            .copied()
            .find(|event| Some(event.greatest_jdut) == self.eclipse_selection)
            .or_else(|| events.first().copied());
        let display =
            |jd| model::display_reference_time(&profile, jd).unwrap_or_else(|| "—".into());
        let mut view = column()
            .child(heading("Eclipses"))
            .child(caption(format!(
                "{}–{} · {} · NASA catalog coverage 2001–2100",
                range.start(),
                range.end(),
                profile.birthplace
            )))
            .child(
                row()
                    .child(button("Solar", !lunar, cx, |view, _| {
                        view.eclipse_lunar = false;
                        view.eclipse_selection = None;
                    }))
                    .child(button("Lunar", lunar, cx, |view, _| {
                        view.eclipse_lunar = true;
                        view.eclipse_selection = None;
                    }))
                    .child(button(
                        "Visible from reference location",
                        visible_only,
                        cx,
                        |view, _| view.eclipse_visible_only = !view.eclipse_visible_only,
                    )),
            );
        let event_list = panel()
            .child("Catalog events")
            .child(data_row([
                "Greatest eclipse".into(),
                "Type".into(),
                "Magnitude".into(),
            ]))
            .when(events.is_empty(), |view| {
                view.child(caption("No matching catalog events in this range."))
            })
            .children(events.into_iter().enumerate().map(|(index, event)| {
                row()
                    .id(index)
                    .child(
                        data_row([
                            display(event.greatest_jdut),
                            event.global_kind.into(),
                            format!("{:.4}", event.magnitude),
                        ])
                        .flex_1(),
                    )
                    .child(button(
                        "Inspect",
                        selected.is_some_and(|item| item.greatest_jdut == event.greatest_jdut),
                        cx,
                        move |view, _| view.eclipse_selection = Some(event.greatest_jdut),
                    ))
            }));
        view = view.child(event_list);
        if let Some(event) = selected {
            let details = panel().child("Local circumstances");
            view = view.child(match local(event, profile.latitude, profile.longitude) {
                Some(result) => details.child(result.kind)
                    .child(caption(if lunar { "Global phase times estimated from NASA catalog durations. The local type reflects the phases above the horizon." }
                        else { "Approximate topocentric disk contacts. Contacts can occur below the local horizon." }))
                    .when(result.rise_during_eclipse_jdut.is_some() || result.set_during_eclipse_jdut.is_some(), |view| {
                        let start = result.rise_during_eclipse_jdut.unwrap_or(result.first_jdut);
                        let end = result.set_during_eclipse_jdut.unwrap_or(result.last_jdut);
                        view.child(caption(format!("Above horizon from {} to {}", display(start), display(end))))
                    })
                    .child(data_row([
                        if lunar { "Penumbral begins" } else { "First contact" }.into(),
                        if lunar { "Global maximum" } else { "Maximum" }.into(),
                        if lunar { "Penumbral ends" } else { "Last contact" }.into(),
                    ]))
                    .child(data_row([display(result.first_jdut), display(result.maximum_jdut), display(result.last_jdut)]))
                    .when_some(result.rise_during_eclipse_jdut, |view, time| view.child(caption(format!("{} during eclipse · {}", if lunar { "Moonrise" } else { "Sunrise" }, display(time)))))
                    .when_some(result.set_during_eclipse_jdut, |view, time| view.child(caption(format!("{} during eclipse · {}", if lunar { "Moonset" } else { "Sunset" }, display(time)))))
                    .when_some(result.umbral_contacts_jdut, |view, [start, end]| view
                        .child(data_row(["Umbral phase begins".into(), "Umbral phase ends".into()]))
                        .child(data_row([display(start), display(end)])))
                    .when_some(result.central_contacts_jdut, |view, [start, end]| view
                        .child(data_row(if lunar { ["Totality begins".into(), "Totality ends".into()] } else { ["Central phase begins".into(), "Central phase ends".into()] }))
                        .child(data_row([display(start), display(end)]))),
                None => details.child("No above-horizon eclipse overlap at the reference location."),
            });
        }
        view.child(self.time_controls(cx))
            .child(button("All tools", false, cx, |view, cx| {
                view.update(cx, |s| s.tool = None)
            }))
            .into_any_element()
    }

    pub(super) fn load_strength_timeline(&mut self, cx: &mut Context<Self>) {
        if self.strength_pending {
            return;
        }
        let session = self.session.read(cx);
        let birth = session.natal.clone();
        let settings = session.settings;
        let body = self.strength_body;
        self.strength_pending = true;
        let worker = std::thread::Builder::new()
            .name("motaa-strength".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || strength_timeline::calculate(&birth, settings, body));
        cx.spawn(async move |view, cx| {
            let result = match worker {
                Ok(worker) => {
                    cx.background_executor()
                        .spawn(async move {
                            worker
                                .join()
                                .map_err(|_| "Strength timeline worker failed.".to_owned())?
                        })
                        .await
                }
                Err(error) => Err(format!("Could not start strength timeline: {error}")),
            };
            let _ = view.update(cx, |view, cx| {
                view.strength_pending = false;
                view.strength_report = Some(result);
                cx.notify();
            });
        })
        .detach();
    }

    fn strength_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view = column()
            .child(heading("Strength timeline"))
            .child(caption(
                "95-year Laghu Parashari Dasha + Maitri + Ashtakavarga strength curve with Antardasha/Pratyantardasha drilldown.",
            ))
            .child(row().children((1..=7).map(|body| {
                button(
                    recovered::BODIES[body],
                    self.strength_body == body,
                    cx,
                    move |view, _| view.strength_body = body,
                )
            })))
            .child(button(
                "Calculate timeline",
                true,
                cx,
                DesktopRoot::load_strength_timeline,
            ));
        if self.strength_pending {
            view = view.child(caption("Calculating annual samples…"));
        }
        if let Some(result) = self.strength_report.as_ref() {
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
                    let selected_age = self.strength_age.unwrap_or(0).min(report.last_age);
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
                                        view.strength_age = Some(selected_age.saturating_sub(1));
                                    }))
                                    .child(button("Next age", false, cx, move |view, _| {
                                        view.strength_age = Some(selected_age.saturating_add(1));
                                    }))
                                    .child(button("+10 yrs", false, cx, move |view, _| {
                                        view.strength_age = Some(selected_age.saturating_add(10));
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
        view.child(button(
            if self.show_birth {
                "Hide birth details"
            } else {
                "Show birth details"
            },
            false,
            cx,
            |view, _| view.show_birth = !view.show_birth,
        ))
        .child(button("All tools", false, cx, |view, cx| {
            view.update(cx, |s| s.tool = None)
        }))
        .into_any_element()
    }

    fn compare_candidate_times(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let settings = session.settings;
        let profile = session.natal.clone();
        let rows = self
            .candidate_inputs
            .iter()
            .enumerate()
            .map(|(index, input)| {
                let label = format!("Candidate {}", index + 1);
                let candidate = (|| {
                    let time = parse_time(input.read(cx).value())?;
                    let result = model::calculate_at(&profile, time)?;
                    let mut ephemeris = result
                        .reference_ephemeris
                        .ok_or("Outside the reference date range.")?;
                    ephemeris.select_ayanamsha(settings.ayanamsha)?;
                    let positions = recovered::positions(&ephemeris, &ephemeris, settings)
                        .ok_or("Missing reference positions.")?;
                    let ascendant = recovered::sign(positions[0]);
                    let moon = ephemeris
                        .longitude("Moon", false)
                        .ok_or("Missing Moon position.")?;
                    let (_, points) = ashtakavarga::calculate(std::array::from_fn(|index| {
                        recovered::sign(positions[(index + 1) % 8])
                    }));
                    let relation = recovered::whole_sign_aspect(5, positions[5], positions[2]);
                    let yogas = interpretations::detect_yogas(&positions).len();
                    let dasha_lord = motaa_engine::dasha::calculate(&profile, moon)
                        .map_or("—", |d| d.major[0].name());
                    Ok::<_, String>([
                        label.clone(),
                        varga::SIGNS[ascendant].into(),
                        motaa_engine::EN[tara::mansion(moon)].into(),
                        format!("{} ({dasha_lord})", points[ascendant]),
                        format!(
                            "{} · {yogas} yoga(s)",
                            if relation {
                                "Jupiter–Moon aspect"
                            } else {
                                "No Jup–Moon aspect"
                            }
                        ),
                    ])
                })();
                candidate.unwrap_or_else(|error| [label, "—".into(), "—".into(), "—".into(), error])
            })
            .collect();
        self.candidate_rows = Some(rows);
    }

    fn candidate_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view = column().child(heading("Yoga / candidate times"))
            .child(caption("Compare up to three birth times using the natal location and timezone. Factors are displayed individually."));
        if self.show_birth {
            view = view
                .child(self.candidate_pickers[0].render(
                    "Candidate 1",
                    false,
                    None,
                    None,
                    |view| Some(&mut view.candidate_pickers[0]),
                    cx,
                ))
                .child(self.candidate_pickers[1].render(
                    "Candidate 2",
                    false,
                    None,
                    None,
                    |view| Some(&mut view.candidate_pickers[1]),
                    cx,
                ))
                .child(self.candidate_pickers[2].render(
                    "Candidate 3",
                    false,
                    None,
                    None,
                    |view| Some(&mut view.candidate_pickers[2]),
                    cx,
                ));
        }
        view = view.child(
            row()
                .child(button(
                    if self.show_birth {
                        "Hide candidate times"
                    } else {
                        "Show candidate times"
                    },
                    false,
                    cx,
                    |view, _| view.show_birth = !view.show_birth,
                ))
                .child(button(
                    "Compare candidates",
                    true,
                    cx,
                    DesktopRoot::compare_candidate_times,
                )),
        );
        if let Some(rows) = &self.candidate_rows {
            view = view.child(
                panel()
                    .child(data_row([
                        "Candidate".into(),
                        "Ascendant".into(),
                        "Moon mansion".into(),
                        "Ascendant bindus".into(),
                        "Relationship".into(),
                    ]))
                    .children(rows.iter().cloned().map(data_row)),
            );
        }
        view.child(button("All tools", false, cx, |view, cx| {
            view.update(cx, |s| s.tool = None)
        }))
        .into_any_element()
    }

    fn sarvatobhadra_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use sarvatobhadra::{GRID, cell, rays};
        let body = self.vedha_body;
        let (positions, natal_pos) = {
            let session = self.session.read(cx);
            let mut settings = session.settings;
            settings.tropical = false;
            let positions = session.calculate(self.vedha_transit).and_then(|result| {
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
            .child(caption(
                "All-three vedha convention · independent sidereal positions",
            ))
            .child(
                row()
                    .child(button("27 mansions", !self.vedha_abhijit, cx, |view, _| {
                        view.vedha_abhijit = false
                    }))
                    .child(button("28 mansions", self.vedha_abhijit, cx, |view, _| {
                        view.vedha_abhijit = true
                    }))
                    .child(button("Natal", !self.vedha_transit, cx, |view, _| {
                        view.vedha_transit = false
                    }))
                    .child(button("Transit", self.vedha_transit, cx, |view, _| {
                        view.vedha_transit = true
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
                            move |view, _| view.vedha_body = index,
                        )
                    })),
            );
        match positions {
            Ok(positions) => {
                let origin = cell(positions[body], self.vedha_abhijit);
                let lines = rays(origin);
                let occupants = sarvatobhadra::cell_occupants(&positions, self.vedha_abhijit);
                let sp = natal_pos
                    .map(|np| sarvatobhadra::special_points(np[2], &positions, self.vedha_abhijit));
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
                            let label = if !self.vedha_abhijit && value == "Abhijit" {
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
                                    .filter(|label| self.vedha_abhijit || *label != "Abhijit")
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

    fn satapati_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let settings = session.settings;
        let active_sector = self.name_sector;
        let scene = chart::ChartScene {
            labels: Vec::new(),
            title: "Name reference",
            circular: true,
            rahu: None,
            houses: None,
            color: PALETTES[self.chart_palette].1,
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
        let lagna_sector = self.satapati_lagna;
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
        let matches = self.name_matches.clone();
        column()
            .child(heading("Satapati name wheel"))
            .child(
                row()
                    .child(div().w(px(230.)).child(self.name_reference.clone()))
                    .child(button("Find matching syllables", true, cx, |view, cx| {
                        let query = view.name_reference.read(cx).value().trim();
                        view.name_matches = name_wheel::syllables()
                            .iter()
                            .enumerate()
                            .filter_map(|(index, text)| (text == query).then_some(index))
                            .collect();
                        view.name_sector = view.name_matches.first().copied();
                        if view.name_sector.is_none() {
                            view.set_status(
                                "No exact matching syllable in the recovered ring.",
                                cx,
                            );
                        } else {
                            view.set_status("", cx);
                        }
                    }))
                    .when_some(active_sector, |r, sec| {
                        r.child(button(
                            "Set as Satapati Lagna",
                            lagna_sector == Some(sec),
                            cx,
                            move |view, _| view.satapati_lagna = Some(sec),
                        ))
                    })
                    .when(lagna_sector.is_some(), |r| {
                        r.child(button("Clear Lagna", false, cx, |view, _| {
                            view.satapati_lagna = None;
                        }))
                    }),
            )
            .when(active_sector.is_some(), |view| {
                view.child(button("Show all sectors", false, cx, |view, cx| {
                    view.name_sector = None;
                    view.name_matches.clear();
                    view.name_reference
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
                        move |view, _| view.name_sector = Some(index),
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

    fn interpretations_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let session = self.session.read(cx);
        let planet = self.interpretation_body;
        let category = self.interpretation;
        let use_bhava = self.interpretation_bhava;
        let use_transit = self.interpretation_transit;
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
                            view.interpretation = choice
                        })
                    }),
                ),
            )
            .child(
                row()
                    .child(button("Natal", !use_transit, cx, |view, _| {
                        view.interpretation_transit = false
                    }))
                    .child(button("Transit", use_transit, cx, |view, _| {
                        view.interpretation_transit = true
                    }))
                    .when(matches!(category, Interpretation::Planet | Interpretation::House), |r| {
                        r.child(button("Rasi house", !use_bhava, cx, |view, _| {
                            view.interpretation_bhava = false
                        }))
                        .child(button("Bhava house", use_bhava, cx, |view, _| {
                            view.interpretation_bhava = true
                        }))
                    }),
            )
            .when(!matches!(category, Interpretation::Period | Interpretation::Yoga), |view| {
                view.child(row().children((1..8).map(|choice| {
                    button(
                        recovered::BODIES[choice],
                        choice == planet,
                        cx,
                        move |view, _| view.interpretation_body = choice,
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

    fn ten_pillars_view(&self, cx: &mut Context<Self>) -> AnyElement {
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
                    None => panel().child("Reference positions unavailable for this date."),
                },
                Err(error) => panel().child(error),
            };
            return column()
                .child(tool_header(
                    "Sky map",
                    "Geocentric ecliptic projection · unit sphere · independent reference",
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
                    None => panel().child("Reference positions unavailable for this date."),
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
                        .child(caption("Desktop astronomical reference"))
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
                    None => panel().child("Reference positions unavailable for this date."),
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
                .when(self.ephemeris_progress, |view| {
                    view.child(caption("Calculating selected month…"))
                })
                .when_some(self.ephemeris_month.as_ref(), |view, result| match result {
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
                    .child(div().w(px(340.)).child(self.tool_search.clone())),
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
