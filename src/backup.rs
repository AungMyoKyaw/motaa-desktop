mod android;

use std::{collections::HashMap, fs::File, io::Read, path::Path};

use motaa_engine::ChartRequest;
use serde::{Deserialize, Serialize};

use crate::model::{AYANAMSHAS, HOUSES, Settings, WorkspaceSession};

const MAX_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
pub struct Record {
    chart: ChartRequest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reference_location: Option<ChartRequest>,
    #[serde(default)]
    reference_location_reset: bool,
    settings: Settings,
    favorite: bool,
    tags: Vec<String>,
    revisions: Vec<Revision>,
}

#[derive(Serialize, Deserialize)]
struct Revision {
    chart: ChartRequest,
    recorded_at: String,
}

#[derive(Serialize, Deserialize)]
struct Backup {
    version: u8,
    profiles: Vec<Record>,
}

impl From<ChartRequest> for Record {
    fn from(chart: ChartRequest) -> Self {
        Self {
            chart,
            reference_location: None,
            reference_location_reset: false,
            settings: Settings::default(),
            favorite: false,
            tags: Vec::new(),
            revisions: Vec::new(),
        }
    }
}

pub fn encode(session: &WorkspaceSession) -> Result<Vec<u8>, String> {
    let connection = session.connection()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let profiles = motaa_store::list(&transaction)?
        .into_iter()
        .map(|profile| {
            let chart =
                serde_json::from_str(&profile.data_json).map_err(|error| error.to_string())?;
            let settings = session.profile_settings(profile.id)?;
            let revisions = motaa_store::history(&transaction, profile.id)?
                .into_iter()
                .rev()
                .map(|revision| {
                    let chart = serde_json::from_str(&revision.data_json)
                        .map_err(|error| error.to_string())?;
                    Ok::<_, String>(Revision {
                        chart,
                        recorded_at: revision.recorded_at,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let reference = session.profile_reference_preference(profile.id)?;
            Ok::<_, String>(Record {
                chart,
                reference_location_reset: matches!(reference, Some(None)),
                reference_location: reference.flatten(),
                settings,
                favorite: profile.favorite,
                tags: profile.tags,
                revisions,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    transaction.commit().map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(&Backup {
        version: 2,
        profiles,
    })
    .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Backup exceeds 10 MiB.".into());
    }
    Ok(bytes)
}

pub fn read(path: &Path) -> Result<Vec<Record>, String> {
    let mut file = File::open(path)
        .map_err(|error| error.to_string())?
        .take(MAX_BYTES + 1);
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Backup exceeds 10 MiB.".into());
    }
    if bytes.starts_with(b"SQLite format 3\0") {
        return android::read(path);
    }
    decode(&bytes)
}

fn decode(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Backup exceeds 10 MiB.".into());
    }
    #[derive(Deserialize)]
    struct BackupEnvelope {
        version: u64,
        profiles: serde_json::Value,
    }

    let json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| format!("Invalid MOTAA backup: {error}"))?;
    if !json.is_object() {
        return Err("A MOTAA backup must be a JSON object.".into());
    }
    let backup: BackupEnvelope =
        serde_json::from_value(json).map_err(|error| format!("Invalid MOTAA backup: {error}"))?;
    let records: Vec<Record> = match backup.version {
        1 => {
            let old: Vec<ChartRequest> =
                serde_json::from_value(backup.profiles).map_err(|error| error.to_string())?;
            old.into_iter().map(Record::from).collect()
        }
        2 => serde_json::from_value(backup.profiles).map_err(|error| error.to_string())?,
        _ => return Err("Unsupported backup version.".into()),
    };
    for record in &records {
        validate(record)?;
    }
    Ok(records)
}

fn validate(record: &Record) -> Result<(), String> {
    let validate_chart = |chart: &ChartRequest| {
        if chart.name.trim().is_empty() {
            return Err("A profile is missing its name.".into());
        }
        motaa_engine::validate(chart)?;
        motaa_engine::effective_utc_offset(chart)?;
        Ok::<_, String>(())
    };
    validate_chart(&record.chart)?;
    if let Some(reference) = &record.reference_location {
        if record.reference_location_reset {
            return Err("Conflicting reference location preferences.".into());
        }
        validate_chart(reference)?;
    }
    if record.settings.ayanamsha >= AYANAMSHAS.len() || record.settings.houses >= HOUSES.len() {
        return Err("Saved calculation choices are outside the supported range.".into());
    }
    if record.tags.len() > 30 || record.tags.iter().any(|tag| tag.len() > 75) {
        return Err("Invalid profile tags.".into());
    }
    for revision in &record.revisions {
        validate_chart(&revision.chart)?;
        chrono::NaiveDateTime::parse_from_str(&revision.recorded_at, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| "Invalid profile revision time.".to_owned())?;
    }
    Ok(())
}

pub fn restore(session: &WorkspaceSession, records: &[Record]) -> Result<(usize, usize), String> {
    let db = session.connection()?;
    let transaction = db
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let mut existing = HashMap::new();
    for profile in motaa_store::list(&transaction)? {
        if let Ok(chart) = serde_json::from_str::<ChartRequest>(&profile.data_json) {
            existing
                .entry(serde_json::to_string(&chart).map_err(|error| error.to_string())?)
                .or_insert(profile.id);
        }
    }
    let mut added = 0;
    for record in records {
        validate(record)?;
        let chart = &record.chart;
        let data = serde_json::to_string(chart).map_err(|error| error.to_string())?;
        if let Some(&id) = existing.get(&data) {
            save_missing_reference(&transaction, id, record)?;
            continue;
        }
        let new = motaa_store::save(&transaction, None, &chart.name, &chart.birthplace, &data)?;
        existing.insert(data.clone(), new.id);
        save_missing_reference(&transaction, new.id, record)?;
        motaa_store::metadata(&transaction, new.id, record.favorite, &record.tags)?;
        let options = serde_json::to_string(&record.settings).map_err(|error| error.to_string())?;
        motaa_store::put_setting(
            &transaction,
            &format!("profile_calculations_{}", new.id),
            &options,
        )?;
        for revision in &record.revisions {
            let data = serde_json::to_string(&revision.chart).map_err(|error| error.to_string())?;
            motaa_store::append_history(
                &transaction,
                new.id,
                &revision.chart.name,
                &data,
                &revision.recorded_at,
            )?;
        }
        added += 1;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok((added, records.len() - added))
}

fn save_missing_reference(
    db: &rusqlite::Connection,
    profile_id: i64,
    record: &Record,
) -> Result<(), String> {
    if record.reference_location.is_some() || record.reference_location_reset {
        let key = format!("profile_reference_location_{profile_id}");
        if motaa_store::setting(db, &key)?.is_none() {
            let value = serde_json::to_string(&record.reference_location)
                .map_err(|error| error.to_string())?;
            motaa_store::put_setting(db, &key, &value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> WorkspaceSession {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        motaa_store::migrate(&db).unwrap();
        WorkspaceSession::new(
            Ok(db),
            chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
    }

    #[test]
    fn invalid_backup_shapes_return_errors() {
        for json in [
            "null",
            "42",
            r#""profiles""#,
            "[2, []]",
            "{}",
            r#"{"version":2}"#,
            r#"{"version":2,"profiles":{}}"#,
            r#"{"version":3,"profiles":[]}"#,
        ] {
            assert!(decode(json.as_bytes()).is_err(), "{json}");
        }
    }

    #[test]
    fn version_two_round_trip_preserves_metadata_settings_and_revisions() {
        let source = session();
        let record = Record {
            settings: Settings {
                ayanamsha: 5,
                houses: 3,
                mean_node: true,
                ..Settings::default()
            },
            favorite: true,
            tags: vec!["reference".into()],
            revisions: vec![Revision {
                chart: ChartRequest {
                    name: "Prior name".into(),
                    ..ChartRequest::default()
                },
                recorded_at: "2025-03-05 02:00:00".into(),
            }],
            ..Record::from(ChartRequest::default())
        };
        assert_eq!(restore(&source, &[record]).unwrap(), (1, 0));
        let destination = session();
        let records = decode(&encode(&source).unwrap()).unwrap();
        assert_eq!(restore(&destination, &records).unwrap(), (1, 0));
        assert_eq!(restore(&destination, &records).unwrap(), (0, 1));
        let profile = destination.profiles().unwrap().remove(0);
        assert!(profile.favorite);
        assert_eq!(profile.tags, ["reference"]);
        assert_eq!(
            destination.profile_settings(profile.id).unwrap().ayanamsha,
            5
        );
        let revisions =
            motaa_store::history(destination.connection().unwrap(), profile.id).unwrap();
        assert_eq!(revisions[0].name, "Prior name");
        assert_eq!(revisions[0].recorded_at, "2025-03-05 02:00:00");
    }

    #[test]
    fn version_one_stays_importable_and_errors_roll_back() {
        let profile = ChartRequest::default();
        let old =
            serde_json::to_vec(&serde_json::json!({"version":1,"profiles":[profile]})).unwrap();
        let records = decode(&old).unwrap();
        let session = session();
        assert_eq!(restore(&session, &records).unwrap(), (1, 0));
        let new = ChartRequest {
            name: "Second profile".into(),
            ..ChartRequest::default()
        };
        let invalid = ChartRequest {
            name: String::new(),
            ..ChartRequest::default()
        };
        assert!(restore(&session, &[new.into(), invalid.into()]).is_err());
        assert_eq!(session.profiles().unwrap().len(), 1);
    }
}
