use axum::{
    extract::{connect_info::ConnectInfo, Path, State},
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fs::OpenOptions,
    io::Write,
    net::{IpAddr, SocketAddr},
    path::{Path as FilePath, PathBuf},
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
    signer: Arc<SigningKey>,
    git_api_base: String,
    http: reqwest::Client,
}

#[derive(Serialize, FromRow, Clone)]
struct Skill {
    workspace_id: String,
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
    git_credential_ref: String,
    git_commit: String,
    source_path: String,
    source_blob_sha: String,
    source_verified_at: String,
    package_digest: String,
    package_signature: String,
    signer_public_key: String,
    repositories: String,
    pilot_repositories: String,
    approved_by: Option<String>,
    approved_at: Option<String>,
    approval_id: Option<String>,
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
struct ReviewApproval {
    reviewer: String,
}
#[derive(Deserialize)]
struct NewReceipt {
    skill_id: String,
    repository: String,
    agent: String,
}
#[derive(Deserialize, Clone)]
struct SourceSkill {
    id: String,
    name: String,
    version: String,
    summary: String,
    targets: Vec<String>,
    owner: String,
    secrets: Vec<String>,
    instructions: String,
    adapters: BTreeMap<String, String>,
    repositories: Vec<String>,
    pilot_repositories: Vec<String>,
}
#[derive(Deserialize)]
struct PublishRequest {
    git_url: String,
    git_credential_ref: Option<String>,
    git_commit: String,
    source_path: String,
}
#[derive(Deserialize)]
struct GitCredentialBindingRequest {
    reference: String,
    git_url: String,
}
#[derive(Deserialize)]
struct InstallCredentialRequest {
    repository: String,
    agent: String,
}

#[derive(Serialize)]
struct PackageEnvelope<'a> {
    schema: &'static str,
    workspace_id: &'a str,
    id: &'a str,
    name: &'a str,
    version: &'a str,
    summary: &'a str,
    targets: &'a [String],
    owner: &'a str,
    secrets: &'a [String],
    instructions: &'a str,
    adapters: &'a BTreeMap<String, String>,
    git_url: &'a str,
    git_commit: &'a str,
    source_path: &'a str,
    source_blob_sha: &'a str,
    source_verified_at: &'a str,
    repositories: &'a [String],
    pilot_repositories: &'a [String],
}

#[derive(Serialize)]
struct ReceiptEnvelope<'a> {
    schema: &'static str,
    id: &'a str,
    workspace_id: &'a str,
    skill_id: &'a str,
    skill_name: &'a str,
    skill_version: &'a str,
    package_digest: &'a str,
    package_signature: &'a str,
    approval_id: &'a str,
    repository: &'a str,
    agent: &'a str,
    ring: &'a str,
    at: &'a str,
    status: &'static str,
}

#[derive(Serialize, FromRow)]
struct Receipt {
    id: String,
    skill: String,
    version: String,
    package_digest: String,
    package_signature: String,
    approval_id: String,
    repository: String,
    agent: String,
    ring: String,
    at: String,
    status: String,
    receipt_signature: String,
    signer_public_key: String,
}

#[derive(Serialize)]
struct Health {
    status: String,
    build_sha: String,
    signer_fingerprint: String,
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
fn hash_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn hash(value: &str) -> String {
    hash_bytes(value.as_bytes())
}
fn random_hex(bytes: usize) -> String {
    let mut value = vec![0_u8; bytes];
    OsRng.fill_bytes(&mut value);
    hex::encode(value)
}
fn json_array(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}
fn json_map(raw: &str) -> BTreeMap<String, String> {
    serde_json::from_str(raw).unwrap_or_default()
}
fn signer_public_key(state: &AppState) -> String {
    hex::encode(state.signer.verifying_key().to_bytes())
}
fn signer_fingerprint(state: &AppState) -> String {
    hash_bytes(&state.signer.verifying_key().to_bytes())
}
fn sign_bytes(state: &AppState, bytes: &[u8]) -> String {
    hex::encode(state.signer.sign(bytes).to_bytes())
}
struct GitSource<'a> {
    url: &'a str,
    commit: &'a str,
    path: &'a str,
    blob_sha: &'a str,
    verified_at: &'a str,
}

