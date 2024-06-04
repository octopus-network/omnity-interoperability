use candid::CandidType;
use ic_log::{
    formatter::buffer::Buffer,
    formatter::humantime::Rfc3339Timestamp,
    writer::{self, ConsoleWriter, InMemoryWriter, Writer},
    Builder, LogSettings, LoggerConfig,
};
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl};
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use log::info;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::{borrow::Cow, cell::RefCell};
use std::{marker::PhantomData, str::FromStr};

use humantime::parse_rfc3339;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use serde_json;
use std::time::UNIX_EPOCH;

type VMem = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static STABLE_LOGS: RefCell<Option<StableBTreeMap<Vec<u8>, Vec<u8>, VMem>>> =RefCell::new(None);
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub timstamp: u64,
    pub log: String,
}

impl Storable for LogEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        let _ = ciborium::ser::into_writer(self, &mut bytes);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let log_entry =
            ciborium::de::from_reader(bytes.as_ref()).expect("failed to decode TokenKey");
        log_entry
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn parse_timestamp(time_str: &Vec<u8>) -> u64 {
    let datetime = parse_rfc3339(&String::from_utf8_lossy(time_str).as_ref())
        .expect("Failed to parse timestamp");
    datetime
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
pub struct StableLogWriter {}
impl StableLogWriter {
    pub fn init_stable_log(stable_log: Option<StableBTreeMap<Vec<u8>, Vec<u8>, VMem>>) {
        STABLE_LOGS.with(|logs| {
            *logs.borrow_mut() = stable_log;
        });
    }
    pub fn get_logs(max_skip_timestamp: u64, offset: usize, limit: usize) -> Vec<String> {
        STABLE_LOGS.with(|cell| {
            if let Some(logs) = cell.borrow().as_ref() {
                logs.iter()
                    .filter(|(time_str, _)| {
                        let timestamp = parse_timestamp(time_str);
                        timestamp >= max_skip_timestamp
                    })
                    .skip(offset)
                    .take(limit)
                    .map(|(_, log)| String::from_utf8_lossy(&log).to_string())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
    }
    fn parse_param<T: FromStr>(req: &HttpRequest, param: &str) -> Result<T, HttpResponse> {
        match req.raw_query_param(param) {
            Some(arg) => match arg.parse() {
                Ok(value) => Ok(value),
                Err(_) => Err(HttpResponseBuilder::bad_request()
                    .with_body_and_content_length(format!(
                        "failed to parse the '{}' parameter",
                        param
                    ))
                    .build()),
            },
            None => Err(HttpResponseBuilder::bad_request()
                .with_body_and_content_length(format!("must provide the '{}' parameter", param))
                .build()),
        }
    }

    pub fn http_request(req: HttpRequest) -> HttpResponse {
        if req.path() == "/logs" {
            let max_skip_timestamp = Self::parse_param::<u64>(&req, "time").unwrap_or(0);
            let offset = match Self::parse_param::<usize>(&req, "offset") {
                Ok(value) => value,
                Err(err) => return err,
            };
            let limit = match Self::parse_param::<usize>(&req, "limit") {
                Ok(value) => value,
                Err(err) => return err,
            };
            info!(
                "request params: max_skip_timestamp: {}, offset: {}, limit: {}",
                max_skip_timestamp, offset, limit
            );

            let logs = Self::get_logs(max_skip_timestamp, offset, limit);
            let logs_json = serde_json::to_string(&logs)
                .unwrap_or_default()
                .replace('\n', "");
            HttpResponseBuilder::ok()
                .header("Content-Type", "application/json; charset=utf-8")
                .with_body_and_content_length(logs_json)
                .build()
        } else {
            HttpResponseBuilder::not_found().build()
        }
    }
}

impl Writer for StableLogWriter {
    fn print(&self, buf: &Buffer) -> std::io::Result<()> {
        STABLE_LOGS.with(|cell| {
            if let Some(logs) = cell.borrow_mut().as_mut() {
                // string format: 2018-02-13T23:08:32.123000000Z
                let timestamp = format!("{}", Rfc3339Timestamp::now());
                logs.insert(timestamp.into_bytes(), buf.bytes().to_vec());
            }
        });

        Ok(())
    }
}

thread_local! {
    static LOGGER_CONFIG: RefCell<Option<LoggerConfig>> = RefCell::new(None);
}

type ForceNotSendAndNotSync = PhantomData<Rc<()>>;

#[derive(Debug, Default)]
/// Handles the runtime logger configuration
pub struct LoggerConfigService(ForceNotSendAndNotSync);

impl LoggerConfigService {
    pub fn init(&self, logger_config: LoggerConfig) {
        LOGGER_CONFIG.with(|config| config.borrow_mut().replace(logger_config));
    }
}

pub fn init_log(stable_log: Option<StableBTreeMap<Vec<u8>, Vec<u8>, VMem>>) {
    let settings = LogSettings {
        in_memory_records: None,
        log_filter: Some("info".to_string()),
        enable_console: true,
    };
    let mut builder =
        Builder::default().parse_filters(settings.log_filter.as_deref().unwrap_or("off"));

    if settings.enable_console {
        builder = builder.add_writer(Box::new(ConsoleWriter {}));
    }

    if let Some(count) = settings.in_memory_records {
        writer::InMemoryWriter::init_buffer(count);
        builder = builder.add_writer(Box::new(InMemoryWriter {}));
    }
    // add StableLogWriter
    StableLogWriter::init_stable_log(stable_log);
    builder = builder.add_writer(Box::new(StableLogWriter {}));

    match builder.try_init() {
        Ok(logger_config) => LoggerConfigService::default().init(logger_config),
        Err(err) => {
            ic_cdk::println!(
                "error configuring the logger. Err({err:?}) \n {}",
                std::panic::Location::caller()
            );
        }
    }
    info!("Logger initialized");
}
