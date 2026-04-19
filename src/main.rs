use tiny_http::{Server, Response, Method, Header};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Deserialize)]
struct Incoming {
    name: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    id: String,
    name: String,
    message: String,
    created_at: u64,
}

fn log(msg: &str) {
    println!("[{}] {}", now(), msg);
}

fn main() {
    let server = Server::http("0.0.0.0:50007").unwrap();
    log("Server started on port 50007");

    for mut request in server.incoming_requests() {
        let method = request.method().to_string();
        let url = request.url().to_string();
        log(&format!("{} {}", method, url));


        let remote_addr = request.remote_addr().map(|a| a.to_string()).unwrap_or_else(|| "unknown".to_string());

        if request.method() == &Method::Options {
            log(&format!("PREFLIGHT from {}", remote_addr));
            let _ = request.respond(cors(Response::from_string("").with_status_code(204)));
        } else if request.method() == &Method::Get && request.url() == "/" {
            log(&format!("STATUS from {}", remote_addr));
            let _ = request.respond(cors(Response::from_string("{\"status\":\"ok\"}")));
        } else if request.method() == &Method::Post && request.url() == "/submit" {
            log(&format!("SUBMIT from {}", remote_addr));

            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).unwrap();
            log(&format!("Body size: {} bytes", body.len()));

            let parsed: Result<Incoming, _> = serde_json::from_str(&body);

            match parsed {
                Ok(data) => {
                    if !valid(&data) {
                        log(&format!("Rejected invalid input from {} (name_len={}, msg_len={})", remote_addr, data.name.len(), data.message.len()));
                        let _ = request.respond(cors(Response::from_string("Invalid input").with_status_code(400)));
                        continue;
                    }

                    let entry = Entry {
                        id: Uuid::new_v4().to_string(),
                        name: data.name,
                        message: data.message,
                        created_at: now(),
                    };

                    log(&format!("New entry id={} name=\"{}\" from {}", entry.id, entry.name, remote_addr));
                    append_entry(entry);
                    log("Entry saved to pending.json");

                    let _ = request.respond(cors(Response::from_string("OK")));
                }
                Err(e) => {
                    log(&format!("Rejected bad JSON from {}: {}", remote_addr, e));
                    let _ = request.respond(cors(Response::from_string("Bad JSON").with_status_code(400)));
                }
            }
        } else {
            log(&format!("Not found: {} {} from {}", method, url, remote_addr));
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
        }
    }
}


fn cors<T: std::io::Read>(response: Response<T>) -> Response<T> {
    response
        .with_header(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap())
        .with_header(Header::from_bytes("Access-Control-Allow-Methods", "POST, GET, OPTIONS").unwrap())
        .with_header(Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap())
}

fn valid(data: &Incoming) -> bool {
    !data.name.is_empty()
        && data.name.len() <= 50
        && !data.message.is_empty()
        && data.message.len() <= 500
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn append_entry(entry: Entry) {
    let path = "data/pending.json";
    fs::create_dir_all("data").unwrap();

    let mut entries: Vec<Entry> = if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    entries.push(entry);

    let tmp = format!("{}.tmp", path);
    fs::write(&tmp, serde_json::to_string_pretty(&entries).unwrap()).unwrap();
    fs::rename(tmp, path).unwrap();
}