fn package_bytes(
    workspace_id: &str,
    input: &SourceSkill,
    source: GitSource<'_>,
) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&PackageEnvelope {
        schema: "team-agent-skill-envelope/v2",
        workspace_id,
        id: &input.id,
        name: &input.name,
        version: &input.version,
        summary: &input.summary,
        targets: &input.targets,
        owner: &input.owner,
        secrets: &input.secrets,
        instructions: &input.instructions,
        adapters: &input.adapters,
        git_url: source.url,
        git_commit: source.commit,
        source_path: source.path,
        source_blob_sha: source.blob_sha,
        source_verified_at: source.verified_at,
        repositories: &input.repositories,
        pilot_repositories: &input.pilot_repositories,
    })
}
fn stored_package_bytes(skill: &Skill) -> Result<Vec<u8>, serde_json::Error> {
    let targets = json_array(&skill.targets);
    let secrets = json_array(&skill.secrets);
    let adapters = json_map(&skill.adapters);
    let repositories = json_array(&skill.repositories);
    let source = GitSource {
        url: &skill.git_url,
        commit: &skill.git_commit,
        path: &skill.source_path,
        blob_sha: &skill.source_blob_sha,
        verified_at: &skill.source_verified_at,
    };
    serde_json::to_vec(&PackageEnvelope {
        schema: "team-agent-skill-envelope/v2",
        workspace_id: &skill.workspace_id,
        id: &skill.id,
        name: &skill.name,
        version: &skill.version,
        summary: &skill.summary,
        targets: &targets,
        owner: &skill.owner,
        secrets: &secrets,
        instructions: &skill.instructions,
        adapters: &adapters,
        git_url: source.url,
        git_commit: source.commit,
        source_path: source.path,
        source_blob_sha: source.blob_sha,
        source_verified_at: source.verified_at,
        repositories: &repositories,
        pilot_repositories: &json_array(&skill.pilot_repositories),
    })
}
fn package_is_valid(state: &AppState, skill: &Skill) -> bool {
    if skill.signer_public_key != signer_public_key(state) {
        return false;
    }
    let Ok(bytes) = stored_package_bytes(skill) else {
        return false;
    };
    if hash_bytes(&bytes) != skill.package_digest {
        return false;
    }
    let Ok(key_vec) = hex::decode(&skill.signer_public_key) else {
        return false;
    };
    let Ok(sig_vec) = hex::decode(&skill.package_signature) else {
        return false;
    };
    let Ok(key_bytes): Result<[u8; 32], _> = key_vec.try_into() else {
        return false;
    };
    let Ok(sig_bytes): Result<[u8; 64], _> = sig_vec.try_into() else {
        return false;
    };
    let Ok(key) = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes) else {
        return false;
    };
    key.verify(&bytes, &Signature::from_bytes(&sig_bytes))
        .is_ok()
}
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() >= 32)
}
async fn owner_workspace(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<String, (StatusCode, &'static str)> {
    let token = bearer(headers).ok_or((
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
    match sqlx::query("INSERT INTO workspaces (id,name,token_hash,created_at) VALUES (?,?,?,?)")
        .bind(&id)
        .bind(input.name.trim())
        .bind(hash(&token))
        .bind(Utc::now().timestamp())
        .execute(&state.db)
        .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"workspace_id":id,"token":token})),
        )
            .into_response(),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The private workspace could not be created.",
        ),
    }
}

const SKILL_SELECT: &str = "SELECT workspace_id,id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_credential_ref,git_commit,source_path,source_blob_sha,source_verified_at,package_digest,package_signature,signer_public_key,repositories,pilot_repositories,approved_by,approved_at,approval_id FROM skills";
async fn list_skills(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    let sql = format!("{SKILL_SELECT} WHERE workspace_id=? ORDER BY created_at DESC");
    match sqlx::query_as::<_, Skill>(&sql)
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
    let signed_payload = stored_package_bytes(&skill)
        .map(hex::encode)
        .unwrap_or_default();
    serde_json::json!({"id":skill.id,"name":skill.name,"version":skill.version,"summary":skill.summary,
        "targets":json_array(&skill.targets),"ring":skill.ring,"updated":skill.updated,"owner":skill.owner,
        "secrets":json_array(&skill.secrets),"instructions":skill.instructions,"adapters":json_map(&skill.adapters),
        "git_url":skill.git_url,"git_credential_ref":skill.git_credential_ref,"git_commit":skill.git_commit,"source_verified_at":skill.source_verified_at,
        "source_path":skill.source_path,"source_blob_sha":skill.source_blob_sha,
        "package_digest":skill.package_digest,"package_signature":skill.package_signature,
        "signer_public_key":skill.signer_public_key,"signed_payload":signed_payload,
        "repositories":json_array(&skill.repositories),
        "pilot_repositories":json_array(&skill.pilot_repositories),
        "approved_by":skill.approved_by,"approved_at":skill.approved_at,"approval_id":skill.approval_id})
}

