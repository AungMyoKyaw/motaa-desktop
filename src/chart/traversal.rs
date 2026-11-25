use gpui::{AnyElement, PathBuilder, canvas, div, point, prelude::*, px, rgb};
use motaa_engine::{
    houses::house_at,
    original_colors::original_x,
    original_positions::{original_x0, original_y0},
};

use crate::{LINE, caption, column, data_row, name_wheel, recovered, row};

const CODES: [&str; 10] = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"];

pub(crate) fn render(
    positions: [f64; 10],
    retrograde: [bool; 10],
    houses: Option<super::HouseGeometry>,
    palette_color: u32,
    side: f32,
) -> AnyElement {
    let rasi = positions.map(recovered::sign);
    let navamsa = positions.map(|angle| recovered::sign(original_x(angle)));
    let bhava = houses.and_then(|houses| {
        let mut sectors = [0; 10];
        for (slot, longitude) in sectors.iter_mut().zip(positions) {
            *slot = (rasi[0] + usize::from(house_at(houses.cusps, longitude)?) - 1) % 12;
        }
        Some(sectors)
    });
    let wheels = div()
        .grid()
        .grid_cols(3)
        .gap_3()
        .child(wheel("Rasi", rasi, palette_color, side))
        .child(match bhava {
            Some(bhava) => wheel("Bhava", bhava, palette_color, side),
            None => column()
                .items_center()
                .child("Bhava")
                .child(caption("House cusps unavailable."))
                .into_any_element(),
        })
        .child(wheel("Navamsa", navamsa, palette_color, side));

    column()
        .child(wheels)
        .child(caption("Positions and mansion subdivisions"))
        .child(data_row(
            [
                "Body",
                "Sign",
                "Within sign",
                "Nakshatra",
                "Quarter",
                "Mansion subdivision",
            ]
            .map(str::to_owned),
        ))
        .children(
            positions
                .into_iter()
                .zip(recovered::BODIES)
                .zip(retrograde)
                .map(|((angle, body), is_retrograde)| {
                    let label = if is_retrograde {
                        format!("{body} (R)")
                    } else {
                        body.into()
                    };
                    position_row(label, angle)
                }),
        )
        .when_some(houses, |view, houses| {
            view.child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_4()
                    .child(house_table("House midpoints", houses.midpoints))
                    .child(house_table("House boundaries", houses.cusps)),
            )
        })
        .into_any_element()
}

fn position_row(label: String, angle: f64) -> gpui::Div {
    let [sign, deg, min, sec, _] = original_y0(angle);
    let [mansion, quarter, part, subpart, remainder] = original_x0(angle);
    data_row([
        label,
        (sign + 1).to_string(),
        format!("{deg}°{min:02}′{sec:02}″"),
        name_wheel::mansion_name(mansion as usize)
            .unwrap_or("—")
            .into(),
        (quarter + 1).to_string(),
        format!("{part}:{subpart:02}:{remainder:02}"),
    ])
}

fn house_table(title: &'static str, angles: [f64; 12]) -> gpui::Div {
    column()
        .gap_1()
        .child(title)
        .child(data_row(
            [
                "House",
                "Sign",
                "Within sign",
                "Nakshatra",
                "Quarter",
                "Subdivision",
            ]
            .map(str::to_owned),
        ))
        .children(
            angles
                .into_iter()
                .enumerate()
                .map(|(index, angle)| position_row(format!("H{}", index + 1), angle)),
        )
}

fn wheel(title: &'static str, sectors: [usize; 10], color: u32, side: f32) -> AnyElement {
    let side = side.clamp(135., 216.);
    let marks = canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let at = |angle, radius| {
                let (x, y) = super::radial_point(angle, radius);
                bounds.origin + point(px(x * side), px(y * side))
            };
            let mut path = PathBuilder::stroke(px(1.));
            for radius in [89. / 180., 20. / 180.] {
                path.move_to(at(0., radius));
                for step in 1..=120 {
                    path.line_to(at(step as f32 * 3., radius));
                }
            }
            for sector in 0..12 {
                let angle = sector as f32 * 30.;
                path.move_to(at(angle, 20. / 180.));
                path.line_to(at(angle, 89. / 180.));
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size_full();
    let mut totals = [0_u8; 12];
    for sector in sectors {
        totals[sector] += 1;
    }
    let mut counts = [0_u8; 12];
    let glyphs = sectors.into_iter().zip(CODES).map(|(sector, code)| {
        let count = counts[sector];
        counts[sector] += 1;
        let spacing = (56. / f32::from(totals[sector].saturating_sub(1).max(1))).min(12.);
        let radius = (80. - f32::from(count) * spacing) / 180.;
        let (x, y) = super::radial_point(sector as f32 * 30. + 15., radius);
        div()
            .absolute()
            .left(px(x * side - 11.))
            .top(px(y * side - 7.))
            .w(px(22.))
            .text_center()
            .text_size(px((spacing * side / 180.).clamp(5., 10.)))
            .child(code)
    });
    row()
        .justify_center()
        .border_b_1()
        .border_color(rgb(LINE))
        .pb_2()
        .child(
            div()
                .relative()
                .size(px(side))
                .child(marks)
                .children(glyphs)
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .justify_center()
                        .items_center()
                        .text_size(px(10.))
                        .child(title),
                ),
        )
        .into_any_element()
}
