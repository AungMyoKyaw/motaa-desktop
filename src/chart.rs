pub(crate) mod comparison;
pub(crate) mod latlon;
mod mode9;
mod rahu;
pub(crate) use rahu::{GROUP_NAMES, RahuWheel, ZONE_COLORS, ZONE_NAMES};
pub(crate) mod traversal;
pub(crate) mod western;

use gpui::{AnyElement, PathBuilder, canvas, div, point, prelude::*, px, rgb};

pub(crate) const TRADITIONAL_PLANETS: [(&str, &str, u32); 9] = [
    ("Sun", "1", 0x216e48),
    ("Moon", "2", 0xb42318),
    ("Mars", "3", super::INK),
    ("Mercury", "4", 0x315f9d),
    ("Jupiter", "5", super::INK),
    ("Venus", "6", 0xb42318),
    ("Saturn", "0", 0x315f9d),
    ("Rahu", "8", super::INK),
    ("Ketu", "9", 0x216e48),
];

pub(crate) const BODY_CODES: [&str; 10] = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"];

pub(crate) fn planet_color(code: &str) -> u32 {
    TRADITIONAL_PLANETS
        .iter()
        .find(|(_, c, _)| *c == code)
        .map_or(super::INK, |(_, _, color)| *color)
}

#[derive(Clone, Copy)]
pub struct HouseGeometry {
    pub cusps: [f64; 12],
    pub midpoints: [f64; 12],
}

const SQUARE_SECTORS: [(f32, f32); 12] = [
    (0.375, 0.04),  // 0 Aries: top-center rectangle
    (0.155, 0.025), // 1 Taurus: top-left upper triangle (y < x)
    (0.025, 0.17),  // 2 Gemini: top-left lower triangle (y > x)
    (0.035, 0.36),  // 3 Cancer: middle-left rectangle
    (0.025, 0.65),  // 4 Leo: bottom-left upper triangle
    (0.21, 0.82),   // 5 Virgo: bottom-left lower triangle
    (0.375, 0.67),  // 6 Libra: bottom-center rectangle
    (0.655, 0.79),  // 7 Scorpio: bottom-right lower triangle (y > x)
    (0.785, 0.65),  // 8 Sagittarius: bottom-right upper triangle (y < x)
    (0.66, 0.36),   // 9 Capricorn: middle-right rectangle
    (0.81, 0.20),   // 10 Aquarius: top-right lower triangle
    (0.655, 0.025), // 11 Pisces: top-right upper triangle
];

pub struct ChartScene {
    pub labels: Vec<(String, f64)>,
    pub title: &'static str,
    pub circular: bool,
    pub rahu: Option<RahuWheel>,
    pub houses: Option<HouseGeometry>,
    pub color: u32,
}

impl ChartScene {
    pub fn render(&self, side: f32) -> AnyElement {
        self.render_with_retrograde(side, &[])
    }

    pub fn render_with_retrograde(&self, side: f32, retrograde: &[bool]) -> AnyElement {
        if let Some(wheel) = self.rahu {
            return wheel.render(side, self.color, &self.labels);
        }
        if self.circular {
            return self.render_mode9(side, &[], None);
        }
        let color = self.color;
        let base = div()
            .relative()
            .size(px(side))
            .flex_shrink_0()
            .bg(rgb(0xfffefc))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let pos = |x, y| bounds.origin + point(px(x * side), px(y * side));
                        let lo = 0.35;
                        let hi = 0.635;

                        // Inner traditional house grid & corner diagonals (1px hairline)
                        let mut inner = PathBuilder::stroke(px(1.));
                        let add_line = |builder: &mut PathBuilder, a: (f32, f32), b: (f32, f32)| {
                            builder.move_to(pos(a.0, a.1));
                            builder.line_to(pos(b.0, b.1));
                        };
                        for n in [lo, hi] {
                            add_line(&mut inner, (n, 0.), (n, 1.));
                            add_line(&mut inner, (0., n), (1., n));
                        }
                        add_line(&mut inner, (0., 0.), (lo, lo));
                        add_line(&mut inner, (1., 0.), (hi, lo));
                        add_line(&mut inner, (0., 1.), (lo, hi));
                        add_line(&mut inner, (1., 1.), (hi, hi));
                        if let Ok(path) = inner.build() {
                            window.paint_path(path, rgb(color));
                        }

                        // Crisp 1.5px outer chart frame for engraved manuscript weight
                        let mut outer = PathBuilder::stroke(px(1.5));
                        add_line(&mut outer, (0., 0.), (1., 0.));
                        add_line(&mut outer, (1., 0.), (1., 1.));
                        add_line(&mut outer, (1., 1.), (0., 1.));
                        add_line(&mut outer, (0., 1.), (0., 0.));
                        if let Ok(path) = outer.build() {
                            window.paint_path(path, rgb(color));
                        }
                    },
                )
                .size_full(),
            );

        let sign_index = |longitude: f64| (longitude.rem_euclid(360.) / 30.) as usize;
        let compact_labels = matches!(self.title, "Bhava" | "Navamsa");
        let (max_code_px, max_deg_px) = if side < 260. { (13., 9.) } else { (19., 10.) };
        let mut occupancy = [0; 12];
        for (_, degree) in &self.labels {
            occupancy[sign_index(*degree)] += 1;
        }
        let mut counts = [0; 12];
        let mut labels = self
            .labels
            .iter()
            .enumerate()
            .map(|(label_idx, (name, degree))| {
                let index = sign_index(*degree);
                let row = counts[index];
                counts[index] += 1;
                let (x, y) = SQUARE_SECTORS[index];
                let height = if matches!(index, 0 | 3 | 6 | 9) {
                    0.26
                } else {
                    0.15
                };
                let line_height = (side * height / occupancy[index].max(1) as f32).min(22.);
                let label = div()
                    .absolute()
                    .left(px(x * side))
                    .top(px(y * side + row as f32 * line_height))
                    .line_height(px(line_height))
                    .whitespace_nowrap();
                let color = planet_color(name);
                if compact_labels {
                    label
                        .text_color(rgb(color))
                        .text_size(px((line_height * 0.86).min(15.)))
                        .child(name.clone())
                } else {
                    let minutes = (degree.rem_euclid(30.) * 60.).floor() as u32;
                    let retro = if retrograde.get(label_idx).copied().unwrap_or(false) {
                        "R"
                    } else {
                        ""
                    };
                    label
                        .flex()
                        .items_center()
                        .gap_1()
                        .line_height(px(line_height))
                        .child(
                            div()
                                .text_color(rgb(color))
                                .text_size(px((line_height * 0.86).min(max_code_px)))
                                .child(name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px((line_height * 0.72).min(max_deg_px)))
                                .child(format!("{}°{:02}′{retro}", minutes / 60, minutes % 60)),
                        )
                }
            });
        base.child(
            div()
                .absolute()
                .left(px(side * 0.35))
                .top(px(side * 0.35))
                .size(px(side * 0.285))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .text_center()
                .text_size(px((side * 0.042).clamp(9., 14.)))
                .child(self.title),
        )
        .children(&mut labels)
        .into_any_element()
    }
}

