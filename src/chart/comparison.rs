use gpui::{AnyElement, Context, PathBuilder, canvas, div, point, prelude::*, px, rgb};
use motaa_engine::{ChartResult, reference_ephemeris::ReferenceEphemeris};

use crate::{DesktopRoot, INK, Settings, button, caption, column, data_row, panel, recovered, row};

const CODES: [&str; 10] = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"];
const TRANSIT: u32 = 0x315f9d;
const NESTED_OFFSETS: [(f32, f32); 12] = [
    (0., -1.),    // 0 Aries (top-center)
    (-0.58, -1.), // 1 Taurus (top-left upper)
    (-1., -0.58), // 2 Gemini (top-left lower)
    (-1., 0.),    // 3 Cancer (middle-left)
    (-1., 0.58),  // 4 Leo (bottom-left upper)
    (-0.58, 1.),  // 5 Virgo (bottom-left lower)
    (0., 1.),     // 6 Libra (bottom-center)
    (0.58, 1.),   // 7 Scorpio (bottom-right lower)
    (1., 0.58),   // 8 Sagittarius (bottom-right upper)
    (1., 0.),     // 9 Capricorn (middle-right)
    (1., -0.58),  // 10 Aquarius (top-right lower)
    (0.58, -1.),  // 11 Pisces (top-right upper)
];

impl DesktopRoot {
    pub(crate) fn comparison(
        &self,
        natal: ChartResult,
        transit: ChartResult,
        settings: Settings,
        side: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (Some(natal), Some(transit)) = (natal.reference_ephemeris, transit.reference_ephemeris)
        else {
            return panel()
                .child("Both epochs need known, supported times.")
                .into_any_element();
        };

        let (Some(natal), Some(transit)) = (
            positions(&natal, settings, self.show_munp),
            positions(&transit, settings, self.show_munp),
        ) else {
            return panel()
                .child("Planet positions are unavailable for one of the epochs.")
                .into_any_element();
        };
        let palette_color = INK;
        let chart = if self.split_comparison {
            div()
                .grid()
                .grid_cols(2)
                .gap_4()
                .children(
                    [("Birth", &natal), ("Transit", &transit)]
                        .into_iter()
                        .enumerate()
                        .map(|(index, (title, positions))| {
                            column().child(title).child(
                                crate::action_control(
                                    ("comparison-split-chart", index),
                                    cx,
                                    move |root, cx| {
                                        root.open_chart_lightbox(
                                            crate::LightboxTarget::ComparisonSplit(index),
                                            cx,
                                        );
                                    },
                                )
                                .child(
                                    super::ChartScene {
                                        labels: positions
                                            .iter()
                                            .map(|(code, _, angle)| ((*code).into(), *angle))
                                            .collect(),
                                        title: "Rasi chart",
                                        circular: false,
                                        rahu: None,
                                        houses: None,
                                        color: palette_color,
                                    }
                                    .render((side * 0.85).min(370.)),
                                ),
                            )
                        }),
                )
                .into_any_element()
        } else {
            crate::action_control("comparison-combined-chart", cx, |root, cx| {
                root.open_chart_lightbox(crate::LightboxTarget::ComparisonCombined, cx);
            })
            .w(px(side))
            .child(nested_chart(&natal, &transit, palette_color, side))
            .into_any_element()
        };
        let positions = column()
            .gap_0()
            .min_w(px(0.))
            .child(data_row(
                ["Body", "Birth longitude", "Transit longitude", "Separation"].map(str::to_owned),
            ))
            .children(natal.iter().filter_map(|(_, name, birth)| {
                let (_, _, current) = transit.iter().find(|(_, other, _)| name == other)?;
                let delta = (current - birth + 180.).rem_euclid(360.) - 180.;
                Some(data_row([
                    (*name).into(),
                    format!("{birth:.3}°"),
                    format!("{current:.3}°"),
                    format!("{:.3}°", delta.abs()),
                ]))
            }));
        let body = if self.split_comparison {
            column().child(chart).child(positions)
        } else {
            row()
                .items_start()
                .gap_6()
                .child(chart)
                .child(positions.flex_1())
        };
        column()
            .child(
                row()
                    .child(button("Combined", !self.split_comparison, cx, |view, _| {
                        view.split_comparison = false
                    }))
                    .child(button(
                        "Side by side",
                        self.split_comparison,
                        cx,
                        |view, _| view.split_comparison = true,
                    ))
                    .child(button("MUNP", self.show_munp, cx, |view, _| {
                        view.show_munp = !view.show_munp
                    })),
            )
            .when(!self.split_comparison, |view| {
                view.child(caption("Birth inside · Transit outside"))
            })
            .child(body)
            .into_any_element()
    }
}

