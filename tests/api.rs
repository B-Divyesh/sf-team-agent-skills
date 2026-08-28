use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use reqwest::blocking::{Client, RequestBuilder};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    process::{Child, Command},
    sync::{Arc, Barrier},
    thread,
    time::Duration,
};

const COMMIT: &str = "7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve port")
        .local_addr()
        .expect("read port")
        .port()
}

fn start_git_verifier() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock Git verifier");
    let address = listener.local_addr().expect("mock address");
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = [0_u8; 4096];
            let size = stream.read(&mut request).unwrap_or(0);
            let request = String::from_utf8_lossy(&request[..size]);
            let requested_commit = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|path| path.rsplit('/').next())
                .unwrap_or("");
            let body = if requested_commit == COMMIT {
                format!(r#"{{"sha":"{COMMIT}"}}"#)
            } else {
                r#"{"sha":"not-the-requested-commit"}"#.to_string()
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://{address}")
}

fn start_server(database_path: &Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_team-agent-skills"))
        .env("DATABASE_PATH", database_path)
        .env("PORT", port.to_string())
        .env("BUILD_SHA", "repair-regression")
        .env("GIT_VERIFY_API_BASE", start_git_verifier())
        .spawn()
        .expect("start server")
}

struct Harness {
    child: Child,
    client: Client,
    base: String,
    data: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
        let data = tempfile::tempdir().expect("temp data");
        let port = free_port();
        let mut child = start_server(&data.path().join("registry.db"), port);
        let client = Client::new();
        let base = format!("http://127.0.0.1:{port}");
        for _ in 0..100 {
            if client
                .get(format!("{base}/health"))
                .send()
                .is_ok_and(|response| response.status().is_success())
            {
                return Self {
                    child,
                    client,
                    base,
                    data,
                };
            }
            thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        panic!("server did not become healthy");
    }

    fn session(&self, name: &str) -> String {
        self.client
            .post(format!("{}/api/session", self.base))
            .json(&json!({"name":name}))
            .send()
            .expect("create workspace")
            .error_for_status()
            .expect("workspace accepted")
            .json::<Value>()
            .expect("workspace json")["token"]
            .as_str()
            .expect("owner key")
            .to_string()
    }

    fn auth(&self, token: &str, method: reqwest::Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{}{}", self.base, path))
            .bearer_auth(token)
    }

    fn create(&self, token: &str, value: &Value) -> Value {
        self.auth(token, reqwest::Method::POST, "/api/skills")
            .json(value)
            .send()
            .expect("publish response")
            .error_for_status()
            .expect("publish accepted")
            .json()
            .expect("publish json")
    }

    fn approve(&self, reviewer_key: &str, reviewer: &str) -> Value {
        self.auth(reviewer_key, reqwest::Method::POST, "/api/review/approve")
            .json(&json!({"reviewer":reviewer}))
            .send()
            .expect("approve response")
            .error_for_status()
            .expect("approval accepted")
            .json()
            .expect("approval json")
    }

    fn restart(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let port = free_port();
        self.child = start_server(&self.data.path().join("registry.db"), port);
        self.base = format!("http://127.0.0.1:{port}");
        for _ in 0..100 {
            if self
                .client
                .get(format!("{}/health", self.base))
                .send()
                .is_ok_and(|response| response.status().is_success())
            {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("restarted server did not become healthy");
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn package(id: &str, name: &str, secrets: Value) -> Value {
    json!({
        "id":id,"name":name,"version":"1.2.3","summary":"Check a release.",
        "targets":["Codex"],"owner":"Mina","secrets":secrets,
        "instructions":"Run tests and inspect the diff before release.",
        "adapters":{"Codex":"Read AGENTS.md and run the configured test command."},
        "git_url":"https://github.com/B-Divyesh/sf-team-agent-skills",
        "git_commit":COMMIT,"repositories":["atlas-api"]
    })
}

#[doc = "@claim:private-workspace"]
#[test]
fn claim_private_workspace() {
    let harness = Harness::new();
    assert_eq!(
        harness
            .client
            .get(format!("{}/api/skills", harness.base))
            .send()
            .unwrap()
            .status(),
        401
    );
    let token_a = harness.session("Team A");
    let token_b = harness.session("Team B");
    let database = std::fs::read(harness.data.path().join("registry.db")).expect("read database");
    assert!(
        !String::from_utf8_lossy(&database).contains(&token_a),
        "owner key is stored only as a hash"
    );
    harness.create(&token_a, &package("shared-id", "Shared package", json!([])));
    let isolated: Vec<Value> = harness
        .auth(&token_b, reqwest::Method::GET, "/api/skills")
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert!(isolated.is_empty());
    let second = harness
        .auth(&token_b, reqwest::Method::POST, "/api/skills")
        .json(&package("shared-id", "Shared package", json!([])))
        .send()
        .unwrap();
    assert_eq!(
        second.status(),
        201,
        "ids and versions are scoped to a workspace"
    );
}

#[doc = "@claim:secret-reference-format"]
#[test]
fn claim_secret_reference_format() {
    let harness = Harness::new();
    let token = harness.session("Secrets");
    let bad = harness
        .auth(&token, reqwest::Method::POST, "/api/skills")
        .json(&package(
            "bad",
            "Bad secret",
            json!(["ghp_actualSecretValue"]),
        ))
        .send()
        .unwrap();
    assert_eq!(bad.status(), 400);
    let good = harness
        .auth(&token, reqwest::Method::POST, "/api/skills")
        .json(&package("good", "Good secret", json!(["GITHUB_TOKEN"])))
        .send()
        .unwrap();
    assert_eq!(good.status(), 201);
}

#[doc = "@claim:git-signed-package"]
#[test]
fn claim_git_signed_package() {
    let harness = Harness::new();
    let token = harness.session("Provenance");
    let mut invented = package("invented", "Invented", json!([]));
    invented["git_commit"] = json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let rejected = harness
        .auth(&token, reqwest::Method::POST, "/api/skills")
        .json(&invented)
        .send()
        .unwrap();
    assert_eq!(rejected.status(), 422, "an invented Git commit is rejected");
    let mut non_github = package("other-host", "Other host", json!([]));
    non_github["git_url"] = json!("https://example.com/not-a-repository");
    assert_eq!(
        harness
            .auth(&token, reqwest::Method::POST, "/api/skills")
            .json(&non_github)
            .send()
            .unwrap()
            .status(),
        400
    );

    let first = harness.create(&token, &package("signed-a", "Signed A", json!([])));
    let mut materially_different = package("signed-b", "Signed B", json!([]));
    materially_different["targets"] = json!(["Claude Code"]);
    materially_different["adapters"] = json!({"Claude Code":"Read CLAUDE.md."});
    materially_different["repositories"] = json!(["web-console"]);
    let second = harness.create(&token, &materially_different);
    assert_ne!(first["package_digest"], second["package_digest"]);
    assert_eq!(first["package_signature"].as_str().unwrap().len(), 128);
    assert_eq!(first["signer_public_key"].as_str().unwrap().len(), 64);
    let payload = hex::decode(first["signed_payload"].as_str().unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&payload)),
        first["package_digest"]
    );
    let key_bytes: [u8; 32] = hex::decode(first["signer_public_key"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let signature_bytes: [u8; 64] = hex::decode(first["package_signature"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    VerifyingKey::from_bytes(&key_bytes)
        .unwrap()
        .verify(&payload, &Signature::from_bytes(&signature_bytes))
        .unwrap();
    assert!(first["source_verified_at"].as_str().unwrap().contains('T'));

    let duplicate = harness
        .auth(&token, reqwest::Method::POST, "/api/skills")
        .json(&package("another-id", "Signed A", json!([])))
        .send()
        .unwrap();
    assert_eq!(
        duplicate.status(),
        409,
        "name and version identify one immutable package per workspace"
    );
}

#[doc = "@claim:independent-one-time-review"]
#[test]
fn claim_independent_one_time_review() {
    let harness = Harness::new();
    let owner = harness.session("Review");
    let created = harness.create(&owner, &package("review-me", "Review me", json!([])));
    let reviewer_key = created["reviewer_key"].as_str().unwrap();
    let database = std::fs::read(harness.data.path().join("registry.db")).expect("read database");
    assert!(
        !String::from_utf8_lossy(&database).contains(reviewer_key),
        "reviewer key is stored only as a hash"
    );
    let owner_cannot_review = harness
        .auth(&owner, reqwest::Method::GET, "/api/review")
        .send()
        .unwrap();
    assert_eq!(owner_cannot_review.status(), 401);
    let independent: Value = harness
        .auth(reviewer_key, reqwest::Method::GET, "/api/review")
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(independent["package_digest"], created["package_digest"]);
    let approval = harness.approve(reviewer_key, "Nora Singh");
    assert_eq!(approval["skill_id"], "review-me");
    let reused = harness
        .auth(reviewer_key, reqwest::Method::POST, "/api/review/approve")
        .json(&json!({"reviewer":"Someone else"}))
        .send()
        .unwrap();
    assert_eq!(reused.status(), 401, "reviewer key is consumed atomically");
}

#[doc = "@claim:governed-execution-receipt"]
#[test]
fn claim_governed_execution_receipt() {
    let harness = Harness::new();
    let owner = harness.session("Receipts");
    let created = harness.create(&owner, &package("governed", "Governed", json!([])));
    let early = harness
        .auth(&owner, reqwest::Method::POST, "/api/receipts")
        .json(&json!({"skill_id":"governed","repository":"atlas-api","agent":"Codex"}))
        .send()
        .unwrap();
    assert_eq!(early.status(), 409);
    let approval = harness.approve(created["reviewer_key"].as_str().unwrap(), "Nora Singh");
    let review_ring = harness
        .auth(&owner, reqwest::Method::POST, "/api/receipts")
        .json(&json!({"skill_id":"governed","repository":"atlas-api","agent":"Codex"}))
        .send()
        .unwrap();
    assert_eq!(review_ring.status(), 409, "review ring is not installable");
    harness
        .auth(&owner, reqwest::Method::PATCH, "/api/skills/governed/ring")
        .json(&json!({"ring":"pilot"}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
    let wrong_agent = harness
        .auth(&owner, reqwest::Method::POST, "/api/receipts")
        .json(&json!({"skill_id":"governed","repository":"atlas-api","agent":"Unassigned Agent"}))
        .send()
        .unwrap();
    assert_eq!(wrong_agent.status(), 403);
    let wrong_repository = harness
        .auth(&owner, reqwest::Method::POST, "/api/receipts")
        .json(&json!({"skill_id":"governed","repository":"other","agent":"Codex"}))
        .send()
        .unwrap();
    assert_eq!(wrong_repository.status(), 403);
    let receipt: Value = harness
        .auth(&owner, reqwest::Method::POST, "/api/receipts")
        .json(&json!({"skill_id":"governed","repository":"atlas-api","agent":"Codex"}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(receipt["approval_id"], approval["id"]);
    assert_eq!(receipt["package_signature"], created["package_signature"]);
    assert_eq!(receipt["receipt_signature"].as_str().unwrap().len(), 128);
    let installed: Value = harness
        .auth(
            &owner,
            reqwest::Method::GET,
            "/api/repositories/atlas-api/install/governed",
        )
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(installed["schema"], "team-agent-skill/v2");
    assert_eq!(
        installed["package"]["package_digest"],
        created["package_digest"]
    );
    let barrier = Arc::new(Barrier::new(40));
    let writes: Vec<_> = (0..40)
        .map(|_| {
            let client = harness.client.clone();
            let base = harness.base.clone();
            let token = owner.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                client
                    .post(format!("{base}/api/receipts"))
                    .bearer_auth(token)
                    .header("X-Forwarded-For", "192.0.2.55")
                    .json(&json!({"skill_id":"governed","repository":"atlas-api","agent":"Codex"}))
                    .send()
                    .unwrap()
                    .error_for_status()
                    .unwrap()
                    .json::<Value>()
                    .unwrap()["id"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
        })
        .collect();
    let ids: std::collections::HashSet<_> = writes
        .into_iter()
        .map(|write| write.join().unwrap())
        .collect();
    assert_eq!(ids.len(), 40, "concurrent receipts have unique ids");
}

#[test]
fn health_and_rate_limit_contract() {
    let harness = Harness::new();
    let health: Value = harness
        .client
        .get(format!("{}/health", harness.base))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(health["build_sha"], "repair-regression");
    assert_eq!(health["signer_fingerprint"].as_str().unwrap().len(), 64);
    for _ in 0..40 {
        let _ = harness
            .client
            .get(format!("{}/api/skills", harness.base))
            .header("X-Forwarded-For", "203.0.113.15, 10.0.0.1")
            .send()
            .unwrap();
    }
    let limited = harness
        .client
        .get(format!("{}/api/skills", harness.base))
        .header("X-Forwarded-For", "203.0.113.15, 10.0.0.1")
        .send()
        .unwrap();
    assert_eq!(limited.status(), 429);
    assert_eq!(limited.headers()["retry-after"], "1");
}

#[test]
fn signing_identity_and_data_survive_restart() {
    let mut harness = Harness::new();
    let token = harness.session("Persistent team");
    let before: Value = harness
        .client
        .get(format!("{}/health", harness.base))
        .send()
        .unwrap()
        .json()
        .unwrap();
    harness.restart();
    let after: Value = harness
        .client
        .get(format!("{}/health", harness.base))
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(before["signer_fingerprint"], after["signer_fingerprint"]);
    let records: Vec<Value> = harness
        .auth(&token, reqwest::Method::GET, "/api/skills")
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
    assert!(records.is_empty(), "workspace owner key survives restart");
}
