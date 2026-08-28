use axum::{
    extract::{Path, State}, http::{header, HeaderMap, HeaderValue, Request, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, routing::{get, patch}, Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{collections::HashMap, env, path::Path as FilePath, sync::{Arc, Mutex}, time::Instant};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Clone)]
struct AppState { db: SqlitePool, build_sha: String, limits: Arc<Mutex<HashMap<String, (Instant, u32)>>> }
#[derive(Serialize, FromRow, Clone)]
struct Skill { id: String, name: String, version: String, summary: String, targets: String, ring: String, updated: String, owner: String, secrets: String }
#[derive(Deserialize)]
struct NewSkill { id: String, name: String, version: String, summary: String, targets: Vec<String>, ring: String, owner: String, secrets: Vec<String> }
#[derive(Deserialize)]
struct RingChange { ring: String }
#[derive(Deserialize)]
struct NewReceipt { skill_id: String, repository: String, agent: String }
#[derive(Serialize, FromRow)]
struct Receipt { id: String, skill: String, version: String, repository: String, agent: String, ring: String, at: String, status: String }
#[derive(Serialize)] struct Health { status: String, build_sha: String }

fn error(status: StatusCode, message: &str) -> Response { (status, Json(serde_json::json!({"error": message}))).into_response() }
fn valid_text(value: &str, max: usize) -> bool { !value.trim().is_empty() && value.len() <= max && !value.contains('\0') }
fn valid_ring(ring: &str) -> bool { matches!(ring, "draft" | "review" | "pilot" | "all") }

