//! Reference ecliptic cusps using apparent sidereal time and true obliquity.
use std::f64::consts::{FRAC_PI_2, PI};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    Equal,
    VehlowPlacidus,
    Placidus,
    Koch,
    Porphyry,
    Regiomontanus,
    Campanus,
    VehlowEqual,
}

impl System {
    pub const OPTIONS: [Self; 8] = [
        Self::Equal,
        Self::VehlowPlacidus,
        Self::Placidus,
        Self::Koch,
        Self::Porphyry,
        Self::Regiomontanus,
        Self::Campanus,
        Self::VehlowEqual,
    ];
}

pub struct Angles {
    pub ascendant: f64,
    pub midheaven: f64,
    ramc: f64,
    latitude: f64,
    obliquity: f64,
}

fn norm(angle: f64) -> f64 {
    angle.rem_euclid(360.)
}

fn project(ra: f64, pole: f64, obliquity: f64) -> f64 {
    norm(
        ra.sin()
            .atan2(ra.cos() * obliquity.cos() - pole.tan() * obliquity.sin())
            .to_degrees(),
    )
}

pub fn angles(jdut: f64, latitude: f64, longitude: f64) -> Angles {
    let t = (jdut - 2451545.) / 36525.;
    let (nutation_longitude, nutation_obliquity) = astro::nutation::nutation(jdut);
    let obliquity = (23.439291111 - 0.013004167 * t).to_radians() + nutation_obliquity;
    let equation_of_equinoxes = (nutation_longitude * obliquity.cos()).to_degrees();
    let ramc = norm(
        280.46061837 + 360.98564736629 * (jdut - 2451545.) + 0.000387933 * t * t
            - t * t * t / 38710000.
            + equation_of_equinoxes
            + longitude,
    )
    .to_radians();
    let latitude = latitude.to_radians();
    Angles {
        ascendant: project(ramc + FRAC_PI_2, latitude, obliquity),
        midheaven: project(ramc, 0., obliquity),
        ramc,
        latitude,
        obliquity,
    }
}

impl Angles {
    pub fn cusps(&self, system: System) -> Result<[f64; 12], String> {
        let asc = self.ascendant;
        let mc = self.midheaven;
        if matches!(system, System::Equal | System::VehlowEqual) {
            let start = if system == System::VehlowEqual {
                asc - 15.
            } else {
                asc
            };
            return Ok(std::array::from_fn(|index| {
                norm(start + index as f64 * 30.)
            }));
        }
        // The original Swiss house routine falls back to Porphyry when
        // Placidus or Koch is requested beyond the polar circles.
        let system = if matches!(system, System::Placidus | System::Koch)
            && self.latitude.abs() + self.obliquity >= FRAC_PI_2
        {
            System::Porphyry
        } else {
            system
        };
        let q1 = norm(asc - mc);
        if system == System::VehlowPlacidus {
            // MOTAA AbstractC0055z.a: divide ASC→IC and IC→DSC into six
            // intervals each, then select the odd boundaries around the wheel.
            return Ok(std::array::from_fn(|index| {
                self.vehlow_placidus_point((2 * index + 23) % 24)
            }));
        }

        let mut output = [0.; 12];
        output[0] = asc;
        output[9] = mc;
        for (ordinal, (index, offset)) in [(10, 30_f64), (11, 60.), (1, 120.), (2, 150.)]
            .into_iter()
            .enumerate()
        {
            let h = offset.to_radians();
            output[index] = match system {
                System::Porphyry => {
                    let (start, width, step) = if ordinal < 2 {
                        (mc, q1, ordinal + 1)
                    } else {
                        (asc, 180. - q1, ordinal - 1)
                    };
                    norm(start + width * step as f64 / 3.)
                }
                System::Regiomontanus => {
                    let pole = (self.latitude.tan() * h.sin()).atan();
                    project(self.ramc + h, pole, self.obliquity)
                }
                System::Campanus => {
                    let pole = (self.latitude.sin() * h.sin()).asin();
                    let hour_angle = (self.latitude.cos() * h.sin()).atan2(h.cos());
                    project(self.ramc + hour_angle, pole, self.obliquity)
                }
                System::Koch => {
                    let declination = (mc.to_radians().sin() * self.obliquity.sin()).asin();
                    let difference = (self.latitude.tan() * declination.tan()).asin() / 3.;
                    let factor = [-2., -1., 1., 2.][ordinal];
                    project(
                        self.ramc + h + factor * difference,
                        self.latitude,
                        self.obliquity,
                    )
                }
                System::Placidus => self.placidus(ordinal, h)?,
                _ => unreachable!("handled equal and MOTAA-specific systems"),
            };
        }
        for index in [0, 1, 2, 9, 10, 11] {
            output[(index + 6) % 12] = norm(output[index] + 180.);
        }
        Ok(output)
    }