fn github_repository(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("https://github.com/")?;
    if rest.contains(['?', '#']) {
        return None;
    }
    let parts: Vec<&str> = rest.trim_end_matches('/').split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let repo = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    if parts[0].is_empty() || repo.is_empty() {
        return None;
    }
    Some((parts[0].to_string(), repo.to_string()))
}
fn canonical_github_url(url: &str) -> Option<String> {
    let (owner, repo) = github_repository(url)?;
    Some(format!("https://github.com/{owner}/{repo}"))
}
fn configured_credential_repository(reference: &str) -> Result<String, (StatusCode, &'static str)> {
    let configured = env::var(format!("GIT_CREDENTIAL_{reference}_REPOSITORY")).map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "That deployment-managed Git credential reference has no repository boundary.",
        )
    })?;
    canonical_github_url(&configured).ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "That deployment-managed Git credential repository boundary is invalid.",
    ))
}
async fn credential_for_source(
    state: &AppState,
    workspace_id: &str,
    git_url: &str,
    credential_ref: Option<&str>,
) -> Result<Option<String>, (StatusCode, &'static str)> {
    let Some(reference) = credential_ref.filter(|reference| !reference.is_empty()) else {
        return Ok(None);
    };
    if !valid_secret_reference(reference) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Use an uppercase Git credential reference name.",
        ));
    }
    let expected_url = canonical_github_url(git_url).ok_or((
        StatusCode::BAD_REQUEST,
        "Use a GitHub repository URL for source verification.",
    ))?;
    let bound_url = sqlx::query_scalar::<_, String>(
        "SELECT git_url FROM git_credential_bindings WHERE workspace_id=? AND credential_ref=?",
    )
    .bind(workspace_id)
    .bind(reference)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The Git credential binding could not be checked.",
        )
    })?;
    if bound_url.as_deref() != Some(expected_url.as_str()) {
        return Err((
            StatusCode::FORBIDDEN,
            "That Git credential reference is not bound to this repository in this workspace.",
        ));
    }
    if configured_credential_repository(reference)? != expected_url {
        return Err((
            StatusCode::FORBIDDEN,
            "That Git credential reference cannot read this repository.",
        ));
    }
    env::var(format!("GIT_CREDENTIAL_{reference}"))
        .map(Some)
        .map_err(|_| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                "That deployment-managed Git credential reference is not available.",
            )
        })
}
async fn bind_git_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GitCredentialBindingRequest>,
) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    if !valid_secret_reference(&input.reference) {
        return error(
            StatusCode::BAD_REQUEST,
            "Use an uppercase Git credential reference name.",
        );
    }
    let Some(git_url) = canonical_github_url(&input.git_url) else {
        return error(
            StatusCode::BAD_REQUEST,
            "Use a GitHub repository URL when binding a Git credential reference.",
        );
    };
    let configured_repository = match configured_credential_repository(&input.reference) {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    if configured_repository != git_url {
        return error(
            StatusCode::FORBIDDEN,
            "That Git credential reference cannot be bound to this repository.",
        );
    }
    if env::var(format!("GIT_CREDENTIAL_{}", input.reference)).is_err() {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "That deployment-managed Git credential reference is not available.",
        );
    }
    let existing_workspace = match sqlx::query_scalar::<_, String>(
        "SELECT workspace_id FROM git_credential_bindings WHERE credential_ref=?",
    )
    .bind(&input.reference)
    .fetch_optional(&state.db)
    .await
    {
        Ok(value) => value,
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The Git credential binding could not be checked.",
            )
        }
    };
    if existing_workspace
        .as_deref()
        .is_some_and(|owner| owner != workspace_id)
    {
        return error(
            StatusCode::FORBIDDEN,
            "That Git credential reference belongs to a different workspace.",
        );
    }
    match sqlx::query("INSERT INTO git_credential_bindings (workspace_id,credential_ref,git_url,created_at) VALUES (?,?,?,?) ON CONFLICT(workspace_id,credential_ref) DO UPDATE SET git_url=excluded.git_url,created_at=excluded.created_at")
        .bind(&workspace_id)
        .bind(&input.reference)
        .bind(&git_url)
        .bind(Utc::now().timestamp())
        .execute(&state.db)
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"reference": input.reference, "git_url": git_url}))).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The Git credential reference could not be bound."),
    }
}
async fn verify_git_source(
    state: &AppState,
    workspace_id: &str,
    url: &str,
    credential_ref: Option<&str>,
    commit: &str,
    source_path: &str,
) -> Result<(SourceSkill, String), (StatusCode, &'static str)> {
    let (owner, repo) = github_repository(url).ok_or((
        StatusCode::BAD_REQUEST,
        "Use a GitHub repository URL for source verification.",
    ))?;
    let credential = credential_for_source(state, workspace_id, url, credential_ref).await?;
    let endpoint = format!(
        "{}/repos/{owner}/{repo}/commits/{commit}",
        state.git_api_base.trim_end_matches('/')
    );
    let mut commit_request = state.http.get(endpoint);
    if let Some(token) = credential.as_deref() {
        commit_request = commit_request.bearer_auth(token);
    }
    let response = commit_request.send().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            "GitHub source verification is unavailable. Try again.",
        )
    })?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "GitHub could not find that commit in the named repository.",
        ));
    }
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            "GitHub source verification is unavailable. Try again.",
        ));
    }
    let body: serde_json::Value = response.json().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            "GitHub returned an unreadable verification response.",
        )
    })?;
    if !body["sha"]
        .as_str()
        .is_some_and(|sha| sha.eq_ignore_ascii_case(commit))
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "GitHub did not verify that exact commit.",
        ));
    }
    if !valid_source_path(source_path) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Use a safe JSON package path inside the repository.",
        ));
    }
    let content_endpoint = format!(
        "{}/repos/{owner}/{repo}/contents/{source_path}?ref={commit}",
        state.git_api_base.trim_end_matches('/')
    );
    let mut content_request = state.http.get(content_endpoint);
    if let Some(token) = credential.as_deref() {
        content_request = content_request.bearer_auth(token);
    }
    let content_response = content_request.send().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            "GitHub package verification is unavailable. Try again.",
        )
    })?;
    if content_response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "GitHub could not find that package file at the verified commit.",
        ));
    }
    if !content_response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            "GitHub package verification is unavailable. Try again.",
        ));
    }
    let content: serde_json::Value = content_response.json().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            "GitHub returned an unreadable package file.",
        )
    })?;
    let blob_sha = content["sha"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or((
            StatusCode::BAD_GATEWAY,
            "GitHub did not return a package blob identifier.",
        ))?;
    let encoded = content["content"].as_str().ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "The source path must point to a UTF-8 JSON skill package.",
    ))?;
    let compact: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = BASE64.decode(compact).map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "The source package is not valid base64 content.",
        )
    })?;
    let source = serde_json::from_slice::<SourceSkill>(&decoded).map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "The source package must be valid JSON with every immutable skill field.",
        )
    })?;
    if !valid_package(&source) {
        return Err((
            StatusCode::BAD_REQUEST,
            "The source package has invalid skill fields or pilot membership.",
        ));
    }
    Ok((source, blob_sha.to_string()))
}
fn valid_source_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value.ends_with(".json")
        && !value.starts_with('/')
        && !value.contains("..")
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}
fn valid_package(input: &SourceSkill) -> bool {
    let targets: HashSet<&str> = input.targets.iter().map(String::as_str).collect();
    let repositories: HashSet<&str> = input.repositories.iter().map(String::as_str).collect();
    valid_text(&input.id, 120)
        && valid_text(&input.name, 100)
        && valid_text(&input.version, 40)
        && valid_text(&input.summary, 500)
        && valid_text(&input.owner, 100)
        && valid_text(&input.instructions, 20_000)
        && !input.targets.is_empty()
        && input.targets.len() <= 8
        && targets.len() == input.targets.len()
        && input.targets.iter().all(|item| valid_text(item, 80))
        && !input.repositories.is_empty()
        && input.repositories.len() <= 32
        && repositories.len() == input.repositories.len()
        && input.repositories.iter().all(|item| valid_text(item, 160))
        && !input.pilot_repositories.is_empty()
        && input.pilot_repositories.len() <= input.repositories.len()
        && input
            .pilot_repositories
            .iter()
            .all(|item| repositories.contains(item.as_str()))
        && input
            .pilot_repositories
            .iter()
            .collect::<HashSet<_>>()
            .len()
            == input.pilot_repositories.len()
        && input.secrets.len() <= 16
        && input
            .secrets
            .iter()
            .all(|item| valid_secret_reference(item))
        && input.adapters.len() == targets.len()
        && input
            .adapters
            .keys()
            .all(|target| targets.contains(target.as_str()))
        && input.adapters.values().all(|text| valid_text(text, 20_000))
}

