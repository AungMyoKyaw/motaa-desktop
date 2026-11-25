use gpui::{AnyElement, Context, Div, div, prelude::*, px, rgb};
use motaa_engine::original_aspects::{original_g, original_h};
use motaa_engine::reference_ephemeris::ReferenceEphemeris;

use crate::{DesktopRoot, LINE, Settings, caption, column, panel, recovered, row};

const HEADERS: [&str; 13] = [
    "Asc", "Sun", "Moon", "Mars", "Merc", "Jup", "Ven", "Sat", "Rahu", "Ura", "Nep", "Plu", "MC",
];
const ASPECTS: [&str; 12] = ["", "☌", "☍", "△", "□", "⚹", "⚻", "⚺", "∠", "⚼", "Q", "bQ"];

pub(crate) fn render(
    planets: &ReferenceEphemeris,
    ascendant: &ReferenceEphemeris,
    settings: Settings,
    palette_color: u32,
    cx: &mut Context<DesktopRoot>,
) -> AnyElement {
    let Some((longitudes, traditional)) = inputs(planets, ascendant, settings) else {
        return panel()
            .child("W-Style needs a supported Ascendant, Midheaven, and planetary positions.")
            .into_any_element();
    };
    let matrix = match original_h(longitudes) {
        Ok(matrix) => matrix.codes,
        Err(error) => return panel().child(error).into_any_element(),
    };

    let body_panel = column()
        .gap_1()
        .child("Planet and angle aspects")
        .child(matrix_header(HEADERS))
        .children(
            matrix
                .iter()
                .enumerate()
                .map(|(index, codes)| matrix_row(HEADERS[index], codes.iter().copied())),
        );

    let house_panel =
        match ascendant.house_midpoints(
            motaa_engine::houses::System::OPTIONS[settings.houses],
            settings.tropical,
        ) {
            Ok(midpoints) => {
                let columns = midpoints.map(|midpoint| {
                    let mut inputs = longitudes;
                    inputs[0] = midpoint;
                    original_g(inputs).map(|codes| codes.codes)
                });
                if let Some(error) = columns.iter().find_map(|column| column.as_ref().err()) {
                    column()
                        .child("House aspects")
                        .child(caption(error.clone()))
                } else {
                    column()
                        .gap_1()
                        .child("House midpoint aspects")
                        .child(matrix_header((1..=12).map(|house| house.to_string())))
                        .children(HEADERS.iter().copied().enumerate().skip(1).map(
                            |(index, name)| {
                                matrix_row(
                                    name,
                                    columns.iter().map(|column| {
                                        column.as_ref().map_or(0, |codes| codes[index])
                                    }),
                                )
                            },
                        ))
                }
            }
            Err(_) => column()
                .child("House aspects")
                .child(caption("House cusps unavailable for this configuration.")),
        };

    let chart = super::ChartScene {
        labels: ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"]
            .into_iter()
            .zip(traditional)
            .map(|(code, angle)| (code.into(), angle))
            .collect(),
        title: "Rasi chart",
        circular: false,
        rahu: None,
        houses: None,
        color: palette_color,
    };

    column()
        .child(caption(
            "W-Style · original angle rules applied to independent astronomical positions",
        ))
        .child(
            div()
                .grid()
                .grid_cols(2)
                .gap_4()
                .child(body_panel)
                .child(house_panel),
        )
        .child(caption(
            "☌ 0° · ☍ 180° · △ 120° · □ 90° (±8°) · ⚹ 60° (±6°) · ⚻ 150° (±3°)",
        ))
        .child(caption("⚺ 30° · ∠ 45° · ⚼ 135° · Q 72° · bQ 144° (±2°)"))
        .child(
            crate::action_control("western-rasi-inset", cx, |root, cx| {
                root.open_chart_lightbox(crate::LightboxTarget::RasiInset, cx);
            })
            .w(px(240.))
            .child(chart.render(240.)),
        )
        .into_any_element()
}

fn inputs(
    planets: &ReferenceEphemeris,
    ascendant: &ReferenceEphemeris,
    settings: Settings,
) -> Option<([f64; 13], [f64; 10])> {
    let traditional = recovered::positions(planets, ascendant, settings)?;
    let mut result = [0.; 13];
    result[..9].copy_from_slice(&traditional[..9]);
    for (slot, planet) in result[9..12].iter_mut().zip(["Uranus", "Neptune", "Pluto"]) {
        *slot = planets.longitude(planet, settings.tropical)?;
    }
    result[12] = ascendant.midheaven(settings.tropical)?;
    Some((result, traditional))
}

fn matrix_header<T: Into<gpui::SharedString>>(labels: impl IntoIterator<Item = T>) -> Div {
    row()
        .gap_0()
        .child(div().w(px(36.)).flex_shrink_0())
        .children(labels.into_iter().map(|label| {
            div()
                .flex_1()
                .min_w(px(0.))
                .text_center()
                .text_size(px(9.))
                .child(label.into())
        }))
}

fn matrix_row(title: &'static str, codes: impl IntoIterator<Item = u8>) -> Div {
    row()
        .gap_0()
        .child(
            div()
                .w(px(36.))
                .flex_shrink_0()
                .text_size(px(10.))
                .child(title),
        )
        .children(codes.into_iter().map(|code| {
            div()
                .flex_1()
                .min_w(px(0.))
                .h(px(22.))
                .text_center()
                .text_size(px(13.))
                .border_b_1()
                .border_r_1()
                .border_color(rgb(LINE))
                .bg(rgb(if code == 0 { 0xfffefc } else { 0xede0bf }))
                .text_color(rgb(0x315f9d))
                .child(ASPECTS[usize::from(code)])
        }))
}