    pub fn house_midpoints(&self, system: System) -> Result<[f64; 12], String> {
        if system == System::VehlowPlacidus {
            return Ok(std::array::from_fn(|index| {
                self.vehlow_placidus_point(2 * index)
            }));
        }
        self.cusps(system).map(midpoints)
    }

    fn vehlow_placidus_point(&self, index: usize) -> f64 {
        let quadrant = norm(self.ascendant - self.midheaven);
        let half = index % 12;
        let arc = (half.min(6) as f64 * (180. - quadrant)
            + half.saturating_sub(6) as f64 * quadrant)
            / 6.;
        norm(self.ascendant + arc + if index >= 12 { 180. } else { 0. })
    }

    fn placidus(&self, ordinal: usize, h: f64) -> Result<f64, String> {
        let offset = if ordinal < 2 {
            h / FRAC_PI_2
        } else {
            (PI - h) / FRAC_PI_2
        };
        let mut ra = self.ramc + h;
        for _ in 0..80 {
            let longitude = project(ra, 0., self.obliquity);
            let dec = (longitude.to_radians().sin() * self.obliquity.sin()).asin();
            let difference = (self.latitude.tan() * dec.tan()).asin();
            let next = self.ramc + h + offset * difference;
            if (next - ra).abs() < 1e-11 {
                return Ok(project(next, 0., self.obliquity));
            }
            ra = next;
        }
        Err("Placidus cusp did not converge at this location and time.".into())
    }
}

pub fn midpoints(cusps: [f64; 12]) -> [f64; 12] {
    std::array::from_fn(|index| {
        norm(cusps[index] + norm(cusps[(index + 1) % 12] - cusps[index]) * 0.5)
    })
}

pub fn house_at(cusps: [f64; 12], longitude: f64) -> Option<u8> {
    (0..12)
        .find(|&index| {
            norm(longitude - cusps[index]) < norm(cusps[(index + 1) % 12] - cusps[index])
        })
        .map(|index| index as u8 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn difference(left: f64, right: f64) -> f64 {
        (left - right + 180.).rem_euclid(360.) - 180.
    }

    #[test]
    fn ascendant_and_seven_standard_house_systems_match_swiss_reference() {
        type Case = (f64, f64, f64, char, f64, f64, [f64; 12]);
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../data/reference-house-oracle.json")).unwrap();
        assert_eq!(cases.len(), 35);
        for (jd, lat, lon, mode, expected_asc, expected_mc, expected) in cases {
            let system = match mode {
                'E' => System::Equal,
                'P' => System::Placidus,
                'K' => System::Koch,
                'O' => System::Porphyry,
                'R' => System::Regiomontanus,
                'C' => System::Campanus,
                'V' => System::VehlowEqual,
                _ => panic!("Unknown fixture system"),
            };
            let angles = angles(jd, lat, lon);
            assert!(difference(angles.ascendant, expected_asc).abs() < 0.02);
            assert!(difference(angles.midheaven, expected_mc).abs() < 0.02);
            for (index, (observed, expected)) in angles
                .cusps(system)
                .unwrap()
                .into_iter()
                .zip(expected)
                .enumerate()
            {
                assert!(
                    difference(observed, expected).abs() < 0.04,
                    "{mode} {}: {observed} vs {expected}, {jd} at {lat}, {lon}",
                    index + 1
                );
            }
        }
    }

    #[test]
    fn motaa_vehlow_bisects_around_ascendant_and_ic() {
        let angles = angles(2451545., 16.84, 96.17);
        let cusps = angles.cusps(System::VehlowPlacidus).unwrap();
        let centers_from_source = angles.house_midpoints(System::VehlowPlacidus).unwrap();
        assert_eq!(centers_from_source[0], angles.ascendant);
        assert!(difference(centers_from_source[9], angles.midheaven).abs() < 1e-10);
        let centers = midpoints(cusps);
        assert!(difference(centers[0], angles.ascendant).abs() <= 5.);
        assert!(difference(cusps[6], cusps[0] + 180.).abs() < 1e-10);
        assert_eq!(house_at(cusps, angles.ascendant), Some(1));
    }
}
