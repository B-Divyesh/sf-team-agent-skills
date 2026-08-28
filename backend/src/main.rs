use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{
    collections::HashMap,
    env,
    path::Path as FilePath,
    sync::{Arc, Mutex},
    time::Instant,
};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    build_sha: String,
    limits: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}

#[derive(Serialize, FromRow, Clone)]
struct Skill {
    id: String,
    name: String,
    version: String,
    summary: String,
    targets: String,
    ring: String,
    updated: String,
    owner: String,
    secrets: String,
    instructions: String,
    adapters: String,
    git_url: String,
    git_commit: String,
    package_digest: String,
    repositories: String,
    approved_by: Option<String>,
    approved_at: Option<String>,
}

#[derive(Deserialize)]
struct NewSession {
    name: String,
}
#[derive(Deserialize)]
struct RingChange {
    ring: String,
}
#[derive(Deserialize)]
struct Approval {
    reviewer: String,
    reviewer_key: String,
}
#[derive(Deserialize)]
struct NewReceipt {
    skill_id: String,
    repository: String,
    agent: String,
}
#[derive(Deserialize)]
struct NewSkill {
    id: String,
    name: String,
    version: String,
    summary: String,
    targets: Vec<String>,
    owner: String,
    secrets: Vec<String>,
    instructions: String,
    adapters: serde_json::Value,
    git_url: String,
    git_commit: String,
    repositories: Vec<String>,
}

#[derive(Serialize, FromRow)]
struct Receipt {
    id: String,
    skill: String,
    version: String,
    package_digest: String,
    repository: String,
    agent: String,
    ring: String,
    at: String,
    status: String,
}

#[derive(Serialize)]
struct Health {
    status: String,
    build_sha: String,
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.contains('\0')
}
fn valid_ring(ring: &str) -> bool {
    matches!(ring, "draft" | "review" | "pilot" | "all")
}
fn valid_secret_reference(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && (2..=64).contains(&value.len())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}
fn valid_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit())
}
fn hash(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn random_hex(bytes: usize) -> String {
    let mut value = vec![0_u8; bytes];
    OsRng.fill_bytes(&mut value);
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn json_array(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

async fn workspace(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<String, (StatusCode, &'static str)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() >= 32)
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Open or restore a private workspace first.",
        ))?;
    sqlx::query_scalar::<_, String>("SELECT id FROM workspaces WHERE token_hash = ?")
        .bind(hash(token))
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The workspace could not be checked.",
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "That workspace key is not active.",
        ))
}

async fn create_session(State(state): State<AppState>, Json(input): Json<NewSession>) -> Response {
    if !valid_text(&input.name, 80) {
        return error(StatusCode::BAD_REQUEST, "Name the workspace and try again.");
    }
    let id = format!("ws-{}", random_hex(8));
    let token = format!("tsr_{}", random_hex(24));
    let reviewer_key = format!("tsr_review_{}", random_hex(24));
    match sqlx::query(
        "INSERT INTO workspaces (id,name,token_hash,reviewer_hash,created_at) VALUES (?,?,?,?,?)",
    )
    .bind(&id)
    .bind(input.name.trim())
    .bind(hash(&token))
    .bind(hash(&reviewer_key))
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"workspace_id":id,"token":token,"reviewer_key":reviewer_key})),
        )
            .into_response(),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The private workspace could not be created.",
        ),
    }
}

async fn list_skills(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    let sql = "SELECT id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_commit,package_digest,repositories,approved_by,approved_at FROM skills WHERE workspace_id=? ORDER BY created_at DESC";
    match sqlx::query_as::<_, Skill>(sql)
        .bind(workspace_id)
        .fetch_all(&state.db)
        .await
    {
        Ok(items) => Json(items.into_iter().map(skill_json).collect::<Vec<_>>()).into_response(),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The registry could not load.",
        ),
    }
}

fn skill_json(skill: Skill) -> serde_json::Value {
    serde_json::json!({
        "id":skill.id,"name":skill.name,"version":skill.version,"summary":skill.summary,
        "targets":json_array(&skill.targets),"ring":skill.ring,"updated":skill.updated,
        "owner":skill.owner,"secrets":json_array(&skill.secrets),"instructions":skill.instructions,
        "adapters":serde_json::from_str::<serde_json::Value>(&skill.adapters).unwrap_or_else(|_| serde_json::json!({})),
        "git_url":skill.git_url,"git_commit":skill.git_commit,"package_digest":skill.package_digest,
        "repositories":json_array(&skill.repositories),"approved_by":skill.approved_by,"approved_at":skill.approved_at
    })
}

