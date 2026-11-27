use std::{io::Write, path::PathBuf};

use gpui::{App, Window, WindowHandle, px, size};
use motaa_engine::ChartRequest;

use crate::{
    DesktopRoot, Interpretation, LightboxTarget, MODES, PALETTES, RahuReference, backup,
    model::Workspace, parse_time, primary_keybinding,
};

pub(crate) fn run(
    smoke: bool,
    store_screenshots: Option<PathBuf>,
    visual_capture: bool,
    window: WindowHandle<DesktopRoot>,
    cx: &mut App,
) {
    if smoke {
        cx.spawn(async move |cx| {
            let native: gpui::AnyWindowHandle = window.into();
            let draw = |cx: &mut gpui::AsyncApp| {
                native
                    .update(cx, |_, window, app| {
                        window.refresh();
                        let _ = window.draw(app);
                    })
                    .expect("draw hidden native window");
            };
            draw(cx);
            let steps = [
                (Workspace::Profiles, primary_keybinding("2")),
                (Workspace::Calculations, primary_keybinding("3")),
                (Workspace::Time, primary_keybinding("4")),
                (Workspace::Advanced, primary_keybinding("5")),
                (Workspace::Charts, primary_keybinding("1")),
            ];
            cx.background_executor()
                .timer(std::time::Duration::from_millis(300))
                .await;
            for (route, shortcut) in steps {
                native
                    .update(cx, |_, window, app| {
                        let keystroke =
                            gpui::Keystroke::parse(&shortcut).expect("valid smoke shortcut");
                        assert!(window.dispatch_keystroke(keystroke, app));
                    })
                    .expect("dispatch native workspace shortcut");
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(250))
                    .await;
                draw(cx);
                window
                    .update(cx, |root, _, cx| {
                        assert!(
                            root.session.read(cx).workspace == route,
                            "expected {}",
                            route.label()
                        );
                    })
                    .expect("verify native workspace");
                println!("Rendered {}", route.label());
            }
            println!("Verified workspace keyboard shortcuts");
            native
                .update(cx, |_, window, app| {
                    let press = |window: &mut Window, app: &mut App, shortcut| {
                        let key = gpui::Keystroke::parse(shortcut).expect("valid focus shortcut");
                        assert!(window.dispatch_keystroke(key, app));
                    };
                    press(window, app, "tab");
                    let first = window.focused(app).expect("first tab stop");
                    press(window, app, "tab");
                    assert_ne!(window.focused(app), Some(first.clone()));
                    press(window, app, "shift-tab");
                    assert_eq!(window.focused(app), Some(first));
                })
                .expect("verify keyboard focus traversal");
            println!("Verified forward and reverse keyboard focus");
            let root_focus = window
                .read_with(cx, |root, _| root.focus.clone())
                .expect("read root focus");
            native
                .update(cx, |_, win, app| {
                    win.focus(&root_focus);
                    for shortcut in ["tab", "tab", "tab", "enter"] {
                        let key = gpui::Keystroke::parse(shortcut).expect("valid activation key");
                        assert!(win.dispatch_keystroke(key, app));
                    }
                })
                .expect("dispatch enter activation");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    assert!(root.session.read(cx).workspace == Workspace::Profiles);
                })
                .expect("verify enter switched to Profiles");
            native
                .update(cx, |_, win, app| {
                    for shortcut in ["shift-tab", "space"] {
                        let key = gpui::Keystroke::parse(shortcut).expect("valid activation key");
                        assert!(win.dispatch_keystroke(key, app));
                    }
                })
                .expect("dispatch space activation");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    assert!(root.session.read(cx).workspace == Workspace::Charts);
                })
                .expect("verify space returned to Charts");
            println!("Verified keyboard control activation");
            let smoke_profile_id = window
                .update(cx, |root, _, cx| {
                    root.open_draft(ChartRequest::default(), None, cx);
                    let draft = root.draft.as_ref().expect("open profile draft");
                    draft
                        .name
                        .update(cx, |input, cx| input.set_value("Smoke Unknown".into(), cx));
                    draft
                        .datetime
                        .update(cx, |input, cx| input.set_value("invalid-date".into(), cx));
                    root.save_draft(cx);
                    assert!(root.draft.is_some());
                    assert!(!root.session.read(cx).status.is_empty());
                    root.draft
                        .as_ref()
                        .unwrap()
                        .datetime
                        .update(cx, |input, cx| {
                            input.set_value("1992-08-14 00:00:00".into(), cx)
                        });
                    root.sync_datetime_pickers(cx);
                    assert_eq!(root.draft.as_ref().unwrap().picker.view_year, 1992);
                    assert_eq!(root.draft.as_ref().unwrap().picker.view_month, 8);
                    root.draft
                        .as_mut()
                        .unwrap()
                        .picker
                        .select_calendar_date(jiff::civil::Date::constant(1992, 8, 14), cx);
                    root.draft.as_mut().unwrap().unknown = true;
                    root.save_draft(cx);
                    assert!(root.draft.is_none());
                    let id = root.selected.expect("saved profile id");
                    assert!(
                        root.session
                            .read(cx)
                            .calculate(false)
                            .unwrap()
                            .reference_ephemeris
                            .is_none()
                    );
                    root.navigate(Workspace::Charts, cx);
                    id
                })
                .expect("verify profile error and unknown birth time");
            window
                .update(cx, |root, _, cx| {
                    root.open_draft(ChartRequest::default(), None, cx);
                    assert!(root.show_profile_editor);
                    root.draft.as_ref().unwrap().name.update(cx, |input, cx| {
                        input.set_value("Unfinished profile".into(), cx)
                    });
                })
                .expect("open a second profile draft");
            draw(cx);
            native
                .update(cx, |_, win, app| {
                    let key = gpui::Keystroke::parse(&primary_keybinding("2")).unwrap();
                    assert!(win.dispatch_keystroke(key, app));
                })
                .expect("open saved profiles while editing a draft");
            draw(cx);
            window
                .read_with(cx, |root, cx| {
                    assert!(root.session.read(cx).workspace == Workspace::Profiles);
                    assert!(!root.show_profile_editor);
                })
                .expect("verify Profiles returns to the library");
            native
                .update(cx, |_, win, app| {
                    win.focus(&root_focus);
                    let tab = gpui::Keystroke::parse("tab").unwrap();
                    // 6 sidebar controls + collapse toggle + quick jump + birth details + search + resume, then row.
                    for _ in 0..12 {
                        assert!(win.dispatch_keystroke(tab.clone(), app));
                    }
                    let enter = gpui::Keystroke::parse("enter").unwrap();
                    assert!(win.dispatch_keystroke(enter, app));
                })
                .expect("activate the saved profile row");
            draw(cx);
            native
                .update(cx, |_, win, app| {
                    win.focus(&root_focus);
                    assert!(win.dispatch_keystroke(
                        gpui::Keystroke::parse(&primary_keybinding("k")).unwrap(),
                        app
                    ));
                })
                .expect("open command palette");
            draw(cx);
            window
                .update(cx, |root, window, cx| {
                    assert!(root.palette_open);
                    root.close_palette(window, cx);
                    assert!(!root.palette_open);
                })
                .expect("verify command palette");
            native
                .update(cx, |_, win, app| {
                    assert!(win.dispatch_keystroke(
                        gpui::Keystroke::parse(&primary_keybinding("b")).unwrap(),
                        app
                    ));
                    assert!(win.dispatch_keystroke(
                        gpui::Keystroke::parse(&primary_keybinding("i")).unwrap(),
                        app
                    ));
                })
                .expect("toggle sidebar and inspector");
            draw(cx);
            window
                .update(cx, |root, _, _| {
                    assert!(root.sidebar_collapsed);
                    assert!(!root.inspector_open);
                    root.sidebar_collapsed = false;
                    root.inspector_open = true;
                })
                .expect("restore sidebar and inspector");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    let session = root.session.read(cx);
                    assert!(session.workspace == Workspace::Charts);
                    assert_eq!(session.natal_id, Some(smoke_profile_id));
                    root.resume_profile_draft(cx);
                    assert!(root.show_profile_editor);
                    assert_eq!(
                        root.draft.as_ref().unwrap().name.read(cx).value(),
                        "Unfinished profile"
                    );
                })
                .expect("verify profile switching preserves the draft");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.navigate(Workspace::Profiles, cx);
                    root.search.update(cx, |input, cx| {
                        input.set_value("__no_such_profile__".into(), cx)
                    });
                })
                .expect("search non-matching profile");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.search
                        .update(cx, |input, cx| input.set_value(String::new(), cx));
                    root.delete_confirmation = Some(smoke_profile_id);
                })
                .expect("open delete confirmation");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.delete_profile(smoke_profile_id, cx);
                    assert!(root.deleted_profile.is_some());
                })
                .expect("delete smoke profile");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.undo_delete(cx);
                    assert_eq!(root.selected, Some(smoke_profile_id));
                    assert!(root.deleted_profile.is_none());
                })
                .expect("undo smoke profile deletion");
            draw(cx);
            println!("Verified profile errors, unknown time, and undo");
            let bad_backup =
                std::env::temp_dir().join(format!("motaa-smoke-bad-{}.json", std::process::id()));
            let good_backup =
                std::env::temp_dir().join(format!("motaa-smoke-good-{}.json", std::process::id()));
            std::fs::write(&bad_backup, b"{not valid json").expect("write bad backup");
            assert!(backup::read(&bad_backup).is_err());
            let _ = std::fs::remove_file(&bad_backup);
            window
                .update(cx, |root, _, cx| {
                    let bytes = backup::encode(root.session.read(cx)).expect("encode backup");
                    std::fs::write(&good_backup, bytes).expect("write good backup");
                    let records = backup::read(&good_backup).expect("read good backup");
                    root.pending_import = Some(records);
                })
                .expect("prepare backup review");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.apply_backup(cx);
                    assert!(root.pending_import.is_none());
                    assert!(
                        root.session
                            .read(cx)
                            .status
                            .contains("Skipped 1 duplicates")
                    );
                    root.delete_profile(smoke_profile_id, cx);
                    root.deleted_profile = None;
                    root.navigate(Workspace::Charts, cx);
                })
                .expect("apply backup review and clean smoke profile");
            let _ = std::fs::remove_file(&good_backup);
            println!("Verified backup review and invalid-file handling");
            for mode in [
                "Rasi",
                "Tricakka",
                "LatLon",
                "Natal + transit",
                "Rasi aspects",
            ] {
                window
                    .update(cx, |root, _, cx| {
                        root.show_munp = true;
                        root.select_mode(mode, cx);
                    })
                    .expect("exercise MUNP mode");
                draw(cx);
            }
            for palette in 0..PALETTES.len() {
                window
                    .update(cx, |root, _, cx| {
                        root.chart_palette = palette;
                        root.select_mode("Rasi", cx);
                    })
                    .expect("exercise chart palette");
                draw(cx);
            }
            window
                .update(cx, |root, _, _| {
                    root.show_munp = false;
                    root.chart_palette = 0;
                })
                .expect("restore default chart flags");
            println!("Verified MUNP modes and chart palettes");
            window
                .update(cx, |root, _, cx| {
                    root.navigate(Workspace::Time, cx);
                    let selected_time = parse_time("2024-07-01 12:00:00").unwrap();
                    root.set_time(selected_time, true, cx);
                    let before = root.session.read(cx).calculate(false).unwrap();
                    root.edit_reference_location(cx);
                    let draft = root.reference_location_draft.as_ref().unwrap();
                    draft
                        .place
                        .update(cx, |input, cx| input.set_value("New York".into(), cx));
                    draft
                        .latitude
                        .update(cx, |input, cx| input.set_value("40 42 46 N".into(), cx));
                    draft
                        .longitude
                        .update(cx, |input, cx| input.set_value("74 0 22 W".into(), cx));
                    let timezone = draft.timezone.clone();
                    timezone.update(cx, |input, cx| input.set_value("Invalid/Zone".into(), cx));
                    root.apply_reference_location(cx);
                    assert!(root.reference_location_draft.is_some());
                    assert!(root.session.read(cx).transit.is_none());
                    timezone.update(cx, |input, cx| {
                        input.set_value("America/New_York".into(), cx)
                    });
                    root.apply_reference_location(cx);
                    assert!(root.reference_location_draft.is_none());
                    let session = root.session.read(cx);
                    assert_eq!(session.reference_profile().birthplace, "New York");
                    assert_eq!(session.time, selected_time);
                    assert_eq!(
                        session
                            .calculate(true)
                            .unwrap()
                            .effective_utc_offset_minutes,
                        -240
                    );
                    assert_eq!(
                        session.calculate(false).unwrap().julian_day_utc,
                        before.julian_day_utc
                    );
                    root.use_birth_location(cx);
                    assert_eq!(
                        root.session.read(cx).reference_profile().birthplace,
                        root.session.read(cx).natal.birthplace
                    );
                })
                .expect("verify native reference location edit");
            println!("Verified independent reference location");
            window
                .update(cx, |root, _, cx| {
                    root.open_draft(ChartRequest::default(), None, cx);
                    root.toggle_osm_search(false, cx);
                    root.toggle_map_picker(false, cx);
                    if let Some(draft) = &mut root.draft {
                        draft.location.tz_open = true;
                        draft
                            .latitude
                            .update(cx, |input, cx| input.set_value("42.2495".into(), cx));
                        draft
                            .longitude
                            .update(cx, |input, cx| input.set_value("-71.0662".into(), cx));
                    }
                    root.autodetect_timezone(false, cx);
                    assert_eq!(
                        root.draft.as_ref().unwrap().timezone.read(cx).value(),
                        "America/New_York"
                    );
                    root.select_timezone_name(false, "Europe/London", cx);
                    assert_eq!(
                        root.draft.as_ref().unwrap().timezone.read(cx).value(),
                        "Europe/London"
                    );
                    assert!(!root.draft.as_ref().unwrap().location.tz_open);
                })
                .expect("exercise location and timezone picker");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.draft = None;
                    root.show_profile_editor = false;
                    root.set_status("", cx);
                })
                .expect("discard smoke profile draft");
            println!("Verified location and timezone picker");
            for mode in MODES {
                window
                    .update(cx, |root, _, cx| root.select_mode(mode, cx))
                    .expect("select chart mode");
                draw(cx);
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(75))
                    .await;
                println!("Exercised chart mode {mode}");
            }
            window
                .update(cx, |root, _, cx| {
                    root.select_mode("Rahu wheel", cx);
                    root.rahu_28 = true;
                    root.rahu_reference = RahuReference::Manual;
                    root.rahu_manual_angle = Some(195.);
                    root.step_rahu_mansion(1, cx);
                    assert_eq!(root.rahu_manual_angle, Some(201.));
                })
                .expect("render 28-division Rahu with manual reference");
            draw(cx);
            cx.background_executor()
                .timer(std::time::Duration::from_millis(125))
                .await;
            window
                .update(cx, |root, _, _| {
                    root.rahu_28 = false;
                    root.rahu_reference = RahuReference::Node;
                })
                .expect("restore Rahu reference");
            println!("Verified Rahu 28-division manual reference");
            for tool in [
                "Ephemeris calendar",
                "Heliocentric model",
                "KP subdivisions",
                "Sky map · 2D",
                "Sky map · 3D",
                "Interpretations",
                "Ten pillars / Ayu Moon",
                "Satapati name wheel",
                "Sarvatobhadra",
                "Strength timeline",
                "Yoga / candidate times",
                "Eclipses",
                "Transit events",
                "Nakshatra groups",
                "Daily Tatkalika",
                "Electional timing",
                "Ashtakavarga",
                "Divisional charts",
            ] {
                window
                    .update(cx, |root, _, cx| {
                        root.update(cx, |session| {
                            session.workspace = Workspace::Advanced;
                            session.tool = Some(tool);
                        })
                    })
                    .expect("select implemented advanced result");
                draw(cx);
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(75))
                    .await;
                println!("Exercised advanced tool {tool}");
            }
            window
                .update(cx, |root, _, cx| {
                    root.satapati_lagna = Some(9);
                    root.kp_horary = Some(1);
                    root.interpretation = Interpretation::Yoga;
                    root.strength_age = Some(10);
                    root.update(cx, |session| {
                        session.workspace = Workspace::Advanced;
                        session.tool = Some("Satapati name wheel");
                    });
                })
                .expect("exercise Satapati Lagna");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.update(cx, |session| session.tool = Some("KP subdivisions"));
                })
                .expect("exercise KP Horary shift");
            draw(cx);
            window
                .update(cx, |root, _, cx| {
                    root.update(cx, |session| session.tool = Some("Interpretations"));
                })
                .expect("exercise Interpretations Yoga");
            draw(cx);
            window
                .update(cx, |root, _, _| {
                    root.satapati_lagna = None;
                    root.kp_horary = None;
                    root.interpretation = Interpretation::Period;
                    root.strength_age = None;
                })
                .expect("reset advanced interactive states");
            println!("Verified premium interactive states");
            window
                .update(cx, |root, _, cx| {
                    root.update(cx, |session| session.tool = Some("Ephemeris calendar"));
                    root.set_time(
                        parse_time("2024-02-01 12:00:00").expect("smoke date"),
                        true,
                        cx,
                    );
                    root.load_ephemeris_month(cx);
                })
                .expect("start monthly ephemeris");
            let mut monthly_checked = false;
            for _ in 0..50 {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;
                monthly_checked = window
                    .update(cx, |root, _, _| match root.ephemeris_month.as_ref() {
                        Some(Ok(month)) => {
                            assert_eq!(month.title, "February 2024");
                            assert_eq!(month.rows.len(), 29);
                            assert_eq!(month.planets.len(), 29);
                            assert_eq!(month.planets[28][0], "2024-02-29");
                            assert!(
                                month.planets[28][1..]
                                    .iter()
                                    .all(|cell| cell.ends_with('°'))
                            );
                            assert_eq!(month.rows[28][0], "2024-02-29");
                            true
                        }
                        Some(Err(error)) => panic!("monthly ephemeris failed: {error}"),
                        None => false,
                    })
                    .expect("check monthly ephemeris");
                if monthly_checked {
                    break;
                }
            }
            assert!(monthly_checked, "monthly ephemeris worker timed out");
            println!("Verified leap-month ephemeris");
            window
                .update(cx, |root, _, cx| {
                    root.set_time(
                        parse_time("2024-03-15 12:00:00").expect("smoke date"),
                        true,
                        cx,
                    );
                    root.update(cx, |session| {
                        session.tool = Some("Transit events");
                        session.settings.tropical = true;
                    });
                    root.search_transits(cx);
                    root.update(cx, |session| session.settings.tropical = false);
                    root.load_strength_timeline(cx);
                })
                .expect("start native analysis workers");
            let mut analyses_checked = false;
            for _ in 0..80 {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;
                analyses_checked = window
                    .update(cx, |root, _, _| {
                        match (&root.transit_report, &root.strength_report) {
                            (Some(Ok(transits)), Some(Ok(strength))) => {
                                assert!(
                                    transits
                                        .events
                                        .iter()
                                        .any(|event| event.body == "Sun" && event.sign == 0)
                                );
                                assert_eq!(strength.samples.len(), 96);
                                true
                            }
                            (Some(Err(error)), _) | (_, Some(Err(error))) => {
                                panic!("analysis failed: {error}")
                            }
                            _ => false,
                        }
                    })
                    .expect("verify analysis workers");
                if analyses_checked {
                    break;
                }
            }
            assert!(analyses_checked, "analysis workers timed out");
            println!("Verified transit and strength workers");
            window
                .update(cx, |root, window, cx| {
                    root.select_mode("Circular chart", cx);
                    root.chart_zoom = 1.5;
                    window.resize(size(px(1024.), px(680.)));
                })
                .expect("render compact zoomed chart");
            cx.background_executor()
                .timer(std::time::Duration::from_millis(250))
                .await;
            window
                .update(cx, |root, window, cx| {
                    let viewport = window.viewport_size();
                    assert!(f32::from(viewport.width) <= 1025.);
                    assert!(f32::from(viewport.height) >= 680.);
                    assert_eq!(root.session.read(cx).mode, "Circular chart");
                })
                .expect("verify compact chart");
            println!("Verified compact chart resize");
            window
                .update(cx, |root, _, cx| root.toggle_circular_detail(cx))
                .expect("open circular chart detail");
            cx.background_executor()
                .timer(std::time::Duration::from_millis(125))
                .await;
            window
                .update(cx, |root, _, cx| {
                    assert!(root.circular_detail);
                    assert_eq!(root.chart_zoom, 1.);
                    root.toggle_circular_detail(cx);
                    assert!(!root.circular_detail);
                    root.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
                    assert_eq!(root.chart_lightbox, Some(LightboxTarget::ActiveChart));
                    root.set_lightbox_zoom(2.25, cx);
                    assert_eq!(root.lightbox_zoom, 2.25);
                    root.close_chart_lightbox(cx);
                    assert!(root.chart_lightbox.is_none());
                    assert_eq!(root.lightbox_zoom, 1.);
                })
                .expect("close circular chart detail");
            println!("Verified circular chart detail navigation");

            for (mode, target) in [
                ("Tricakka", LightboxTarget::TricakkaSubchart(0)),
                ("Tricakka", LightboxTarget::TricakkaSubchart(1)),
                ("Tricakka", LightboxTarget::TricakkaSubchart(2)),
                ("Natal + transit", LightboxTarget::ComparisonCombined),
                ("Natal + transit", LightboxTarget::ComparisonSplit(1)),
                ("Western aspects", LightboxTarget::RasiInset),
                ("Rasi aspects", LightboxTarget::RasiInset),
            ] {
                window
                    .update(cx, |root, _, cx| {
                        root.select_mode(mode, cx);
                        root.open_chart_lightbox(target, cx);
                    })
                    .expect("open chart detail");
                draw(cx);
                window
                    .update(cx, |root, _, cx| {
                        root.close_chart_lightbox(cx);
                    })
                    .expect("close chart detail");
            }
            println!("Rendered compact Tricakka, comparison, and aspect details");
            cx.update(|cx| cx.quit()).expect("quit smoke window");
        })
        .detach();
    } else if let Some(capture_dir) = store_screenshots {
        cx.spawn(async move |cx| {
            let executor = cx.background_executor().clone();
            let wait = |ms| executor.timer(std::time::Duration::from_millis(ms));
            let wait_for_ack = |slug: &str| {
                let done_path = capture_dir.join(format!("{slug}.done"));
                let executor = executor.clone();
                async move {
                    for _ in 0..150 {
                        if done_path.exists() {
                            break;
                        }
                        executor.timer(std::time::Duration::from_millis(100)).await;
                    }
                }
            };
            let emit = |name: &str| {
                println!("STORE_CAPTURE_READY {name}");
                let _ = std::io::stdout().flush();
            };
            wait(1200).await;
            let chart_shots: &[(&str, &str, bool)] = &[
                ("01-rasi-chart", "Rasi", true),
                ("02-tricakka-chart", "Tricakka", true),
                ("03-dasha-periods", "Dasha periods", false),
            ];
            for &(slug, mode, munp) in chart_shots {
                window
                    .update(cx, |root, window, cx| {
                        root.navigate(Workspace::Charts, cx);
                        root.inspector_open = true;
                        root.sidebar_collapsed = false;
                        root.show_munp = munp;
                        root.select_mode(mode, cx);
                        window.refresh();
                    })
                    .expect("switch store screenshot chart mode");
                wait(600).await;
                emit(slug);
                wait_for_ack(slug).await;
            }
            let workspace_shots: &[(&str, Workspace)] = &[
                ("04-time-analysis", Workspace::Time),
                ("05-profiles", Workspace::Profiles),
                ("06-advanced-tools", Workspace::Advanced),
            ];
            for &(slug, route) in workspace_shots {
                window
                    .update(cx, |root, window, cx| {
                        root.navigate(route, cx);
                        root.inspector_open = true;
                        root.sidebar_collapsed = false;
                        window.refresh();
                    })
                    .expect("switch store screenshot workspace");
                wait(600).await;
                emit(slug);
                wait_for_ack(slug).await;
            }
            cx.update(|cx| cx.quit()).expect("quit store screenshots");
        })
        .detach();
    } else if visual_capture {
        cx.spawn(async move |cx| {
            let executor = cx.background_executor().clone();
            let wait = |ms| executor.timer(std::time::Duration::from_millis(ms));
            let emit = |name: &str| {
                println!("CAPTURE_READY {name}");
                let _ = std::io::stdout().flush();
            };
            wait(900).await;
            emit("01-circular-chart");
            wait(700).await;
            let modes: &[(&str, &str, bool)] = &[
                ("02-rasi-munp", "Rasi", true),
                ("03-tricakka", "Tricakka", true),
                ("04-nakshatra-traversal", "Nakshatra traversal", false),
                ("05-latlon", "LatLon", true),
                ("06-western-aspects", "Western aspects", false),
                ("07-natal-transit", "Natal + transit", true),
                ("08-dasha-periods", "Dasha periods", false),
                ("09-rahu-wheel", "Rahu wheel", false),
                ("10-rasi-aspects", "Rasi aspects", true),
            ];
            for &(slug, mode, munp) in modes {
                window
                    .update(cx, |root, window, cx| {
                        root.show_munp = munp;
                        root.all_strengths = mode == "Rasi aspects";
                        root.select_mode(mode, cx);
                        window.refresh();
                    })
                    .expect("switch capture mode");
                wait(350).await;
                emit(slug);
                wait(700).await;
            }
            for (slug, route) in [
                ("11-time-analysis", Workspace::Time),
                ("12-profiles", Workspace::Profiles),
                ("13-calculations", Workspace::Calculations),
                ("14-advanced", Workspace::Advanced),
            ] {
                window
                    .update(cx, |root, window, cx| {
                        root.navigate(route, cx);
                        window.refresh();
                    })
                    .expect("switch capture workspace");
                wait(350).await;
                emit(slug);
                wait(700).await;
            }
            window
                .update(cx, |root, window, cx| {
                    root.navigate(Workspace::Charts, cx);
                    root.select_mode("Circular chart", cx);
                    window.resize(size(px(1024.), px(680.)));
                    window.refresh();
                })
                .expect("resize capture window");
            wait(450).await;
            emit("15-compact-1024x680");
            wait(700).await;
            window
                .update(cx, |root, window, cx| {
                    root.select_mode("Rahu wheel", cx);
                    root.rahu_reference = RahuReference::Moon;
                    window.refresh();
                })
                .expect("capture compact Rahu with Moon reference");
            wait(350).await;
            emit("16-rahu-compact-moon");
            wait(700).await;
            window
                .update(cx, |root, window, cx| {
                    root.rahu_28 = true;
                    root.rahu_reference = RahuReference::Manual;
                    root.step_rahu_mansion(1, cx);
                    root.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
                    window.refresh();
                })
                .expect("capture manual Rahu lightbox");
            wait(350).await;
            emit("17-rahu-manual-lightbox");
            wait(700).await;
            for (slug, mode, target) in [
                ("18-compact-tricakka", "Tricakka", None),
                (
                    "19-compact-bhava-detail",
                    "Tricakka",
                    Some(LightboxTarget::TricakkaSubchart(1)),
                ),
                ("20-compact-traversal", "Nakshatra traversal", None),
                ("21-compact-split-transit", "Natal + transit", None),
                ("22-compact-combined-transit", "Natal + transit", None),
            ] {
                window
                    .update(cx, |root, window, cx| {
                        root.select_mode(mode, cx);
                        root.show_munp = true;
                        root.split_comparison = slug == "21-compact-split-transit";
                        if let Some(target) = target {
                            root.open_chart_lightbox(target, cx);
                        }
                        window.refresh();
                    })
                    .expect("switch compact chart capture");
                wait(350).await;
                emit(slug);
                wait(700).await;
            }
            cx.update(|cx| cx.quit()).expect("quit visual capture");
        })
        .detach();
    }
}
