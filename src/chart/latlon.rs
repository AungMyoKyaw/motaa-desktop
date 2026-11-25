use gpui::{
    AnyElement, Image, ImageFormat, PathBuilder, canvas, div, img, point, prelude::*, px, relative,
    rgb,
};
use motaa_engine::reference_ephemeris::ReferenceEphemeris;
use std::sync::{Arc, OnceLock};

use super::super::{caption, column, data_row};

fn x(longitude: f64) -> f32 {
    ((longitude + 180.).rem_euclid(360.) / 360.) as f32
}

fn y(declination: f64) -> f32 {
    ((136. - declination * 3.4225) / 271.) as f32
}

pub(crate) fn render(
    ephemeris: ReferenceEphemeris,
    ascendant: Option<&ReferenceEphemeris>,
    cusps: Option<[f64; 12]>,
    observer_longitude: f64,
    settings: crate::Settings,
    show_munp: bool,
) -> AnyElement {
    let tropical = settings.tropical;
    let rates = ephemeris.longitude_rates(tropical, settings.ayanamsha);
    static MAP: OnceLock<Arc<Image>> = OnceLock::new();
    let image = MAP.get_or_init(|| {
        Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../../design/research/world-map.png").to_vec(),
        ))
    });
    let equinox = if tropical {
        0.
    } else {
        ephemeris.bodies.first().map_or(0., |body| {
            (body.tropical_longitude_deg - body.sidereal_longitude_deg).rem_euclid(360.)
        })
    };
    let midpoints = ascendant.and_then(|epoch| {
        epoch
            .house_midpoints(
                motaa_engine::houses::System::OPTIONS[settings.houses],
                tropical,
            )
            .ok()
    });
    let shift = midpoints.map_or(0., |values| {
        let pixels = ((values[9] - observer_longitude) * 1200. / 360.).trunc();
        (pixels / 1200.).rem_euclid(1.) as f32
    });
    let points: Vec<_> = ephemeris
        .bodies
        .iter()
        .map(|body| {
            let longitude = if tropical {
                body.tropical_longitude_deg
            } else {
                body.sidereal_longitude_deg
            };
            let declination =
                ephemeris.declination(body.tropical_longitude_deg, body.ecliptic_latitude_deg);
            (body, longitude, declination)
        })
        .collect();
    let height = px(190.);
    let map = div()
        .relative()
        .w_full()
        .h(height)
        .overflow_hidden()
        .children([shift - 1., shift].map(|left| {
            img(image.clone())
                .absolute()
                .left(relative(left))
                .top(px(0.))
                .size_full()
                .object_fit(gpui::ObjectFit::Fill)
        }))
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let at =
                        |x, y| bounds.origin + point(bounds.size.width * x, bounds.size.height * y);
                    if let Some(cusps) = cusps {
                        let mut lines = PathBuilder::stroke(px(0.8));
                        for cusp in cusps {
                            lines.move_to(at(x(cusp), 0.));
                            lines.line_to(at(x(cusp), 1.));
                        }
                        if let Ok(path) = lines.build() {
                            window.paint_path(path, rgb(0x647389));
                        }
                    }
                    let mut curve = PathBuilder::stroke(px(1.2));
                    for step in 0..=240 {
                        let fraction = step as f64 / 240.;
                        let lon = (fraction * 360. - 180. + equinox).to_radians();
                        let pos = at(
                            fraction as f32,
                            ((136. - lon.sin() * 80.42875) / 271.) as f32,
                        );
                        if step == 0 {
                            curve.move_to(pos);
                        } else {
                            curve.line_to(pos);
                        }
                    }
                    if let Ok(path) = curve.build() {
                        window.paint_path(path, rgb(0x28251f));
                    }
                },
            )
            .size_full(),
        )
        .children(points.iter().map(|(body, longitude, declination)| {
            div()
                .absolute()
                .left(relative(x(*longitude)))
                .top(relative(y(*declination)))
                .text_size(px(10.))
                .text_color(rgb(0x28251f))
                .bg(rgb(0xfffefc))
                .rounded(px(3.))
                .px_1()
                .child(body.name.clone())
        }));
    let houses =
        div()
            .relative()
            .w_full()
            .h(px(19.))
            .children(
                cusps
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, cusp)| {
                        caption(format!("H{}", index + 1))
                            .absolute()
                            .left(relative(x(cusp)))
                    }),
            );
    let rate_text = |name: &str| {
        rates
            .as_ref()
            .ok()
            .and_then(|rates| rates.iter().find(|(body, _)| *body == name))
            .map_or_else(|| "—".into(), |(_, rate)| format!("{rate:+.4}"))
    };
    let node_rate = rate_text(if settings.mean_node {
        "Mean node"
    } else {
        "True node"
    });
    let table = column()
        .gap_1()
        .flex_1()
        .min_w(px(0.))
        .child(data_row(
            [
                "Body",
                "Longitude",
                "Declination",
                "Latitude",
                "Motion °/day",
                "Distance",
            ]
            .map(str::to_owned),
        ))
        .children(points.iter().map(|(body, longitude, declination)| {
            data_row([
                body.name.clone(),
                format!("{longitude:.3}°"),
                format!("{declination:.3}°"),
                format!("{:.3}°", body.ecliptic_latitude_deg),
                rate_text(&body.name),
                body.distance_au()
                    .map_or_else(|| "—".into(), |distance| format!("{distance:.5} AU")),
            ])
        }))
        .children(
            ["Rahu", "Ketu"]
                .into_iter()
                .zip(ephemeris.lunar_nodes(settings.mean_node, tropical))
                .zip(ephemeris.lunar_nodes(settings.mean_node, true))
                .map(|((name, longitude), tropical_longitude)| {
                    let declination = ephemeris.declination(tropical_longitude, 0.);
                    data_row([
                        name.into(),
                        format!("{longitude:.3}°"),
                        format!("{declination:.3}°"),
                        "0.000°".into(),
                        node_rate.clone(),
                        "—".into(),
                    ])
                }),
        );
    let inset = ascendant
        .and_then(|epoch| crate::recovered::positions(&ephemeris, epoch, settings))
        .map(|angles| {
            let mut labels: Vec<_> = ["Asc", "1", "2", "3", "4", "5", "6", "0", "8", "9"]
                .into_iter()
                .zip(angles)
                .map(|(label, angle)| (label.into(), angle))
                .collect();
            if show_munp && let Some(epoch) = ascendant {
                labels.extend(crate::chart_view::munp_labels(&ephemeris, epoch, settings));
            }
            super::ChartScene {
                labels,
                title: "Rasi chart",
                circular: false,
                rahu: None,
                houses: None,
                color: crate::INK,
            }
            .render(218.)
        });
    column()
        .gap_1()
        .child(caption(
            "Planetary coordinates · independent astronomical reference",
        ))
        .child(map)
        .child(
            div()
                .relative()
                .w_full()
                .h(px(19.))
                .children(midpoints.into_iter().flat_map(|values| {
                    [(0, "East"), (3, "Below"), (6, "West"), (9, "Above")].map(|(index, text)| {
                        caption(text).absolute().left(relative(x(values[index])))
                    })
                })),
        )
        .child(houses)
        .child(
            crate::row()
                .items_start()
                .gap_4()
                .when_some(inset, |row, inset| row.child(inset))
                .child(table),
        )
        .into_any_element()
}
