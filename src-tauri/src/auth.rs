use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Hr,
}

impl Role {
    fn from_db(_s: &str) -> Self {
        Role::Hr
    }

    fn as_db(&self) -> &'static str {
        "hr"
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub role: Role,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: i64,
    pub username: String,
    pub role: Role,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = PasswordHash::new(hash).ok();
    parsed
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

pub fn create_user(conn: &Connection, username: &str, password: &str) -> Result<UserInfo, String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("اسم المستخدم مطلوب".into());
    }
    if password.len() < 6 {
        return Err("كلمة المرور يجب أن تكون 6 أحرف على الأقل".into());
    }
    let password_hash = hash_password(password)?;
    conn.execute(
        "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
        params![username, password_hash, Role::Hr.as_db()],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "اسم المستخدم مستخدم مسبقاً".to_string()
        } else {
            e.to_string()
        }
    })?;
    let id = conn.last_insert_rowid();
    Ok(UserInfo {
        id,
        username: username.to_string(),
        role: Role::Hr,
    })
}

pub fn authenticate(
    conn: &Connection,
    username: &str,
    password: &str,
) -> Result<UserInfo, String> {
    let username = username.trim();
    let row = conn
        .query_row(
            "SELECT id, username, password_hash, role FROM users WHERE username = ?1 COLLATE NOCASE",
            params![username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| "اسم المستخدم أو كلمة المرور غير صحيحة".to_string())?;

    if !verify_password(password, &row.2) {
        return Err("اسم المستخدم أو كلمة المرور غير صحيحة".into());
    }
    let role = Role::from_db(&row.3);
    Ok(UserInfo {
        id: row.0,
        username: row.1,
        role,
    })
}

pub fn list_users(conn: &Connection) -> Result<Vec<UserInfo>, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, role FROM users ORDER BY username")
        .map_err(|e| e.to_string())?;
    let users = stmt
        .query_map([], |row| {
            let role_str: String = row.get(2)?;
            Ok(UserInfo {
                id: row.get(0)?,
                username: row.get(1)?,
                role: Role::from_db(&role_str),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(users)
}

pub fn session_from_user(user: &UserInfo) -> Session {
    Session {
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
    }
}
