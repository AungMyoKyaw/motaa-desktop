use super::*;
use gpui::Focusable;

#[derive(Clone)]
pub(super) enum PaletteAction {
    Mode(&'static str),
    Tool(&'static str),
    Profile(i64),
    Workspace(Workspace),
    NewBirthChart,
    JumpNow,
    ToggleSidebar,
    ToggleInspector,
    ToggleBirthDetails,
    OpenFullView,
}

#[derive(Clone)]
pub(super) struct PaletteEntry {
    pub(super) category: &'static str,
    pub(super) label: String,
    pub(super) hint: String,
    pub(super) action: PaletteAction,
}

impl DesktopRoot {
    pub(super) fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = true;
        self.palette_index = 0;
        self.palette_query
            .update(cx, |input, cx| input.set_value(String::new(), cx));
        let focus = self.palette_query.read(cx).focus_handle(cx);
        window.focus(&focus);
        cx.notify();
    }

    pub(super) fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = false;
        self.palette_index = 0;
        window.focus(&self.focus);
        cx.notify();
    }

    pub(super) fn palette_entries(&self, cx: &App) -> Vec<PaletteEntry> {
        let query = self.palette_query.read(cx).value().trim().to_lowercase();
        let mut entries = Vec::new();

        for mode in MODES {
            entries.push(PaletteEntry {
                category: "Chart Mode",
                label: mode.to_owned(),
                hint: "Charts".into(),
                action: PaletteAction::Mode(mode),
            });
        }

        for row in TOOLS {
            for tool in row {
                entries.push(PaletteEntry {
                    category: "Advanced Tool",
                    label: tool.to_owned(),
                    hint: "Advanced".into(),
                    action: PaletteAction::Tool(tool),
                });
            }
        }
        for sky in ["Sky map · 2D", "Sky map · 3D"] {
            entries.push(PaletteEntry {
                category: "Advanced Tool",
                label: sky.to_owned(),
                hint: "Advanced".into(),
                action: PaletteAction::Tool(sky),
            });
        }

        if let Ok(profiles) = self.session.read(cx).profiles() {
            for profile in profiles {
                let place = if profile.birthplace.is_empty() {
                    "Saved profile".to_owned()
                } else {
                    profile.birthplace
                };
                entries.push(PaletteEntry {
                    category: "Profile",
                    label: profile.name,
                    hint: place,
                    action: PaletteAction::Profile(profile.id),
                });
            }
        }

        for workspace in Workspace::ALL {
            entries.push(PaletteEntry {
                category: "Workspace",
                label: workspace.label().to_owned(),
                hint: String::new(),
                action: PaletteAction::Workspace(workspace),
            });
        }

        entries.extend([
            PaletteEntry {
                category: "Action",
                label: "New birth chart".into(),
                hint: String::new(),
                action: PaletteAction::NewBirthChart,
            },
            PaletteEntry {
                category: "Action",
                label: "Jump to current time (Now)".into(),
                hint: String::new(),
                action: PaletteAction::JumpNow,
            },
            PaletteEntry {
                category: "Action",
                label: "Toggle sidebar rail".into(),
                hint: String::new(),
                action: PaletteAction::ToggleSidebar,
            },
            PaletteEntry {
                category: "Action",
                label: "Toggle contextual inspector".into(),
                hint: String::new(),
                action: PaletteAction::ToggleInspector,
            },
            PaletteEntry {
                category: "Action",
                label: "Toggle birth details privacy".into(),
                hint: "Privacy".into(),
                action: PaletteAction::ToggleBirthDetails,
            },
            PaletteEntry {
                category: "Action",
                label: "Open full chart view".into(),
                hint: "Lightbox".into(),
                action: PaletteAction::OpenFullView,
            },
        ]);

        if query.is_empty() {
            entries.truncate(18);
            return entries;
        }

        entries
            .into_iter()
            .filter(|entry| {
                entry.label.to_lowercase().contains(&query)
                    || entry.category.to_lowercase().contains(&query)
                    || entry.hint.to_lowercase().contains(&query)
            })
            .take(18)
            .collect()
    }