pub(crate) type Position = (&'static str, &'static str, f64);

pub(crate) fn positions(
    epoch: &ReferenceEphemeris,
    settings: Settings,
    show_munp: bool,
) -> Option<Vec<Position>> {
    let values = recovered::positions(epoch, epoch, settings)?;
    let mut positions: Vec<_> = CODES
        .into_iter()
        .zip(recovered::BODIES)
        .zip(values)
        .map(|((code, name), angle)| (code, name, angle))
        .collect();
    if show_munp {
        positions.extend(
            [
                ("M", "Midheaven", epoch.midheaven(settings.tropical)),
                ("U", "Uranus", epoch.longitude("Uranus", settings.tropical)),
                (
                    "N",
                    "Neptune",
                    epoch.longitude("Neptune", settings.tropical),
                ),
                ("P", "Pluto", epoch.longitude("Pluto", settings.tropical)),
            ]
            .into_iter()
            .filter_map(|(code, name, angle)| Some((code, name, angle?))),
        );
    }
    Some(positions)
}

pub(crate) fn nested_chart(
    natal: &[Position],
    transit: &[Position],
    line_color: u32,
    side: f32,
) -> AnyElement {
    let marks = canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let mut path = PathBuilder::stroke(px(1.));
            let pos = |x, y| bounds.origin + point(px(x * side / 1370.), px(y * side / 1370.));
            let mut line = |(x1, y1), (x2, y2)| {
                path.move_to(pos(x1, y1));
                path.line_to(pos(x2, y2));
            };
            for (low, high) in [(5., 1365.), (205., 1165.)] {
                for coordinate in [low, 545., 815., high] {
                    line((coordinate, low), (coordinate, high));
                    line((low, coordinate), (high, coordinate));
                }
            }
            for (x, inner_x, center_x) in [(5., 205., 545.), (1365., 1165., 815.)] {
                for (y, inner_y, center_y) in [(5., 205., 545.), (1365., 1165., 815.)] {
                    line((x, y), (inner_x, inner_y));
                    line((inner_x, inner_y), (center_x, center_y));
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(line_color));
            }
        },
    )
    .size_full();
    let mut chart = div()
        .relative()
        .size(px(side))
        .bg(rgb(0xfffefc))
        .child(
            div()
                .absolute()
                .left(px(side * 545. / 1370.))
                .top(px(side * 545. / 1370.))
                .size(px(side * 270. / 1370.))
                .bg(rgb(0xf1ecdf))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.))
                .child("MOTAA"),
        )
        .child(marks);
    for (positions, radius, color) in [(natal, 0.29, INK), (transit, 0.43, TRANSIT)] {
        for (index, (dx, dy)) in NESTED_OFFSETS.into_iter().enumerate() {
            let codes = positions
                .iter()
                .filter(|(_, _, angle)| recovered::sign(*angle) == index)
                .map(|(code, _, _)| *code)
                .collect::<Vec<_>>();
            if codes.is_empty() {
                continue;
            }
            let width = side * if radius > 0.4 { 0.14 } else { 0.17 };
            let label = div()
                .absolute()
                .left(px((0.5 + dx * radius) * side - width / 2.))
                .top(px((0.5 + dy * radius) * side - 7.))
                .w(px(width))
                .text_center()
                .text_color(rgb(color))
                .text_size(px(10.))
                .child(codes.join(" "));
            chart = chart.child(label);
        }
    }
    chart.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::NESTED_OFFSETS;

    #[test]
    fn nested_offsets_proceed_counter_clockwise_from_top_center() {
        assert_eq!(NESTED_OFFSETS[0], (0., -1.));
        assert_eq!(NESTED_OFFSETS[3], (-1., 0.));
        assert_eq!(NESTED_OFFSETS[6], (0., 1.));
        assert_eq!(NESTED_OFFSETS[9], (1., 0.));

        // Top-left (signs 1, 2) has negative x and y; bottom-right (signs 7, 8) has positive x and y.
        assert!(NESTED_OFFSETS[1].0 < 0. && NESTED_OFFSETS[1].1 < 0.);
        assert!(NESTED_OFFSETS[2].0 < 0. && NESTED_OFFSETS[2].1 < 0.);
        assert!(NESTED_OFFSETS[7].0 > 0. && NESTED_OFFSETS[7].1 > 0.);
        assert!(NESTED_OFFSETS[8].0 > 0. && NESTED_OFFSETS[8].1 > 0.);
    }
}
