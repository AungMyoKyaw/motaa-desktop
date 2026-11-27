use gpui::{AnyElement, div, prelude::*, px, rgb};
use motaa_engine::reference_ephemeris::ReferenceBody;

pub struct Projection {
    pub azimuth: f64,
    pub tilted: bool,
}

impl Projection {
    /// Project a unit vector in ecliptic coordinates. The local horizon and
    /// physical distances are intentionally outside this reference view.
    fn point(&self, longitude: f64, latitude: f64) -> [f64; 3] {
        let longitude = (longitude - self.azimuth).to_radians();
        let latitude = latitude.to_radians();
        let x = latitude.cos() * longitude.cos();
        let y = latitude.cos() * longitude.sin();
        let z = latitude.sin();
        if self.tilted {
            let tilt = 56_f64.to_radians();
            [
                x,
                y * tilt.cos() - z * tilt.sin(),
                y * tilt.sin() + z * tilt.cos(),
            ]
        } else {
            [x * (1. - z * 0.35), y * (1. - z * 0.35), z]
        }
    }

    pub fn render(&self, bodies: &[ReferenceBody]) -> AnyElement {
        let side = 410.;
        let mut vertices: Vec<_> = bodies
            .iter()
            .map(|body| {
                (
                    body.name.as_str(),
                    self.point(body.tropical_longitude_deg, body.ecliptic_latitude_deg),
                )
            })
            .collect();
        vertices.sort_by(|a, b| a.1[2].total_cmp(&b.1[2]));
        let plot = div()
            .relative()
            .size(px(side))
            .bg(rgb(crate::PAPER))
            .overflow_hidden();
        let grid = (0..72).map(|tick| {
            let [x, y, _] = self.point(f64::from(tick) * 5., 0.);
            marker(x, y, side)
                .size(px(3.))
                .rounded_full()
                .bg(rgb(crate::LINE))
        });
        let nodes = vertices.into_iter().map(|(name, [x, y, _])| {
            marker(x, y, side)
                .flex()
                .items_center()
                .gap_1()
                .child(div().size(px(7.)).rounded_full().bg(rgb(crate::GOLD)))
                .child(
                    div()
                        .text_size(px(11.))
                        .bg(rgb(crate::PAPER))
                        .child(name.to_owned()),
                )
        });
        plot.children(grid).children(nodes).into_any_element()
    }
}

fn marker(x: f64, y: f64, side: f32) -> gpui::Div {
    div()
        .absolute()
        .left(px(side * (0.45 + x as f32 * 0.33)))
        .top(px(side * (0.48 + y as f32 * 0.33)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilted_projection_preserves_unit_vector_length() {
        let projection = Projection {
            azimuth: 35.,
            tilted: true,
        };
        let point = projection.point(90., -14.);
        assert!((point.iter().map(|value| value.powi(2)).sum::<f64>() - 1.).abs() < 1e-12);
    }
}
