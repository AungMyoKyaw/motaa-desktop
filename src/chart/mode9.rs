use super::{ChartScene, radial_point};
use gpui::{
    AnyElement, Bounds, PathBuilder, Pixels, Point, canvas, div, point, prelude::*, px, rgb, size,
};
use std::sync::OnceLock;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RingData {
    display_names: [String; 27],
    rulers: [String; 27],
    quarter_codes: Vec<String>,
    house_colors: [String; 12],
}

fn ring_data() -> &'static RingData {
    static DATA: OnceLock<RingData> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../design/source/circle-data.json"))
            .expect("valid Mode 9 source tables")
    })
}

pub(super) fn plot(angle: f32, radius: f32, side: f32) -> Point<Pixels> {
    let (x, y) = radial_point(angle, radius / 480.);
    point(px(x * side), px(y * side))
}

pub(super) fn marker_angle(bin: usize) -> f32 {
    bin as f32 * 360. / 108. + 1.6
}

fn house_marker_angle(longitude: f64) -> f32 {
    let [mansion, quarter, ..] = motaa_engine::original_positions::original_x0(longitude);
    marker_angle((4 * mansion + quarter) as usize)
}

pub(super) fn planet_bin(longitude: f64) -> usize {
    ((longitude / (360. / 27.) * 4.) as usize) % 108
}

pub(super) fn planet_radius(occupied: &mut [u8; 108], bin: usize) -> f32 {
    let radius = 210. - 12. * f32::from(occupied[bin]);
    occupied[bin] += 1;
    radius
}

fn digit(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '၀'..='၉' => char::from(b'0' + (ch as u32 - '၀' as u32) as u8),
            _ => ch,
        })
        .collect()
}

pub(super) fn ring_label(
    label: impl Into<gpui::SharedString>,
    angle: f32,
    radius: f32,
    side: f32,
    font_size: f32,
    color: u32,
) -> gpui::Div {
    let pos = plot(angle, radius, side);
    let font = px(font_size * side / 480.);
    let width = font * 5.;
    let height = font * 1.2;
    div()
        .absolute()
        .left(pos.x - width / 2.)
        .top(pos.y - height / 2.)
        .w(width)
        .h(height)
        .text_center()
        .text_size(font)
        .line_height(height)
        .whitespace_nowrap()
        .text_color(rgb(color))
        .child(label.into())
}

impl ChartScene {
    pub(crate) fn render_mode9(
        &self,
        side: f32,
        center_lines: &[String],
        selected_syllable: Option<usize>,
    ) -> AnyElement {
        let data = ring_data();
        let houses = self.houses;
        let scale = side / 480.;
        let markers = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let center = bounds.origin + point(px(side / 2.), px(side / 2.));
                let mut circle = |radius: f32, color| {
                    let r = px(radius * scale);
                    window.paint_quad(gpui::quad(
                        Bounds::new(center - point(r, r), size(r * 2., r * 2.)),
                        r,
                        rgb(color),
                        px(0.),
                        gpui::transparent_black(),
                        Default::default(),
                    ));
                };
                for (radius, color) in [
                    (240., 0xf0ead0),
                    (220., 0xfffdf6),
                    (145., 0xf0ead0),
                    (125., 0xfffdf6),
                    (95., 0xf0ead0),
                    (78., 0xfffdf6),
                    (60., 0xede0aa),
                ] {
                    circle(radius, color);
                }

                let mut lines = |width: f32, color, segments: &[(Point<Pixels>, Point<Pixels>)]| {
                    let mut path = PathBuilder::stroke(px(width * scale));
                    for &(first, second) in segments {
                        path.move_to(bounds.origin + first);
                        path.line_to(bounds.origin + second);
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, rgb(color));
                    }
                };
                if let Some(index) = selected_syllable {
                    let angle = crate::name_wheel::center(index) as f32;
                    let half_cell = 180. / 108.;
                    lines(
                        18.,
                        0xe6c370,
                        &[(
                            plot(angle - half_cell, 230., side),
                            plot(angle + half_cell, 230., side),
                        )],
                    );
                }
                let spoke =
                    |angle, inner, outer| (plot(angle, inner, side), plot(angle, outer, side));
                let mut subdivisions = Vec::new();
                for cell in 0..108 {
                    let angle = cell as f32 * 360. / 108.;
                    subdivisions.push(spoke(angle, 145., 220.));
                    if cell % 4 != 0 {
                        subdivisions.push(spoke(angle, 125., 145.));
                        subdivisions.push(spoke(angle, 220., 240.));
                    }
                }
                lines(0.45, 0xd7c78e, &subdivisions);