async fn create_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut request): Json<PublishRequest>,
) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    request.git_commit.make_ascii_lowercase();
    if !valid_commit(&request.git_commit) || !valid_source_path(&request.source_path) {
        return error(
            StatusCode::BAD_REQUEST,
            "Use a 40-character commit and a safe JSON package path.",
        );
    }
    let (input, source_blob_sha) = match verify_git_source(
        &state,
        &workspace_id,
        &request.git_url,
        request.git_credential_ref.as_deref(),
        &request.git_commit,
        &request.source_path,
    )
    .await
    {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    let source_verified_at = Utc::now().to_rfc3339();
    let bytes = match package_bytes(
        &workspace_id,
        &input,
        GitSource {
            url: &request.git_url,
            commit: &request.git_commit,
            path: &request.source_path,
            blob_sha: &source_blob_sha,
            verified_at: &source_verified_at,
        },
    ) {
        Ok(value) => value,
        Err(_) => return error(StatusCode::BAD_REQUEST, "The package could not be encoded."),
    };
    let signed_payload = hex::encode(&bytes);
    let digest = hash_bytes(&bytes);
    let signature = sign_bytes(&state, &bytes);
    let public_key = signer_public_key(&state);
    let reviewer_key = format!("tsr_review_{}", random_hex(24));
    let updated = Utc::now().to_rfc3339();
    let targets = serde_json::to_string(&input.targets).unwrap_or_default();
    let secrets = serde_json::to_string(&input.secrets).unwrap_or_default();
    let adapters = serde_json::to_string(&input.adapters).unwrap_or_default();
    let repositories = serde_json::to_string(&input.repositories).unwrap_or_default();
    let pilot_repositories = serde_json::to_string(&input.pilot_repositories).unwrap_or_default();
    let result = sqlx::query("INSERT INTO skills (workspace_id,id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_credential_ref,git_commit,source_path,source_blob_sha,source_verified_at,package_digest,package_signature,signer_public_key,repositories,pilot_repositories,reviewer_hash,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&workspace_id).bind(&input.id).bind(&input.name).bind(&input.version).bind(&input.summary).bind(&targets)
        .bind("draft").bind(&updated).bind(&input.owner).bind(&secrets).bind(&input.instructions).bind(&adapters)
        .bind(&request.git_url).bind(request.git_credential_ref.as_deref().unwrap_or("")).bind(&request.git_commit).bind(&request.source_path).bind(&source_blob_sha).bind(&source_verified_at).bind(&digest).bind(&signature)
        .bind(&public_key).bind(&repositories).bind(&pilot_repositories).bind(hash(&reviewer_key)).bind(Utc::now().timestamp()).execute(&state.db).await;
    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id":input.id,"name":input.name,"version":input.version,
            "summary":input.summary,"targets":input.targets,"ring":"draft","updated":updated,"owner":input.owner,
            "secrets":input.secrets,"instructions":input.instructions,"adapters":input.adapters,"git_url":request.git_url,"git_credential_ref":request.git_credential_ref,
            "git_commit":request.git_commit,"source_path":request.source_path,"source_blob_sha":source_blob_sha,"source_verified_at":source_verified_at,"package_digest":digest,
            "package_signature":signature,"signer_public_key":public_key,"signed_payload":signed_payload,
            "repositories":input.repositories,"pilot_repositories":input.pilot_repositories,
            "approved_by":null,"approved_at":null,"approval_id":null,"reviewer_key":reviewer_key}))).into_response(),
        Err(value) if value.to_string().contains("UNIQUE") => error(StatusCode::CONFLICT, "That skill id or name and version already exists in this workspace."),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The skill package could not be published."),
    }
}