async fn list_skills(State(state): State<AppState>) -> Response {
    match sqlx::query_as::<_, Skill>("SELECT id,name,version,summary,targets,ring,updated,owner,secrets FROM skills ORDER BY updated DESC").fetch_all(&state.db).await {
        Ok(items) => Json(items.into_iter().map(skill_json).collect::<Vec<_>>()).into_response(), Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The registry could not load."),
    }
}
fn skill_json(skill: Skill) -> serde_json::Value {
    serde_json::json!({"id":skill.id,"name":skill.name,"version":skill.version,"summary":skill.summary,"targets":serde_json::from_str::<Vec<String>>(&skill.targets).unwrap_or_default(),"ring":skill.ring,"updated":skill.updated,"owner":skill.owner,"secrets":serde_json::from_str::<Vec<String>>(&skill.secrets).unwrap_or_default()})
}
async fn create_skill(State(state): State<AppState>, Json(input): Json<NewSkill>) -> Response {
    if !valid_text(&input.id, 120) || !valid_text(&input.name, 100) || !valid_text(&input.version, 40) || !valid_text(&input.summary, 500) || !valid_text(&input.owner, 100) || !valid_ring(&input.ring) || input.targets.len() > 8 || input.secrets.len() > 16 { return error(StatusCode::BAD_REQUEST, "Check the skill fields and try again."); }
    let updated = Utc::now().format("%d %b").to_string();
    let targets = serde_json::to_string(&input.targets).unwrap_or_default(); let secrets = serde_json::to_string(&input.secrets).unwrap_or_default();
    let result = sqlx::query("INSERT INTO skills (id,name,version,summary,targets,ring,updated,owner,secrets) VALUES (?,?,?,?,?,?,?,?,?)").bind(&input.id).bind(&input.name).bind(&input.version).bind(&input.summary).bind(&targets).bind(&input.ring).bind(&updated).bind(&input.owner).bind(&secrets).execute(&state.db).await;
    match result { Ok(_) => Json(serde_json::json!({"id":input.id,"name":input.name,"version":input.version,"summary":input.summary,"targets":input.targets,"ring":input.ring,"updated":updated,"owner":input.owner,"secrets":input.secrets})).into_response(), Err(_) => error(StatusCode::CONFLICT, "That skill id already exists."), }
}
async fn change_ring(State(state): State<AppState>, Path(id): Path<String>, Json(input): Json<RingChange>) -> Response {
    if !valid_ring(&input.ring) { return error(StatusCode::BAD_REQUEST, "Choose a valid release ring."); }
    match sqlx::query("UPDATE skills SET ring=?, updated=? WHERE id=?").bind(&input.ring).bind(Utc::now().format("%d %b").to_string()).bind(id).execute(&state.db).await { Ok(result) if result.rows_affected() == 1 => StatusCode::NO_CONTENT.into_response(), Ok(_) => error(StatusCode::NOT_FOUND, "That skill no longer exists."), Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The release ring could not save."), }
}
async fn list_receipts(State(state): State<AppState>) -> Response {
    let sql = "SELECT receipts.id, skills.name AS skill, skills.version, receipts.repository, receipts.agent, receipts.ring, receipts.at, receipts.status FROM receipts JOIN skills ON skills.id=receipts.skill_id ORDER BY receipts.created_at DESC LIMIT 100";
    match sqlx::query_as::<_, Receipt>(sql).fetch_all(&state.db).await { Ok(items) => Json(items).into_response(), Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The receipts could not load."), }
}
async fn create_receipt(State(state): State<AppState>, Json(input): Json<NewReceipt>) -> Response {
    if !valid_text(&input.skill_id, 120) || !valid_text(&input.repository, 160) || !valid_text(&input.agent, 80) { return error(StatusCode::BAD_REQUEST, "Name a skill, repository, and agent."); }
    let Some((version, ring)) = sqlx::query_as::<_, (String, String)>("SELECT version, ring FROM skills WHERE id=?").bind(&input.skill_id).fetch_optional(&state.db).await.unwrap_or(None) else { return error(StatusCode::NOT_FOUND, "That skill does not exist."); };
    let id = format!("rcpt-{}", &uuid_like()); let at = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    match sqlx::query("INSERT INTO receipts (id,skill_id,repository,agent,ring,at,status,created_at) VALUES (?,?,?,?,?,?,?,?)").bind(&id).bind(&input.skill_id).bind(&input.repository).bind(&input.agent).bind(&ring).bind(&at).bind("Recorded").bind(Utc::now().timestamp()).execute(&state.db).await { Ok(_) => Json(serde_json::json!({"id":id,"version":version,"ring":ring,"at":at,"status":"Recorded"})).into_response(), Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The receipt could not save."), }
}
fn uuid_like() -> String { format!("{:X}", Utc::now().timestamp_nanos_opt().unwrap_or_default()).chars().rev().take(6).collect::<String>().chars().rev().collect() }
async fn health(State(state): State<AppState>) -> impl IntoResponse { Json(Health { status: "ok".into(), build_sha: state.build_sha }) }
async fn index() -> Response { match tokio::fs::read("dist/index.html").await { Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes).into_response(), Err(_) => error(StatusCode::SERVICE_UNAVAILABLE, "Frontend files are not built."), } }
async fn rate_limit(State(state): State<AppState>, headers: HeaderMap, request: Request<axum::body::Body>, next: Next) -> Response {
    if request.uri().path() == "/health" { return next.run(request).await; }
    let ip = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()).and_then(|v| v.split(',').next()).unwrap_or("local").trim().to_string();
    let denied = { let mut map = state.limits.lock().expect("rate limit lock"); let entry = map.entry(ip).or_insert((Instant::now(), 0)); if entry.0.elapsed().as_secs() >= 1 { *entry = (Instant::now(), 0); } entry.1 += 1; entry.1 > 40 };
    if denied { return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "1")], Json(serde_json::json!({"error":"Too many requests. Wait one second."}))).into_response(); }
    next.run(request).await
}
async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response { let mut response = next.run(request).await; let headers = response.headers_mut(); headers.insert("x-content-type-options", HeaderValue::from_static("nosniff")); headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin")); headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; base-uri 'self'; form-action 'self' https://api.sociobot.in; frame-ancestors 'none'")); response }
async fn initialise(db: &SqlitePool) -> Result<(), sqlx::Error> { sqlx::query("CREATE TABLE IF NOT EXISTS skills (id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL, summary TEXT NOT NULL, targets TEXT NOT NULL, ring TEXT NOT NULL, updated TEXT NOT NULL, owner TEXT NOT NULL, secrets TEXT NOT NULL)").execute(db).await?; sqlx::query("CREATE TABLE IF NOT EXISTS receipts (id TEXT PRIMARY KEY, skill_id TEXT NOT NULL REFERENCES skills(id), repository TEXT NOT NULL, agent TEXT NOT NULL, ring TEXT NOT NULL, at TEXT NOT NULL, status TEXT NOT NULL, created_at INTEGER NOT NULL)").execute(db).await?; Ok(()) }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".into())).init();
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "data/registry.db".into()); if let Some(parent) = FilePath::new(&db_path).parent() { let _ = std::fs::create_dir_all(parent); }
    let database_url = format!("sqlite:{}?mode=rwc", db_path); let db = SqlitePoolOptions::new().max_connections(5).connect(&database_url).await.expect("database opens"); initialise(&db).await.expect("database schema");
    let state = AppState { db, build_sha: env::var("BUILD_SHA").unwrap_or_else(|_| "dev".into()), limits: Arc::new(Mutex::new(HashMap::new())) };
    info!(config="DATABASE_PATH generated/default when absent", "Team Skills Registry started");
    let app = Router::new().route("/health", get(health)).route("/api/skills", get(list_skills).post(create_skill)).route("/api/skills/:id/ring", patch(change_ring)).route("/api/receipts", get(list_receipts).post(create_receipt)).route("/", get(index)).route("/demo", get(index)).route("/registry", get(index)).route("/privacy", get(index)).route("/terms", get(index)).with_state(state.clone()).fallback_service(ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"))).layer(middleware::from_fn(security_headers)).layer(middleware::from_fn_with_state(state, rate_limit));
    let port = env::var("PORT").unwrap_or_else(|_| "8080".into()); let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.expect("port binds"); info!(port, "listening"); axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.expect("server exits");
}
async fn shutdown_signal() { let _ = tokio::signal::ctrl_c().await; }
