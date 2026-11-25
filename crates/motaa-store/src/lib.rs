//! Private, Rust-owned SQLite data store, migrations, revisions, settings and legacy imports.
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub birthplace: String,
    pub data_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub favorite: bool,
    pub tags: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileHistory {
    pub id: i64,
    pub profile_id: i64,
    pub name: String,
    pub data_json: String,
    pub recorded_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImportReport {
    pub source: String,
    pub records_imported: usize,
    pub comments_imported: usize,
    pub duplicates_skipped: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyRecord {
    pub id: i64,
    pub source: String,
    pub original_table: String,
    pub content: String,
    pub imported_at: String,
}

fn ensure_column(db: &Connection, column: &str, sql: &str) -> Result<(), String> {
    let mut stmt = db
        .prepare("PRAGMA table_info(profiles)")
        .map_err(|e| e.to_string())?;
    let names = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if !names.iter().any(|name| name == column) {
        db.execute_batch(sql).map_err(|e| e.to_string())?
    }
    Ok(())
}
pub fn migrate(db: &Connection) -> Result<(), String> {
    db.execute_batch(
        "PRAGMA foreign_keys=ON;
 CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY);
 CREATE TABLE IF NOT EXISTS profiles(
   id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL,
   birthplace TEXT NOT NULL DEFAULT '', data_json TEXT NOT NULL,
   created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
   updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
 );
 CREATE INDEX IF NOT EXISTS idx_profiles_name ON profiles(name COLLATE NOCASE);
 INSERT OR IGNORE INTO schema_migrations(version) VALUES(1);",
    )
    .map_err(|e| e.to_string())?;
    ensure_column(
        db,
        "favorite",
        "ALTER TABLE profiles ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        db,
        "tags_json",
        "ALTER TABLE profiles ADD COLUMN tags_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS profile_history(
   id INTEGER PRIMARY KEY AUTOINCREMENT, profile_id INTEGER NOT NULL,
   name TEXT NOT NULL, data_json TEXT NOT NULL, recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
 );
 CREATE INDEX IF NOT EXISTS idx_profile_history_parent ON profile_history(profile_id,id DESC);
 CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value_json TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS legacy_records(
   id INTEGER PRIMARY KEY AUTOINCREMENT, source TEXT NOT NULL,
   original_table TEXT NOT NULL, content TEXT NOT NULL, imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
   UNIQUE(source, original_table, content)
 );
 INSERT OR IGNORE INTO schema_migrations(version) VALUES(2);")
 .map_err(|e|e.to_string())?;
    Ok(())
}
pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?
    }
    let db = Connection::open(path).map_err(|e| e.to_string())?;
    migrate(&db)?;
    Ok(db)
}
fn map_profile(r: &rusqlite::Row) -> rusqlite::Result<Profile> {
    let tags: String = r.get(7)?;
    Ok(Profile {
        id: r.get(0)?,
        name: r.get(1)?,
        birthplace: r.get(2)?,
        data_json: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
        favorite: r.get::<_, i64>(6)? != 0,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
    })
}
const SELECT_PROFILE: &str =
    "SELECT id,name,birthplace,data_json,created_at,updated_at,favorite,tags_json FROM profiles";