async fn review_package(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(key) = bearer(&headers) else {
        return error(StatusCode::UNAUTHORIZED, "Enter the package reviewer key.");
    };
    let sql = format!("{SKILL_SELECT} WHERE reviewer_hash=? AND reviewer_used_at IS NULL");
    match sqlx::query_as::<_, Skill>(&sql)
        .bind(hash(key))
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(skill)) if package_is_valid(&state, &skill) => {
            Json(skill_json(skill)).into_response()
        }
        Ok(Some(_)) => error(
            StatusCode::CONFLICT,
            "The signed package failed verification.",
        ),
        Ok(None) => error(
            StatusCode::UNAUTHORIZED,
            "That reviewer key is invalid or already used.",
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The review package could not load.",
        ),
    }
}
async fn approve_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ReviewApproval>,
) -> Response {
    let Some(key) = bearer(&headers) else {
        return error(StatusCode::UNAUTHORIZED, "Enter the package reviewer key.");
    };
    if !valid_text(&input.reviewer, 100) {
        return error(StatusCode::BAD_REQUEST, "Name the reviewer and try again.");
    }
    let key_hash = hash(key);
    let mut tx = match state.db.begin().await {
        Ok(value) => value,
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Review could not start."),
    };
    let review_sql = format!(
        "{SKILL_SELECT} WHERE reviewer_hash=? AND reviewer_used_at IS NULL AND approved_at IS NULL"
    );
    let row = sqlx::query_as::<_, Skill>(&review_sql)
        .bind(&key_hash)
        .fetch_optional(&mut *tx)
        .await;
    let skill = match row {
        Ok(Some(value)) if package_is_valid(&state, &value) => value,
        Ok(Some(_)) => {
            return error(
                StatusCode::CONFLICT,
                "The signed package failed verification.",
            )
        }
        Ok(None) => {
            return error(
                StatusCode::UNAUTHORIZED,
                "That reviewer key is invalid or already used.",
            )
        }
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The package could not be checked.",
            )
        }
    };
    let workspace_id = skill.workspace_id;
    let skill_id = skill.id;
    let package_digest = skill.package_digest;
    let now = Utc::now().to_rfc3339();
    let approval_id = format!("apr-{}", random_hex(8));
    let result = sqlx::query("UPDATE skills SET approved_by=?,approved_at=?,approval_id=?,reviewer_used_at=?,ring='review',updated=? WHERE workspace_id=? AND id=? AND reviewer_hash=? AND reviewer_used_at IS NULL")
        .bind(input.reviewer.trim()).bind(&now).bind(&approval_id).bind(&now).bind(&now).bind(&workspace_id).bind(&skill_id).bind(&key_hash).execute(&mut *tx).await;
    if !matches!(result, Ok(ref value) if value.rows_affected() == 1) {
        return error(StatusCode::CONFLICT, "This package was already reviewed.");
    }
    if sqlx::query("INSERT INTO approvals (id,workspace_id,skill_id,package_digest,reviewer,approved_at) VALUES (?,?,?,?,?,?)")
        .bind(&approval_id).bind(&workspace_id).bind(&skill_id).bind(&package_digest).bind(input.reviewer.trim()).bind(&now).execute(&mut *tx).await.is_err() || tx.commit().await.is_err() {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "The approval record could not save.");
    }
    Json(serde_json::json!({"id":approval_id,"skill_id":skill_id,"package_digest":package_digest,"reviewer":input.reviewer.trim(),"approved_at":now,"ring":"review"})).into_response()
}

