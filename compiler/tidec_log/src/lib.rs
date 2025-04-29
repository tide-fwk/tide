//! This crate allows tools to enable rust logging.
//!
//! Suppose you're working on `tidec_lir` and want to run a minimal standalone
//! program that can be debugged with access to `debug!` logs emitted by
//! `tidec_lir`. You can do this by writing:
//!
//! ```toml
//! [dependencies]
//! tidec_lir = { path = "../tidec_lir" }
//! tidec_log = { path = "../tidec_log" }
//! ```
//!
//! And in your `main.rs`:
//!
//! ```rust
//! fn main() {
//!     tidec_log::Logger::init_logger(tidec_log::LoggerConfig::from_env("LOG")).unwrap();
//!     // Your test code using tidec_lir...
//! }
//! ```
//!
//! Then run your program with:
//!
//! ```bash
//! LOG=debug cargo run
//! ```
//!
//! For convenience, you can also include this at the top of `main`:
//!
//! ```rust
//! std::env::set_var("LOG", "debug");
//! ```
//!
//! This allows you to simply run `cargo run` and still see debug output.
//!
//! ---
//!
//! The `tidec_log` crate exists as a minimal, self-contained logger setup,
//! allowing you to enable logging without depending on the much larger
//! `tidec` crate. This helps you iterate quickly on individual compiler
//! components like `tidec_lir`, without requiring full rebuilds of the entire
//! compiler stack.

use std::{env::VarError, fs::File, io::IsTerminal, path::PathBuf};
use tracing::Subscriber;
use tracing_subscriber::{
    EnvFilter, Layer, fmt::layer, prelude::*, registry::LookupSpan, util::TryInitError,
};

/// The logger for the `tidec` crate.
pub struct Logger;

/// The writer for the logger.
/// This is used to determine where the logs will be written to.
pub enum LogWriter {
    /// Write to stdout.
    Stdout,
    /// Write to stderr.
    Stderr,
    /// Write to a file.
    File(PathBuf),
}

/// The configuration for the logger.
pub struct LoggerConfig {
    /// The writer for the logger.
    pub log_writer: LogWriter,
    /// The filter for the logger.
    /// This is a string that can be "debug", "info", "warn", "error", or "trace".
    pub filter: Result<String, VarError>,
    /// Whether to use color in the logger.
    /// This is a string that can be "always", "never", or "auto".
    pub color: Result<String, VarError>,
    /// Whether to show line numbers in the logger.
    /// If this is set to "1", line numbers will be shown otherwise they will not.
    pub line_numbers: Result<String, VarError>,
}

/// The error type for the logger.
#[derive(Debug)]
pub enum LogError {
    /// The color value is not valid.
    ColorNotValid(String),
    /// The color value is not a valid unicode string.
    NotUnicode(String),
    /// Wrapping an IO error.
    IoError(std::io::Error),
    /// Wrapping a TryInitError.
    TryInitError(TryInitError),
}

impl LoggerConfig {
    /// Create a new logger configuration from the given environment variable.
    pub fn from_env(env_var: &str) -> Result<Self, VarError> {
        let filter = std::env::var(format!("{}_FILTER", env_var));
        let color = std::env::var(format!("{}_COLOR", env_var));
        let log_writer = std::env::var(format!("{}_LOG_WRITER", env_var))
            .map(|s| match s.as_str() {
                "stdout" => LogWriter::Stdout,
                "stderr" => LogWriter::Stderr,
                _ => LogWriter::File(s.into()),
            })
            .unwrap_or(LogWriter::Stderr);
        let line_numbers = std::env::var(format!("{}_LINE_NUMBERS", env_var));

        Ok(LoggerConfig {
            filter,
            color,
            log_writer,
            line_numbers,
        })
    }
}

impl Logger {
    /// Initialize the logger with the given values for the filter, coloring, and other options env variables.
    pub fn init_logger(cfg: LoggerConfig) -> Result<(), LogError> {
        let filter = match cfg.filter {
            Ok(filter) => EnvFilter::new(filter),
            Err(_) => EnvFilter::default().add_directive(tracing::Level::INFO.into()),
        };

        let color_log = match cfg.color {
            Ok(color) => match color.as_str() {
                "always" => true,
                "never" => false,
                "auto" => std::io::stderr().is_terminal(),
                e => return Err(LogError::ColorNotValid(e.to_string())),
            },
            Err(VarError::NotPresent) => std::io::stderr().is_terminal(),
            Err(VarError::NotUnicode(os_string)) => {
                return Err(LogError::NotUnicode(
                    os_string.to_string_lossy().to_string(),
                ));
            }
        };

        let line_numbers = match cfg.line_numbers {
            Ok(line_numbers) => &line_numbers == "1",
            Err(_) => false,
        };

        let layer = Self::create_layer(cfg.log_writer, color_log, line_numbers);

        let subscriber = tracing_subscriber::Registry::default()
            .with(filter)
            .with(layer);

        let _ = subscriber
            .try_init()
            .map_err(|e| LogError::TryInitError(e))
            .map_err(|e| e);

        Ok(())
    }

    fn create_layer<S>(
        log_writer: LogWriter,
        color_log: bool,
        line_numbers: bool,
    ) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        S: Subscriber,
        for<'a> S: LookupSpan<'a>,
    {
        let layer = layer()
            .with_ansi(color_log)
            .with_target(true)
            .with_line_number(line_numbers);

        match log_writer {
            LogWriter::Stdout => Box::new(layer.with_writer(std::io::stdout)),
            LogWriter::Stderr => Box::new(layer.with_writer(std::io::stderr)),
            LogWriter::File(path) => {
                let file = File::create(path).expect("Failed to create log file");
                Box::new(layer.with_writer(file))
            }
        }
    }
}

impl std::error::Error for LogError {}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::ColorNotValid(s) => write!(f, "Color not valid: {}", s),
            LogError::NotUnicode(s) => write!(f, "Not unicode: {}", s),
            LogError::IoError(e) => write!(f, "IO error: {}", e),
            LogError::TryInitError(e) => write!(f, "TryInit error: {:?}", e),
        }
    }
}
