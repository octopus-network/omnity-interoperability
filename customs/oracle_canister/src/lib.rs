mod oracle;
mod types;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};
use ic_log::{writer::Logs, LogSettings, LoggerConfig};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static LOGGER_CONFIG: RefCell<Option<LoggerConfig>> = const { RefCell::new(None) };
    static CUSTOMS_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
    static INDEXER_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
}

pub(crate) fn customs_principal() -> Principal {
    CUSTOMS_PRINCIPAL.with(|p| p.borrow().clone().expect("not initialized"))
}

pub(crate) fn indexer_principal() -> Principal {
    INDEXER_PRINCIPAL.with(|p| p.borrow().clone().expect("not initialized"))
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Args {
    pub customs: Principal,
    pub indexer: Principal,
}

pub fn init_ic_log() {
    let settings = LogSettings {
        in_memory_records: Some(128),
        log_filter: Some("info".to_string()),
        enable_console: true,
    };
    match ic_log::init_log(&settings) {
        Ok(logger_config) => LoggerConfigService::default().init(logger_config),
        Err(err) => {
            ic_cdk::println!("error configuring the logger. Err: {:?}", err)
        }
    }
    log::info!("Logger initialized");
}

type ForceNotSendAndNotSync = PhantomData<Rc<()>>;

#[derive(Debug, Default)]
/// Handles the runtime logger configuration
pub struct LoggerConfigService(ForceNotSendAndNotSync);

impl LoggerConfigService {
    /// Sets a new LoggerConfig
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

#[query]
pub fn get_log_records(count: usize, offset: usize) -> Logs {
    ic_log::take_memory_records(count, offset)
}

#[init]
pub fn init(args: Args) {
    init_ic_log();
    CUSTOMS_PRINCIPAL.with(|p| p.replace(Some(args.customs)));
    INDEXER_PRINCIPAL.with(|p| p.replace(Some(args.indexer)));
    oracle::fetch_then_submit(5);
}

#[pre_upgrade]
fn pre_upgrade() {
    let customs = CUSTOMS_PRINCIPAL.with(|p| p.take());
    let indexer = INDEXER_PRINCIPAL.with(|p| p.take());
    ic_cdk::storage::stable_save((customs, indexer)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    init_ic_log();
    let (customs, indexer): (Option<Principal>, Option<Principal>) =
        ic_cdk::storage::stable_restore().unwrap();
    CUSTOMS_PRINCIPAL.with(|p| p.replace(customs));
    INDEXER_PRINCIPAL.with(|p| p.replace(indexer));
    oracle::fetch_then_submit(5);
}

ic_cdk::export_candid!();
