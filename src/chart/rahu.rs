use gpui::{AnyElement, PathBuilder, canvas, div, prelude::*, px, rgb};
use motaa_engine::{
    original_geometry::original_l0, original_m0::original_m0, original_positions::original_b0,
};

use super::mode9::{dot, marker_angle, planet_bin, planet_color, planet_radius, plot, ring_label};
use crate::{INK, MUTED, PAPER, SOFT, name_wheel};

const MANSION: f32 = 360. / 27.;
pub(crate) const GROUP_NAMES: [&str; 6] = [
    "Jamma",
    "Kamma",
    "Sanghatika",
    "Samudayika",
    "Adhana",
    "Vinasa",
];
const SUN_NAMES: [&str; 9] = [
    "လောင်",
    "ဝိဒြု",
    "သူလ",
    "သန္နိ",
    "ကေတု",
    "ဥက္ကာ",
    "ဘူမိ",
    "ဝဇိရ",
    "နိဃာတ",
];
pub(crate) const ZONE_NAMES: [&str; 10] = [
    "မတ",
    "ဇီဝ",
    "ပုပ္ဖိတ",
    "ဖလိတ",
    "နိဖ္ဖလ",
    "ဇဋိတ",
    "ဂူဒ",
    "ရာဇဿ",
    "ထမဿ",
    "ဝုဒ္ဓ",
];
pub(crate) const ZONE_COLORS: [u32; 10] = [
    0x533337, 0xffffe1, 0xfffba8, 0xffc172, 0xff893e, 0xd36d2a, 0xeee4e5, 0xbd9da1, 0x9c7479,
    0x6b4247,
];

#[derive(Clone, Copy)]
pub(crate) struct RahuWheel {
    pub use_28: bool,
    pub node: Option<f64>,
    pub sun: Option<f64>,
    pub reference: Option<f64>,
    pub reference_name: &'static str,
}

impl RahuWheel {
    pub fn groups(self) -> [Option<usize>; 6] {
        self.reference.map_or([None; 6], |angle| {
            name_wheel::jamma_groups(angle, self.use_28)
        })
    }

    pub fn zones(self) -> Option<[f64; 10]> {
        self.node.map(original_b0)
    }

