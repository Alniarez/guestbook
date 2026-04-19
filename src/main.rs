use tiny_http::{Server, Response, Method};
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

        if request.method() == &Method::Get && request.url() == "/status" {
            let _ = request.respond(Response::from_string("{\"status\":\"ok\"}").with_status_code(200));
        } else if request.method() == &Method::Post && request.url() == "/submit" {

            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).unwrap();

            let parsed: Result<Incoming, _> = serde_json::from_str(&body);

            match parsed {
                Ok(data) => {
                    if !valid(&data) {
                        log("Rejected: invalid input");
                        let _ = request.respond(Response::from_string("Invalid input").with_status_code(400));
                        continue;
                    }

                    let entry = Entry {
                        id: Uuid::new_v4().to_string(),
                        name: data.name,
                        message: data.message,
                        created_at: now(),
                    };

                    log(&format!("New entry from \"{}\"", entry.name));
                    append_entry(entry);

                    let _ = request.respond(Response::from_string("OK"));
                }
                Err(_) => {
                    log("Rejected: bad JSON");
                    let _ = request.respond(Response::from_string("Bad JSON").with_status_code(400));
                }
            }
        } else {
            log(&format!("Not found: {} {}", method, url));
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
        }
    }
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
