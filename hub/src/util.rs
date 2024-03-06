use log::{debug, info};

use std::cell::RefCell;

use std::marker::PhantomData;
use std::rc::Rc;

use ic_log::writer::Logs;
use ic_log::{LogSettings, LoggerConfig};

use crate::auth;
use auth::is_owner;

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

    /// Changes the logger filter at runtime
    pub fn set_logger_filter(&self, filter: &str) {
        LOGGER_CONFIG.with(|config| match *config.borrow_mut() {
            Some(ref logger_config) => {
                logger_config.update_filters(filter);
            }
            None => panic!("LoggerConfig not initialized"),
        });
    }
}

pub fn init_log() {
    let settings = LogSettings {
        in_memory_records: Some(256),
        log_filter: Some("info".to_string()),
        enable_console: true,
    };
    match ic_log::init_log(&settings) {
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

#[ic_cdk::query]
pub fn get_log_records(count: usize, offset: usize) -> Logs {
    debug!("collecting {count} log records");
    ic_log::take_memory_records(count, offset)
}

#[ic_cdk::update(guard = "is_owner")]
pub async fn set_logger_filter(filter: String) {
    LoggerConfigService::default().set_logger_filter(&filter);
    debug!("log filter set to {filter}");
}

#[cfg(test)]
mod tests {
    use ic_log::take_memory_records;
    use log::*;

    use super::*;

    #[test]
    fn update_filter_at_runtime() {
        // log level: debug < info < error

        //default log level: info
        init_log();
        info!("This info should be printed");
        debug!("This debug should NOT be printed");
        error!("This error should be printed");

        // debug level
        LoggerConfigService::default().set_logger_filter("debug");
        info!("This info should be printed");
        debug!("This debug should be printed");
        error!("This error should be printed");

        // error
        LoggerConfigService::default().set_logger_filter("error");
        info!("This info should NOT be printed");
        debug!("This debug should NOT be printed");
        error!("This error should be printed");

        LoggerConfigService::default().set_logger_filter("info");
        info!("This info should be printed");
        debug!("This debug should NOT be printed");
        error!("This error should be printed");

        let log_records = take_memory_records(5, 0);
        for r in log_records.logs.iter() {
            print!("log_record: {:#?}", r)
        }
    }
}