    pub fn render(self, side: f32, color: u32, bodies: &[(String, f64)]) -> AnyElement {
        let groups = self.groups();
        let sun = self.sun.map(|angle| {
            if self.use_28 {
                original_m0(angle)
            } else {
                original_l0(angle)
            }
        });
        let zones = self.zones();
        let marks = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let at = |angle, radius| bounds.origin + plot(angle, radius, side);
                let mut arc = |start: f32, sweep: f32, radius, width, color| {
                    if sweep <= 0. {
                        return;
                    }
                    let steps = (sweep / 2.).ceil() as u32;
                    let mut path = PathBuilder::stroke(px(width * side / 480.));
                    path.move_to(at(start, radius));
                    for step in 1..=steps {
                        path.line_to(at(start + sweep * step as f32 / steps as f32, radius));
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, rgb(color));
                    }
                };
                // N0 keeps each classification in its own band: 157–180, 141–157, 125–141.
                if let Some(zones) = zones {
                    for (index, start) in zones.into_iter().enumerate() {
                        let sweep = (zones[(index + 1) % zones.len()] - start).rem_euclid(360.);
                        arc(start as f32, sweep as f32, 168., 23., ZONE_COLORS[index]);
                    }
                }
                if let Some(members) = sun {
                    for index in members.into_iter().filter(|i| (0..27).contains(i)) {
                        arc(index as f32 * MANSION, MANSION, 149., 16., 0xf6be00);
                    }
                }
                for index in groups.into_iter().flatten() {
                    arc(index as f32 * MANSION, MANSION, 133., 16., 0xffdf11);
                }
                for radius in [60., 125., 140., 157., 180., 220., 239.5] {
                    arc(0., 360., radius, 0.85, color);
                }
                let mut spokes = PathBuilder::stroke(px(0.65 * side / 480.));
                let mut spoke = |angle, inner, outer| {
                    spokes.move_to(at(angle, inner));
                    spokes.line_to(at(angle, outer));
                };
                for index in 0..108 {
                    let angle = index as f32 * 360. / 108.;
                    spoke(angle, 180., 220.);
                    spoke(angle, 220., 239.5);
                    if index % 4 == 0 {
                        spoke(angle, 60., 157.);
                    }
                }
                for sign in 0..12 {
                    spoke(sign as f32 * 30., 180., 220.);
                }
                if let Some(zones) = zones {
                    for angle in zones {
                        spoke(angle as f32, 157., 180.);
                    }
                }
                if let Ok(path) = spokes.build() {
                    window.paint_path(path, rgb(color));
                }
            },
        )
        .size_full();

        let angle = |index: usize| (index as f32 + 0.5) * MANSION;
        let mut wheel = div()
            .relative()
            .size(px(side))
            .flex_shrink_0()
            .bg(rgb(PAPER))
            .child(marks)
            .children(
                name_wheel::syllables()
                    .iter()
                    .enumerate()
                    .map(|(index, syllable)| {
                        let font = if syllable.chars().count() > 3 {
                            4.2
                        } else {
                            7.5
                        };
                        ring_label(
                            syllable.clone(),
                            name_wheel::center(index) as f32,
                            230.,
                            side,
                            font,
                            INK,
                        )
                    }),
            );
        const RULERS: [u8; 9] = [9, 6, 1, 2, 3, 8, 5, 0, 4];
        for index in 0..27 {
            let name = name_wheel::mansion_name(index).unwrap_or_default();
            let name = match name {
                "Purvaphalguni" => "P. Phalg.",
                "Uttaraphalguni" => "U. Phalg.",
                "Purvashadha" => "P. Ashad.",
                "Uttarashadha" => "U. Ashad.",
                "Purvabhadrapada" => "P. Bhad.",
                "Uttarabhadrapada" => "U. Bhad.",
                name => name,
            };
            wheel = wheel
                .child(ring_label(name, angle(index), 110., side, 6.5, INK))
                .child(ring_label(
                    (index + 1).to_string(),
                    angle(index),
                    86.,
                    side,
                    8.,
                    INK,
                ))
                .child(ring_label(
                    RULERS[index % 9].to_string(),
                    angle(index),
                    70.,
                    side,
                    7.,
                    0x315f9d,
                ));
        }
        if let Some(sun) = sun {
            wheel = wheel.children(
                SUN_NAMES
                    .into_iter()
                    .zip(sun)
                    .filter(|(_, index)| (0..27).contains(index))
                    .map(|(name, index)| {
                        ring_label(name, angle(index as usize), 149., side, 8., INK)
                    }),
            );
        }
        wheel = wheel.children(
            ["Jamma", "Kamma", "Sangha", "Samu", "Adhana", "Vinasa"]
                .into_iter()
                .zip(groups)
                .filter_map(|(name, index)| {
                    Some(ring_label(name, angle(index?), 133., side, 8., INK))
                }),
        );

        let mut occupied = [0; 108];
        for (code, longitude) in bodies {
            let bin = planet_bin(*longitude);
            let radius = planet_radius(&mut occupied, bin);
            wheel = wheel.child(dot(
                code.clone(),
                marker_angle(bin),
                radius,
                side,
                12.,
                planet_color(code, *longitude),
                PAPER,
            ));
        }
        let hub_size = px(side / 4.);
        wheel
            .child(
                div()
                    .absolute()
                    .left(px(side * 0.375))
                    .top(px(side * 0.375))
                    .size(hub_size)
                    .rounded_full()
                    .bg(rgb(SOFT))
                    .border_1()
                    .border_color(rgb(color))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(MUTED))
                    .text_size(px((side / 33.).clamp(9., 15.)))
                    .child(self.reference_name)
                    .child(
                        div().text_color(rgb(INK)).child(
                            self.reference
                                .map_or_else(|| "—".into(), |angle| format!("{angle:.3}°")),
                        ),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_jamma_reference_preserves_the_rahu_zone_boundaries() {
        let node = RahuWheel {
            use_28: false,
            node: Some(100.113),
            sun: Some(120.),
            reference: Some(100.113),
            reference_name: "Rahu",
        };
        let manual = RahuWheel {
            use_28: true,
            reference: Some(201.),
            reference_name: "Manual",
            ..node
        };
        assert_eq!(node.zones(), manual.zones());
        assert_eq!(node.groups()[0], Some(7));
        assert_eq!(manual.groups()[0], Some(15));
        let without_ephemeris = RahuWheel {
            node: None,
            sun: None,
            ..manual
        };
        assert!(without_ephemeris.zones().is_none());
        assert_eq!(without_ephemeris.groups(), manual.groups());
    }
}
