use jiff::civil::{Date, DateTime};
use motaa_engine::{
    ChartRequest, dasha, original_colors::original_c0, original_strengths::original_d0,
};

use crate::{ashtakavarga, model, model::Settings, recovered, ten_pillars};

#[derive(Debug)]
pub struct Sample {
    pub age: u8,
    pub when: DateTime,
    pub sign_class: u8,
    pub mansion_class: i32,
    pub natal_points: u8,
    pub maitri: &'static str,
    pub combined_score: i32,
    pub lord: &'static str,
    pub major_index: Option<usize>,
}

pub struct Timeline {
    pub name: String,
    pub body: &'static str,
    pub samples: Vec<Sample>,
    pub last_age: u8,
    pub ayanamsha: usize,
    pub dasha: dasha::Timeline,
}

pub fn calculate(
    profile: &ChartRequest,
    mut settings: Settings,
    body: usize,
) -> Result<Timeline, String> {
    if profile.unknown_birth_time {
        return Err("The lifetime timeline requires a known natal time.".into());
    }
    settings.tropical = false;
    let year = i16::try_from(profile.year).map_err(|_| "Invalid birth date for timeline.")?;
    let birth = Date::new(year, profile.month as i8, profile.day as i8)
        .map(|date| {
            date.at(
                profile.hour as i8,
                profile.minute as i8,
                profile.second as i8,
                0,
            )
        })
        .map_err(|_| "Invalid birth date for timeline.")?;
    let mut natal = motaa_engine::calculate(profile)?
        .reference_ephemeris
        .ok_or("Birth date is outside the supported ephemeris range.")?;
    natal.select_ayanamsha(settings.ayanamsha)?;
    let natal_positions =
        recovered::positions(&natal, &natal, settings).ok_or("Natal positions unavailable.")?;
    let dasha = dasha::calculate(profile, natal_positions[2])?;
    let (points, _) = ashtakavarga::calculate(std::array::from_fn(|index| {
        recovered::sign(natal_positions[(index + 1) % 8])
    }));
    let mut samples = Vec::new();
    for age in 0..=95_u8 {
        let when = birth
            .checked_add(jiff::Span::new().months(i32::from(age) * 12))
            .map_err(|_| "Invalid birth anniversary.")?;
        if when.year() > 2100 {
            break;
        }
        let result = model::calculate_at(profile, when)?;
        let mut ephemeris = result
            .reference_ephemeris
            .ok_or("Planetary positions unavailable for anniversary.")?;
        ephemeris.select_ayanamsha(settings.ayanamsha)?;
        let name = recovered::BODIES[body];
        let longitude = ephemeris
            .longitude(name, false)
            .ok_or("Transit planet unavailable.")?;
        let when_jd = dasha::day_number(when);
        let major_match = dasha
            .major
            .iter()
            .enumerate()
            .find(|(_, period)| period.contains(when_jd));
        let lord_name = major_match.map_or("—", |(_, period)| period.name());
        let lord_idx = recovered::BODIES
            .iter()
            .position(|&b| b == lord_name)
            .unwrap_or(body);
        let maitri = ten_pillars::panchadha_maitri(body, lord_idx, &natal_positions);
        let maitri_bonus = match maitri {
            "Self" | "Best Friend" => 2,
            "Friend" => 1,
            "Neutral" => 0,
            "Enemy" => -1,
            _ => -2,
        };
        let sign_class = original_c0(body as u8, longitude);
        let mansion_class = original_d0(body as u8, longitude)?;
        let natal_points = points[body - 1][recovered::sign(longitude)];
        let combined_score =
            i32::from(sign_class) + mansion_class + i32::from(natal_points) + maitri_bonus;

        samples.push(Sample {
            age,
            when,
            sign_class,
            mansion_class,
            natal_points,
            maitri,
            combined_score,
            lord: lord_name,
            major_index: major_match.map(|(i, _)| i),
        });
    }
    let last_age = samples
        .last()
        .ok_or("No reference samples in the available range.")?
        .age;
    Ok(Timeline {
        name: profile.name.clone(),
        body: recovered::BODIES[body],
        samples,
        last_age,
        ayanamsha: settings.ayanamsha,
        dasha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annual_samples_stop_at_reference_range_and_stay_in_class_limits() {
        let profile = ChartRequest {
            year: 2098,
            ..Default::default()
        };
        let report = calculate(&profile, Settings::default(), 2).unwrap();
        assert_eq!(report.last_age, 2);
        assert_eq!(report.samples.len(), 3);
        for sample in report.samples {
            assert!(sample.sign_class <= 5);
            assert!((0..=3).contains(&sample.mansion_class));
            assert!(sample.natal_points <= 8);
        }
    }

    #[test]
    fn timeline_matches_selected_sidereal_positions() {
        let profile = ChartRequest {
            year: 2098,
            ..Default::default()
        };
        let settings = Settings {
            ayanamsha: 3,
            tropical: true,
            ..Settings::default()
        };
        let report = calculate(&profile, settings, 2).unwrap();
        assert_eq!(report.ayanamsha, 3);
        let mut chart = motaa_engine::calculate(&profile)
            .unwrap()
            .reference_ephemeris
            .unwrap();
        chart.select_ayanamsha(3).unwrap();
        let moon = chart.longitude("Moon", false).unwrap();
        assert_eq!(report.samples[0].sign_class, original_c0(2, moon));
        assert_eq!(
            report.samples[0].mansion_class,
            original_d0(2, moon).unwrap()
        );
        assert_eq!(
            report.samples[0].lord,
            dasha::calculate(&profile, moon).unwrap().major[0].name()
        );
    }
}
