use rusqlite::Connection;
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection, rusqlite::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
    }
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE COLLATE NOCASE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin', 'hr')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS staff (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            work_center TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            file_number TEXT NOT NULL DEFAULT '',
            civil_id TEXT NOT NULL UNIQUE,
            job_title TEXT NOT NULL DEFAULT '',
            hire_date TEXT,
            grade TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '',
            qualification TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_staff_name ON staff(name);
        CREATE INDEX IF NOT EXISTS idx_staff_civil ON staff(civil_id);
        CREATE INDEX IF NOT EXISTS idx_staff_work_center ON staff(work_center);
        ",
    )?;
    conn.execute("UPDATE users SET role = 'hr' WHERE role = 'admin'", [])?;
    Ok(())
}

pub fn user_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
}
