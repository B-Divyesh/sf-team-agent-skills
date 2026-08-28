use std::{net::TcpListener, path::Path, process::{Child, Command}, thread, time::Duration};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve a test port");
    listener.local_addr().expect("read test port").port()
}

fn start_server(database_path: &Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_team-agent-skills"))
        .env("DATABASE_PATH", database_path)
        .env("PORT", port.to_string())
        .env("BUILD_SHA", "container-regression")
        .spawn()
        .expect("start registry server")
}

fn request(port: u16, forwarded_for: &str, path: &str) -> String {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect to registry");
    stream.set_read_timeout(Some(Duration::from_secs(2))).expect("set timeout");
    write!(stream, "GET {path} HTTP/1.1\r\nHost: localhost\r\nX-Forwarded-For: {forwarded_for}\r\nConnection: close\r\n\r\n").expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    response
}

fn wait_for_health(port: u16) -> String {
    for _ in 0..40 {
        if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
            use std::io::{Read, Write};
            let _ = stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            if response.contains("200 OK") { return response; }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("registry did not become healthy");
}

#[test]
fn release_binary_serves_health_and_limits_forwarded_clients() {
    let database = tempfile::tempdir().expect("create temp data directory");
    let port = free_port();
    let mut child = start_server(&database.path().join("registry.db"), port);

    let result = std::panic::catch_unwind(|| {
        let health = wait_for_health(port);
        assert!(health.contains("\"build_sha\":\"container-regression\""));

        for _ in 0..40 {
            assert!(request(port, "203.0.113.15, 10.0.0.1", "/api/skills").starts_with("HTTP/1.1 200"));
        }
        let limited = request(port, "203.0.113.15, 10.0.0.1", "/api/skills");
        assert!(limited.starts_with("HTTP/1.1 429"));
        assert!(limited.to_ascii_lowercase().contains("retry-after: 1"));
        assert!(request(port, "203.0.113.16, 10.0.0.1", "/api/skills").starts_with("HTTP/1.1 200"));
    });

    let _ = child.kill();
    let _ = child.wait();
    result.expect("container runtime regression check");
}
