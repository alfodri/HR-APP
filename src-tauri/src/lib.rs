mod auth;
mod db;
mod import;
mod staff;

#[cfg(feature = "dev-server")]
pub mod dev_server;

use auth::{authenticate, create_user, list_users, session_from_user, Session, UserInfo};
use import::ImportSummary;
use rusqlite::Connection;
use staff::{Staff, StaffInput};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub session: Mutex<Option<Session>>,
}

fn with_db<F, T>(state: &AppState, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let conn = state.db.lock().map_err(|_| "خطأ داخلي".to_string())?;
    f(&conn)
}

fn require_session(state: &AppState) -> Result<Session, String> {
    state
        .session
        .lock()
        .map_err(|_| "خطأ داخلي".to_string())?
        .clone()
        .ok_or_else(|| "يجب تسجيل الدخول".to_string())
}

#[tauri::command]
fn needs_setup(state: tauri::State<AppState>) -> Result<bool, String> {
    with_db(&state, |conn| {
        let count = db::user_count(conn).map_err(|e| e.to_string())?;
        Ok(count == 0)
    })
}

#[tauri::command]
fn setup_initial(
    state: tauri::State<AppState>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    with_db(&state, |conn| {
        let count = db::user_count(conn).map_err(|e| e.to_string())?;
        if count > 0 {
            return Err("تم إعداد النظام مسبقاً".into());
        }
        let user = create_user(conn, &username, &password)?;
        *state
            .session
            .lock()
            .map_err(|_| "خطأ داخلي".to_string())? = Some(session_from_user(&user));
        Ok(user)
    })
}

#[tauri::command]
fn login(
    state: tauri::State<AppState>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    with_db(&state, |conn| {
        let user = authenticate(conn, &username, &password)?;
        *state
            .session
            .lock()
            .map_err(|_| "خطأ داخلي".to_string())? = Some(session_from_user(&user));
        Ok(user)
    })
}

#[tauri::command]
fn logout(state: tauri::State<AppState>) -> Result<(), String> {
    *state
        .session
        .lock()
        .map_err(|_| "خطأ داخلي".to_string())? = None;
    Ok(())
}

#[tauri::command]
fn get_session(state: tauri::State<AppState>) -> Result<Option<UserInfo>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "خطأ داخلي".to_string())?;
    Ok(session.as_ref().map(|s| UserInfo {
        id: s.user_id,
        username: s.username.clone(),
        role: s.role.clone(),
    }))
}

#[tauri::command]
fn list_hr_users(state: tauri::State<AppState>) -> Result<Vec<UserInfo>, String> {
    require_session(&state)?;
    with_db(&state, list_users)
}

#[tauri::command]
fn create_hr_user(
    state: tauri::State<AppState>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    require_session(&state)?;
    with_db(&state, |conn| create_user(conn, &username, &password))
}

#[tauri::command]
fn list_staff(
    state: tauri::State<AppState>,
    query: Option<String>,
) -> Result<Vec<Staff>, String> {
    require_session(&state)?;
    with_db(&state, |conn| staff::list_staff(conn, query.as_deref()))
}

#[tauri::command]
fn get_staff(state: tauri::State<AppState>, id: i64) -> Result<Staff, String> {
    require_session(&state)?;
    with_db(&state, |conn| staff::get_staff(conn, id))
}

#[tauri::command]
fn create_staff_record(
    state: tauri::State<AppState>,
    input: StaffInput,
) -> Result<Staff, String> {
    require_session(&state)?;
    with_db(&state, |conn| staff::create_staff(conn, input))
}

#[tauri::command]
fn update_staff_record(
    state: tauri::State<AppState>,
    id: i64,
    input: StaffInput,
) -> Result<Staff, String> {
    require_session(&state)?;
    with_db(&state, |conn| staff::update_staff(conn, id, input))
}

#[tauri::command]
fn delete_staff_record(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    require_session(&state)?;
    with_db(&state, |conn| staff::delete_staff(conn, id))
}

#[tauri::command]
fn work_center_suggestions(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    require_session(&state)?;
    with_db(&state, staff::work_center_suggestions)
}

#[tauri::command]
fn staff_total(state: tauri::State<AppState>) -> Result<i64, String> {
    require_session(&state)?;
    with_db(&state, staff::staff_count)
}

#[tauri::command]
fn import_staff_excel(
    state: tauri::State<AppState>,
    path: String,
) -> Result<ImportSummary, String> {
    require_session(&state)?;
    with_db(&state, |conn| import::import_staff_excel(conn, std::path::Path::new(&path)))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            let db_path = dir.join("hr_app.db");
            let conn = db::open_db(&db_path).expect("failed to open database");
            app.manage(AppState {
                db: Mutex::new(conn),
                session: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            needs_setup,
            setup_initial,
            login,
            logout,
            get_session,
            list_hr_users,
            create_hr_user,
            list_staff,
            get_staff,
            create_staff_record,
            update_staff_record,
            delete_staff_record,
            work_center_suggestions,
            staff_total,
            import_staff_excel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
