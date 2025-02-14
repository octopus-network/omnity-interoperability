use ic_canister_log::export as export_logs;
use ic_canister_log::{declare_log_buffer, GlobalBuffer, Sink};
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use serde::Deserialize;
use time::OffsetDateTime;

declare_log_buffer!(name = DEBUG_BUF, capacity = 1000);
declare_log_buffer!(name = INFO_BUF, capacity = 1000);
declare_log_buffer!(name = WARNING_BUF, capacity = 1000);
declare_log_buffer!(name = ERROR_BUF, capacity = 1000);
declare_log_buffer!(name = CRITICAL_BUF, capacity = 1000);

pub struct PrintProxySink(&'static str, &'static GlobalBuffer);

impl Sink for PrintProxySink {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        ic_cdk::println!("{} {}:{} {}", self.0, entry.file, entry.line, entry.message);
        self.1.append(entry)
    }
}

pub const DEBUG: PrintProxySink = PrintProxySink("DEBUG", &DEBUG_BUF);
pub const INFO: PrintProxySink = PrintProxySink("INFO", &INFO_BUF);
pub const WARNING: PrintProxySink = PrintProxySink("WARNING", &WARNING_BUF);
pub const ERROR: PrintProxySink = PrintProxySink("ERROR", &ERROR_BUF);
pub const CRITICAL: PrintProxySink = PrintProxySink("WARNING", &CRITICAL_BUF);

#[derive(Clone, serde::Serialize, Deserialize, Debug, Copy)]
pub enum Priority {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub canister_id: String,
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

pub fn http_log(req: HttpRequest, enable_debug: bool) -> HttpResponse {
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

    let limit = match req.raw_query_param("limit") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build()
            }
        },
        None => 1000,
    };

    let offset = match req.raw_query_param("offset") {
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
    if enable_debug {
        merge_log(&mut entries, &DEBUG_BUF, Priority::DEBUG);
    }
    merge_log(&mut entries, &INFO_BUF, Priority::INFO);
    merge_log(&mut entries, &WARNING_BUF, Priority::WARNING);
    merge_log(&mut entries, &ERROR_BUF, Priority::ERROR);
    merge_log(&mut entries, &CRITICAL_BUF, Priority::CRITICAL);
    entries
        .entries
        .retain(|entry| entry.timestamp >= max_skip_timestamp);
    entries
        .entries
        .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let logs = entries
        .entries
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect::<Vec<_>>();
    HttpResponseBuilder::ok()
        .header("Content-Type", "application/json; charset=utf-8")
        .with_body_and_content_length(serde_json::to_string(&logs).unwrap_or_default())
        .build()
}

fn merge_log(entries: &mut Log, buffer: &'static GlobalBuffer, priority: Priority) {
    let canister_id = ic_cdk::api::id();
    for entry in export_logs(buffer) {
        entries.entries.push(LogEntry {
            timestamp: entry.timestamp,
            canister_id: canister_id.to_string(),
            time_str: OffsetDateTime::from_unix_timestamp_nanos(entry.timestamp as i128)
                .unwrap()
                .to_string(),
            counter: entry.counter,
            priority: priority,
            file: entry.file.to_string(),
            line: entry.line,
            message: entry.message,
        });
    }
}
