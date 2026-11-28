use super::*;

#[derive(Clone, Copy)]
enum ProfileAction {
    OpenChart,
    Edit,
    Duplicate,
    UseForTransit,
}

struct LocationFields<'a> {
    place: &'a Entity<TextInput>,
    latitude: &'a Entity<TextInput>,
    longitude: &'a Entity<TextInput>,
    timezone: &'a Entity<TextInput>,
    search: &'a mut Option<Entity<TextInput>>,
    location: &'a mut location_picker::LocationPickerState,
}

impl DesktopRoot {
    pub(super) fn open_saved_profile(
        &mut self,
        id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.choose_profile(id, ProfileAction::OpenChart, window, cx);
    }

    pub(super) fn profiles_view(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.show_profile_editor
            && let Some(draft) = &self.draft
        {
            let effective_dt = parse_time(draft.datetime.read(cx).value().as_ref())
                .unwrap_or_else(|_| self.session.read(cx).time);
            return column()
                .child(heading(if draft.id.is_some() {
                    "Edit birth profile"
                } else {
                    "New birth profile"
                }))
                .child(
                    panel()
                        .child(field("Name", draft.name.clone()))
                        .child(draft.picker.render(
                            "Birth date and time",
                            draft.unknown,
                            Some(|view, _| {
                                if let Some(draft) = &mut view.draft {
                                    draft.unknown = !draft.unknown;
                                }
                            }),
                            None,
                            |view| view.draft.as_mut().map(|d| &mut d.picker),
                            cx,
                        ))
                        .child(self.location_and_timezone_section(false, effective_dt, cx))
                        .child(
                            row()
                                .child(button(
                                    "Save profile",
                                    true,
                                    cx,
                                    DesktopRoot::save_draft as Callback,
                                ))
                                .child(button("Discard draft", false, cx, |view, cx| {
                                    view.draft = None;
                                    view.show_profile_editor = false;
                                    view.set_status("Draft discarded.", cx);
                                })),
                        ),
                )
                .into_any_element();
        }
        let s = self.session.read(cx);
        let natal_id = s.natal_id;
        let natal = s.natal.name.clone();
        let transit = s
            .transit
            .as_ref()
            .map_or("Current reference time", |profile| &profile.name)
            .to_owned();
        let search = self.search.read(cx).value().trim().to_lowercase();
        let entries = s.profiles();
        let mut view = column()
            .gap_4()
            .child(heading("Saved profiles"))
            .child(
                panel()
                    .py_3()
                    .px_4()
                    .gap_2()
                    .child(section_label("ACTIVE CHART CONTEXT"))
                    .child(
                        row()
                            .gap_8()
                            .child(
                                row().gap_2().child(caption("Natal")).child(
                                    div()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(GOLD))
                                        .child(natal),
                                ),
                            )
                            .child(
                                row().gap_2().child(caption("Transit")).child(
                                    div()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(rgb(INK))
                                        .child(transit),
                                ),
                            ),
                    ),
            )
            .child(
                row()
                    .w_full()
                    .justify_between()
                    .flex_wrap()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(220.))
                            .max_w(px(360.))
                            .child(render_input(&self.search)),
                    )
                    .child(
                        row()
                            .gap_2()
                            .child(compact_button(
                                "profiles-export",
                                "Export JSON",
                                false,
                                cx,
                                DesktopRoot::export_backup,
                            ))
                            .when(
                                !self.reading_backup && self.pending_import.is_none(),
                                |bar| {
                                    bar.child(compact_button(
                                        "profiles-import",
                                        "Import backup",
                                        false,
                                        cx,
                                        DesktopRoot::inspect_backup,
                                    ))
                                },
                            )
                            .child(if self.draft.is_some() {
                                compact_button(
                                    "profiles-resume-draft",
                                    "Resume draft",
                                    true,
                                    cx,
                                    DesktopRoot::resume_profile_draft,
                                )
                            } else {
                                compact_button_in(
                                    "profiles-new",
                                    "+ New profile",
                                    false,
                                    cx,
                                    |view, window, cx| {
                                        view.open_draft(ChartRequest::default(), None, window, cx)
                                    },
                                )
                            }),
                    ),
            );
        if let Some(pending) = &self.pending_import {
            view = view.child(
                panel()
                    .child(format!("Review backup · {} profiles", pending.len()))
                    .child(caption(
                        "Existing profiles keep their chart data. Import can restore missing reference locations.",
                    ))
                    .child(
                        row()
                            .child(button(
                                "Import profiles",
                                true,
                                cx,
                                DesktopRoot::apply_backup,
                            ))
                            .child(button("Cancel import", false, cx, |view, _| {
                                view.pending_import = None
                            })),
                    ),
            );
        }
        match entries {
            Ok(profiles) => {
                let matches: Vec<_> = profiles
                    .into_iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&search)
                            || p.birthplace.to_lowercase().contains(&search)
                    })
                    .collect();
                let count = matches.len();
                let is_empty = matches.is_empty();
                view = view.child(
                    column()
                        .gap_2()
                        .when(!is_empty, |col| {
                            col.child(row().justify_between().child(caption(format!(
                                "{count} {}",
                                if count == 1 { "profile" } else { "profiles" }
                            ))))
                            .child(
                                panel()
                                    .p_0()
                                    .gap_0()
                                    .overflow_hidden()
                                    .child(
                                        row()
                                            .w_full()
                                            .bg(rgb(SOFT))
                                            .border_b_1()
                                            .border_color(rgb(LINE))
                                            .child(
                                                table_header_row([
                                                    "Name",
                                                    "Birth date",
                                                    "Birthplace",
                                                    "Calculation",
                                                ])
                                                .border_b_0()
                                                .flex_1()
                                                .min_w(px(0.)),
                                            )
                                            .child(div().w(px(48.)).flex_shrink_0()),
                                    )
                                    .children(matches.into_iter().map(|profile| {
                                        let id = profile.id;
                                        let menu_open = self.selected == Some(id);
                                        let active = natal_id == Some(id);
                                        let name_for_menu = profile.name.clone();
                                        let date = if self.show_birth {
                                            serde_json::from_str::<ChartRequest>(&profile.data_json)
                                                .ok()
                                                .map_or_else(
                                                    || "Unavailable".into(),
                                                    |request| {
                                                        format!(
                                                            "{:04}-{:02}-{:02}",
                                                            request.year,
                                                            request.month,
                                                            request.day
                                                        )
                                                    },
                                                )
                                        } else {
                                            "Hidden".into()
                                        };
                                        let calculation =
                                            self.session.read(cx).profile_settings(id).map_or_else(
                                                |_| "Unavailable".into(),
                                                |settings| {
                                                    format!(
                                                        "{} · {}",
                                                        if settings.tropical {
                                                            "Tropical"
                                                        } else {
                                                            "Sidereal"
                                                        },
                                                        AYANAMSHAS[settings.ayanamsha]
                                                    )
                                                },
                                            );
                                        row()
                                            .w_full()
                                            .gap_0()
                                            .border_b_1()
                                            .border_color(rgb(SUBTLE_LINE))
                                            .bg(rgb(if active { SOFT } else { PAPER }))
                                            .hover(|item| {
                                                item.bg(rgb(if active {
                                                    SELECTED
                                                } else {
                                                    HEADER_BG
                                                }))
                                            })
                                            .child(
                                                action_control_in(
                                                    ("profile-row", id as usize),
                                                    cx,
                                                    move |view, window, cx| {
                                                        view.selected = None;
                                                        view.delete_confirmation = None;
                                                        view.choose_profile(
                                                            id,
                                                            ProfileAction::OpenChart,
                                                            window,
                                                            cx,
                                                        );
                                                    },
                                                )
                                                .flex_1()
                                                .min_w(px(0.))
                                                .px_2()
                                                .py(px(10.))
                                                .gap_3()
                                                .child(
                                                    row()
                                                        .flex_1()
                                                        .min_w(px(0.))
                                                        .gap_2()
                                                        .overflow_hidden()
                                                        .child(
                                                            div()
                                                                .font_weight(if active {
                                                                    gpui::FontWeight::SEMIBOLD
                                                                } else {
                                                                    gpui::FontWeight::MEDIUM
                                                                })
                                                                .text_color(rgb(INK))
                                                                .child(profile.name),
                                                        )
                                                        .when(active, |name_cell| {
                                                            name_cell
                                                                .child(badge("Active", Tone::Gold))
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.))
                                                        .overflow_hidden()
                                                        .font_family("Menlo")
                                                        .text_size(px(12.))
                                                        .text_color(rgb(MUTED))
                                                        .child(date),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.))
                                                        .overflow_hidden()
                                                        .text_size(px(13.))
                                                        .text_color(rgb(INK))
                                                        .child(profile.birthplace),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w(px(0.))
                                                        .overflow_hidden()
                                                        .text_size(px(12.5))
                                                        .text_color(rgb(MUTED))
                                                        .child(calculation),
                                                ),
                                            )
                                            .child(
                                                div()
                                                    .id(("profile-management", id as usize))
                                                    .relative()
                                                    .w(px(48.))
                                                    .flex_shrink_0()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .on_mouse_down(
                                                        gpui::MouseButton::Left,
                                                        |_, _, cx| cx.stop_propagation(),
                                                    )
                                                    .child(
                                                        action_control(
                                                            ("profile-more-btn", id as usize),
                                                            cx,
                                                            move |view, _| {
                                                                view.selected = (view.selected
                                                                    != Some(id))
                                                                .then_some(id);
                                                                view.delete_confirmation = None;
                                                            },
                                                        )
                                                        .w(px(28.))
                                                        .h(px(26.))
                                                        .justify_center()
                                                        .rounded(px(6.))
                                                        .text_size(px(15.))
                                                        .text_color(rgb(if menu_open {
                                                            GOLD
                                                        } else {
                                                            MUTED
                                                        }))
                                                        .bg(if menu_open {
                                                            rgb(SELECTED).into()
                                                        } else {
                                                            gpui::transparent_black()
                                                        })
                                                        .hover(|s| {
                                                            s.bg(rgb(SELECTED)).text_color(rgb(INK))
                                                        })
                                                        .child(ui_icon(
                                                            "ui/ellipsis",
                                                            15.,
                                                            if menu_open { GOLD } else { MUTED },
                                                        )),
                                                    )
                                                    .when(menu_open, |anchor| {
                                                        anchor.child(self.profile_actions_popover(
                                                            id,
                                                            &name_for_menu,
                                                            cx,
                                                        ))
                                                    }),
                                            )
                                    })),
                            )
                        })
                        .when(is_empty, |v| {
                            v.child(empty_state(
                                if search.is_empty() {
                                    "No saved profiles yet"
                                } else {
                                    "No matching profiles"
                                },
                                if search.is_empty() {
                                    "Create a birth profile or import a JSON/Android backup."
                                } else {
                                    "Try another name or birthplace query."
                                },
                                search.is_empty().then(|| {
                                    button_in("New profile", true, cx, |view, window, cx| {
                                        view.open_draft(ChartRequest::default(), None, window, cx)
                                    })
                                }),
                            ))
                        }),
                );
            }
            Err(error) => view = view.child(panel().child(error)),
        }
        if self.deleted_profile.is_some() {
            view = view.child(row().child(compact_button(
                "profiles-undo-delete",
                "Undo deletion",
                false,
                cx,
                DesktopRoot::undo_delete,
            )));
        }
        view.into_any_element()
    }

    fn profile_actions_popover(&self, id: i64, name: &str, cx: &mut Context<Self>) -> AnyElement {
        let confirming = self.delete_confirmation == Some(id);
        let title = format!("Delete {name}?");
        anchored_popover(
            ("profile-actions-popover", id as usize),
            gpui::Anchor::TopRight,
            204.,
            |menu| {
                menu.when(confirming, |menu| {
                    menu.p_2()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(12.5))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(INK))
                                .child(title.clone()),
                        )
                        .child(caption("You can undo right after deleting."))
                        .child(
                            row()
                                .justify_end()
                                .gap_2()
                                .child(compact_button(
                                    ("cancel-delete", id as usize),
                                    "Cancel",
                                    false,
                                    cx,
                                    |view, _| {
                                        view.delete_confirmation = None;
                                    },
                                ))
                                .child(
                                    action_control(
                                        ("confirm-delete", id as usize),
                                        cx,
                                        move |view, cx| {
                                            view.selected = None;
                                            view.delete_profile(id, cx);
                                        },
                                    )
                                    .px(px(10.))
                                    .h(px(26.))
                                    .justify_center()
                                    .rounded(px(RADIUS_CONTROL))
                                    .text_size(px(12.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .bg(rgb(ERROR))
                                    .text_color(rgb(PAPER))
                                    .hover(|s| s.opacity(0.9))
                                    .child("Delete"),
                                ),
                        )
                })
                .when(!confirming, |menu| {
                    menu.child(menu_item_in(
                        ("profile-open", id as usize),
                        "Open chart",
                        false,
                        cx,
                        move |view, window, cx| {
                            view.selected = None;
                            view.choose_profile(id, ProfileAction::OpenChart, window, cx);
                        },
                    ))
                    .child(menu_item_in(
                        ("profile-transit", id as usize),
                        "Use for transit",
                        false,
                        cx,
                        move |view, window, cx| {
                            view.selected = None;
                            view.choose_profile(id, ProfileAction::UseForTransit, window, cx);
                        },
                    ))
                    .child(menu_divider())
                    .child(menu_item_in(
                        ("profile-edit", id as usize),
                        "Edit profile",
                        false,
                        cx,
                        move |view, window, cx| {
                            view.selected = None;
                            view.choose_profile(id, ProfileAction::Edit, window, cx);
                        },
                    ))
                    .child(menu_item_in(
                        ("profile-dup", id as usize),
                        "Duplicate",
                        false,
                        cx,
                        move |view, window, cx| {
                            view.selected = None;
                            view.choose_profile(id, ProfileAction::Duplicate, window, cx);
                        },
                    ))
                    .child(menu_divider())
                    .child(menu_item_danger(
                        ("profile-del", id as usize),
                        "Delete profile",
                        cx,
                        move |view, _| {
                            view.delete_confirmation = Some(id);
                        },
                    ))
                })
            },
        )
    }

    fn choose_profile(
        &mut self,
        id: i64,
        action: ProfileAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let result = (|| {
            let session = self.session.read(cx);
            let profile = session
                .profiles()?
                .into_iter()
                .find(|profile| profile.id == id)
                .ok_or("Profile unavailable.")?;
            let mut request: ChartRequest =
                serde_json::from_str(&profile.data_json).map_err(|error| error.to_string())?;
            match action {
                ProfileAction::Edit => self.open_draft(request, Some(id), window, cx),
                ProfileAction::Duplicate => {
                    if self.draft.is_some() {
                        self.navigate(Workspace::Profiles, cx);
                        self.set_status("Save or discard the open draft first.", cx);
                    } else {
                        request.name.push_str(" — copy");
                        self.open_draft(request, None, window, cx);
                        if let Some(draft) = &mut self.draft {
                            draft.copy_from = Some(id);
                        }
                    }
                }
                ProfileAction::UseForTransit => self.update(cx, |s| {
                    s.transit = Some(request);
                    s.reference_source = Some(model::ReferenceSource::SelectedProfile(id));
                    s.status = "Transit location set.".into();
                }),
                ProfileAction::OpenChart => {
                    let settings = session.profile_settings(id)?;
                    let reference = session.profile_reference_location(id)?;
                    self.navigate(Workspace::Charts, cx);
                    self.update(cx, |s| {
                        s.activate_natal(id, request, reference);
                        s.settings = settings;
                        s.settings_draft = settings;
                    });
                }
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            self.set_status(error, cx);
        }
    }

    pub(super) fn active_town_popup(&self) -> Option<bool> {
        if self.draft.as_ref().is_some_and(|d| d.town_search.is_some()) {
            Some(false)
        } else if self
            .reference_location_draft
            .as_ref()
            .is_some_and(|d| d.town_search.is_some())
        {
            Some(true)
        } else {
            None
        }
    }

    pub(super) fn close_town_popup(&mut self, reference: bool) {
        if reference {
            if let Some(d) = &mut self.reference_location_draft {
                d.town_search = None;
            }
        } else if let Some(d) = &mut self.draft {
            d.town_search = None;
        }
    }

    pub(super) fn toggle_town_picker(
        &mut self,
        reference: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_town_popup(!reference);
        let field = if reference {
            self.reference_location_draft
                .as_mut()
                .map(|draft| &mut draft.town_search)
        } else {
            self.draft.as_mut().map(|draft| &mut draft.town_search)
        };
        if let Some(field) = field {
            *field = if field.is_some() {
                None
            } else {
                let input = text_input("", "Town name in Myanmar script", window, cx);
                subscribe_input(&input, window, cx, |_, _, cx| cx.notify());
                self.town_scroll.set_offset(gpui::point(px(0.), px(0.)));
                Some(input)
            };
        }
    }

    pub(super) fn town_picker_overlay(
        &self,
        reference: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        modal_dialog(
            "town-backdrop",
            "town-dialog",
            580.,
            None,
            "Myanmar Town Directory",
            "534-town offline directory",
            "Close",
            self.town_picker_panel(reference, cx).into_any_element(),
            cx,
            move |root, _| root.close_town_popup(reference),
        )
    }

    pub(super) fn town_picker_panel(&self, reference: bool, cx: &mut Context<Self>) -> Div {
        let search = if reference {
            self.reference_location_draft
                .as_ref()
                .and_then(|d| d.town_search.as_ref())
        } else {
            self.draft.as_ref().and_then(|d| d.town_search.as_ref())
        };
        let Some(search) = search else {
            return panel();
        };
        let query = search.read(cx).value().trim().to_owned();
        let matching: Vec<(usize, &crate::Town)> = self
            .towns
            .towns
            .iter()
            .enumerate()
            .filter(|(_, town)| town.name_my.contains(&query))
            .collect();
        let count = matching.len();
        let total = self.towns.towns.len();
        panel()
            .bg(rgb(SOFT))
            .gap_2()
            .child(field("Search Myanmar towns", search.clone()))
            .child(caption(format!("{count} of {total} towns")))
            .when(matching.is_empty(), |panel| {
                panel.child(caption("No matching towns."))
            })
            .when(!matching.is_empty(), |panel| {
                use gpui::component::scroll::ScrollableElement as _;
                panel.child(
                    div()
                        .relative()
                        .w_full()
                        .h(px(400.))
                        .child(
                            div()
                                .id("town-list-scroll")
                                .w_full()
                                .h(px(400.))
                                .overflow_y_scroll()
                                .track_scroll(&self.town_scroll)
                                .on_scroll(cx.listener(|_, _, _, cx| cx.notify()))
                                .child(column().gap_1().children(matching.into_iter().map(
                                    |(index, town)| {
                                        let coords = format!(
                                            "{:.2}°, {:.2}°",
                                            town.latitude, town.longitude
                                        );
                                        action_control_in(
                                            ("town", index),
                                            cx,
                                            move |view, window, cx| {
                                                view.select_town(index, reference, window, cx);
                                            },
                                        )
                                        .w_full()
                                        .px_3()
                                        .py(px(6.))
                                        .rounded(px(RADIUS_FIELD))
                                        .border_1()
                                        .border_color(rgb(SUBTLE_LINE))
                                        .bg(rgb(PAPER))
                                        .hover(|s| s.bg(rgb(SELECTED)))
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(rgb(INK))
                                                .child(town.name_my.clone()),
                                        )
                                        .child(
                                            div()
                                                .font_family("Menlo")
                                                .text_size(px(11.5))
                                                .text_color(rgb(MUTED))
                                                .child(coords),
                                        )
                                    },
                                ))),
                        )
                        .vertical_scrollbar(&self.town_scroll),
                )
            })
    }

    fn select_town(
        &mut self,
        index: usize,
        reference: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(town) = self.towns.towns.get(index) else {
            return;
        };
        let (name_my, lat, lon, raw_lat, raw_lon) = (
            town.name_my.clone(),
            town.latitude.to_string(),
            town.longitude.to_string(),
            town.latitude,
            town.longitude,
        );
        let Some(fields) = self.location_fields_mut(reference) else {
            return;
        };
        for (input, value) in [
            (fields.place, name_my),
            (fields.latitude, lat),
            (fields.longitude, lon),
            (fields.timezone, "Asia/Yangon".into()),
        ] {
            input.update(cx, |field, cx| field.set_value(value, window, cx));
        }
        fields.location.map_center_lat = raw_lat;
        fields.location.map_center_lon = raw_lon;
        *fields.search = None;
    }

    fn location_fields_mut(&mut self, reference: bool) -> Option<LocationFields<'_>> {
        if reference {
            self.reference_location_draft
                .as_mut()
                .map(|d| LocationFields {
                    place: &d.place,
                    latitude: &d.latitude,
                    longitude: &d.longitude,
                    timezone: &d.timezone,
                    search: &mut d.town_search,
                    location: &mut d.location,
                })
        } else {
            self.draft.as_mut().map(|d| LocationFields {
                place: &d.place,
                latitude: &d.latitude,
                longitude: &d.longitude,
                timezone: &d.timezone,
                search: &mut d.town_search,
                location: &mut d.location,
            })
        }
    }

    pub(super) fn delete_profile(&mut self, id: i64, cx: &mut Context<Self>) {
        self.delete_confirmation = None;
        if self
            .draft
            .as_ref()
            .is_some_and(|draft| draft.id == Some(id) || draft.copy_from == Some(id))
        {
            self.set_status(
                "Save or discard the draft before deleting this profile.",
                cx,
            );
            return;
        }
        let result = (|| {
            let session = self.session.read(cx);
            let existing = session
                .profiles()?
                .into_iter()
                .find(|profile| profile.id == id)
                .ok_or("Profile unavailable.")?;
            if !motaa_store::delete(session.connection()?, id)? {
                return Err("Profile was already removed.".into());
            }
            Ok::<_, String>(existing)
        })();
        match result {
            Ok(profile) => {
                self.selected = None;
                self.deleted_profile = Some(profile);
                self.update(cx, |s| {
                    if s.natal_id == Some(id) {
                        s.natal = ChartRequest {
                            name: "Anonymous sample".into(),
                            ..ChartRequest::default()
                        };
                        s.natal_id = None;
                        s.settings = Settings::default();
                        s.settings_draft = s.settings;
                    }
                    if s.reference_source == Some(model::ReferenceSource::SelectedProfile(id))
                        || s.reference_source == Some(model::ReferenceSource::NatalProfile(id))
                    {
                        s.clear_reference_location();
                    }
                    s.status = "Profile deleted.".into();
                });
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    pub(super) fn undo_delete(&mut self, cx: &mut Context<Self>) {
        let Some(profile) = &self.deleted_profile else {
            return;
        };
        let result = self
            .session
            .read(cx)
            .connection()
            .and_then(|db| motaa_store::restore_deleted(db, profile));
        match result {
            Ok(()) => {
                self.selected = Some(profile.id);
                self.deleted_profile = None;
                self.set_status("Profile restored.", cx);
            }
            Err(error) => self.set_status(error, cx),
        }
    }

    fn export_backup(&mut self, cx: &mut Context<Self>) {
        let result = backup::encode(self.session.read(cx));
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(error) => return self.set_status(error, cx),
        };
        let directory = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let picker = cx.prompt_for_new_path(&directory, Some("motaa-profiles.json"));
        cx.spawn(async move |view, cx| {
            let message = match picker.await {
                Ok(Ok(Some(path))) => {
                    cx.background_executor()
                        .spawn(async move {
                            match std::fs::OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(&path)
                            {
                                Ok(mut file) => {
                                    match file.write_all(&bytes).and_then(|_| file.sync_all()) {
                                        Ok(()) => format!("Backup exported to {}", path.display()),
                                        Err(error) => {
                                            drop(file);
                                            let _ = std::fs::remove_file(&path);
                                            format!("Could not write backup: {error}")
                                        }
                                    }
                                }
                                Err(error) => format!("Could not create backup: {error}"),
                            }
                        })
                        .await
                }
                Ok(Ok(None)) => return,
                Ok(Err(error)) => error.to_string(),
                Err(error) => error.to_string(),
            };
            let _ = view.update(cx, |view, cx| view.set_status(message, cx));
        })
        .detach();
    }

    fn inspect_backup(&mut self, cx: &mut Context<Self>) {
        if self.reading_backup || self.pending_import.is_some() {
            return;
        }
        self.reading_backup = true;
        self.set_status("Choose a backup to import.", cx);
        let picker = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Choose a MOTAA backup or Android profile database".into()),
        });
        cx.spawn(async move |view, cx| {
            let result = match picker.await {
                Ok(Ok(paths)) => match paths.and_then(|paths| paths.into_iter().next()) {
                    Some(path) => {
                        let _ = view.update(cx, |view, cx| view.set_status("Reading backup…", cx));
                        cx.background_executor()
                            .spawn(async move { backup::read(&path) })
                            .await
                            .map(Some)
                    }
                    None => Ok(None),
                },
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = view.update(cx, |view, cx| {
                view.reading_backup = false;
                match result {
                    Ok(Some(profiles)) if profiles.is_empty() => {
                        view.set_status("This backup contains no profiles.", cx);
                    }
                    Ok(Some(profiles)) => {
                        view.pending_import = Some(profiles);
                        view.set_status("Backup ready for review.", cx);
                    }
                    Ok(None) => view.set_status("", cx),
                    Err(error) => view.set_status(error, cx),
                }
            });
        })
        .detach();
    }

    pub(super) fn apply_backup(&mut self, cx: &mut Context<Self>) {
        let Some(records) = &self.pending_import else {
            return;
        };
        let result = backup::restore(self.session.read(cx), records);
        match result {
            Ok((added, skipped)) => {
                self.pending_import = None;
                self.set_status(
                    format!("Imported {added} profiles. Skipped {skipped} duplicates."),
                    cx,
                );
            }
            Err(error) => self.set_status(error, cx),
        }
    }
}
