use std::path::PathBuf;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub file_path: String,
    pub width: i64,
    pub height: i64,
    pub created_at: String,
    pub tags: String,
}

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("better-screenshoot.db"))
}

fn with_connection<F, T>(app: &AppHandle, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let path = db_path(app)?;
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS captures (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]'
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    f(&conn)
}

pub(crate) fn insert_record_with_conn(
    conn: &Connection,
    id: &str,
    file_path: &str,
    width: u32,
    height: u32,
    created_at: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO captures (id, file_path, width, height, created_at, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, file_path, width as i64, height as i64, created_at, "[]"],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn get_history_with_conn(
    conn: &Connection,
    limit: u32,
) -> Result<Vec<HistoryRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, file_path, width, height, created_at, tags FROM captures ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([limit as i64], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                file_path: row.get(1)?,
                width: row.get(2)?,
                height: row.get(3)?,
                created_at: row.get(4)?,
                tags: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub async fn insert_record(
    app: &AppHandle,
    id: &str,
    file_path: &str,
    width: u32,
    height: u32,
    created_at: &str,
) -> Result<(), String> {
    with_connection(app, |conn| {
        insert_record_with_conn(conn, id, file_path, width, height, created_at)
    })
}

#[tauri::command]
pub async fn get_history(app: AppHandle, limit: Option<u32>) -> Result<Vec<HistoryRecord>, String> {
    let limit = limit.unwrap_or(100);
    with_connection(&app, |conn| get_history_with_conn(conn, limit))
}

pub fn get_record_by_id(app: &AppHandle, id: &str) -> Result<HistoryRecord, String> {
    with_connection(app, |conn| {
        conn.query_row(
            "SELECT id, file_path, width, height, created_at, tags FROM captures WHERE id = ?1",
            params![id],
            |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    width: row.get(2)?,
                    height: row.get(3)?,
                    created_at: row.get(4)?,
                    tags: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    })
}

pub fn delete_record_if_exists(app: &AppHandle, id: &str) -> Result<(), String> {
    let file_path = with_connection(app, |conn| {
        let result = conn.query_row(
            "SELECT file_path FROM captures WHERE id = ?1",
            params![id],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(path) => {
                conn.execute("DELETE FROM captures WHERE id = ?1", params![id])
                    .map_err(|e| e.to_string())?;
                Ok(Some(path))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    })?;

    if let Some(path) = file_path {
        let file = PathBuf::from(path);
        if file.exists() {
            std::fs::remove_file(&file).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_history_item(app: AppHandle, id: String) -> Result<(), String> {
    delete_record_if_exists(&app, &id)?;
    let _ = app.emit("history-changed", ());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE captures (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn insert_and_fetch_round_trip() {
        let conn = setup_in_memory();
        insert_record_with_conn(
            &conn,
            "id1",
            "/tmp/cap.png",
            1920,
            1080,
            "2026-01-01T00:00:00Z",
        )
        .unwrap();
        let rows = get_history_with_conn(&conn, 10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "id1");
    }

    #[test]
    fn history_is_ordered_newest_first() {
        let conn = setup_in_memory();
        insert_record_with_conn(&conn, "id1", "/a.png", 1, 1, "2026-01-01T00:00:00Z").unwrap();
        insert_record_with_conn(&conn, "id2", "/b.png", 1, 1, "2026-01-02T00:00:00Z").unwrap();
        insert_record_with_conn(&conn, "id3", "/c.png", 1, 1, "2026-01-03T00:00:00Z").unwrap();
        let rows = get_history_with_conn(&conn, 10).unwrap();
        assert_eq!(rows[0].id, "id3");
    }

    #[test]
    fn limit_is_respected() {
        let conn = setup_in_memory();
        for i in 0..5u32 {
            insert_record_with_conn(
                &conn,
                &format!("id{i}"),
                "/f.png",
                1,
                1,
                &format!("2026-01-0{}T00:00:00Z", i + 1),
            )
            .unwrap();
        }
        let rows = get_history_with_conn(&conn, 3).unwrap();
        assert_eq!(rows.len(), 3);
    }
}

pub fn migrations() -> Vec<tauri_plugin_sql::Migration> {
    use tauri_plugin_sql::{Migration, MigrationKind};
    vec![Migration {
        version: 1,
        description: "create captures table",
        sql: "CREATE TABLE IF NOT EXISTS captures (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]'
        );",
        kind: MigrationKind::Up,
    }]
}
