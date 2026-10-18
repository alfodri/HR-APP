use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub id: i64,
    pub work_center: String,
    pub name: String,
    pub file_number: String,
    pub civil_id: String,
    pub job_title: String,
    pub hire_date: Option<String>,
    pub grade: String,
    pub email: String,
    pub phone: String,
    pub qualification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffInput {
    pub work_center: String,
    pub name: String,
    pub file_number: String,
    pub civil_id: String,
    pub job_title: String,
    pub hire_date: Option<String>,
    pub grade: String,
    pub email: String,
    pub phone: String,
    pub qualification: String,
}

fn validate_input(input: &StaffInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("الاسم مطلوب".into());
    }
    if input.civil_id.trim().is_empty() {
        return Err("الرقم المدني مطلوب".into());
    }
    Ok(())
}

fn row_to_staff(row: &rusqlite::Row<'_>) -> Result<Staff, rusqlite::Error> {
    Ok(Staff {
        id: row.get(0)?,
        work_center: row.get(1)?,
        name: row.get(2)?,
        file_number: row.get(3)?,
        civil_id: row.get(4)?,
        job_title: row.get(5)?,
        hire_date: row.get(6)?,
        grade: row.get(7)?,
        email: row.get(8)?,
        phone: row.get(9)?,
        qualification: row.get(10)?,
    })
}

const STAFF_SELECT: &str = "
    SELECT id, work_center, name, file_number, civil_id, job_title,
           hire_date, grade, email, phone, qualification
    FROM staff
";

pub fn list_staff(conn: &Connection, query: Option<&str>) -> Result<Vec<Staff>, String> {
    let q = query.map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(term) = q {
        let like = format!("%{term}%");
        let sql = format!(
            "{STAFF_SELECT} WHERE name LIKE ?1 OR civil_id LIKE ?1 OR file_number LIKE ?1 OR work_center LIKE ?1 ORDER BY name"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], row_to_staff)
            .map_err(|e| e.to_string())?;
        collect_rows(rows)
    } else {
        let mut stmt = conn
            .prepare(&format!("{STAFF_SELECT} ORDER BY name"))
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], row_to_staff).map_err(|e| e.to_string())?;
        collect_rows(rows)
    }
}

fn collect_rows(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> Result<Staff, rusqlite::Error>>,
) -> Result<Vec<Staff>, String> {
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_staff(conn: &Connection, id: i64) -> Result<Staff, String> {
    conn.query_row(
        &format!("{STAFF_SELECT} WHERE id = ?1"),
        params![id],
        row_to_staff,
    )
    .map_err(|_| "الموظف غير موجود".to_string())
}

pub fn create_staff(conn: &Connection, input: StaffInput) -> Result<Staff, String> {
    validate_input(&input)?;
    let civil_id = input.civil_id.trim().to_string();
    conn.execute(
        "INSERT INTO staff (
            work_center, name, file_number, civil_id, job_title, hire_date,
            grade, email, phone, qualification, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",
        params![
            input.work_center.trim(),
            input.name.trim(),
            input.file_number.trim(),
            civil_id,
            input.job_title.trim(),
            input.hire_date,
            input.grade.trim(),
            input.email.trim(),
            input.phone.trim(),
            input.qualification.trim(),
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "الرقم المدني مسجل مسبقاً".to_string()
        } else {
            e.to_string()
        }
    })?;
    get_staff(conn, conn.last_insert_rowid())
}

pub fn update_staff(conn: &Connection, id: i64, input: StaffInput) -> Result<Staff, String> {
    validate_input(&input)?;
    let civil_id = input.civil_id.trim().to_string();
    let updated = conn.execute(
        "UPDATE staff SET
            work_center = ?1, name = ?2, file_number = ?3, civil_id = ?4,
            job_title = ?5, hire_date = ?6, grade = ?7, email = ?8,
            phone = ?9, qualification = ?10, updated_at = datetime('now')
         WHERE id = ?11",
        params![
            input.work_center.trim(),
            input.name.trim(),
            input.file_number.trim(),
            civil_id,
            input.job_title.trim(),
            input.hire_date,
            input.grade.trim(),
            input.email.trim(),
            input.phone.trim(),
            input.qualification.trim(),
            id,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "الرقم المدني مسجل لموظف آخر".to_string()
        } else {
            e.to_string()
        }
    })?;
    if updated == 0 {
        return Err("الموظف غير موجود".into());
    }
    get_staff(conn, id)
}

pub fn delete_staff(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM staff WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("الموظف غير موجود".into());
    }
    Ok(())
}

pub fn work_center_suggestions(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT work_center FROM staff WHERE work_center != '' ORDER BY work_center",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<String>, _>>().map_err(|e| e.to_string())
}

pub fn upsert_staff_by_civil_id(conn: &Connection, input: StaffInput) -> Result<bool, String> {
    validate_input(&input)?;
    let civil_id = input.civil_id.trim();
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM staff WHERE civil_id = ?1",
            params![civil_id],
            |_| Ok(()),
        )
        .is_ok();
    if exists {
        let id: i64 = conn
            .query_row(
                "SELECT id FROM staff WHERE civil_id = ?1",
                params![civil_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        update_staff(conn, id, input)?;
        Ok(false)
    } else {
        create_staff(conn, input)?;
        Ok(true)
    }
}

pub fn staff_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM staff", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}