async fn create_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<NewSkill>,
) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    let adapters = match serde_json::to_string(&input.adapters) {
        Ok(value) if input.adapters.is_object() && value.len() <= 20_000 => value,
        _ => {
            return error(
                StatusCode::BAD_REQUEST,
                "Provide adapter instructions as an object.",
            )
        }
    };
    if !valid_text(&input.id, 120)
        || !valid_text(&input.name, 100)
        || !valid_text(&input.version, 40)
        || !valid_text(&input.summary, 500)
        || !valid_text(&input.owner, 100)
        || !valid_text(&input.instructions, 20_000)
        || !valid_text(&input.git_url, 500)
        || !input.git_url.starts_with("https://")
        || !valid_commit(&input.git_commit)
        || input.targets.is_empty()
        || input.targets.len() > 8
        || input.repositories.is_empty()
        || input.repositories.len() > 32
        || input.secrets.len() > 16
        || input
            .secrets
            .iter()
            .any(|item| !valid_secret_reference(item))
        || input.repositories.iter().any(|item| !valid_text(item, 160))
    {
        return error(
            StatusCode::BAD_REQUEST,
            "Check every package field and use secret names, never secret values.",
        );
    }
    let targets = serde_json::to_string(&input.targets).unwrap_or_default();
    let secrets = serde_json::to_string(&input.secrets).unwrap_or_default();
    let repositories = serde_json::to_string(&input.repositories).unwrap_or_default();
    let digest = hash(&format!(
        "{}\n{}\n{}\n{}\n{}",
        input.version, input.instructions, adapters, input.git_url, input.git_commit
    ));
    let updated = Utc::now().to_rfc3339();
    let result = sqlx::query("INSERT INTO skills (id,workspace_id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_commit,package_digest,repositories,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&input.id).bind(&workspace_id).bind(&input.name).bind(&input.version).bind(&input.summary)
        .bind(&targets).bind("draft").bind(&updated).bind(&input.owner).bind(&secrets)
        .bind(&input.instructions).bind(&adapters).bind(&input.git_url).bind(input.git_commit.to_ascii_lowercase())
        .bind(&digest).bind(&repositories).bind(Utc::now().timestamp()).execute(&state.db).await;
    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({
            "id":input.id,"name":input.name,"version":input.version,"summary":input.summary,
            "targets":input.targets,"ring":"draft","updated":updated,"owner":input.owner,
            "secrets":input.secrets,"instructions":input.instructions,"adapters":input.adapters,
            "git_url":input.git_url,"git_commit":input.git_commit.to_ascii_lowercase(),
            "package_digest":digest,"repositories":input.repositories,"approved_by":null,"approved_at":null
        }))).into_response(),
        Err(value) if value.to_string().contains("UNIQUE") => error(StatusCode::CONFLICT, "That immutable skill version already exists."),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The skill package could not be published."),
    }
}

async fn approve_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Approval>,
) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    if !valid_text(&input.reviewer, 100) {
        return error(StatusCode::BAD_REQUEST, "Name the reviewer and try again.");
    }
    let reviewer_allowed = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM workspaces WHERE id=? AND reviewer_hash=?",
    )
    .bind(&workspace_id)
    .bind(hash(&input.reviewer_key))
    .fetch_one(&state.db)
    .await
    .unwrap_or(0)
        == 1;
    if !reviewer_allowed {
        return error(
            StatusCode::FORBIDDEN,
            "A valid reviewer key is required for approval.",
        );
    }
    let now = Utc::now().to_rfc3339();
    let mut tx = match state.db.begin().await {
        Ok(v) => v,
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Review could not start."),
    };
    let result = sqlx::query("UPDATE skills SET approved_by=?,approved_at=?,ring='review',updated=? WHERE id=? AND workspace_id=? AND approved_at IS NULL")
        .bind(input.reviewer.trim()).bind(&now).bind(&now).bind(&id).bind(&workspace_id).execute(&mut *tx).await;
    match result {
        Ok(result) if result.rows_affected() == 1 => {
            let approval_id = format!("apr-{}", random_hex(6));
            if sqlx::query("INSERT INTO approvals (id,workspace_id,skill_id,reviewer,approved_at) VALUES (?,?,?,?,?)")
                .bind(&approval_id).bind(&workspace_id).bind(&id).bind(input.reviewer.trim()).bind(&now).execute(&mut *tx).await.is_err()
                || tx.commit().await.is_err() {
                return error(StatusCode::INTERNAL_SERVER_ERROR, "The approval record could not save.");
            }
            Json(serde_json::json!({"id":approval_id,"reviewer":input.reviewer.trim(),"approved_at":now,"ring":"review"})).into_response()
        }
        Ok(_) => error(
            StatusCode::CONFLICT,
            "That version is missing or already reviewed.",
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The approval could not save.",
        ),
    }
}

