use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use ic_log::{
    formatter::buffer::Buffer,
    writer::{self, ConsoleWriter, InMemoryWriter, Writer},
    Builder, LogSettings, LoggerConfig,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableLog as IcStableLog,
};

use log::info;

type VMem = VirtualMemory<DefaultMemoryImpl>;
pub type StableLog = IcStableLog<Vec<u8>, VMem, VMem>;
const LOG_INDEX_MEMORY_ID: MemoryId = MemoryId::new(0);
const LOG_DATA_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // The log of the customs state modifications.
    static STABLE_LOGS: RefCell<Option<StableLog>> = MEMORY_MANAGER
        .with(|m|
              RefCell::new(
                  Some(StableLog::init(
                      m.borrow().get(LOG_INDEX_MEMORY_ID),
                      m.borrow().get(LOG_DATA_MEMORY_ID)
                  ).expect("failed to initialize stable log"))
              )
        );
    // static STABLE_LOGS: RefCell<Option<StableLog>> =RefCell::new(None);
}

pub struct StableLogWriter {}
impl StableLogWriter {
    // pub fn init_stable_log(stable_log: Option<StableLog>) {
    //     STABLE_LOGS.with(|logs| {
    //         *logs.borrow_mut() = stable_log;
    //     });
    // }
    pub fn get_logs(offset: usize, limit: usize) -> Vec<String> {
        STABLE_LOGS.with(|cell| {
            if let Some(logs) = cell.borrow().as_ref() {
                logs.iter()
                    .skip(offset)
                    .take(limit)
                    .map(|log| String::from_utf8_lossy(&log).to_string())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
    }
}

impl Writer for StableLogWriter {
    fn print(&self, buf: &Buffer) -> std::io::Result<()> {
        STABLE_LOGS.with(|cell| {
            if let Some(logs) = cell.borrow().as_ref() {
                let _ = logs.append(&buf.bytes().to_vec());
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

pub fn init_log(_stable_log: Option<StableLog>) {
    let settings = LogSettings {
        in_memory_records: Some(256),
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
    // StableLogWriter::init_stable_log(stable_log);
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

#[cfg(test)]
mod tests {
    use ic_log::take_memory_records;
    use log::*;

    use super::*;

    #[test]
    fn update_filter_at_runtime() {
        // log level: debug < info < error

        //default log level: info
        init_log(None);
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

    #[test]
    fn test_stable_log() {
        // log level: debug < info < error

        //default log level: info
        init_log(None);
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

        let logs = StableLogWriter::get_logs(0, 12);
        for r in logs.iter() {
            print!("log_record: {}", r)
        }
    }
}