async fn change_ring(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RingChange>,
) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
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
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    let sql = "SELECT id,skill_name AS skill,skill_version AS version,package_digest,package_signature,approval_id,repository,agent,ring,at,status,receipt_signature,signer_public_key FROM receipts WHERE workspace_id=? ORDER BY created_at DESC LIMIT 100";
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
fn repository_is_released(skill: &Skill, repository: &str) -> bool {
    match skill.ring.as_str() {
        "all" => json_array(&skill.repositories)
            .iter()
            .any(|item| item == repository),
        "pilot" => json_array(&skill.pilot_repositories)
            .iter()
            .any(|item| item == repository),
        _ => false,
    }
}
async fn issue_install_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<InstallCredentialRequest>,
) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
        Err((status, message)) => return error(status, message),
    };
    if !valid_text(&input.repository, 160) || !valid_text(&input.agent, 80) {
        return error(
            StatusCode::BAD_REQUEST,
            "Name an assigned repository and agent.",
        );
    }
    let sql = format!("{SKILL_SELECT} WHERE id=? AND workspace_id=?");
    let skill = match sqlx::query_as::<_, Skill>(&sql)
        .bind(&id)
        .bind(&workspace_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "That skill version no longer exists.",
            )
        }
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The skill version could not be checked.",
            )
        }
    };
    if !json_array(&skill.repositories).contains(&input.repository)
        || !json_array(&skill.targets).contains(&input.agent)
    {
        return error(
            StatusCode::FORBIDDEN,
            "This credential must match an assigned repository and agent.",
        );
    }
    let credential = format!("tsr_install_{}", random_hex(24));
    match sqlx::query("INSERT INTO install_credentials (workspace_id,skill_id,repository,agent,token_hash,created_at) VALUES (?,?,?,?,?,?)")
        .bind(&workspace_id).bind(&skill.id).bind(&input.repository).bind(&input.agent).bind(hash(&credential)).bind(Utc::now().timestamp()).execute(&state.db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"credential":credential,"skill_id":skill.id,"repository":input.repository,"agent":input.agent}))).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The scoped install credential could not be issued."),
    }
}
async fn create_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<NewReceipt>,
) -> Response {
    let workspace_id = match owner_workspace(&headers, &state).await {
        Ok(value) => value,
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
    let sql = format!("{SKILL_SELECT} WHERE id=? AND workspace_id=? AND approved_at IS NOT NULL AND ring IN ('pilot','all')");
    let skill = match sqlx::query_as::<_, Skill>(&sql)
        .bind(&input.skill_id)
        .bind(&workspace_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return error(
                StatusCode::CONFLICT,
                "Only an installed pilot or full release can record a run.",
            )
        }
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The skill version could not be checked.",
            )
        }
    };
    if !package_is_valid(&state, &skill) {
        return error(
            StatusCode::CONFLICT,
            "The signed package failed verification.",
        );
    }
    if !repository_is_released(&skill, &input.repository) {
        return error(
            StatusCode::FORBIDDEN,
            "This repository is not in the active release ring.",
        );
    }
    if !json_array(&skill.targets).contains(&input.agent) {
        return error(
            StatusCode::FORBIDDEN,
            "This skill version has no adapter for that agent.",
        );
    }
    let Some(approval_id) = skill.approval_id.as_deref() else {
        return error(StatusCode::CONFLICT, "The package has no approval record.");
    };
    let id = format!("rcpt-{}", random_hex(8));
    let at = Utc::now().to_rfc3339();
    let receipt_bytes = serde_json::to_vec(&ReceiptEnvelope {
        schema: "team-agent-execution-receipt/v2",
        id: &id,
        workspace_id: &workspace_id,
        skill_id: &skill.id,
        skill_name: &skill.name,
        skill_version: &skill.version,
        package_digest: &skill.package_digest,
        package_signature: &skill.package_signature,
        approval_id,
        repository: &input.repository,
        agent: &input.agent,
        ring: &skill.ring,
        at: &at,
        status: "Recorded",
    })
    .unwrap_or_default();
    let receipt_signature = sign_bytes(&state, &receipt_bytes);
    let public_key = signer_public_key(&state);
    match sqlx::query("INSERT INTO receipts (id,workspace_id,skill_id,skill_name,skill_version,package_digest,package_signature,approval_id,repository,agent,ring,at,status,receipt_signature,signer_public_key,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(&workspace_id).bind(&skill.id).bind(&skill.name).bind(&skill.version).bind(&skill.package_digest)
        .bind(&skill.package_signature).bind(approval_id).bind(&input.repository).bind(&input.agent).bind(&skill.ring).bind(&at)
        .bind("Recorded").bind(&receipt_signature).bind(&public_key).bind(Utc::now().timestamp()).execute(&state.db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id":id,"skill":skill.name,"version":skill.version,
            "package_digest":skill.package_digest,"package_signature":skill.package_signature,"approval_id":approval_id,
            "repository":input.repository,"agent":input.agent,"ring":skill.ring,"at":at,"status":"Recorded",
            "receipt_signature":receipt_signature,"signer_public_key":public_key}))).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The receipt could not save."),
    }
}

