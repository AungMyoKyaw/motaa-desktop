//! Desktop astronomical reference with V31.1 time and sidereal conventions.
//! VSOP87/ELP2000-derived positions via astro 2.0.0, MIT; documented in
//! third_party/astro-LICENSE.md. Intended for dates 1800-2100 inclusive.
use astro::{lunar, nutation, planet, pluto, precess, sun, time};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceBody {
    pub name: String,
    pub tropical_longitude_deg: f64,
    pub sidereal_longitude_deg: f64,
    pub ecliptic_latitude_deg: f64,
    pub distance: f64,
    pub distance_unit: String,
}
impl ReferenceBody {
    pub fn distance_au(&self) -> Option<f64> {
        match self.distance_unit.as_str() {
            "AU" => Some(self.distance),
            "km" => Some(self.distance / 149_597_870.7),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceAspect {
    pub first: String,
    pub second: String,
    pub kind: String,
    pub exact_angle_deg: f64,
    pub separation_deg: f64,
    pub orb_deg: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceEphemeris {
    pub julian_ephemeris_day: f64,
    #[serde(default)]
    observation: Option<Observation>,
    #[serde(default)]
    sidereal_offset_deg: Option<f64>,
    pub delta_t_seconds: f64,
    pub bodies: Vec<ReferenceBody>,
    pub estimated_lahiri_ayanamsha_deg: f64,
    pub ascendant_tropical_deg: f64,
    pub ascendant_sidereal_deg: f64,
    pub equal_house_cusps_tropical_deg: Vec<f64>,
    pub equal_house_cusps_sidereal_deg: Vec<f64>,
    pub aspects: Vec<ReferenceAspect>,
    pub source: String,
    pub limitations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Observation {
    jdut: f64,
    latitude: f64,
    longitude: f64,
}

fn norm(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}
fn add_body(
    items: &mut Vec<ReferenceBody>,
    name: &str,
    longitude: f64,
    latitude: f64,
    distance: f64,
    unit: &str,
    aya: f64,
) {
    items.push(ReferenceBody {
        name: name.into(),
        tropical_longitude_deg: norm(longitude.to_degrees()),
        sidereal_longitude_deg: norm(longitude.to_degrees() - aya),
        ecliptic_latitude_deg: norm(latitude.to_degrees() + 180.) - 180.,
        distance,
        distance_unit: unit.into(),
    });
}
fn aspect_pairs(bodies: &[ReferenceBody]) -> Vec<ReferenceAspect> {
    let mut found = vec![];
    for (i, a) in bodies.iter().enumerate() {
        for b in &bodies[i + 1..] {
            let difference = (a.tropical_longitude_deg - b.tropical_longitude_deg).abs();
            let separation = difference.min(360.0 - difference);
            for (name, angle, orb) in [
                ("Conjunction", 0.0_f64, 8.0_f64),
                ("Sextile", 60.0, 5.0),
                ("Square", 90.0, 6.0),
                ("Trine", 120.0, 6.0),
                ("Opposition", 180.0, 8.0),
            ] {
                if (separation - angle).abs() <= orb {
                    found.push(ReferenceAspect {
                        first: a.name.clone(),
                        second: b.name.clone(),
                        kind: name.into(),
                        exact_angle_deg: angle,
                        separation_deg: separation,
                        orb_deg: (separation - angle).abs(),
                    });
                    break;
                }
            }
        }
    }
    found
}
// Monthly samples from the V31.1 runtime, taken at Gregorian midnight UT.
fn android_delta_t(jdut: f64) -> Option<f64> {
    static MONTHS: std::sync::OnceLock<Vec<(f64, f64)>> = std::sync::OnceLock::new();
    let months = MONTHS.get_or_init(|| {
        include_str!("../data/motaa-v31.1-delta-t.csv")
            .lines()
            .map(|line| {
                let (julian_day, seconds) = line.split_once(',').expect("Delta T sample");
                (
                    julian_day.parse().expect("sample date"),
                    seconds.parse().expect("sample seconds"),
                )
            })
            .collect()
    });
    let index = months
        .partition_point(|&(date, _)| date <= jdut)
        .checked_sub(1)?;
    let (date, seconds) = months[index];
    if jdut == date {
        return Some(seconds);
    }
    let &(next_date, next_seconds) = months.get(index + 1)?;
    Some(seconds + (next_seconds - seconds) * (jdut - date) / (next_date - date))
}

pub(crate) fn reference_delta_t(jdut: f64, year: i32, month: u8) -> f64 {
    android_delta_t(jdut).unwrap_or_else(|| time::delta_t(year, month))
}

// Annual aberration shifts a direction toward the observer's orbital velocity.
fn annual_aberration(longitude: f64, latitude: f64, velocity_over_c: [f64; 3]) -> (f64, f64) {
    let (sin_lon, cos_lon) = longitude.sin_cos();
    let (sin_lat, cos_lat) = latitude.sin_cos();
    let [x, y, z] = velocity_over_c;
    let longitude_shift = (-x * sin_lon + y * cos_lon) / cos_lat;
    let latitude_shift = -(x * cos_lon + y * sin_lon) * sin_lat + z * cos_lat;
    (longitude + longitude_shift, latitude + latitude_shift)
}

fn earth_velocity_over_c(jde: f64) -> [f64; 3] {
    let cartesian = |day| {
        let (lon, lat, radius) = planet::heliocent_coords(&planet::Planet::Earth, day);
        [
            radius * lat.cos() * lon.cos(),
            radius * lat.cos() * lon.sin(),
            radius * lat.sin(),
        ]
    };
    let before = cartesian(jde - 0.005);
    let after = cartesian(jde + 0.005);
    std::array::from_fn(|index| (after[index] - before[index]) / (0.01 * 173.144632674))
}

pub fn calculate(
    jdut: f64,
    year: i32,
    month: u8,
    latitude: f64,
    longitude: f64,
) -> Result<ReferenceEphemeris, String> {
    if !(1800..=2100).contains(&year) {
        return Err(
            "Reference ephemeris supported for 1800–2100; original APK calendar still available"
                .into(),
        );
    }
    if !jdut.is_finite() || !latitude.is_finite() || !longitude.is_finite() {
        return Err("Invalid reference ephemeris inputs".into());
    }
    let delta = reference_delta_t(jdut, year, month);
    let jde = jdut + delta / 86400.0;
    let aya =
        crate::ayanamsha::android_apparent(jde, 0).ok_or("Reference ayanamsha outside range.")?;
    let nutation = nutation::nutation(jde).0;
    let mut bodies = vec![];
    let (sun_pos, sun_distance) = sun::geocent_ecl_pos(jde);
    add_body(
        &mut bodies,
        "Sun",
        sun_pos.long + nutation - (20.4898_f64 / (3600. * sun_distance)).to_radians(),
        sun_pos.lat,
        sun_distance,
        "AU",
        aya,
    );
    let (moon_pos, moon_distance) = lunar::geocent_ecl_pos(jde);
    add_body(
        &mut bodies,
        "Moon",
        moon_pos.long + nutation,
        moon_pos.lat,
        moon_distance,
        "km",
        aya,
    );
    let velocity = earth_velocity_over_c(jde);
    for (name, p) in [
        ("Mercury", planet::Planet::Mercury),
        ("Venus", planet::Planet::Venus),
        ("Mars", planet::Planet::Mars),
        ("Jupiter", planet::Planet::Jupiter),
        ("Saturn", planet::Planet::Saturn),
        ("Uranus", planet::Planet::Uranus),
        ("Neptune", planet::Planet::Neptune),
    ] {
        let (pos, dist) = planet::geocent_apprnt_ecl_coords(&p, jde);
        let (longitude, latitude) = annual_aberration(pos.long, pos.lat, velocity);
        add_body(
            &mut bodies,
            name,
            longitude + nutation,
            latitude,
            dist,
            "AU",
            aya,
        );
    }
    // The Pluto series is referred to J2000. Transform it to the Earth's
    // ecliptic of date before subtracting the heliocentric vectors.
    let (earth_long, earth_lat, earth_r) = planet::heliocent_coords(&planet::Planet::Earth, jde);
    let pluto_geocentric = |time| {
        let (pluto_long, pluto_lat, pluto_r) = pluto::heliocent_pos(time);
        let (pluto_long, pluto_lat) =
            precess::precess_ecl_coords(pluto_long, pluto_lat, 2451545., jde);
        planet::geocent_geomet_ecl_coords(
            earth_long, earth_lat, earth_r, pluto_long, pluto_lat, pluto_r,
        )
    };
    let light_time = pluto_geocentric(jde).3;
    let (pluto_long, pluto_lat, pluto_distance, _) = pluto_geocentric(jde - light_time);
    let (pluto_long, pluto_lat) = annual_aberration(pluto_long, pluto_lat, velocity);
    add_body(
        &mut bodies,
        "Pluto",
        pluto_long + nutation,
        pluto_lat,
        pluto_distance,
        "AU",
        aya,
    );
    if bodies
        .iter()
        .any(|b| !b.tropical_longitude_deg.is_finite() || !b.distance.is_finite())
    {
        return Err("Astronomy library returned invalid coordinates".into());
    }
    let asc = crate::houses::angles(jdut, latitude, longitude).ascendant;
    let tropical = (0..12)
        .map(|i| norm(asc + i as f64 * 30.0))
        .collect::<Vec<_>>();
    let sidereal = tropical.iter().map(|d| norm(d - aya)).collect::<Vec<_>>();
    let aspects = aspect_pairs(&bodies);
    Ok(ReferenceEphemeris{
        julian_ephemeris_day:jde,delta_t_seconds:delta,
        observation: Some(Observation { jdut, latitude, longitude }),
        sidereal_offset_deg: Some(aya),
        bodies,estimated_lahiri_ayanamsha_deg:aya,
        ascendant_tropical_deg:asc,ascendant_sidereal_deg:norm(asc-aya),
        equal_house_cusps_tropical_deg:tropical,equal_house_cusps_sidereal_deg:sidereal,
        aspects,
        source:"Analytic astronomy with V31.1 Delta T and sidereal conventions.".into(),
        limitations:vec![
         "Planet longitudes, lunar nodes and house angles are checked against sampled V31.1 outputs.".into(),
         "The analytic lunar state can shift true-node longitudes and fractional Dasha timing.".into(),
         "The Pluto position series is published for 1885–2099.".into(),
        ],
    })
}
impl ReferenceEphemeris {
    pub fn select_ayanamsha(&mut self, mode: usize) -> Result<(), String> {
        if self.observation.is_none() {
            return Err("Chart observation unavailable.".into());
        }
        let offset = crate::ayanamsha::android_apparent(self.julian_ephemeris_day, mode)
            .ok_or("Unsupported ayanamsha.")?;
        self.sidereal_offset_deg = Some(offset);
        for body in &mut self.bodies {
            body.sidereal_longitude_deg = norm(body.tropical_longitude_deg - offset);
        }
        self.ascendant_sidereal_deg = norm(self.ascendant_tropical_deg - offset);
        self.equal_house_cusps_sidereal_deg = self
            .equal_house_cusps_tropical_deg
            .iter()
            .map(|&longitude| norm(longitude - offset))
            .collect();
        Ok(())
    }

    fn house_angles(
        &self,
        tropical: bool,
        select: impl FnOnce(crate::houses::Angles) -> Result<[f64; 12], String>,
    ) -> Result<[f64; 12], String> {
        let sample = self
            .observation
            .as_ref()
            .ok_or("Chart observation unavailable.")?;
        let values = select(crate::houses::angles(
            sample.jdut,
            sample.latitude,
            sample.longitude,
        ))?;
        Ok(if tropical {
            values
        } else {
            values.map(|angle| norm(angle - self.sidereal_offset()))
        })
    }

    pub fn cusps(
        &self,
        system: crate::houses::System,
        tropical: bool,
    ) -> Result<[f64; 12], String> {
        self.house_angles(tropical, |angles| angles.cusps(system))
    }

    pub fn house_midpoints(
        &self,
        system: crate::houses::System,
        tropical: bool,
    ) -> Result<[f64; 12], String> {
        self.house_angles(tropical, |angles| angles.house_midpoints(system))
    }

    /// Centered numerical derivative of the selected zodiac longitude, in degrees per day.
    pub fn longitude_rates(
        &self,
        tropical: bool,
        ayanamsha: usize,
    ) -> Result<Vec<(&str, f64)>, String> {
        let observation = self
            .observation
            .as_ref()
            .ok_or("Chart observation unavailable.")?;
        // Keep fallback Delta T samples in the same month at the supported
        // range edges, where an instant can fall outside the Android table.
        let (year, month, _) =
            time::date_frm_julian_day(observation.jdut).map_err(str::to_owned)?;
        let sample = |jd: f64| {
            let mut ephemeris = calculate(
                jd,
                i32::from(year),
                month,
                observation.latitude,
                observation.longitude,
            )?;
            if !tropical {
                ephemeris.select_ayanamsha(ayanamsha)?;
            }
            Ok::<_, String>(ephemeris)
        };
        const HALF_INTERVAL: f64 = 1. / 1440.;
        let before_jd = observation.jdut - HALF_INTERVAL;
        let after_jd = observation.jdut + HALF_INTERVAL;
        let before = sample(before_jd)?;
        let after = sample(after_jd)?;
        let rate = |start: f64, end: f64| {
            let change = (end - start + 180.).rem_euclid(360.) - 180.;
            change / (after_jd - before_jd)
        };
        let mut rates: Vec<_> = self
            .bodies
            .iter()
            .filter_map(|body| {
                let start = before.longitude(&body.name, tropical)?;
                let end = after.longitude(&body.name, tropical)?;
                Some((body.name.as_str(), rate(start, end)))
            })
            .collect();
        for (label, mean) in [("True node", false), ("Mean node", true)] {
            rates.push((
                label,
                rate(
                    before.lunar_nodes(mean, tropical)[0],
                    after.lunar_nodes(mean, tropical)[0],
                ),
            ));
        }
        Ok(rates)
    }

    /// Equatorial declination from the apparent tropical ecliptic coordinates.
    pub fn declination(&self, tropical_longitude: f64, latitude: f64) -> f64 {
        let t = (self.julian_ephemeris_day - 2451545.) / 36525.;
        let obliquity = (23.439291111 - 0.013004167 * t).to_radians()
            + nutation::nutation(self.julian_ephemeris_day).1;
        let latitude = latitude.to_radians();
        (latitude.sin() * obliquity.cos()
            + latitude.cos() * obliquity.sin() * tropical_longitude.to_radians().sin())
        .clamp(-1., 1.)
        .asin()
        .to_degrees()
    }

    pub fn midheaven(&self, tropical: bool) -> Option<f64> {
        let sample = self.observation.as_ref()?;
        let angle = crate::houses::angles(sample.jdut, sample.latitude, sample.longitude).midheaven;
        Some(if tropical {
            angle
        } else {
            norm(angle - self.sidereal_offset())
        })
    }

    fn sidereal_offset(&self) -> f64 {
        self.sidereal_offset_deg
            .unwrap_or(self.estimated_lahiri_ayanamsha_deg)
    }

    pub fn longitude(&self, name: &str, tropical: bool) -> Option<f64> {
        self.bodies
            .iter()
            .find(|body| body.name == name)
            .map(|body| {
                if tropical {
                    body.tropical_longitude_deg
                } else {
                    body.sidereal_longitude_deg
                }
            })
    }

    pub fn lunar_nodes(&self, mean: bool, tropical: bool) -> [f64; 2] {
        let century = time::julian_cent(self.julian_ephemeris_day);
        let node = if mean {
            lunar::mn_ascend_node(century) + nutation::nutation(self.julian_ephemeris_day).0
        } else {
            osculating_node(self.julian_ephemeris_day)
        };
        let offset = if tropical { 0. } else { self.sidereal_offset() };
        let rahu = norm(node.to_degrees() - offset);
        [rahu, norm(rahu + 180.)]
    }

    pub fn heliocentric_positions(&self) -> Vec<(&'static str, f64, f64, f64)> {
        Self::heliocentric_at(self.julian_ephemeris_day)
    }

    pub fn heliocentric_longitude_rates(&self) -> Vec<(&'static str, f64)> {
        let jde = self.julian_ephemeris_day;
        Self::heliocentric_at(jde - 0.5)
            .into_iter()
            .zip(Self::heliocentric_at(jde + 0.5))
            .map(|((name, before, _, _), (_, after, _, _))| {
                (name, (after - before + 180.).rem_euclid(360.) - 180.)
            })
            .collect()
    }

    /// Sample a year of ecliptic positions surrounding the chart epoch.
    pub fn heliocentric_year(&self) -> [[(f64, f64); 9]; 49] {
        std::array::from_fn(|index| {
            let offset = (index as f64 - 24.) * (365.25 / 48.);
            let positions = Self::heliocentric_at(self.julian_ephemeris_day + offset);
            std::array::from_fn(|body| {
                let (_, longitude, latitude, radius) = positions[body];
                let angle = longitude.to_radians();
                let projected_radius = radius * latitude.to_radians().cos();
                (
                    projected_radius * angle.cos(),
                    projected_radius * angle.sin(),
                )
            })
        })
    }

    fn heliocentric_at(jde: f64) -> Vec<(&'static str, f64, f64, f64)> {
        use planet::Planet;
        [
            ("Mercury", Planet::Mercury),
            ("Venus", Planet::Venus),
            ("Earth", Planet::Earth),
            ("Mars", Planet::Mars),
            ("Jupiter", Planet::Jupiter),
            ("Saturn", Planet::Saturn),
            ("Uranus", Planet::Uranus),
            ("Neptune", Planet::Neptune),
        ]
        .into_iter()
        .map(|(name, planet)| {
            let (longitude, latitude, distance) = planet::heliocent_coords(&planet, jde);
            (
                name,
                norm(longitude.to_degrees()),
                latitude.to_degrees(),
                distance,
            )
        })
        .chain(std::iter::once_with(|| {
            let (longitude, latitude, distance) = pluto::heliocent_pos(jde);
            let (longitude, latitude) =
                precess::precess_ecl_coords(longitude, latitude, 2451545., jde);
            (
                "Pluto",
                norm(longitude.to_degrees()),
                latitude.to_degrees(),
                distance,
            )
        }))
        .collect()
    }
}

fn osculating_node(jde: f64) -> f64 {
    let moon_vector = |time| {
        let (position, distance) = lunar::geocent_ecl_pos(time);
        [
            distance * position.lat.cos() * position.long.cos(),
            distance * position.lat.cos() * position.long.sin(),
            distance * position.lat.sin(),
        ]
    };
    let position = moon_vector(jde);
    let before = moon_vector(jde - 0.001);
    let after = moon_vector(jde + 0.001);
    let velocity: [f64; 3] = std::array::from_fn(|index| after[index] - before[index]);
    // z-axis crossed with the lunar orbital normal gives the ascending node.
    let hx = position[1] * velocity[2] - position[2] * velocity[1];
    let hy = position[2] * velocity[0] - position[0] * velocity[2];
    hx.atan2(-hy) + nutation::nutation(jde).0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lunar_nodes_are_opposite_and_apply_selected_zodiac() {
        let ephemeris = calculate(2451545., 2000, 1, 16.8409, 96.1735).unwrap();
        let [rahu, ketu] = ephemeris.lunar_nodes(true, true);
        let [sidereal, _] = ephemeris.lunar_nodes(true, false);
        assert!((rahu - 125.0445).abs() < 0.01);
        assert!(((ketu - rahu).rem_euclid(360.) - 180.).abs() < 1e-10);
        assert!((rahu - sidereal - ephemeris.estimated_lahiri_ayanamsha_deg).abs() < 1e-10);
        assert_ne!(ephemeris.lunar_nodes(false, true), [rahu, ketu]);
        let heliocentric = ephemeris.heliocentric_positions();
        assert_eq!(heliocentric.len(), 9);
        assert!((heliocentric[2].3 - 1.).abs() < 0.03);
    }

    #[test]
    fn heliocentric_rates_preserve_body_order_and_earths_daily_motion() {
        let chart = calculate(2451545., 2000, 1, 16.8409, 96.1735).unwrap();
        let rates = chart.heliocentric_longitude_rates();
        let positions = chart.heliocentric_positions();
        assert_eq!(rates.len(), positions.len());
        for ((body, rate), (position_body, _, _, _)) in rates.iter().zip(positions) {
            assert_eq!(*body, position_body);
            assert!(rate.is_finite());
        }
        assert_eq!(rates[2].0, "Earth");
        assert!((0.98..1.04).contains(&rates[2].1));
    }

    #[test]
    fn orbit_plot_marks_the_selected_epoch() {
        let chart = calculate(2451545., 2000, 1, 16.8409, 96.1735).unwrap();
        let orbit = chart.heliocentric_year();
        for ((x, y), (_, longitude, latitude, radius)) in
            orbit[24].into_iter().zip(chart.heliocentric_positions())
        {
            let angle = longitude.to_radians();
            let radius = radius * latitude.to_radians().cos();
            assert!((x - radius * angle.cos()).abs() < 1e-12);
            assert!((y - radius * angle.sin()).abs() < 1e-12);
        }
        assert_ne!(orbit[0], orbit[48]);
    }

    #[test]
    fn j2000_sun_reference() {
        let x = calculate(2451545.0, 2000, 1, 16.8409, 96.1735).unwrap();
        assert_eq!(x.bodies.len(), 10);
        let sun = x.bodies.iter().find(|b| b.name == "Sun").unwrap();
        assert!(
            (sun.tropical_longitude_deg - 280.37).abs() < 0.5,
            "{}",
            sun.tropical_longitude_deg
        );
        assert!(
            x.bodies
                .iter()
                .all(|b| b.tropical_longitude_deg >= 0.0 && b.tropical_longitude_deg < 360.0)
        );
        assert_eq!(x.equal_house_cusps_tropical_deg.len(), 12);
    }
    #[test]
    fn houses_regular() {
        let x = calculate(2460000.5, 2023, 2, 13.75, 100.5).unwrap();
        for w in x.equal_house_cusps_tropical_deg.windows(2) {
            assert!((w[1] - w[0]).rem_euclid(360.0) > 29.9)
        }
    }
    #[test]
    fn rejects_unsupported_eras() {
        assert!(calculate(100000.0, 1500, 1, 0.0, 0.0).is_err())
    }
    #[test]
    fn aspect_exact() {
        let bodies = vec![
            ReferenceBody {
                name: "A".into(),
                tropical_longitude_deg: 0.0,
                sidereal_longitude_deg: 0.0,
                ecliptic_latitude_deg: 0.0,
                distance: 1.0,
                distance_unit: "AU".into(),
            },
            ReferenceBody {
                name: "B".into(),
                tropical_longitude_deg: 180.0,
                sidereal_longitude_deg: 0.0,
                ecliptic_latitude_deg: 0.0,
                distance: 1.0,
                distance_unit: "AU".into(),
            },
        ];
        assert_eq!(aspect_pairs(&bodies)[0].kind, "Opposition");
    }
}
