use colog::format::CologStyle;
use env_logger::Builder;
use log::Level;

pub struct CustomLog;

impl CologStyle for CustomLog {
    fn level_token(&self, level: &Level) -> &str {
        match *level {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }
}

pub fn init(verbose: bool) {
    let mut builder = Builder::new();
    builder.format(colog::formatter(CustomLog));
    if verbose {
        builder.filter(None, log::LevelFilter::Info);
    } else {
        builder.filter(None, log::LevelFilter::Warn);
    }

    builder.init();
}
