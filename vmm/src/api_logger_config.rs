use std::result;

use data_model::vm::{LoggerDescription, LoggerError, LoggerLevel};
use logger::{Level, LOGGER};

type Result<T> = result::Result<T, LoggerError>;

pub fn init_logger(api_logger: LoggerDescription) -> Result<()> {
    //there are 3 things we need to get out: the level, whether to show it and whether to show the origin of the log
    let level = from_api_level(api_logger.level);

    if let Some(val) = level {
        LOGGER.set_level(val);
    }

    if let Some(val) = api_logger.show_log_origin {
        LOGGER.set_include_origin(val, val);
    }

    if let Some(val) = api_logger.show_level {
        LOGGER.set_include_level(val);
    }

    if let Err(ref e) = LOGGER.init(Some(api_logger.path)) {
        return Err(LoggerError::InitializationFailure(e.to_string()));
    } else {
        Ok(())
    }
}

fn from_api_level(api_level: Option<LoggerLevel>) -> Option<Level> {
    if let Some(val) = api_level {
        match val {
            LoggerLevel::Error => Some(Level::Error),
            LoggerLevel::Warning => Some(Level::Warn),
            LoggerLevel::Info => Some(Level::Info),
            LoggerLevel::Debug => Some(Level::Debug),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader};

    fn validate_logs(
        log_path: &str,
        expected: &[(&'static str, &'static str, &'static str)],
    ) -> bool {
        let f = File::open(log_path).unwrap();
        let mut reader = BufReader::new(f);
        let mut res = true;
        let mut line = String::new();
        for tuple in expected {
            line.clear();
            reader.read_line(&mut line).unwrap();
            res &= line.contains(&tuple.0);
            res &= line.contains(&tuple.1);
            res &= line.contains(&tuple.2);
        }
        res
    }

    #[test]
    fn test_init_logger_from_api() {
        let desc = LoggerDescription {
            path: String::from(""),
            level: None,
            show_level: None,
            show_log_origin: None,
        };
        assert!(init_logger(desc).is_err());

        let filename = "tmp.log";
        let desc = LoggerDescription {
            path: String::from(filename),
            level: Some(LoggerLevel::Warning),
            show_level: Some(true),
            show_log_origin: Some(true),
        };
        let res = init_logger(desc).is_ok();

        if !res {
            let _x = fs::remove_file(filename);
        }

        assert!(res);

        info!("info");
        warn!("warning");
        error!("error");

        // info should not be outputted
        let res = validate_logs(
            filename,
            &[
                ("[WARN", "logger_config.rs", "warn"),
                ("[ERROR", "logger_config.rs", "error"),
            ],
        );
        let _x = fs::remove_file(filename);
        assert!(res);
    }

    #[test]
    fn test_from_api_level() {
        assert_eq!(from_api_level(Some(LoggerLevel::Error)), Some(Level::Error));
        assert_eq!(
            from_api_level(Some(LoggerLevel::Warning)),
            Some(Level::Warn)
        );
        assert_eq!(from_api_level(Some(LoggerLevel::Info)), Some(Level::Info));
        assert_eq!(from_api_level(Some(LoggerLevel::Debug)), Some(Level::Debug));
    }
}
