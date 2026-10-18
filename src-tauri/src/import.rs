use calamine::{open_workbook_auto, Data, Reader};
use chrono::{Duration, NaiveDate};
use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;

use crate::staff::{StaffInput, upsert_staff_by_civil_id};

#[derive(Debug, Clone, Serialize)]
pub struct ImportSummary {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
    pub skip_messages: Vec<String>,
}

const HEADER_WORK_CENTER: &str = "مركز العمل";
const HEADER_NAME: &str = "الاسم";
const HEADER_FILE: &str = "رقم الملف";
const HEADER_CIVIL: &str = "الرقم المدني";
const HEADER_JOB: &str = "المسمي الوظيفي";
const HEADER_HIRE: &str = "تاريخ التعيين";
const HEADER_GRADE: &str = "الدرجة";
const HEADER_EMAIL: &str = "البريد الإلكتروني الوزاري";
const HEADER_PHONE: &str = "هاتف";
const HEADER_QUAL: &str = "المؤهل العلمي";

pub fn parse_hire_date(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(serial) = s.parse::<i64>() {
        if (20_000..=80_000).contains(&serial) {
            return excel_serial_to_ymd(serial).map(|d| d.format("%Y-%m-%d").to_string());
        }
    }
    if let Ok(serial) = s.parse::<f64>() {
        let serial = serial.round() as i64;
        if (20_000..=80_000).contains(&serial) {
            return excel_serial_to_ymd(serial).map(|d| d.format("%Y-%m-%d").to_string());
        }
    }
    let formats = ["%d-%m-%Y", "%d/%m/%Y", "%Y-%m-%d"];
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d.format("%Y-%m-%d").to_string());
        }
    }
    None
}

fn excel_serial_to_ymd(serial: i64) -> Option<NaiveDate> {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    epoch.checked_add_signed(Duration::days(serial))
}

fn normalize_header(h: &str) -> String {
    h.trim().replace('\u{00a0}', " ").trim().to_string()
}

fn cell_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 && *f >= 1.0 && *f < 1e15 {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::DateTime(dt) => excel_serial_to_ymd(dt.as_f64().round() as i64)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
        Data::Bool(b) => b.to_string(),
        Data::Error(_) | Data::Empty => String::new(),
        _ => String::new(),
    }
}

struct ColumnMap {
    work_center: Option<usize>,
    name: Option<usize>,
    file_number: Option<usize>,
    civil_id: Option<usize>,
    job_title: Option<usize>,
    hire_date: Option<usize>,
    grade: Option<usize>,
    email: Option<usize>,
    phone: Option<usize>,
    qualification: Option<usize>,
}

impl ColumnMap {
    fn from_headers(headers: &[String]) -> Self {
        let mut map = ColumnMap {
            work_center: None,
            name: None,
            file_number: None,
            civil_id: None,
            job_title: None,
            hire_date: None,
            grade: None,
            email: None,
            phone: None,
            qualification: None,
        };
        for (i, h) in headers.iter().enumerate() {
            let n = normalize_header(h);
            match n.as_str() {
                s if s.starts_with(HEADER_WORK_CENTER) => map.work_center = Some(i),
                s if s == HEADER_NAME => map.name = Some(i),
                s if s.starts_with(HEADER_FILE) => map.file_number = Some(i),
                s if s.starts_with(HEADER_CIVIL) => map.civil_id = Some(i),
                s if s.starts_with(HEADER_JOB) => map.job_title = Some(i),
                s if s.starts_with(HEADER_HIRE) => map.hire_date = Some(i),
                s if s.starts_with(HEADER_GRADE) => map.grade = Some(i),
                s if s.starts_with(HEADER_EMAIL) => map.email = Some(i),
                s if s == HEADER_PHONE => map.phone = Some(i),
                s if s.starts_with(HEADER_QUAL) => map.qualification = Some(i),
                _ => {}
            }
        }
        map
    }

    fn get(row: &[Data], idx: Option<usize>) -> String {
        idx.and_then(|i| row.get(i))
            .map(cell_string)
            .unwrap_or_default()
    }
}

