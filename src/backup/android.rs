use super::{Record, Settings, validate};
use motaa_engine::ChartRequest;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

// V31.1 UserDBActivity reads one field per MY_TABLE row in the active
// profile database, and one space-delimited profile per comments row.
pub(super) fn read(path: &Path) -> Result<Vec<Record>, String> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| error.to_string())?;
    db.pragma_update(None, "trusted_schema", false)
        .map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    let mut recognized = false;
    for (table, query) in [
        ("MY_TABLE", "SELECT Content FROM MY_TABLE ORDER BY rowid"),
        ("comments", "SELECT comment FROM comments ORDER BY rowid"),
    ] {
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1 COLLATE NOCASE)",
            [table],
            |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        if !exists {
            continue;
        }
        recognized = true;
        let mut stmt = db.prepare(query).map_err(|error| error.to_string())?;
        let fields = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        if table == "MY_TABLE" {
            if !fields.is_empty() {
                records.push(
                    parse(&fields.iter().map(String::as_str).collect::<Vec<_>>())
                        .map_err(|error| format!("Android active profile: {error}"))?,
                );
            }
        } else {
            for (index, row) in fields.iter().enumerate() {
                records
                    .push(parse_saved(row).map_err(|error| {
                        format!("Android saved profile {}: {error}", index + 1)
                    })?);
            }
        }
    }
    if !recognized {
        return Err("Choose a MOTAA Android profile database or a desktop JSON backup.".into());
    }
    Ok(records)
}

// V31.1 serializes saved profiles by joining fields with a literal space.
// Recover unescaped names by locating the fixed numeric and settings fields.
fn parse_saved(row: &str) -> Result<Record, String> {
    let fields: Vec<_> = row.trim_end_matches(' ').split(' ').collect();
    if let Ok(record) = parse(&fields) {
        return Ok(record);
    }
    let mut recovered = None;
    for birth_start in 1..fields.len().saturating_sub(22) {
        let birth_end = birth_start + 9;
        if !matches!(fields[birth_end - 1], "EGMT" | "WGMT") {
            continue;
        }
        for coordinates in birth_end + 1..fields.len().saturating_sub(12) {
            if !matches!(fields[coordinates + 8], "Tropical" | "Sidereal") {
                continue;
            }
            let name = fields[..birth_start].join(" ");
            let birthplace = fields[birth_end..coordinates].join(" ");
            let mut record = vec![name.as_str()];
            record.extend_from_slice(&fields[birth_start..birth_end]);
            record.push(birthplace.as_str());
            record.extend_from_slice(&fields[coordinates..coordinates + 13]);

            let tail = &fields[coordinates + 13..];
            let reference_name;
            if tail.len() >= 13 {
                let offset_start = tail.len() - 11;
                reference_name = tail[1..offset_start].join(" ");
                record.extend([tail[0], reference_name.as_str()]);
                record.extend_from_slice(&tail[offset_start..]);
            } else {
                record.extend_from_slice(tail);
            }
            if let Ok(record) = parse(&record) {
                if recovered.is_some() {
                    return Err("Ambiguous space-delimited Android record.".into());
                }
                recovered = Some(record);
            }
        }
    }
    recovered.map_or_else(|| parse(&fields), Ok)
}

fn number<T: std::str::FromStr>(value: &str, field: &str) -> Result<T, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("Invalid {field}."))
}

fn choice(value: &str, choices: &[&str], field: &str) -> Result<usize, String> {
    choices
        .iter()
        .position(|item| item.eq_ignore_ascii_case(value.trim()))
        .ok_or_else(|| format!("Unrecognized {field}."))
}

fn timezone_offset(fields: &[&str]) -> Result<i32, String> {
    let offset_hours: i32 = number(fields[0], "time zone hours")?;
    let offset_minutes: i32 = number(fields[1], "time zone minutes")?;
    if !(0..=14).contains(&offset_hours) || !(0..60).contains(&offset_minutes) {
        return Err("Invalid time zone offset.".into());
    }
    let west = choice(fields[2], &["EGMT", "WGMT"], "time zone direction")? == 1;
    Ok((60 * offset_hours + offset_minutes) * if west { -1 } else { 1 })
}