async fn install_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((repository, agent, id)): Path<(String, String, String)>,
) -> Response {
    let Some(credential) = bearer(&headers) else {
        return error(
            StatusCode::UNAUTHORIZED,
            "Use a repository and agent scoped install credential.",
        );
    };
    let workspace_id: String = match sqlx::query_scalar("SELECT workspace_id FROM install_credentials WHERE token_hash=? AND skill_id=? AND repository=? AND agent=?")
        .bind(hash(credential)).bind(&id).bind(&repository).bind(&agent).fetch_optional(&state.db).await {
        Ok(Some(value)) => value,
        Ok(None) => return error(StatusCode::UNAUTHORIZED, "That install credential cannot read this package."),
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "The install credential could not be checked."),
    };
    let sql = format!("{SKILL_SELECT} WHERE id=? AND workspace_id=? AND approved_at IS NOT NULL AND ring IN ('pilot','all')");
    match sqlx::query_as::<_, Skill>(&sql).bind(id).bind(workspace_id).fetch_optional(&state.db).await {
        Ok(Some(skill)) if !repository_is_released(&skill, &repository) => error(StatusCode::FORBIDDEN, "This repository is not in the active release ring."),
        Ok(Some(skill)) if !json_array(&skill.targets).contains(&agent) => error(StatusCode::FORBIDDEN, "This skill version has no adapter for that agent."),
        Ok(Some(skill)) if !package_is_valid(&state, &skill) => error(StatusCode::CONFLICT, "The signed package failed verification."),
        Ok(Some(skill)) => Json(serde_json::json!({"schema":"team-agent-skill/v2","repository":repository,"agent":agent,"package":skill_json(skill)})).into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "No reviewed released package matches that request."),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "The package could not be installed."),
    }
}

