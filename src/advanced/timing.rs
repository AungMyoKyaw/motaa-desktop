use super::*;

impl DesktopRoot {
    pub(crate) fn load_ephemeris_month(&mut self, cx: &mut Context<Self>) {
        if self.ephemeris.progress {
            return;
        }
        let session = self.session.read(cx);
        let date = session.time.date();
        let profile = session.reference_profile().clone();
        let settings = session.settings;
        self.ephemeris.request_id = self.ephemeris.request_id.wrapping_add(1);
        let request_id = self.ephemeris.request_id;
        self.ephemeris.progress = true;
        // astro's VSOP routines use large stack frames in debug builds. The
        // macOS dispatch pool has a much smaller stack than the window thread.
        spawn_named_worker(
            "motaa-ephemeris",
            "Monthly ephemeris",
            cx,
            move || {
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
                            .ok_or("Planetary positions unavailable.")?;
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
            },
            move |view, result, _| {
                if request_id == view.ephemeris.request_id {
                    view.ephemeris.progress = false;
                    view.ephemeris.month = Some(result);
                }
            },
        );
    }

    pub(super) fn electional_view(&self, cx: &mut Context<Self>) -> AnyElement {
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

    pub(crate) fn search_transits(&mut self, cx: &mut Context<Self>) {
        if self.transits.searching {
            return;
        }
        let session = self.session.read(cx);
        let profile = session.reference_profile().clone();
        let start = session.time;
        let settings = session.settings;
        self.transits.searching = true;
        spawn_named_worker(
            "motaa-transits",
            "Transit search",
            cx,
            move || transits::calculate(profile, start, settings.tropical, settings.ayanamsha),
            |view, result, _| {
                view.transits.searching = false;
                view.transits.report = Some(result);
            },
        );
    }

    pub(super) fn transits_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut view = column()
            .child(heading("Transit events"))
            .child(caption(
                "Sign ingresses and planetary stations over the following 30 days.",
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
                            button(label, self.transits.filter == filter, cx, move |view, _| {
                                view.transits.filter = filter
                            })
                        }),
                    ),
            );
        if self.transits.searching {
            use gpui::component::Sizable as _;
            view = view.child(
                row()
                    .gap_2()
                    .child(gpui::component::spinner::Spinner::new().xsmall())
                    .child(caption("Calculating transit events…")),
            );
        }
        match self.transits.report.as_ref() {
            Some(Ok(report)) => {
                let display = |jd| {
                    model::display_reference_time(&report.profile, jd).unwrap_or_else(|| "—".into())
                };
                let rows: Vec<_> = report
                    .events
                    .iter()
                    .filter(|event| match self.transits.filter {
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
                                    entry.child(button_in(
                                        "Go to event",
                                        false,
                                        cx,
                                        move |view, window, cx| {
                                            view.set_time(time, true, window, cx);
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

    pub(super) fn daily_tatkalika_view(&self, cx: &mut Context<Self>) -> AnyElement {
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

    pub(super) fn eclipses_view(&self, cx: &mut Context<Self>) -> AnyElement {
        use motaa_engine::eclipses::{catalog, local};
        let session = self.session.read(cx);
        let profile = session.reference_profile().clone();
        let year = session.time.year();
        let range = year..=(year + 4).min(2100);
        let lunar = self.eclipses.lunar;
        let visible_only = self.eclipses.visible_only;
        let events: Vec<_> = catalog()
            .filter(|event| event.lunar == lunar && range.contains(&(event.year as i16)))
            .filter(|&event| {
                !visible_only || local(event, profile.latitude, profile.longitude).is_some()
            })
            .collect();
        let selected = events
            .iter()
            .copied()
            .find(|event| Some(event.greatest_jdut) == self.eclipses.selection)
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
                        view.eclipses.lunar = false;
                        view.eclipses.selection = None;
                    }))
                    .child(button("Lunar", lunar, cx, |view, _| {
                        view.eclipses.lunar = true;
                        view.eclipses.selection = None;
                    }))
                    .child(button(
                        "Visible from reference location",
                        visible_only,
                        cx,
                        |view, _| view.eclipses.visible_only = !view.eclipses.visible_only,
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
                        move |view, _| view.eclipses.selection = Some(event.greatest_jdut),
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
                    let time = parse_time(input.read(cx).value().as_ref())?;
                    let result = model::calculate_at(&profile, time)?;
                    let mut ephemeris = result
                        .reference_ephemeris
                        .ok_or("Outside the reference date range.")?;
                    ephemeris.select_ayanamsha(settings.ayanamsha)?;
                    let positions = recovered::positions(&ephemeris, &ephemeris, settings)
                        .ok_or("Planetary positions unavailable.")?;
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

    pub(super) fn candidate_view(&self, cx: &mut Context<Self>) -> AnyElement {
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
}
