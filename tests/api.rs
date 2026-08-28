use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use std::{
    net::TcpListener,
    path::Path,
    process::{Child, Command},
    thread,
    time::Duration,
};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve port")
        .local_addr()
        .expect("read port")
        .port()
}
fn start_server(database_path: &Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_team-agent-skills"))
        .env("DATABASE_PATH", database_path)
        .env("PORT", port.to_string())
        .env("BUILD_SHA", "repair-regression")
        .spawn()
        .expect("start server")
}
fn wait_for_health(client: &Client, base: &str) {
    for _ in 0..80 {
        if client
            .get(format!("{base}/health"))
            .send()
            .is_ok_and(|r| r.status().is_success())
        {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("server did not become healthy");
}
fn session(client: &Client, base: &str, name: &str) -> (String, String) {
    let result = client
        .post(format!("{base}/api/session"))
        .json(&json!({"name":name}))
        .send()
        .expect("create workspace")
        .error_for_status()
        .expect("session accepted")
        .json::<Value>()
        .expect("session json");
    (
        result["token"].as_str().expect("token").to_string(),
        result["reviewer_key"]
            .as_str()
            .expect("reviewer key")
            .to_string(),
    )
}
fn auth(
    client: &Client,
    token: &str,
    method: reqwest::Method,
    url: String,
) -> reqwest::blocking::RequestBuilder {
    client.request(method, url).bearer_auth(token)
}
fn package(id: &str, secrets: Value) -> Value {
    json!({
        "id":id,"name":"Release safety","version":"1.2.3","summary":"Check a release.",
        "targets":["Codex"],"owner":"Mina","secrets":secrets,
        "instructions":"Run tests and inspect the diff before release.",
        "adapters":{"Codex":"Read AGENTS.md and run the configured test command."},
        "git_url":"https://github.com/example/repo",
        "git_commit":"7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2",
        "repositories":["atlas-api"]
    })
}

#[test]
fn release_binary_enforces_private_reviewed_repository_packages() {
    let database = tempfile::tempdir().expect("temp data");
    let port = free_port();
    let mut child = start_server(&database.path().join("registry.db"), port);
    let result = std::panic::catch_unwind(|| {
        let client = Client::new();
        let base = format!("http://127.0.0.1:{port}");
        wait_for_health(&client, &base);
        let health: Value = client
            .get(format!("{base}/health"))
            .send()
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(health["build_sha"], "repair-regression");

        let public = client.get(format!("{base}/api/skills")).send().unwrap();
        assert_eq!(
            public.status(),
            401,
            "registry reads require a workspace key"
        );

        let (token_a, reviewer_a) = session(&client, &base, "Team A");
        let (token_b, reviewer_b) = session(&client, &base, "Team B");
        let database_bytes =
            std::fs::read(database.path().join("registry.db")).expect("read database");
        let database_text = String::from_utf8_lossy(&database_bytes);
        assert!(
            !database_text.contains(&token_a),
            "owner key is stored only as a hash"
        );
        assert!(
            !database_text.contains(&reviewer_a),
            "reviewer key is stored only as a hash"
        );
        let bad_secret = auth(
            &client,
            &token_a,
            reqwest::Method::POST,
            format!("{base}/api/skills"),
        )
        .json(&package("bad-secret", json!(["ghp_actualSecretValue"])))
        .send()
        .unwrap();
        assert_eq!(bad_secret.status(), 400, "secret values are rejected");

        let created: Value = auth(
            &client,
            &token_a,
            reqwest::Method::POST,
            format!("{base}/api/skills"),
        )
        .json(&package("release-safety", json!(["GITHUB_TOKEN"])))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
        assert_eq!(created["ring"], "draft");
        assert_eq!(
            created["instructions"],
            "Run tests and inspect the diff before release."
        );
        assert_eq!(
            created["git_commit"],
            "7fa45d6e0ca5274ed376bc86a0c8c6f1d959aad2"
        );
        assert_eq!(created["package_digest"].as_str().unwrap().len(), 64);

        let isolated: Vec<Value> = auth(
            &client,
            &token_b,
            reqwest::Method::GET,
            format!("{base}/api/skills"),
        )
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
        assert!(isolated.is_empty(), "another workspace cannot read Team A");

        let early_release = auth(
            &client,
            &token_a,
            reqwest::Method::PATCH,
            format!("{base}/api/skills/release-safety/ring"),
        )
        .json(&json!({"ring":"pilot"}))
        .send()
        .unwrap();
        assert_eq!(
            early_release.status(),
            409,
            "pilot requires an approval record"
        );
        let forged_review = auth(
            &client,
            &token_a,
            reqwest::Method::POST,
            format!("{base}/api/skills/release-safety/approve"),
        )
        .json(&json!({"reviewer":"Forged name","reviewer_key":"tsr_review_wrong"}))
        .send()
        .unwrap();
        assert_eq!(
            forged_review.status(),
            403,
            "review requires its separate key"
        );
        let foreign_approval = auth(
            &client,
            &token_b,
            reqwest::Method::POST,
            format!("{base}/api/skills/release-safety/approve"),
        )
        .json(&json!({"reviewer":"Intruder","reviewer_key":reviewer_b}))
        .send()
        .unwrap();
        assert_eq!(
            foreign_approval.status(),
            409,
            "another workspace cannot approve a package"
        );

        auth(
            &client,
            &token_a,
            reqwest::Method::POST,
            format!("{base}/api/skills/release-safety/approve"),
        )
        .json(&json!({"reviewer":"Nora Singh","reviewer_key":reviewer_a}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
        auth(
            &client,
            &token_a,
            reqwest::Method::PATCH,
            format!("{base}/api/skills/release-safety/ring"),
        )
        .json(&json!({"ring":"pilot"}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();

        let denied_repo = auth(
            &client,
            &token_a,
            reqwest::Method::GET,
            format!("{base}/api/repositories/other/install/release-safety"),
        )
        .send()
        .unwrap();
        assert_eq!(denied_repo.status(), 403);
        let installed: Value = auth(
            &client,
            &token_a,
            reqwest::Method::GET,
            format!("{base}/api/repositories/atlas-api/install/release-safety"),
        )
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
        assert_eq!(installed["schema"], "team-agent-skill/v1");
        assert_eq!(installed["package"]["version"], "1.2.3");

        let receipt: Value = auth(
            &client,
            &token_a,
            reqwest::Method::POST,
            format!("{base}/api/receipts"),
        )
        .json(&json!({"skill_id":"release-safety","repository":"atlas-api","agent":"Codex"}))
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .unwrap();
        assert_eq!(receipt["version"], "1.2.3");
        assert_eq!(receipt["package_digest"], created["package_digest"]);

        for _ in 0..40 {
            let _: Response = client
                .get(format!("{base}/api/skills"))
                .header("X-Forwarded-For", "203.0.113.15, 10.0.0.1")
                .send()
                .unwrap();
        }
        let limited = client
            .get(format!("{base}/api/skills"))
            .header("X-Forwarded-For", "203.0.113.15, 10.0.0.1")
            .send()
            .unwrap();
        assert_eq!(limited.status(), 429);
        assert_eq!(limited.headers()["retry-after"], "1");
    });
    let _ = child.kill();
    let _ = child.wait();
    result.expect("API regression check");
}