pub fn import_staff_excel(conn: &Connection, path: &Path) -> Result<ImportSummary, String> {
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "لا توجد أوراق في الملف".to_string())?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<Vec<Data>> = Vec::new();
    for row in range.rows() {
        rows.push(row.to_vec());
    }
    if rows.is_empty() {
        return Err("الملف فارغ".into());
    }

    let header_cells: Vec<String> = rows[0].iter().map(cell_string).collect();
    let cols = ColumnMap::from_headers(&header_cells);
    if cols.civil_id.is_none() || cols.name.is_none() {
        return Err("رؤوس الأعمدة غير معروفة. تأكد من وجود الاسم والرقم المدني".into());
    }

    let mut summary = ImportSummary {
        added: 0,
        updated: 0,
        skipped: 0,
        skip_messages: Vec::new(),
    };

    for (row_idx, row) in rows.iter().enumerate().skip(1) {
        let civil_id = ColumnMap::get(row, cols.civil_id);
        let name = ColumnMap::get(row, cols.name);
        if civil_id.is_empty() {
            summary.skipped += 1;
            if summary.skip_messages.len() < 50 {
                summary
                    .skip_messages
                    .push(format!("صف {}: بدون رقم مدني", row_idx + 1));
            }
            continue;
        }
        if name.is_empty() {
            summary.skipped += 1;
            if summary.skip_messages.len() < 50 {
                summary
                    .skip_messages
                    .push(format!("صف {}: بدون اسم", row_idx + 1));
            }
            continue;
        }

        let hire_raw = ColumnMap::get(row, cols.hire_date);
        let hire_date = parse_hire_date(&hire_raw);

        let input = StaffInput {
            work_center: ColumnMap::get(row, cols.work_center),
            name,
            file_number: ColumnMap::get(row, cols.file_number),
            civil_id,
            job_title: ColumnMap::get(row, cols.job_title),
            hire_date,
            grade: ColumnMap::get(row, cols.grade),
            email: ColumnMap::get(row, cols.email),
            phone: ColumnMap::get(row, cols.phone),
            qualification: ColumnMap::get(row, cols.qualification),
        };

        match upsert_staff_by_civil_id(conn, input) {
            Ok(true) => summary.added += 1,
            Ok(false) => summary.updated += 1,
            Err(e) => {
                summary.skipped += 1;
                if summary.skip_messages.len() < 50 {
                    summary
                        .skip_messages
                        .push(format!("صف {}: {}", row_idx + 1, e));
                }
            }
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db;
    use std::env;

    #[test]
    fn parse_excel_serial_date() {
        let d = parse_hire_date("39816").expect("serial");
        assert_eq!(d, "2009-01-03");
    }

    #[test]
    fn parse_dash_and_slash_dates() {
        assert_eq!(parse_hire_date("22-02-1997"), Some("1997-02-22".into()));
        assert_eq!(parse_hire_date("15/09/2021"), Some("2021-09-15".into()));
    }

    #[test]
    fn empty_db_then_import_flow() {
        use crate::auth::create_user;
        let db_path = std::env::temp_dir().join("hr_app_flow_test.db");
        let _ = std::fs::remove_file(&db_path);
        let conn = open_db(&db_path).expect("db");
        assert_eq!(crate::db::user_count(&conn).unwrap(), 0);
        assert_eq!(crate::staff::staff_count(&conn).unwrap(), 0);
        create_user(&conn, "hr1", "secret12").expect("user");
        assert_eq!(crate::db::user_count(&conn).unwrap(), 1);
        assert_eq!(crate::staff::staff_count(&conn).unwrap(), 0);
    }

    #[test]
    fn import_staff_new_xlsx() {
        let root = env::current_dir().expect("cwd");
        let path = if root.ends_with("src-tauri") {
            root.join("../staff-new.xlsx")
        } else {
            root.join("staff-new.xlsx")
        };
        assert!(path.exists(), "missing staff-new.xlsx at {}", path.display());
        let db_path = std::env::temp_dir().join("hr_app_import_test.db");
        let _ = std::fs::remove_file(&db_path);
        let conn = open_db(&db_path).expect("db");
        let summary = import_staff_excel(&conn, &path).expect("import");
        assert_eq!(summary.added, 228);
        assert_eq!(summary.updated, 0);
        assert_eq!(summary.skipped, 0);
        let count = crate::staff::staff_count(&conn).expect("count");
        assert_eq!(count, 228);
    }
}
