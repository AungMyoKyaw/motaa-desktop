use gpui::{AnyElement, ClipboardItem, Context, div, img, prelude::*, px, rgb};

use crate::{
    DesktopRoot, GOLD, INK, LINE, MUTED, PAPER, SOFT, SUBTLE_LINE, Tone, action_control, app_icon,
    badge, caption, column, heading, panel, pill_shadow, row, section_label, ui_icon,
};

const ORIGINAL_APP_URL: &str = "https://play.google.com/store/apps/details?id=motaa.myanmar";
const NUMAN_MOTAA_URL: &str = "https://web.facebook.com/numan.motaa";
const KOPAKHAN_URL: &str = "https://web.facebook.com/AstrologerKoPaKhan";
const REPO_URL: &str = "https://github.com/AungMyoKyaw/motaa-desktop";

fn link_action_button(
    id: impl Into<gpui::ElementId>,
    label: &'static str,
    icon_path: &'static str,
    primary: bool,
    cx: &mut Context<DesktopRoot>,
    action: impl Fn(&mut DesktopRoot, &mut Context<DesktopRoot>) + 'static,
) -> AnyElement {
    action_control(id, cx, action)
        .px(px(10.))
        .h(px(28.))
        .gap(px(6.))
        .justify_center()
        .rounded(px(crate::RADIUS_CONTROL))
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .border_1()
        .border_color(rgb(if primary { GOLD } else { LINE }))
        .bg(rgb(if primary { GOLD } else { PAPER }))
        .text_color(rgb(if primary { PAPER } else { INK }))
        .when(primary, |el| el.shadow(pill_shadow()))
        .hover(|style| style.bg(rgb(if primary { crate::ACTION_HOVER } else { SOFT })))
        .active(|style| {
            style.bg(rgb(if primary {
                crate::ACTION_PRESSED
            } else {
                crate::SELECTED
            }))
        })
        .child(ui_icon(
            icon_path,
            12.5,
            if primary { PAPER } else { MUTED },
        ))
        .child(label)
        .into_any_element()
}

fn credit_link_row(
    id: &'static str,
    title: &'static str,
    role: &'static str,
    url: &'static str,
    cx: &mut Context<DesktopRoot>,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .p_3()
        .rounded(px(8.))
        .bg(rgb(SOFT))
        .border_1()
        .border_color(rgb(SUBTLE_LINE))
        .child(
            column()
                .gap_1()
                .flex_1()
                .min_w(px(0.))
                .child(
                    div()
                        .text_size(px(13.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(INK))
                        .child(title),
                )
                .child(caption(role))
                .child(
                    div()
                        .font_family("Menlo")
                        .text_size(px(11.5))
                        .text_color(rgb(GOLD))
                        .child(url),
                ),
        )
        .child(
            row()
                .flex_shrink_0()
                .gap_2()
                .child(link_action_button(
                    (id, 0usize),
                    "Open",
                    "ui/external-link",
                    true,
                    cx,
                    move |view, cx| {
                        cx.open_url(url);
                        view.set_status(format!("Opened {title} in browser."), cx);
                    },
                ))
                .child(link_action_button(
                    (id, 1usize),
                    "Copy",
                    "ui/copy",
                    false,
                    cx,
                    move |view, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(url.to_owned()));
                        view.set_status(format!("Copied {title} link to clipboard."), cx);
                    },
                )),
        )
        .into_any_element()
}

impl DesktopRoot {
    pub(crate) fn about_view(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .max_w(px(860.))
            .pb_6()
            .child(
                column()
                    .w_full()
                    .gap_3()
                    .child(
                        panel()
                            .w_full()
                            .flex_shrink_0()
                            .gap_3()
                            .child(
                                row()
                                    .w_full()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        img(app_icon())
                                            .w(px(48.))
                                            .h(px(48.))
                                            .rounded(px(10.))
                                            .flex_shrink_0(),
                                    )
                                    .child(
                                        column()
                                            .w_full()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .gap_1()
                                            .child(
                                                row()
                                                    .w_full()
                                                    .flex_wrap()
                                                    .gap_2()
                                                    .items_center()
                                                    .child(heading("About MOTAA Desktop"))
                                                    .child(badge(
                                                        format!("v{}", env!("CARGO_PKG_VERSION")),
                                                        Tone::Gold,
                                                    ))
                                                    .child(badge(
                                                        "Reverse-Engineered Port",
                                                        Tone::Neutral,
                                                    )),
                                            )
                                            .child(caption(
                                                "Offline-first Myanmar astrology workspace built with Rust and GPUI.",
                                            ).w_full()),
                                    ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .text_size(px(13.5))
                                    .line_height(px(20.))
                                    .text_color(rgb(INK))
                                    .child(
                                        "MOTAA Desktop is a reverse-engineered desktop version of the original MOTAA Myanmar Android application. The astrological algorithms, traditional Burmese chart geometry, and calculations in this app are ported from the original Android release and the work of its creators.",
                                    ),
                            ),
                    )
                    .child(
                        panel()
                            .w_full()
                            .flex_shrink_0()
                            .gap_3()
                            .child(section_label("ORIGINAL APPLICATION"))
                            .child(caption(
                                "This desktop workspace is a port of the original Android release on Google Play:",
                            ).w_full())
                            .child(credit_link_row(
                                "about-original-app",
                                "MOTAA Myanmar (Google Play)",
                                "Original Android application from which MOTAA Desktop was reverse-engineered.",
                                ORIGINAL_APP_URL,
                                cx,
                            )),
                    )
                    .child(
                        panel()
                            .w_full()
                            .flex_shrink_0()
                            .gap_3()
                            .child(section_label("SPECIAL THANKS & CREDITS"))
                            .child(caption(
                                "The algorithms and astrological calculations in this application are ports of the work by these creators. Special thanks to:",
                            ).w_full())
                            .child(credit_link_row(
                                "about-numan-motaa",
                                "Numan MOTAA",
                                "Original MOTAA Myanmar creator — algorithms and calculation port.",
                                NUMAN_MOTAA_URL,
                                cx,
                            ))
                            .child(credit_link_row(
                                "about-kopakhan",
                                "AstrologerKoPaKhan",
                                "Astrological methodology and calculations behind the port.",
                                KOPAKHAN_URL,
                                cx,
                            )),
                    )
                    .child(
                        panel()
                            .w_full()
                            .flex_shrink_0()
                            .gap_3()
                            .child(section_label("DESKTOP EDITION"))
                            .child(caption(
                                "Rust + GPUI offline-first desktop workspace · License: GNU Affero General Public License v3.0 (AGPL-3.0) · Local-first SQLite storage with no accounts or telemetry.",
                            ).w_full())
                            .child(credit_link_row(
                                "about-repo",
                                "MOTAA Desktop Source Repository",
                                "Open-source desktop port repository, releases, and documentation.",
                                REPO_URL,
                                cx,
                            )),
                    ),
            )
            .into_any_element()
    }
}
