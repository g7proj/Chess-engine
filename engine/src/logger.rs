use lazy_static::lazy_static;
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

lazy_static! {
    static ref CURRENT_LOG_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Info); // Change this to set log level
    static ref LOG_FILE_PATH: Mutex<String> = Mutex::new("engine_debug.log".to_string());
}

pub struct Logger;

impl Logger {
    pub fn init(log_level: LogLevel, log_path: &str) {
        *CURRENT_LOG_LEVEL.lock().unwrap() = log_level; // Set the current log level
        *LOG_FILE_PATH.lock().unwrap() = log_path.to_string(); // Set the log file path

        // Clear the log file at initialization
        let _ = OpenOptions::new().create(true).write(true).truncate(true).open(log_path);
    }

    /**
     * Write a debug message to engine_debug.log
     */
    fn log(log_level: LogLevel, msg: &str) {
        let current: std::sync::MutexGuard<'_, LogLevel> = CURRENT_LOG_LEVEL.lock().unwrap();
        if (log_level as u8) < (*current as u8) {
            return; // Skip logging if the message level is below the current log level
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("engine_debug.log")
        {
            let _ = writeln!(file, "{}", msg);
        }
    }

    /**
     * Convenience functions for different log levels
     */
    pub fn debug(msg: &str) {
        Self::log(LogLevel::Debug, msg);
    }

    pub fn info(msg: &str) {
        Self::log(LogLevel::Info, msg);
    }

    pub fn warning(msg: &str) {
        Self::log(LogLevel::Warning, msg);
    }

    pub fn error(msg: &str) {
        Self::log(LogLevel::Error, msg);
    }
}
