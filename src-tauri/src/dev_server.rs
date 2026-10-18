use crate::auth::{
    authenticate, create_user, list_users, session_from_user, Session, UserInfo,
};
use crate::db;
use crate::import::ImportSummary;
use crate::staff::{Staff, StaffInput};
use axum::{
    body::Bytes,
    extract::{Multipart, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

const DEV_DB_DIR: &str = "dev-data";

pub struct DevState {
    db: Mutex<Connection>,
    sessions: Mutex<HashMap<String, Session>>,
}

#[derive(Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    user: UserInfo,
    token: String,
}

#[derive(Deserialize)]
struct CreateUserBody {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct StaffQuery {
    query: Option<String>,
}

fn dev_db_path() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if dir.ends_with("src-tauri") {
        dir.pop();
    }
    dir.join(DEV_DB_DIR).join("hr_app.db")
}

fn json_error(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn require_dev_session(state: &Arc<DevState>, headers: &HeaderMap) -> Result<Session, Response> {
    let token = bearer_token(headers).ok_or_else(|| {
        json_error(StatusCode::UNAUTHORIZED, "يجب تسجيل الدخول")
    })?;
    let sessions = state.sessions.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    sessions
        .get(&token)
        .cloned()
        .ok_or_else(|| json_error(StatusCode::UNAUTHORIZED, "انتهت الجلسة"))
}

pub async fn run() {
    let path = dev_db_path();
    let conn = db::open_db(&path).expect("dev database");
    let state = Arc::new(DevState {
        db: Mutex::new(conn),
        sessions: Mutex::new(HashMap::new()),
    });

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:1420".parse().unwrap(),
            "http://127.0.0.1:1420".parse().unwrap(),
        ])
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/needs_setup", get(needs_setup))
        .route("/api/setup", post(setup))
        .route("/api/login", post(login_route))
        .route("/api/logout", post(logout_route))
        .route("/api/session", get(session_route))
        .route("/api/users", get(list_users_route).post(create_user_route))
        .route("/api/staff", get(list_staff_route).post(create_staff_route))
        .route(
            "/api/staff/total",
            get(staff_total_route),
        )
        .route(
            "/api/staff/{id}",
            get(get_staff_route)
                .put(update_staff_route)
                .delete(delete_staff_route),
        )
        .route("/api/work_centers", get(work_centers_route))
        .route("/api/import", post(import_route))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:1421")
        .await
        .expect("bind 1421");
    println!(
        "hr-dev API on http://127.0.0.1:1421 (DB: {})",
        path.display()
    );
    axum::serve(listener, app).await.expect("serve");
}

async fn needs_setup(State(state): State<Arc<DevState>>) -> Result<Json<bool>, Response> {
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let count = db::user_count(&conn).map_err(|e| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;
    Ok(Json(count == 0))
}

async fn setup(
    State(state): State<Arc<DevState>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<AuthResponse>, Response> {
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let count = db::user_count(&conn).map_err(|e| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;
    if count > 0 {
        return Err(json_error(StatusCode::BAD_REQUEST, "تم إعداد النظام مسبقاً"));
    }
    let user = create_user(&conn, &body.username, &body.password).map_err(|e| {
        json_error(StatusCode::BAD_REQUEST, &e)
    })?;
    let token = uuid::Uuid::new_v4().to_string();
    state
        .sessions
        .lock()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي"))?
        .insert(token.clone(), session_from_user(&user));
    Ok(Json(AuthResponse { user, token }))
}

async fn login_route(
    State(state): State<Arc<DevState>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<AuthResponse>, Response> {
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let user = authenticate(&conn, &body.username, &body.password).map_err(|e| {
        json_error(StatusCode::UNAUTHORIZED, &e)
    })?;
    let token = uuid::Uuid::new_v4().to_string();
    state
        .sessions
        .lock()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي"))?
        .insert(token.clone(), session_from_user(&user));
    Ok(Json(AuthResponse { user, token }))
}

async fn logout_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
) -> Result<StatusCode, Response> {
    if let Some(token) = bearer_token(&headers) {
        state
            .sessions
            .lock()
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي"))?
            .remove(&token);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn session_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
) -> Result<Json<Option<UserInfo>>, Response> {
    let token = bearer_token(&headers);
    if token.is_none() {
        return Ok(Json(None));
    }
    let sessions = state.sessions.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let session = sessions.get(&token.unwrap());
    Ok(Json(session.map(|s| UserInfo {
        id: s.user_id,
        username: s.username.clone(),
        role: s.role.clone(),
    })))
}

async fn list_users_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserInfo>>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let users = list_users(&conn).map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(users))
}

async fn create_user_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    Json(body): Json<CreateUserBody>,
) -> Result<Json<UserInfo>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let user = create_user(&conn, &body.username, &body.password)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(user))
}

async fn list_staff_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    Query(q): Query<StaffQuery>,
) -> Result<Json<Vec<Staff>>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let rows = crate::staff::list_staff(&conn, q.query.as_deref())
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(rows))
}

async fn staff_total_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
) -> Result<Json<i64>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let n = crate::staff::staff_count(&conn)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(n))
}

async fn get_staff_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<Staff>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let row = crate::staff::get_staff(&conn, id)
        .map_err(|e| json_error(StatusCode::NOT_FOUND, &e))?;
    Ok(Json(row))
}

async fn create_staff_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    Json(input): Json<StaffInput>,
) -> Result<Json<Staff>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let row = crate::staff::create_staff(&conn, input)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(row))
}

async fn update_staff_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(input): Json<StaffInput>,
) -> Result<Json<Staff>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let row = crate::staff::update_staff(&conn, id, input)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(row))
}

async fn delete_staff_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    crate::staff::delete_staff(&conn, id)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn work_centers_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<String>>, Response> {
    require_dev_session(&state, &headers)?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let list = crate::staff::work_center_suggestions(&conn)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    Ok(Json(list))
}

async fn import_route(
    State(state): State<Arc<DevState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ImportSummary>, Response> {
    require_dev_session(&state, &headers)?;
    let mut file_bytes: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e.to_string()))?
    {
        if field.name() == Some("file") {
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e.to_string()))?,
            );
            break;
        }
    }
    let bytes = file_bytes.ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "ملف مفقود"))?;
    let tmp = std::env::temp_dir().join(format!("hr_import_{}.xlsx", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, &bytes).map_err(|e| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;
    let conn = state.db.lock().map_err(|_| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "خطأ داخلي")
    })?;
    let summary = crate::import::import_staff_excel(&conn, &tmp)
        .map_err(|e| json_error(StatusCode::BAD_REQUEST, &e))?;
    let _ = std::fs::remove_file(&tmp);
    Ok(Json(summary))
}