pub fn list(db: &Connection) -> Result<Vec<Profile>, String> {
    let mut stmt = db
        .prepare(&format!(
            "{SELECT_PROFILE} ORDER BY favorite DESC,updated_at DESC,id DESC"
        ))
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_profile).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub fn save(
    db: &Connection,
    id: Option<i64>,
    name: &str,
    birthplace: &str,
    data: &str,
) -> Result<Profile, String> {
    if name.trim().is_empty() || name.len() > 200 || birthplace.len() > 300 {
        return Err("Invalid profile name/location".into());
    }
    if data.len() > 1_000_000 {
        return Err("Profile data too large".into());
    }
    serde_json::from_str::<serde_json::Value>(data).map_err(|e| e.to_string())?;
    let next_id = if let Some(id) = id {
        let old = db
            .query_row(
                "SELECT name,data_json FROM profiles WHERE id=?1",
                params![id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or("Profile not found")?;
        db.execute(
            "INSERT INTO profile_history(profile_id,name,data_json) VALUES(?1,?2,?3)",
            params![id, old.0, old.1],
        )
        .map_err(|e| e.to_string())?;
        db.execute("UPDATE profiles SET name=?1,birthplace=?2,data_json=?3,updated_at=CURRENT_TIMESTAMP WHERE id=?4",
   params![name,birthplace,data,id]).map_err(|e|e.to_string())?;
        id
    } else {
        db.execute(
            "INSERT INTO profiles(name,birthplace,data_json) VALUES(?1,?2,?3)",
            params![name, birthplace, data],
        )
        .map_err(|e| e.to_string())?;
        db.last_insert_rowid()
    };
    db.query_row(
        &format!("{SELECT_PROFILE} WHERE id=?1"),
        params![next_id],
        map_profile,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Profile disappeared".into())
}
pub fn metadata(
    db: &Connection,
    id: i64,
    favorite: bool,
    tags: &[String],
) -> Result<Profile, String> {
    if tags.len() > 30 || tags.iter().any(|t| t.len() > 75) {
        return Err("Invalid tags".into());
    }
    let tags_json = serde_json::to_string(tags).map_err(|e| e.to_string())?;
    let n = db
        .execute(
            "UPDATE profiles SET favorite=?1,tags_json=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3",
            params![i64::from(favorite), tags_json, id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Profile not found".into());
    }
    db.query_row(
        &format!("{SELECT_PROFILE} WHERE id=?1"),
        params![id],
        map_profile,
    )
    .map_err(|e| e.to_string())
}
pub fn delete(db: &Connection, id: i64) -> Result<bool, String> {
    db.execute("DELETE FROM profiles WHERE id=?1", params![id])
        .map(|n| n > 0)
        .map_err(|e| e.to_string())
}
pub fn restore_deleted(db: &Connection, profile: &Profile) -> Result<(), String> {
    let tags = serde_json::to_string(&profile.tags).map_err(|error| error.to_string())?;
    db.execute(
        "INSERT INTO profiles(id, name, birthplace, data_json, created_at, updated_at, favorite, tags_json)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![profile.id, profile.name, profile.birthplace, profile.data_json,
            profile.created_at, profile.updated_at, profile.favorite, tags],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn append_history(
    db: &Connection,
    id: i64,
    name: &str,
    data: &str,
    recorded_at: &str,
) -> Result<(), String> {
    db.execute(
        "INSERT INTO profile_history(profile_id,name,data_json,recorded_at) VALUES(?1,?2,?3,?4)",
        params![id, name, data, recorded_at],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn history(db: &Connection, id: i64) -> Result<Vec<ProfileHistory>, String> {
    let mut stmt=db.prepare("SELECT id,profile_id,name,data_json,recorded_at FROM profile_history WHERE profile_id=?1 ORDER BY id DESC").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map(params![id], |r| {
            Ok(ProfileHistory {
                id: r.get(0)?,
                profile_id: r.get(1)?,
                name: r.get(2)?,
                data_json: r.get(3)?,
                recorded_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub fn setting(db: &Connection, key: &str) -> Result<Option<String>, String> {
    if key.len() > 100 {
        return Err("Setting key too long".into());
    }
    db.query_row(
        "SELECT value_json FROM settings WHERE key=?1",
        params![key],
        |r| r.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}
pub fn put_setting(db: &Connection, key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 100 || value.len() > 65_536 {
        return Err("Invalid setting".into());
    }
    serde_json::from_str::<serde_json::Value>(value).map_err(|e| e.to_string())?;
    db.execute("INSERT INTO settings(key,value_json) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json",
  params![key,value]).map_err(|e|e.to_string())?;
    Ok(())
}
fn table_exists(db: &Connection, name: &str) -> Result<bool, String> {
    db.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
        params![name],
        |r| r.get::<_, i64>(0),
    )
    .optional()
    .map(|o| o.is_some())
    .map_err(|e| e.to_string())
}
pub fn import_legacy(db: &mut Connection, path: &Path) -> Result<LegacyImportReport, String> {
    let external = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    let source = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("legacy.sqlite")
        .to_string();
    let contents = table_exists(&external, "MY_TABLE")?;
    let comments = table_exists(&external, "comments")?;
    if !contents && !comments {
        return Err("Not a supported MOTAA legacy database (no MY_TABLE or comments)".into());
    }
    let mut recs = 0;
    let mut coms = 0;
    let mut dup = 0;
    let txn = db.transaction().map_err(|e| e.to_string())?;
    for (table, column, is_comment) in [
        ("MY_TABLE", "Content", false),
        ("comments", "comment", true),
    ] {
        if !(if is_comment { comments } else { contents }) {
            continue;
        }
        // Table and column names here are hardcoded/constant, never constructed from file content.
        let sql = if is_comment {
            "SELECT comment FROM comments"
        } else {
            "SELECT Content FROM MY_TABLE"
        };
        let mut stmt = external.prepare(sql).map_err(|e| e.to_string())?;
        let records = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for text in records {
            let n=txn.execute("INSERT OR IGNORE INTO legacy_records(source,original_table,content) VALUES(?1,?2,?3)",
    params![source,table,text]).map_err(|e|e.to_string())?;
            if n == 0 {
                dup += 1
            } else if is_comment {
                coms += 1
            } else {
                recs += 1
            }
        }
        let _ = column;
    }
    txn.commit().map_err(|e| e.to_string())?;
    Ok(LegacyImportReport {
        source,
        records_imported: recs,
        comments_imported: coms,
        duplicates_skipped: dup,
    })
}
pub fn legacy_records(db: &Connection) -> Result<Vec<LegacyRecord>, String> {
    let mut stmt=db.prepare("SELECT id,source,original_table,content,imported_at FROM legacy_records ORDER BY id DESC").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(LegacyRecord {
                id: r.get(0)?,
                source: r.get(1)?,
                original_table: r.get(2)?,
                content: r.get(3)?,
                imported_at: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
/// SQLite snapshot including WAL contents. Fails instead of overwriting an existing backup.
pub fn backup_to(db: &Connection, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        return Err("Backup target already exists".into());
    }
    db.execute(
        "VACUUM INTO ?1",
        params![destination.to_string_lossy().as_ref()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn restore_from(db: &mut Connection, source_path: &Path) -> Result<(), String> {
    use rusqlite::backup::Backup;
    let source =
        Connection::open_with_flags(source_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
    if !table_exists(&source, "profiles")? || !table_exists(&source, "schema_migrations")? {
        return Err("Not a valid MOTAA Desktop backup".into());
    }
    let backup = Backup::new(&source, db).map_err(|e| e.to_string())?;
    backup
        .run_to_completion(100, std::time::Duration::from_millis(30), None)
        .map_err(|e| e.to_string())?;
    drop(backup);
    migrate(db)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn save_update_delete() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        let p = save(&db, None, "Test", "Yangon", r#"{"year":2000}"#).unwrap();
        assert_eq!(list(&db).unwrap().len(), 1);
        let updated = save(&db, Some(p.id), "New", "Mandalay", r#"{"year":2001}"#).unwrap();
        assert_eq!(updated.id, p.id);
        assert_eq!(updated.name, "New");
        assert_eq!(history(&db, p.id).unwrap().len(), 1);
        let flagged = metadata(&db, p.id, true, &["family".into(), "important".into()]).unwrap();
        assert!(flagged.favorite);
        assert_eq!(flagged.tags, vec!["family", "important"]);
        assert!(delete(&db, p.id).unwrap());
        assert!(list(&db).unwrap().is_empty());
    }
    #[test]
    fn rejects_invalid_input() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        assert!(save(&db, None, "", "", "{}").is_err());
        assert!(save(&db, None, "x", "", "bad").is_err());
    }
    #[test]
    fn settings_roundtrip() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        put_setting(&db, "theme", r#""dark""#).unwrap();
        assert_eq!(setting(&db, "theme").unwrap(), Some(r#""dark""#.into()));
    }
    #[test]
    fn legacy_import_preserves_raw_and_dedupes() {
        let root = std::env::temp_dir().join(format!("motaa-legacy-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let legacy = root.join("commments.db");
        {
            let l = Connection::open(&legacy).unwrap();
            l.execute_batch("CREATE TABLE MY_TABLE (Content TEXT NOT NULL); INSERT INTO MY_TABLE VALUES('ပထမ'),('ဒုတိယ'); CREATE TABLE comments(_id INTEGER PRIMARY KEY,comment TEXT NOT NULL); INSERT INTO comments VALUES(1,'မင်္ဂလာပါ');").unwrap();
        }
        let mut db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        let a = import_legacy(&mut db, &legacy).unwrap();
        assert_eq!(a.records_imported, 2);
        assert_eq!(a.comments_imported, 1);
        assert_eq!(legacy_records(&db).unwrap().len(), 3);
        assert_eq!(
            import_legacy(&mut db, &legacy).unwrap().duplicates_skipped,
            3
        );
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn backup_and_restore() {
        let root = std::env::temp_dir().join(format!("motaa-backup-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let backup = root.join("snapshot.sqlite");
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        save(&db, None, "hello", "Mandalay", r#"{"year":2024}"#).unwrap();
        backup_to(&db, &backup).unwrap();
        let mut restore = Connection::open_in_memory().unwrap();
        migrate(&restore).unwrap();
        restore_from(&mut restore, &backup).unwrap();
        assert_eq!(list(&restore).unwrap()[0].name, "hello");
        let _ = std::fs::remove_dir_all(root);
    }
}