                let mut house_lines = Vec::new();
                if let Some(super::HouseGeometry { cusps, midpoints }) = houses {
                    for (index, midpoint) in midpoints.into_iter().enumerate() {
                        let tip = plot(house_marker_angle(midpoint), 160., side);
                        let first = plot(house_marker_angle(cusps[index]), 150., side);
                        let next = plot(house_marker_angle(cusps[(index + 1) % 12]), 150., side);
                        house_lines.push((house_color(index), [(first, tip), (tip, next)]));
                    }
                }
                for (color, segs) in house_lines {
                    lines(1.8, color, &segs);
                }

                let mut path = PathBuilder::stroke(px(0.85 * scale));
                for radius in [60., 78., 95., 125., 145., 220., 239.5] {
                    for step in 0..=96 {
                        let pt = bounds.origin + plot(step as f32 * 3.75, radius, side);
                        if step == 0 {
                            path.move_to(pt);
                        } else {
                            path.line_to(pt);
                        }
                    }
                }
                for mansion in 0..27 {
                    let angle = mansion as f32 * 360. / 27.;
                    let (a, b) = spoke(angle, 60., 145.);
                    path.move_to(bounds.origin + a);
                    path.line_to(bounds.origin + b);
                    let (c, d) = spoke(angle, 220., 240.);
                    path.move_to(bounds.origin + c);
                    path.line_to(bounds.origin + d);
                }
                for sign in 0..12 {
                    let (a, b) = spoke(sign as f32 * 30., 145., 240.);
                    path.move_to(bounds.origin + a);
                    path.line_to(bounds.origin + b);
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(0x28251f));
                }
            },
        )
        .size_full();

        let mut wheel = div()
            .relative()
            .size(px(side))
            .flex_shrink_0()
            .child(markers);
        for (index, syllable) in crate::name_wheel::syllables().iter().enumerate() {
            let angle = crate::name_wheel::center(index) as f32;
            let font = if syllable.chars().count() > 3 {
                4.2
            } else {
                6.4
            };
            wheel = wheel
                .child(ring_label(
                    syllable.clone(),
                    angle,
                    230.,
                    side,
                    font,
                    0x28251f,
                ))
                .child(ring_label(
                    digit(&data.quarter_codes[index]),
                    angle,
                    135.,
                    side,
                    6.5,
                    0x28251f,
                ));
        }
        for (index, name) in data.display_names.iter().enumerate() {
            let angle = (index as f32 + 0.5) * 360. / 27.;
            let name = if name.chars().count() > 9 {
                format!("{}.", name.chars().take(7).collect::<String>())
            } else {
                name.clone()
            };
            wheel = wheel
                .child(ring_label(name, angle, 110., side, 6.3, 0x28251f))
                .child(ring_label(
                    digit(&data.rulers[index]),
                    angle,
                    86.,
                    side,
                    8.,
                    0x315f9d,
                ))
                .child(ring_label(
                    (index + 1).to_string(),
                    angle,
                    68.,
                    side,
                    7.,
                    0x28251f,
                ));
        }
        if let Some(super::HouseGeometry { cusps, midpoints }) = houses {
            for (index, cusp) in cusps.into_iter().enumerate() {
                wheel = wheel
                    .child(dot(
                        "",
                        house_marker_angle(cusp),
                        150.,
                        side,
                        8.,
                        0x28251f,
                        0xffffff,
                    ))
                    .child(dot(
                        (index + 1).to_string(),
                        house_marker_angle(midpoints[index]),
                        160.,
                        side,
                        8.,
                        house_color(index),
                        0x181511,
                    ));
            }
        }
        let mut occupied = [0; 108];
        for (code, longitude) in &self.labels {
            let bin = planet_bin(*longitude);
            let radius = planet_radius(&mut occupied, bin);
            let color = planet_color(code, *longitude);
            wheel = wheel.child(dot(
                code.clone(),
                marker_angle(bin),
                radius,
                side,
                14.,
                color,
                0xffffff,
            ));
        }
        let count = center_lines.len() as f32;
        wheel = wheel.child(
            ring_label("MOTAA", 15., 7. * count, side, 12., 0x28251f).font_family("Baskerville"),
        );
        for (index, line) in center_lines.iter().enumerate() {
            let offset = 10. * (index as f32 + 1. - count / 2.);
            let clipped: String = line.chars().take(25).collect();
            wheel = wheel.child(ring_label(clipped, -165., offset, side, 7., 0x28251f));
        }
        wheel.into_any_element()
    }
}