fn radial_point(degrees: f32, radius: f32) -> (f32, f32) {
    let angle = (degrees + 75.).to_radians();
    (0.5 + radius * angle.cos(), 0.5 - radius * angle.sin())
}

#[cfg(test)]
mod tests {
    use super::SQUARE_SECTORS;

    #[test]
    fn square_sectors_match_counter_clockwise_myanmar_rasi_layout() {
        let lo = 0.35;
        let hi = 0.635;

        // Cardinal signs (0 Aries, 3 Cancer, 6 Libra, 9 Capricorn) in rectangular boxes:
        // top-center, middle-left, bottom-center, middle-right.
        let (x0, y0) = SQUARE_SECTORS[0];
        assert!(x0 >= lo && x0 <= hi && y0 < lo);

        let (x3, y3) = SQUARE_SECTORS[3];
        assert!(x3 < lo && y3 >= lo && y3 <= hi);

        let (x6, y6) = SQUARE_SECTORS[6];
        assert!(x6 >= lo && x6 <= hi && y6 > hi);

        let (x9, y9) = SQUARE_SECTORS[9];
        assert!(x9 > hi && y9 >= lo && y9 <= hi);

        // Corner triangles: counter-clockwise around the square.
        // Top-left corner diagonal runs (0, 0) -> (lo, lo): y < x is upper (1 Taurus), y > x is lower (2 Gemini).
        let (x1, y1) = SQUARE_SECTORS[1];
        assert!(x1 < lo && y1 < lo && y1 < x1);
        let (x2, y2) = SQUARE_SECTORS[2];
        assert!(x2 < lo && y2 < lo && y2 > x2);

        // Bottom-left corner diagonal runs (0, 1) -> (lo, hi): y = 1 - (1 - hi) / lo * x.
        let bl_diag = |x: f32| 1. - (1. - hi) / lo * x;
        let (x4, y4) = SQUARE_SECTORS[4];
        assert!(x4 < lo && y4 > hi && y4 < bl_diag(x4));
        let (x5, y5) = SQUARE_SECTORS[5];
        assert!(x5 < lo && y5 > hi && y5 > bl_diag(x5));

        // Bottom-right corner diagonal runs (hi, hi) -> (1, 1): y > x is lower (7 Scorpio), y < x is upper (8 Sagittarius).
        let (x7, y7) = SQUARE_SECTORS[7];
        assert!(x7 > hi && y7 > hi && y7 > x7);
        let (x8, y8) = SQUARE_SECTORS[8];
        assert!(x8 > hi && y8 > hi && y8 < x8);

        // Top-right corner diagonal runs (hi, lo) -> (1, 0): y = lo * (1 - x) / (1 - hi).
        let tr_diag = |x: f32| lo * (1. - x) / (1. - hi);
        let (x10, y10) = SQUARE_SECTORS[10];
        assert!(x10 > hi && y10 < lo && y10 > tr_diag(x10));
        let (x11, y11) = SQUARE_SECTORS[11];
        assert!(x11 > hi && y11 < lo && y11 < tr_diag(x11));
    }

    #[test]
    fn body_codes_align_with_recovered_bodies_and_colors() {
        assert_eq!(super::BODY_CODES.len(), crate::recovered::BODIES.len());
        assert_eq!(super::BODY_CODES[0], "Asc");
        assert_eq!(super::planet_color("Asc"), crate::INK);
        for (body, code, color) in super::TRADITIONAL_PLANETS {
            let idx = crate::recovered::BODIES
                .iter()
                .position(|&b| b == body)
                .expect("traditional planet in BODIES");
            assert_eq!(super::BODY_CODES[idx], code);
            assert_eq!(super::planet_color(code), color);
        }
    }
}
