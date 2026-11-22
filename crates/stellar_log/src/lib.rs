use stellar_app::{app::App, plugins::Plugin};

pub enum LogLevel {
    Environment,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn is_env(&self) -> bool {
        matches!(self, LogLevel::Environment)
    }
}

pub struct LogPlugin {
    pub log_level: LogLevel,
}

impl Plugin for LogPlugin {
    fn build(&self, _app: &mut App) {
        let mut builder = env_logger::builder();

        if !self.log_level.is_env() {
            use log::LevelFilter;
            builder.filter_level(match self.log_level {
                LogLevel::Trace => LevelFilter::Trace,
                LogLevel::Debug => LevelFilter::Debug,
                LogLevel::Info => LevelFilter::Info,
                LogLevel::Warn => LevelFilter::Warn,
                LogLevel::Error => LevelFilter::Error,
                _ => LevelFilter::Off,
            });
        }

        builder.try_init().unwrap();

        log::info!("Initialised logger!");
    }
}

impl Default for LogPlugin {
    fn default() -> Self {
        Self::new(LogLevel::Environment)
    }
}

impl LogPlugin {
    pub fn new(log_level: LogLevel) -> Self {
        Self { log_level }
    }

    pub fn environment() -> Self {
        Self::new(LogLevel::Environment)
    }

    pub fn trace() -> Self {
        Self::new(LogLevel::Trace)
    }

    pub fn debug() -> Self {
        Self::new(LogLevel::Debug)
    }

    pub fn info() -> Self {
        Self::new(LogLevel::Info)
    }

    pub fn warn() -> Self {
        Self::new(LogLevel::Warn)
    }

    pub fn error() -> Self {
        Self::new(LogLevel::Error)
    }
}
