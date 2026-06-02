use anyhow::Result;
use serde::Deserialize;
use std::io::Write;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub level: String,
    pub writer: LogWriter,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum LogWriter {
    #[serde(rename = "file")]
    File {
        directory: String,
        file_name_prefix: String,
    },
    #[serde(rename = "stdout")]
    Stdout,
}

pub fn init_logging(config: &Config) -> Result<WorkerGuard> {
    let (writer, ansi): (Box<dyn Write + Send + 'static>, bool) = match &config.writer {
        LogWriter::File {
            directory,
            file_name_prefix,
        } => (
            Box::new(tracing_appender::rolling::daily(
                directory,
                file_name_prefix,
            )),
            false,
        ),
        LogWriter::Stdout => (Box::new(std::io::stdout()), true),
    };
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(writer);
    let filter = EnvFilter::try_new(config.level.as_str().to_lowercase())?;
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(ansi)
        .with_writer(non_blocking);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
    Ok(worker_guard)
}
