use crate::recovered::sign;

pub const SIGNS: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Varga {
    Hora = 2,
    Drekkana,
    Chaturthamsa,
    Panchamsa,
    Shashthamsa,
    Saptamsa,
    Navamsa = 9,
    Dashamsa,
    Dwadashamsa = 12,
}

impl Varga {
    pub const ALL: [Self; 9] = [
        Self::Hora,
        Self::Drekkana,
        Self::Chaturthamsa,
        Self::Panchamsa,
        Self::Shashthamsa,
        Self::Saptamsa,
        Self::Navamsa,
        Self::Dashamsa,
        Self::Dwadashamsa,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Hora => "Hora",
            Self::Drekkana => "Drekkana",
            Self::Chaturthamsa => "Chaturthamsa",
            Self::Panchamsa => "Panchamsa",
            Self::Shashthamsa => "Shashthamsa",
            Self::Saptamsa => "Saptamsa",
            Self::Navamsa => "Navamsa",
            Self::Dashamsa => "Dashamsa",
            Self::Dwadashamsa => "Dwadashamsa",
        }
    }

    /// Parashara conventions except D5 and D6, which follow Narasimha Rao.
    /// Each part is start-inclusive and its longitude is rescaled to 30°.
    pub fn longitude(self, longitude: f64) -> f64 {
        let natal_sign = sign(longitude);
        let order = self as u8;
        let scaled = longitude.rem_euclid(30.) * f64::from(order);
        let part = ((scaled / 30.) as usize).min(usize::from(order) - 1);
        let odd = natal_sign.is_multiple_of(2);
        let mapped = match self {
            Self::Hora => {
                if odd == (part == 0) {
                    4
                } else {
                    3
                }
            }
            Self::Drekkana => natal_sign + 4 * part,
            Self::Chaturthamsa => natal_sign + 3 * part,
            Self::Panchamsa => {
                if odd {
                    [0, 10, 8, 2, 6][part]
                } else {
                    [1, 5, 11, 9, 7][part]
                }
            }
            Self::Shashthamsa => usize::from(!odd) * 6 + part,
            Self::Saptamsa => natal_sign + usize::from(!odd) * 6 + part,
            Self::Navamsa => natal_sign * 9 + part,
            Self::Dashamsa => natal_sign + usize::from(!odd) * 8 + part,
            Self::Dwadashamsa => natal_sign + part,
        };
        (mapped % 12) as f64 * 30. + scaled.rem_euclid(30.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisional_signs_follow_worked_examples_and_half_open_boundaries() {
        for (varga, input, expected) in [
            (Varga::Hora, 0., 4),
            (Varga::Hora, 15., 3),
            (Varga::Hora, 30., 3),
            (Varga::Hora, 45., 4),
            (Varga::Drekkana, 140., 0),
            (Varga::Chaturthamsa, 7.5, 3),
            (Varga::Panchamsa, 256., 8),
            (Varga::Panchamsa, 226., 11),
            (Varga::Shashthamsa, 71., 2),
            (Varga::Shashthamsa, 229., 9),
            (Varga::Shashthamsa, 34.999, 6),
            (Varga::Shashthamsa, 35., 7),
            (Varga::Saptamsa, 30., 7),
            (Varga::Navamsa, 30., 9),
            (Varga::Dashamsa, 30., 9),
            (Varga::Dwadashamsa, 32.5, 2),
        ] {
            assert_eq!(
                sign(varga.longitude(input)),
                expected,
                "{} at {input}",
                varga.name()
            );
        }
    }
}
