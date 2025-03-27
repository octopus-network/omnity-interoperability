use ic_canister_log::{declare_log_buffer, export as export_logs, GlobalBuffer, Sink};
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};

use serde::Deserialize;
use std::str::FromStr;
use time::OffsetDateTime;

// High-priority messages.
declare_log_buffer!(name = INFO_BUF, capacity = 1000);

// Low-priority info messages.
declare_log_buffer!(name = DEBUG_BUF, capacity = 1000);
declare_log_buffer!(name = WARNING_BUF, capacity = 1000);
declare_log_buffer!(name = ERROR_BUF, capacity = 1000);
declare_log_buffer!(name = CRITICAL_BUF, capacity = 1000);

pub const INFO: PrintProxySink = PrintProxySink("INFO", &INFO_BUF);
pub const DEBUG: PrintProxySink = PrintProxySink("DEBUG", &DEBUG_BUF);
pub const WARNING: PrintProxySink = PrintProxySink("WARNING", &WARNING_BUF);
pub const ERROR: PrintProxySink = PrintProxySink("ERROR", &ERROR_BUF);
pub const CRITICAL: PrintProxySink = PrintProxySink("WARNING", &CRITICAL_BUF);

pub struct PrintProxySink(&'static str, &'static GlobalBuffer);

impl Sink for PrintProxySink {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        ic_cdk::println!("{} {}:{} {}", self.0, entry.file, entry.line, entry.message);
        self.1.append(entry)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, serde::Serialize)]
pub enum Priority {
    INFO,
    DEBUG,
    WARNING,
    ERROR,
    CRITICAL,
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Priority::INFO),
            "debug" => Ok(Priority::DEBUG),
            _ => Err("could not recognize priority".to_string()),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, serde::Serialize)]
pub enum Sort {
    Ascending,
    Descending,
}

