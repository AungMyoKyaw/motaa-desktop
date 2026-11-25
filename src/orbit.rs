use gpui::{Div, PathBuilder, canvas, point, prelude::*, px, rgb};
use motaa_engine::reference_ephemeris::ReferenceEphemeris;

use crate::{caption, panel, row};

const COLORS: [u32; 9] = [
    0x62686f, 0x8a5e00, 0x226a95, 0xaf3928, 0x83512d, 0x705426, 0x187a72, 0x3858ae, 0x814a80,
];

pub fn render(ephemeris: &ReferenceEphemeris) -> Div {
    let paths = ephemeris.heliocentric_year();
    let positions = ephemeris.heliocentric_positions();
    panel()
        .child("Heliocentric paths")
        .child(caption(
            "One year around the reference date · Ecliptic projection · Logarithmic radial scale",
        ))
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let center = bounds.center();
                    let radius = bounds.size.width.min(bounds.size.height) / 2. - px(12.);
                    let maximum = 46_f64.ln();
                    let project = |(x, y): (f64, f64)| {
                        let distance = x.hypot(y);
                        let scale = if distance > 0. {
                            distance.ln_1p() / (distance * maximum)
                        } else {
                            0.
                        };
                        center + point(radius * (x * scale) as f32, radius * (-y * scale) as f32)
                    };
                    let mut sun = PathBuilder::stroke(px(5.));
                    sun.move_to(center - point(px(2.), px(0.)));
                    sun.line_to(center + point(px(2.), px(0.)));
                    if let Ok(path) = sun.build() {
                        window.paint_path(path, rgb(0xaa6500));
                    }
                    for (index, color) in COLORS.into_iter().enumerate() {
                        let mut path = PathBuilder::stroke(px(1.5));
                        path.move_to(project(paths[0][index]));
                        for sample in &paths[1..] {
                            path.line_to(project(sample[index]));
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(color));
                        }
                        let current = project(paths[24][index]);
                        let mut marker = PathBuilder::stroke(px(5.));
                        marker.move_to(current - point(px(2.), px(0.)));
                        marker.line_to(current + point(px(2.), px(0.)));
                        if let Ok(marker) = marker.build() {
                            window.paint_path(marker, rgb(color));
                        }
                    }
                },
            )
            .w_full()
            .h(px(320.)),
        )
        .child(
            row().flex_wrap().children(
                positions
                    .into_iter()
                    .zip(COLORS)
                    .map(|((name, _, _, _), color)| caption(name).text_color(rgb(color))),
            ),
        )
}
