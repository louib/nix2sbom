extern crate log;
extern crate simple_logging;

use std::env;
use std::str::FromStr;

const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

pub fn init() {
    simple_logging::log_to_stderr(get_log_level());
}

fn get_log_level() -> log::LevelFilter {
    let log_level_string = match env::var("NIX2SBOM_LOG_LEVEL") {
        Ok(level) => level,
        Err(_) => return DEFAULT_LOG_LEVEL,
    };

    let log_level = match log::LevelFilter::from_str(&log_level_string) {
        Ok(level) => level,
        Err(_) => {
            eprintln!("Invalid log level value {}", log_level_string);
            return DEFAULT_LOG_LEVEL;
        }
    };

    return log_level;
}