impl FromStr for Sort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(Sort::Ascending),
            "desc" => Ok(Sort::Descending),
            _ => Err("could not recognize sort order".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, serde::Serialize)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn push_logs(&mut self, priority: Priority) {
        let logs = match priority {
            Priority::INFO => export_logs(&INFO_BUF),
            Priority::DEBUG => export_logs(&DEBUG_BUF),
            Priority::WARNING => export_logs(&WARNING_BUF),
            Priority::ERROR => export_logs(&ERROR_BUF),
            Priority::CRITICAL => export_logs(&CRITICAL_BUF),
        };
        for entry in logs {
            self.entries.push(LogEntry {
                canister_id: ic_cdk::api::id().to_string(),
                timestamp: entry.timestamp,
                time_str: OffsetDateTime::from_unix_timestamp_nanos(entry.timestamp as i128)
                    .unwrap()
                    .to_string(),
                counter: entry.counter,
                priority,
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
    }

    pub fn push_all(&mut self) {
        self.push_logs(Priority::INFO);
        self.push_logs(Priority::DEBUG);
        self.push_logs(Priority::WARNING);
        self.push_logs(Priority::ERROR);
        self.push_logs(Priority::CRITICAL);
    }

    pub fn serialize_logs(&self, max_body_size: usize) -> String {
        let mut entries_json: String = serde_json::to_string(&self).unwrap_or_default();

        if entries_json.len() > max_body_size {
            let mut left = 0;
            let mut right = self.entries.len();

            while left < right {
                let mid = left + (right - left) / 2;
                let mut temp_log = self.clone();
                temp_log.entries.truncate(mid);
                let temp_entries_json = serde_json::to_string(&temp_log).unwrap_or_default();

                if temp_entries_json.len() <= max_body_size {
                    entries_json = temp_entries_json;
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
        }
        entries_json
    }

    pub fn sort_logs(&mut self, sort_order: Sort) {
        match sort_order {
            Sort::Ascending => self.sort_asc(),
            Sort::Descending => self.sort_desc(),
        }
    }

    pub fn sort_asc(&mut self) {
        self.entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    pub fn sort_desc(&mut self) {
        self.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
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

#[cfg(test)]
mod tests {
    use proptest::{prop_assert, proptest};

    use super::*;

    fn info_log_entry_with_timestamp(timestamp: u64) -> LogEntry {
        LogEntry {
            canister_id: ic_cdk::api::id().to_string(),
            timestamp,
            time_str: OffsetDateTime::from_unix_timestamp_nanos(timestamp as i128)
                .unwrap()
                .to_string(),
            priority: Priority::INFO,
            file: String::default(),
            line: 0,
            message: String::default(),
            counter: 0,
        }
    }

    fn is_ascending(log: &Log) -> bool {
        for i in 0..log.entries.len() - 1 {
            if log.entries[i].timestamp > log.entries[i + 1].timestamp {
                return false;
            }
        }
        true
    }

    fn is_descending(log: &Log) -> bool {
        for i in 0..log.entries.len() - 1 {
            if log.entries[i].timestamp < log.entries[i + 1].timestamp {
                return false;
            }
        }
        true
    }

    proptest! {
        #[test]
        fn logs_always_fit_in_message(
            number_of_entries in 1..100_usize,
            entry_size in 1..10000_usize,
            max_body_size in 100..10000_usize
        ) {
            let mut entries: Vec<LogEntry> = vec![];
            for _ in 0..number_of_entries {
                entries.push(LogEntry {
                    canister_id: ic_cdk::api::id().to_string(),
                    timestamp: 0,
                    time_str: "0".into(),
                    priority: Priority::INFO,
                    file: String::default(),
                    line: 0,
                    message: "1".repeat(entry_size),
                    counter: 0,
                });
            }
            let log = Log { entries };
            let truncated_logs_json_len = log.serialize_logs(max_body_size).len();
            prop_assert!(truncated_logs_json_len <= max_body_size);
        }
    }

    #[test]
    fn sorting_order() {
        let mut log = Log { entries: vec![] };
        log.entries.push(info_log_entry_with_timestamp(2));
        log.entries.push(info_log_entry_with_timestamp(0));
        log.entries.push(info_log_entry_with_timestamp(1));
        log.sort_asc();
        assert!(is_ascending(&log));

        log.sort_desc();
        assert!(is_descending(&log));

        log.sort_logs(Sort::Ascending);
        assert!(is_ascending(&log));

        log.sort_logs(Sort::Descending);
        assert!(is_descending(&log));
    }

    #[test]
    fn simple_logs_truncation() {
        let mut entries: Vec<LogEntry> = vec![];
        const MAX_BODY_SIZE: usize = 3_000_000;

        for _ in 0..10 {
            entries.push(LogEntry {
                canister_id: ic_cdk::api::id().to_string(),
                timestamp: 0,
                time_str: "0".into(),
                priority: Priority::INFO,
                file: String::default(),
                line: 0,
                message: String::default(),
                counter: 0,
            });
        }
        let log = Log {
            entries: entries.clone(),
        };
        let small_len = serde_json::to_string(&log).unwrap_or_default().len();

        entries.push(LogEntry {
            canister_id: ic_cdk::api::id().to_string(),
            timestamp: 0,
            time_str: "0".into(),
            priority: Priority::INFO,
            file: String::default(),
            line: 0,
            message: "1".repeat(MAX_BODY_SIZE),
            counter: 0,
        });
        let log = Log { entries };
        let entries_json = serde_json::to_string(&log).unwrap_or_default();
        assert!(entries_json.len() > MAX_BODY_SIZE);

        let truncated_logs_json = log.serialize_logs(MAX_BODY_SIZE);

        assert_eq!(small_len, truncated_logs_json.len());
    }

    #[test]
    fn one_entry_too_big() {
        let mut entries: Vec<LogEntry> = vec![];
        const MAX_BODY_SIZE: usize = 3_000_000;

        entries.push(LogEntry {
            canister_id: ic_cdk::api::id().to_string(),
            timestamp: 0,
            time_str: "0".into(),
            priority: Priority::INFO,
            file: String::default(),
            line: 0,
            message: "1".repeat(MAX_BODY_SIZE),
            counter: 0,
        });
        let log = Log { entries };
        let truncated_logs_json_len = log.serialize_logs(MAX_BODY_SIZE).len();
        assert!(truncated_logs_json_len < MAX_BODY_SIZE);
        assert_eq!("{\"entries\":[]}", log.serialize_logs(MAX_BODY_SIZE));
    }

    #[test]
    fn should_truncate_last_entry() {
        let log_entries = vec![
            info_log_entry_with_timestamp(0),
            info_log_entry_with_timestamp(1),
            info_log_entry_with_timestamp(2),
        ];
        let log_with_2_entries = Log {
            entries: {
                let mut entries = log_entries.clone();
                entries.pop();
                entries
            },
        };
        let log_with_3_entries = Log {
            entries: log_entries,
        };

        let serialized_log_with_2_entries = log_with_2_entries.serialize_logs(usize::MAX);
        let serialized_log_with_3_entries =
            log_with_3_entries.serialize_logs(serialized_log_with_2_entries.len());

        assert_eq!(serialized_log_with_3_entries, serialized_log_with_2_entries);
    }
}