fn parse(fields: &[&str]) -> Result<Record, String> {
    if fields.len() < 24 {
        return Err("The birth record is incomplete.".into());
    }
    let mut record = Record::from(ChartRequest {
        name: fields[0].to_owned(),
        day: number(fields[1], "birth day")?,
        month: number(fields[2], "birth month")?,
        year: number(fields[3], "birth year")?,
        hour: number(fields[4], "birth hour")?,
        minute: number(fields[5], "birth minute")?,
        second: number(fields[6], "birth second")?,
        timezone_offset_minutes: timezone_offset(&fields[7..10])?,
        timezone_id: None,
        birthplace: fields[10].to_owned(),
        latitude: crate::parse_coordinate(&fields[11..15].join(" "), true)?,
        longitude: crate::parse_coordinate(&fields[15..19].join(" "), false)?,
        ..ChartRequest::default()
    });
    record.settings = Settings {
        tropical: choice(fields[19], &["Sidereal", "Tropical"], "zodiac")? == 1,
        ayanamsha: choice(
            fields[20],
            &[
                "LAHIRI",
                "FAGAN_BRADLEY",
                "DELUCE",
                "RAMAN",
                "USHASHASHI",
                "KRISHNAMURTI",
                "DJWHAL_KHUL",
                "YUKTESHWAR",
                "JN_BHASIN",
            ],
            "ayanamsha",
        )?,
        houses: choice(
            fields[21],
            &[
                "Equal",
                "Vehlow_Placidus",
                "Placidus",
                "Koch",
                "Porphyrius",
                "Regiomontanus",
                "Campanus",
                "Vehlow_Equal",
            ],
            "house system",
        )?,
        mean_node: choice(fields[22], &["True_Rahu", "Mean_Rahu"], "lunar node")? == 1,
        fifteen_degree_display: choice(fields[23], &["5D_Aspect", "15D_Aspect"], "aspect display")?
            == 1,
        ..Settings::default()
    };
    if (26..37).contains(&fields.len()) && !fields[25].trim().is_empty() {
        return Err("The Android reference location is incomplete.".into());
    }
    if fields.len() >= 37 && !fields[25].trim().is_empty() {
        record.reference_location = Some(ChartRequest {
            birthplace: fields[25].to_owned(),
            timezone_offset_minutes: timezone_offset(&fields[26..29])?,
            timezone_id: None,
            latitude: crate::parse_coordinate(&fields[29..33].join(" "), true)?,
            longitude: crate::parse_coordinate(&fields[33..37].join(" "), false)?,
            ..record.chart.clone()
        });
    }
    validate(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn recovers_unescaped_spaces_in_android_saved_profiles() {
        let source = "Birth Name 15 6 1990 12 20 4 5 0 WGMT San Francisco 37 46 0 N 122 25 0 W Tropical RAMAN Porphyrius Mean_Rahu 15D_Aspect 2 New York City 4 0 WGMT 40 42 0 N 74 0 0 W";
        let record = parse_saved(source).unwrap();
        assert_eq!(record.chart.name, "Birth Name");
        assert_eq!(record.chart.birthplace, "San Francisco");
        let reference = record.reference_location.unwrap();
        assert_eq!(reference.birthplace, "New York City");
        assert_eq!(reference.timezone_offset_minutes, -240);
        assert!(parse_saved(source.trim_end_matches(" W")).is_err());
    }

    #[test]
    fn legacy_short_saved_records_import_without_reference_location() {
        let short = "Short Name 15 6 1990 12 20 4 5 0 WGMT Boston 42 0 0 N 71 0 0 W Tropical RAMAN Porphyrius Mean_Rahu 15D_Aspect";
        let record = parse_saved(short).unwrap();
        assert_eq!(record.chart.name, "Short Name");
        assert_eq!(record.chart.birthplace, "Boston");
        assert!(record.reference_location.is_none());
        assert!(record.settings.mean_node);
        // Android V31.1 treats 25 fields (display flag only) the same way.
        assert!(
            parse_saved(&format!("{short} 1"))
                .unwrap()
                .reference_location
                .is_none()
        );
        // A partial reference location is rejected rather than guessed.
        assert!(parse_saved(&format!("{short} 1 Place")).is_err());
    }

    #[test]
    #[ignore = "requires a saved-profile database from the isolated Android emulator"]
    fn reads_saved_profiles_written_by_v31_1() {
        let path = std::env::var_os("MOTAA_ANDROID_ORIGINAL_SAVED_DATABASE").unwrap();
        let records = super::super::read(Path::new(&path)).unwrap();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.chart.birthplace, "Pakhangyi");
        assert_eq!(record.chart.timezone_offset_minutes, 390);
        assert_eq!(record.settings.houses, 1);
        let mut ephemeris = motaa_engine::calculate(&record.chart)
            .unwrap()
            .reference_ephemeris
            .unwrap();
        ephemeris
            .select_ayanamsha(record.settings.ayanamsha)
            .unwrap();
        let centers = ephemeris
            .house_midpoints(
                motaa_engine::houses::System::OPTIONS[record.settings.houses],
                record.settings.tropical,
            )
            .unwrap();
        for (index, expected) in [(9, "Virgo 12°36′"), (3, "Pisces 12°36′")] {
            assert_eq!(crate::chart_view::chart_longitude(centers[index]), expected);
        }
        let positions =
            crate::recovered::positions(&ephemeris, &ephemeris, record.settings).unwrap();
        // Mode 9 labels in the original Pakhangyi chart. The independent
        // ephemeris can put a longitude on the other side of a minute boundary.
        for ((body, angle), (sign, degrees, minutes)) in
            crate::recovered::BODIES.into_iter().zip(positions).zip([
                ("Sagittarius", 2, 36),
                ("Sagittarius", 18, 7),
                ("Sagittarius", 15, 16),
                ("Libra", 2, 5),
                ("Sagittarius", 11, 54),
                ("Sagittarius", 2, 43),
                ("Scorpio", 8, 38),
                ("Libra", 20, 28),
                ("Taurus", 22, 13),
                ("Scorpio", 22, 13),
            ])
        {
            let readout = crate::chart_view::chart_longitude(angle);
            assert!(
                (-1..=1).any(|offset| {
                    let value = degrees * 60 + minutes + offset;
                    readout == format!("{sign} {}°{:02}′", value / 60, value % 60)
                }),
                "{body}: {readout}"
            );
        }
        let reference = record.reference_location.as_ref().unwrap();
        assert_eq!(reference.birthplace, "TechnoHalo");
        assert_eq!(reference.timezone_offset_minutes, 390);
        assert!((reference.latitude - (21. + 59. / 60. + 59. / 3600.)).abs() < 1e-10);
        assert!((reference.longitude - (96. + 7. / 60. + 35. / 3600.)).abs() < 1e-10);
    }

    #[test]
    fn reads_android_sqlite_and_restores_operable_profiles() {
        let records = if let Some(path) = std::env::var_os("MOTAA_ANDROID_PROFILE_DATABASE") {
            super::super::read(Path::new(&path)).unwrap()
        } else {
            let path = std::env::temp_dir().join(format!(
                "motaa-android-import-{}.sqlite",
                std::process::id()
            ));
            {
                let db = Connection::open(&path).unwrap();
                db.execute_batch("CREATE TABLE MY_TABLE(Content TEXT); CREATE TABLE comments(_id INTEGER PRIMARY KEY, comment TEXT);").unwrap();
                let active = [
                    "Birth Name",
                    "3",
                    "1",
                    "1984",
                    "5",
                    "45",
                    "0",
                    "6",
                    "30",
                    "EGMT",
                    "Birth Place",
                    "21",
                    "32",
                    "8",
                    "N",
                    "95",
                    "11",
                    "56",
                    "E",
                    "Sidereal",
                    "LAHIRI",
                    "Vehlow_Equal",
                    "True_Rahu",
                    "5D_Aspect",
                ];
                for field in active {
                    db.execute("INSERT INTO MY_TABLE VALUES (?1)", [field])
                        .unwrap();
                }
                let saved = "Natal Record 15 6 1990 12 20 4 5 0 WGMT Boston 42 0 0 N 71 0 0 W Tropical RAMAN Porphyrius Mean_Rahu 15D_Aspect 2 Current Place 6 30 EGMT 16 48 0 N 96 10 0 E";
                db.execute("INSERT INTO comments(comment) VALUES (?1)", [saved])
                    .unwrap();
            }
            let records = super::super::read(&path).unwrap();
            std::fs::remove_file(path).unwrap();
            records
        };
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].chart.name, "Birth Name");
        assert_eq!(records[0].chart.timezone_offset_minutes, 390);
        assert_eq!(records[1].chart.timezone_offset_minutes, -300);
        assert_eq!(records[1].chart.longitude, -71.);
        assert_eq!(records[1].settings.ayanamsha, 3);
        let mut session = crate::model::WorkspaceSession::new(
            motaa_store::open(Path::new(":memory:")),
            NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
        assert_eq!(super::super::restore(&session, &records).unwrap(), (2, 0));
        assert_eq!(super::super::restore(&session, &records).unwrap(), (0, 2));
        let profiles = session.profiles().unwrap();
        let natal = profiles
            .iter()
            .find(|profile| profile.name == "Natal Record")
            .unwrap();
        let reference = session
            .profile_reference_location(natal.id)
            .unwrap()
            .unwrap();
        assert_eq!(reference.birthplace, "Current Place");
        assert_eq!(reference.timezone_offset_minutes, 390);
        assert_eq!(reference.latitude, 16.8);
        let read_chart = |profile: &motaa_store::Profile| {
            serde_json::from_str::<ChartRequest>(&profile.data_json).unwrap()
        };
        session.activate_natal(natal.id, read_chart(natal), Some(reference));
        assert_eq!(session.reference_profile().birthplace, "Current Place");

        let other = profiles
            .iter()
            .find(|profile| profile.id != natal.id)
            .unwrap();
        session.activate_natal(other.id, read_chart(other), None);
        assert_eq!(session.reference_profile().birthplace, "Birth Place");
        assert!(session.reference_source.is_none());

        session.transit = Some(read_chart(natal));
        session.reference_source = Some(crate::model::ReferenceSource::SelectedProfile(natal.id));
        session.activate_natal(other.id, read_chart(other), None);
        assert_eq!(session.reference_profile().birthplace, "Boston");
        session.activate_natal(natal.id, read_chart(natal), None);
        let mut edited_reference = read_chart(natal);
        edited_reference.birthplace = "Edited transit location".into();
        session
            .store_reference_for_active_profile(Some(&edited_reference))
            .unwrap();
        assert_eq!(
            session
                .profile_reference_location(natal.id)
                .unwrap()
                .unwrap()
                .birthplace,
            "Edited transit location"
        );
        let restored = super::super::decode(&super::super::encode(&session).unwrap()).unwrap();
        assert!(restored.iter().any(|record| {
            record
                .reference_location
                .as_ref()
                .is_some_and(|reference| reference.birthplace == "Edited transit location")
        }));
        assert_eq!(session.profile_settings(natal.id).unwrap().houses, 4);
        session.store_reference_for_active_profile(None).unwrap();
        assert_eq!(super::super::restore(&session, &records).unwrap(), (0, 2));
        assert!(
            session
                .profile_reference_location(natal.id)
                .unwrap()
                .is_none()
        );
        let decoded = super::super::decode(&super::super::encode(&session).unwrap()).unwrap();
        let fresh = crate::model::WorkspaceSession::new(
            motaa_store::open(Path::new(":memory:")),
            session.time,
        );
        assert_eq!(super::super::restore(&fresh, &decoded).unwrap(), (2, 0));
        assert_eq!(super::super::restore(&fresh, &records).unwrap(), (0, 2));
        let fresh_natal = fresh
            .profiles()
            .unwrap()
            .into_iter()
            .find(|profile| profile.name == natal.name)
            .unwrap();
        assert!(matches!(
            fresh.profile_reference_preference(fresh_natal.id).unwrap(),
            Some(None)
        ));
    }
}