async fn change_ring(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RingChange>,
) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    if !valid_ring(&input.ring) {
        return error(StatusCode::BAD_REQUEST, "Choose a valid release ring.");
    }
    let requires_approval = matches!(input.ring.as_str(), "pilot" | "all");
    let sql = if requires_approval {
        "UPDATE skills SET ring=?,updated=? WHERE id=? AND workspace_id=? AND approved_at IS NOT NULL"
    } else {
        "UPDATE skills SET ring=?,updated=? WHERE id=? AND workspace_id=?"
    };
    match sqlx::query(sql)
        .bind(&input.ring)
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .bind(workspace_id)
        .execute(&state.db)
        .await
    {
        Ok(result) if result.rows_affected() == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) if requires_approval => error(
            StatusCode::CONFLICT,
            "A recorded review is required before this release.",
        ),
        Ok(_) => error(
            StatusCode::NOT_FOUND,
            "That skill version no longer exists.",
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The release ring could not save.",
        ),
    }
}

async fn list_receipts(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    let sql = "SELECT id,skill_name AS skill,skill_version AS version,package_digest,repository,agent,ring,at,status FROM receipts WHERE workspace_id=? ORDER BY created_at DESC LIMIT 100";
    match sqlx::query_as::<_, Receipt>(sql)
        .bind(workspace_id)
        .fetch_all(&state.db)
        .await
    {
        Ok(items) => Json(items).into_response(),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The receipts could not load.",
        ),
    }
}

async fn create_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<NewReceipt>,
) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    if !valid_text(&input.skill_id, 120)
        || !valid_text(&input.repository, 160)
        || !valid_text(&input.agent, 80)
    {
        return error(
            StatusCode::BAD_REQUEST,
            "Name a skill, repository, and agent.",
        );
    }
    let row = sqlx::query_as::<_, (String,String,String,String,String)>("SELECT name,version,ring,package_digest,repositories FROM skills WHERE id=? AND workspace_id=? AND approved_at IS NOT NULL")
        .bind(&input.skill_id).bind(&workspace_id).fetch_optional(&state.db).await;
    let (name, version, ring, digest, repositories) = match row {
        Ok(Some(v)) => v,
        Ok(None) => {
            return error(
                StatusCode::CONFLICT,
                "Only a reviewed skill version can record a run.",
            )
        }
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The skill version could not be checked.",
            )
        }
    };
    if !json_array(&repositories).contains(&input.repository) {
        return error(
            StatusCode::FORBIDDEN,
            "This skill version is not assigned to that repository.",
        );
    }
    let id = format!("rcpt-{}", random_hex(5));
    let at = Utc::now().to_rfc3339();
    match sqlx::query("INSERT INTO receipts (id,workspace_id,skill_id,skill_name,skill_version,package_digest,repository,agent,ring,at,status,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(&workspace_id).bind(&input.skill_id).bind(&name).bind(&version).bind(&digest)
        .bind(&input.repository).bind(&input.agent).bind(&ring).bind(&at).bind("Recorded").bind(Utc::now().timestamp()).execute(&state.db).await
    {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id":id,"skill":name,"version":version,"package_digest":digest,"repository":input.repository,"agent":input.agent,"ring":ring,"at":at,"status":"Recorded"}))).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The receipt could not save."),
    }
}

async fn install_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((repository, id)): Path<(String, String)>,
) -> Response {
    let workspace_id = match workspace(&headers, &state).await {
        Ok(v) => v,
        Err((status, message)) => return error(status, message),
    };
    let sql = "SELECT id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_commit,package_digest,repositories,approved_by,approved_at FROM skills WHERE id=? AND workspace_id=? AND approved_at IS NOT NULL AND ring IN ('pilot','all')";
    match sqlx::query_as::<_, Skill>(sql).bind(id).bind(workspace_id).fetch_optional(&state.db).await {
        Ok(Some(skill)) if json_array(&skill.repositories).contains(&repository) => Json(serde_json::json!({"schema":"team-agent-skill/v1","repository":repository,"package":skill_json(skill)})).into_response(),
        Ok(Some(_)) => error(StatusCode::FORBIDDEN, "This released version is not assigned to that repository."),
        Ok(None) => error(StatusCode::NOT_FOUND, "No reviewed released package matches that request."),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The package could not be installed."),
    }
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(Health {
        status: "ok".into(),
        build_sha: state.build_sha,
    })
}
async fn index() -> Response {
    match tokio::fs::read("dist/index.html").await {
        Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes).into_response(),
        Err(_) => error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Frontend files are not built.",
        ),
    }
}

async fn rate_limit(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.split(',').next())
        .unwrap_or("local")
        .trim()
        .to_string();
    let denied = {
        let mut map = state.limits.lock().expect("rate limit lock");
        let entry = map.entry(ip).or_insert((Instant::now(), 0));
        if entry.0.elapsed().as_secs() >= 1 {
            *entry = (Instant::now(), 0);
        }
        entry.1 += 1;
        entry.1 > 40
    };
    if denied {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "1")],
            Json(serde_json::json!({"error":"Too many requests. Wait one second."})),
        )
            .into_response();
    }
    next.run(request).await
}