    pub(super) fn execute_palette_entry(
        &mut self,
        entry: PaletteEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_palette(window, cx);
        match entry.action {
            PaletteAction::Mode(mode) => self.select_mode(mode, cx),
            PaletteAction::Tool(tool) => self.update(cx, |session| {
                session.tool = Some(tool);
                session.workspace = Workspace::Advanced;
            }),
            PaletteAction::Profile(id) => {
                self.open_saved_profile(id, cx);
            }
            PaletteAction::Workspace(route) => self.navigate(route, cx),
            PaletteAction::NewBirthChart => self.open_draft(ChartRequest::default(), None, cx),
            PaletteAction::JumpNow => self.set_time(self.session.read(cx).now(), true, cx),
            PaletteAction::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
            }
            PaletteAction::ToggleInspector => {
                self.inspector_open = !self.inspector_open;
            }
            PaletteAction::ToggleBirthDetails => {
                self.show_birth = !self.show_birth;
            }
            PaletteAction::OpenFullView => {
                self.navigate(Workspace::Charts, cx);
                self.open_chart_lightbox(LightboxTarget::ActiveChart, cx);
            }
        }
        cx.notify();
    }

    pub(super) fn palette_overlay(&self, cx: &mut Context<Self>) -> AnyElement {
        use gpui::{MouseButton, rgba};
        let entries = self.palette_entries(cx);
        let selected_idx = if entries.is_empty() {
            0
        } else {
            self.palette_index.min(entries.len() - 1)
        };

        div()
            .id("command-palette-backdrop")
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .absolute()
            .inset_0()
            .bg(rgba(0x28251f66))
            .flex()
            .items_start()
            .justify_center()
            .pt(px(84.))
            .px_4()
            .on_click(cx.listener(|root, _, window, cx| root.close_palette(window, cx)))
            .child(
                div()
                    .id("command-palette-dialog")
                    .w(px(560.))
                    .max_w_full()
                    .bg(rgb(PAPER))
                    .border_1()
                    .border_color(rgb(LINE))
                    .rounded(px(RADIUS_PANEL))
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .on_key_down(cx.listener(|root, event: &gpui::KeyDownEvent, window, cx| {
                        let entries = root.palette_entries(cx);
                        match event.keystroke.key.as_str() {
                            "escape" => {
                                root.close_palette(window, cx);
                                cx.stop_propagation();
                            }
                            "down" => {
                                if !entries.is_empty() {
                                    root.palette_index = (root.palette_index + 1) % entries.len();
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }
                            "up" => {
                                if !entries.is_empty() {
                                    root.palette_index = if root.palette_index == 0 {
                                        entries.len() - 1
                                    } else {
                                        root.palette_index - 1
                                    };
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }
                            "enter" => {
                                if let Some(entry) = entries.get(root.palette_index).cloned() {
                                    root.execute_palette_entry(entry, window, cx);
                                }
                                cx.stop_propagation();
                            }
                            _ => {}
                        }
                    }))
                    .child(
                        div()
                            .p_3()
                            .border_b_1()
                            .border_color(rgb(LINE))
                            .bg(rgb(HEADER_BG))
                            .child(self.palette_query.clone()),
                    )
                    .child(
                        div()
                            .id("command-palette-results")
                            .max_h(px(380.))
                            .overflow_y_scroll()
                            .p_2()
                            .when(entries.is_empty(), |list| {
                                list.child(div().py_6().flex().justify_center().child(caption(
                                    "No matching modes, tools, profiles, or actions.",
                                )))
                            })
                            .children(entries.into_iter().enumerate().map(|(index, entry)| {
                                let is_selected = index == selected_idx;
                                let entry_for_click = entry.clone();
                                div()
                                    .id(("palette-item", index))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded(px(RADIUS_CONTROL))
                                    .cursor_pointer()
                                    .bg(rgb(if is_selected { SELECTED } else { PAPER }))
                                    .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                                    .on_click(cx.listener(move |root, _, window, cx| {
                                        root.execute_palette_entry(
                                            entry_for_click.clone(),
                                            window,
                                            cx,
                                        );
                                    }))
                                    .child(
                                        row()
                                            .gap_2()
                                            .min_w(px(0.))
                                            .child(badge(entry.category, Tone::Neutral))
                                            .child(
                                                div()
                                                    .text_size(px(13.5))
                                                    .font_weight(if is_selected {
                                                        gpui::FontWeight::MEDIUM
                                                    } else {
                                                        gpui::FontWeight::NORMAL
                                                    })
                                                    .text_color(rgb(INK))
                                                    .child(entry.label),
                                            ),
                                    )
                                    .child(caption(entry.hint))
                            })),
                    )
                    .child(
                        row()
                            .px_3()
                            .py_2()
                            .border_t_1()
                            .border_color(rgb(SUBTLE_LINE))
                            .bg(rgb(SOFT))
                            .justify_between()
                            .child(caption("↑↓ Navigate · Enter Open · Esc Close"))
                            .child(caption("Quick Jump")),
                    ),
            )
            .into_any_element()
    }
}