fn house_color(index: usize) -> u32 {
    u32::from_str_radix(&ring_data().house_colors[index][1..], 16)
        .expect("valid Mode 9 house color")
}

pub(super) fn planet_color(code: &str, longitude: f64) -> u32 {
    let index = match code {
        "0" => 7,
        code => match code.parse::<u8>() {
            Ok(value @ 1..=9) => value,
            _ => return 0x151515,
        },
    };
    match motaa_engine::original_aspects::original_d0(index, longitude) {
        1 => 0x0000ff,
        2 => 0x4c9900,
        3 => 0xff0000,
        _ => 0x151515,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dot(
    text: impl Into<gpui::SharedString>,
    angle: f32,
    radius: f32,
    side: f32,
    diameter: f32,
    background: u32,
    foreground: u32,
) -> gpui::Div {
    let pos = plot(angle, radius, side);
    let diameter = px(diameter * side / 480.);
    div()
        .absolute()
        .left(pos.x - diameter / 2.)
        .top(pos.y - diameter / 2.)
        .size(diameter)
        .rounded_full()
        .bg(rgb(background))
        .text_color(rgb(foreground))
        .text_size(diameter * 0.62)
        .line_height(diameter)
        .flex()
        .items_center()
        .justify_center()
        .child(text.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode9_tables_and_marker_geometry_match_source() {
        assert_eq!(ring_data().quarter_codes.len(), 108);
        assert_eq!(digit("၉၀"), "90");
        assert!((marker_angle(planet_bin(195.247)) - 194.93333).abs() < 0.0001);
        let boundary = 3.33333333331;
        assert_eq!(planet_bin(boundary), 0);
        assert_eq!(house_marker_angle(boundary), marker_angle(1));

        let mut occupants = [0; 108];
        assert_eq!(planet_radius(&mut occupants, planet_bin(1.)), 210.);
        assert_eq!(planet_radius(&mut occupants, planet_bin(2.)), 198.);
        assert_eq!(planet_radius(&mut occupants, planet_bin(4.)), 210.);
        assert_eq!(planet_color("1", 0.), 0xff0000);
        assert_eq!(planet_color("1", 28.), 0x0000ff);
        assert_eq!(planet_color("1", 148.), 0x0000ff);
    }

    #[test]
    fn mode9_planet_anchors_match_the_android_pakhangyi_capture() {
        // The Android capture's wheel is 405 px across and begins at y=151.
        // Longitudes come from the V31.1 numeric oracle for the same chart.
        let mut occupied = [0; 108];
        for (body, longitude, x, y) in [
            ("Ascendant", 242.613291, 331., 475.),
            ("Sun", 258.123246, 360., 433.),
            ("Moon", 255.275271, 356., 442.),
            ("Mars", 182.085453, 161., 526.),
            ("Mercury", 251.915689, 350., 451.),
            ("Jupiter", 242.721741, 324., 468.),
            ("Venus", 218.640643, 272., 516.),
            ("Saturn", 200.475147, 223., 529.),
            ("Rahu", 52.226504, 97., 211.),
            ("Ketu", 232.226504, 308., 495.),
        ] {
            let bin = planet_bin(longitude);
            let radius = planet_radius(&mut occupied, bin);
            let anchor = plot(marker_angle(bin), radius, 405.);
            let dx = f32::from(anchor.x) - x;
            let dy = f32::from(anchor.y) + 151. - y;
            assert!(dx.hypot(dy) < 3., "{body}: {dx:.1}, {dy:.1}");
        }
    }
}
