use ic_canister_log::declare_log_buffer;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use serde_derive::Deserialize;
use ic_canister_log::export as export_logs;
use time::OffsetDateTime;

// High-priority messages.
declare_log_buffer!(name = P0, capacity = 1000);

// Low-priority info messages.
declare_log_buffer!(name = P1, capacity = 1000);

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub enum Priority {
    P0,
    P1,
}

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub time_str: String,
    pub priority: Priority,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub counter: u64,
}

#[derive(Clone, Default, serde::Serialize, Deserialize, Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

pub fn http_request(req: HttpRequest) -> HttpResponse {
    if req.path() == "/logs" {
        use std::str::FromStr;
        let max_skip_timestamp = match req.raw_query_param("time") {
            Some(arg) => match u64::from_str(arg) {
                Ok(value) => value,
                Err(_) => {
                    return HttpResponseBuilder::bad_request()
                        .with_body_and_content_length("failed to parse the 'time' parameter")
                        .build()
                }
            },
            None => 0,
        };

        let mut entries: Log = Default::default();
        for entry in export_logs(&P0) {

            entries.entries.push(LogEntry {
                timestamp: entry.timestamp,
                time_str: OffsetDateTime::from_unix_timestamp_nanos(entry.timestamp as i128).unwrap().to_string(),
                counter: entry.counter,
                priority: Priority::P0,
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
        for entry in export_logs(&P1) {
            entries.entries.push(LogEntry {
                timestamp: entry.timestamp,
                time_str: OffsetDateTime::from_unix_timestamp_nanos(entry.timestamp as i128).unwrap().to_string(),
                counter: entry.counter,
                priority: Priority::P1,
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
        entries
            .entries
            .retain(|entry| entry.timestamp >= max_skip_timestamp);
        HttpResponseBuilder::ok()
            .header("Content-Type", "application/json; charset=utf-8")
            .with_body_and_content_length(serde_json::to_string(&entries).unwrap_or_default())
            .build()
    } else {
        HttpResponseBuilder::not_found().build()
    }
}