async fn trust(State(state): State<AppState>) -> impl IntoResponse {
    Json(
        serde_json::json!({"algorithm":"Ed25519","public_key":signer_public_key(&state),"fingerprint":signer_fingerprint(&state)}),
    )
}
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(Health {
        status: "ok".into(),
        build_sha: state.build_sha.clone(),
        signer_fingerprint: signer_fingerprint(&state),
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
    // Factory ingress overwrites X-Forwarded-For before it reaches this
    // container. Its first hop is the trusted client identity. Direct local
    // runs retain a peer-address fallback for a stable per-client bucket.
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .and_then(|value| value.parse::<IpAddr>().ok())
        .map(|value| format!("forwarded:{value}"))
        .or_else(|| {
            request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|peer| format!("peer:{}", peer.0.ip()))
        })
        .unwrap_or_else(|| "unknown".to_string());
    let denied = {
        let mut map = state.limits.lock().expect("rate limit lock");
        map.retain(|_, (started, _)| started.elapsed().as_secs() < 2);
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
async fn migrate_workspaces(db: &SqlitePool) -> Result<(), sqlx::Error> {
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='workspaces'",
    )
    .fetch_optional(db)
    .await?;
    if existing
        .as_deref()
        .is_some_and(|sql| sql.contains("reviewer_hash"))
    {
        sqlx::query("ALTER TABLE workspaces RENAME TO workspaces_legacy_v1")
            .execute(db)
            .await?;
        sqlx::query("CREATE TABLE workspaces (id TEXT PRIMARY KEY,name TEXT NOT NULL,token_hash TEXT NOT NULL UNIQUE,created_at INTEGER NOT NULL)")
            .execute(db).await?;
        sqlx::query("INSERT INTO workspaces (id,name,token_hash,created_at) SELECT id,name,token_hash,created_at FROM workspaces_legacy_v1")
            .execute(db).await?;
        sqlx::query("DROP TABLE workspaces_legacy_v1")
            .execute(db)
            .await?;
    } else if existing.is_none() {
        sqlx::query("CREATE TABLE workspaces (id TEXT PRIMARY KEY,name TEXT NOT NULL,token_hash TEXT NOT NULL UNIQUE,created_at INTEGER NOT NULL)")
            .execute(db).await?;
    }
    Ok(())
}
async fn migrate_skills(db: &SqlitePool) -> Result<(), sqlx::Error> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type='table' AND name='skills'")
            .fetch_optional(db)
            .await?;
    if existing
        .as_deref()
        .is_some_and(|sql| sql.contains("PRIMARY KEY(workspace_id,id)"))
    {
        return Ok(());
    }
    if existing.is_some() {
        sqlx::query("ALTER TABLE skills RENAME TO skills_legacy_v1")
            .execute(db)
            .await?;
    }
    sqlx::query("CREATE TABLE skills (workspace_id TEXT NOT NULL,id TEXT NOT NULL,name TEXT NOT NULL,version TEXT NOT NULL,summary TEXT NOT NULL,targets TEXT NOT NULL,ring TEXT NOT NULL,updated TEXT NOT NULL,owner TEXT NOT NULL,secrets TEXT NOT NULL,instructions TEXT NOT NULL,adapters TEXT NOT NULL,git_url TEXT NOT NULL,git_commit TEXT NOT NULL,source_path TEXT NOT NULL DEFAULT '',source_blob_sha TEXT NOT NULL DEFAULT '',source_verified_at TEXT NOT NULL DEFAULT '',package_digest TEXT NOT NULL,package_signature TEXT NOT NULL DEFAULT '',signer_public_key TEXT NOT NULL DEFAULT '',repositories TEXT NOT NULL,pilot_repositories TEXT NOT NULL DEFAULT '[]',approved_by TEXT,approved_at TEXT,approval_id TEXT,reviewer_hash TEXT NOT NULL DEFAULT '',reviewer_used_at TEXT,created_at INTEGER NOT NULL,PRIMARY KEY(workspace_id,id),UNIQUE(workspace_id,name,version))").execute(db).await?;
    if existing.is_some() {
        sqlx::query("INSERT OR IGNORE INTO skills (workspace_id,id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_commit,package_digest,repositories,approved_by,approved_at,created_at) SELECT workspace_id,id,name,version,summary,targets,ring,updated,owner,secrets,instructions,adapters,git_url,git_commit,package_digest,repositories,approved_by,approved_at,created_at FROM skills_legacy_v1").execute(db).await?;
        sqlx::query("DROP TABLE skills_legacy_v1")
            .execute(db)
            .await?;
    }
    Ok(())
}
async fn initialise(db: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA foreign_keys=ON").execute(db).await?;
    migrate_workspaces(db).await?;
    migrate_skills(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS approvals (id TEXT PRIMARY KEY,workspace_id TEXT NOT NULL,skill_id TEXT NOT NULL,package_digest TEXT NOT NULL DEFAULT '',reviewer TEXT NOT NULL,approved_at TEXT NOT NULL)").execute(db).await?;
    add_column(
        db,
        "ALTER TABLE approvals ADD COLUMN package_digest TEXT NOT NULL DEFAULT ''",
    )
    .await;
    sqlx::query("CREATE TABLE IF NOT EXISTS receipts (id TEXT PRIMARY KEY,workspace_id TEXT NOT NULL,skill_id TEXT NOT NULL,skill_name TEXT NOT NULL,skill_version TEXT NOT NULL,package_digest TEXT NOT NULL,package_signature TEXT NOT NULL DEFAULT '',approval_id TEXT NOT NULL DEFAULT '',repository TEXT NOT NULL,agent TEXT NOT NULL,ring TEXT NOT NULL,at TEXT NOT NULL,status TEXT NOT NULL,receipt_signature TEXT NOT NULL DEFAULT '',signer_public_key TEXT NOT NULL DEFAULT '',created_at INTEGER NOT NULL)").execute(db).await?;
    for statement in [
        "ALTER TABLE receipts ADD COLUMN package_signature TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE receipts ADD COLUMN approval_id TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE receipts ADD COLUMN receipt_signature TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE receipts ADD COLUMN signer_public_key TEXT NOT NULL DEFAULT ''",
    ] {
        add_column(db, statement).await;
    }
    for statement in [
        "ALTER TABLE skills ADD COLUMN source_path TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN source_blob_sha TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE skills ADD COLUMN pilot_repositories TEXT NOT NULL DEFAULT '[]'",
        "ALTER TABLE skills ADD COLUMN git_credential_ref TEXT NOT NULL DEFAULT ''",
    ] {
        add_column(db, statement).await;
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS install_credentials (workspace_id TEXT NOT NULL,skill_id TEXT NOT NULL,repository TEXT NOT NULL,agent TEXT NOT NULL,token_hash TEXT NOT NULL UNIQUE,created_at INTEGER NOT NULL)").execute(db).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS git_credential_bindings (workspace_id TEXT NOT NULL,credential_ref TEXT NOT NULL,git_url TEXT NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(workspace_id,credential_ref))").execute(db).await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS git_credential_reference_unique ON git_credential_bindings (credential_ref)").execute(db).await?;
    Ok(())
}
fn signing_key(path: &FilePath) -> Result<(SigningKey, bool), String> {
    if path.exists() {
        let encoded = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        let bytes: [u8; 32] = hex::decode(encoded.trim())
            .map_err(|error| error.to_string())?
            .try_into()
            .map_err(|_| "signing key has the wrong length".to_string())?;
        return Ok((SigningKey::from_bytes(&bytes), false));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .and_then(|mut file| file.write_all(hex::encode(bytes).as_bytes()))
        .map_err(|error| error.to_string())?;
    Ok((SigningKey::from_bytes(&bytes), true))
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
    let key_path = env::var("SIGNING_KEY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| FilePath::new(&db_path).with_file_name("registry-signing.key"));
    let (signer, generated) = signing_key(&key_path).expect("signing identity loads");
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
        signer: Arc::new(signer),
        git_api_base: env::var("GIT_VERIFY_API_BASE")
            .unwrap_or_else(|_| "https://api.github.com".into()),
        http: reqwest::Client::builder()
            .user_agent("team-agent-skills/1.2")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("HTTP client builds"),
    };
    info!(
        database = if env::var_os("DATABASE_PATH").is_some() {
            "supplied"
        } else {
            "generated/default"
        },
        signing_identity = if generated {
            "generated"
        } else {
            "persisted/supplied"
        },
        "Team Skills Registry started"
    );
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/trust", get(trust))
        .route("/api/session", post(create_session))
        .route("/api/git-credentials", post(bind_git_credential))
        .route("/api/skills", get(list_skills).post(create_skill))
        .route("/api/review", get(review_package))
        .route("/api/review/approve", post(approve_review))
        .route("/api/skills/:id/ring", patch(change_ring))
        .route(
            "/api/skills/:id/install-credentials",
            post(issue_install_credential),
        )
        .route("/api/receipts", get(list_receipts).post(create_receipt))
        .route(
            "/api/repositories/:repository/agents/:agent/install/:id",
            get(install_skill),
        )
        .route("/", get(index))
        .route("/demo", get(index))
        .route("/registry", get(index))
        .route("/review", get(index))
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("server exits");
}
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