async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
    let path = request.uri().path();
    let asset = path.starts_with("/assets/");
    let image = path.ends_with(".webp") || path.ends_with(".svg");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"));
    if asset {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }
    if image {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=604800"),
        );
    }
    response
}

async fn add_column(db: &SqlitePool, sql: &str) {
    let _ = sqlx::query(sql).execute(db).await;
}
async fn initialise(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA foreign_keys=ON").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS workspaces (id TEXT PRIMARY KEY,name TEXT NOT NULL,token_hash TEXT NOT NULL UNIQUE,reviewer_hash TEXT NOT NULL,created_at INTEGER NOT NULL)").execute(db).await?;
    add_column(
        db,
        "ALTER TABLE workspaces ADD COLUMN reviewer_hash TEXT NOT NULL DEFAULT ''",
    )
    .await;
    sqlx::query("CREATE TABLE IF NOT EXISTS skills (id TEXT PRIMARY KEY,workspace_id TEXT NOT NULL,name TEXT NOT NULL,version TEXT NOT NULL,summary TEXT NOT NULL,targets TEXT NOT NULL,ring TEXT NOT NULL,updated TEXT NOT NULL,owner TEXT NOT NULL,secrets TEXT NOT NULL,instructions TEXT NOT NULL,adapters TEXT NOT NULL,git_url TEXT NOT NULL,git_commit TEXT NOT NULL,package_digest TEXT NOT NULL,repositories TEXT NOT NULL,approved_by TEXT,approved_at TEXT,created_at INTEGER NOT NULL)").execute(db).await?;
    for statement in [
        "ALTER TABLE skills ADD COLUMN workspace_id TEXT NOT NULL DEFAULT 'legacy'",
        "ALTER TABLE skills ADD COLUMN instructions TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN adapters TEXT NOT NULL DEFAULT '{}'",
        "ALTER TABLE skills ADD COLUMN git_url TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN git_commit TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN package_digest TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN repositories TEXT NOT NULL DEFAULT '[]'",
        "ALTER TABLE skills ADD COLUMN approved_by TEXT",
        "ALTER TABLE skills ADD COLUMN approved_at TEXT",
        "ALTER TABLE skills ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0",
    ] {
        add_column(db, statement).await;
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS approvals (id TEXT PRIMARY KEY,workspace_id TEXT NOT NULL,skill_id TEXT NOT NULL,reviewer TEXT NOT NULL,approved_at TEXT NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS receipts (id TEXT PRIMARY KEY,workspace_id TEXT NOT NULL,skill_id TEXT NOT NULL,skill_name TEXT NOT NULL,skill_version TEXT NOT NULL,package_digest TEXT NOT NULL,repository TEXT NOT NULL,agent TEXT NOT NULL,ring TEXT NOT NULL,at TEXT NOT NULL,status TEXT NOT NULL,created_at INTEGER NOT NULL)").execute(db).await?;
    for statement in [
        "ALTER TABLE receipts ADD COLUMN workspace_id TEXT NOT NULL DEFAULT 'legacy'",
        "ALTER TABLE receipts ADD COLUMN skill_name TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE receipts ADD COLUMN skill_version TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE receipts ADD COLUMN package_digest TEXT NOT NULL DEFAULT ''",
    ] {
        add_column(db, statement).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "data/registry.db".into());
    if let Some(parent) = FilePath::new(&db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let database_url = format!("sqlite:{db_path}?mode=rwc");
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("database opens");
    initialise(&db).await.expect("database schema");
    let state = AppState {
        db,
        build_sha: env::var("BUILD_SHA").unwrap_or_else(|_| "dev".into()),
        limits: Arc::new(Mutex::new(HashMap::new())),
    };
    info!(
        config =
            "DATABASE_PATH generated/default when absent; workspace keys generated per request",
        "Team Skills Registry started"
    );
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/session", post(create_session))
        .route("/api/skills", get(list_skills).post(create_skill))
        .route("/api/skills/:id/approve", post(approve_skill))
        .route("/api/skills/:id/ring", patch(change_ring))
        .route("/api/receipts", get(list_receipts).post(create_receipt))
        .route(
            "/api/repositories/:repository/install/:id",
            get(install_skill),
        )
        .route("/", get(index))
        .route("/demo", get(index))
        .route("/registry", get(index))
        .route("/privacy", get(index))
        .route("/terms", get(index))
        .with_state(state.clone())
        .fallback_service(ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html")))
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn_with_state(state, rate_limit));
    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("port binds");
    info!(port, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server exits");
}
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
