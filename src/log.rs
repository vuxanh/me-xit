use std::env::{self, VarError};

use logger::{LogConfig, LogSender, Logger};

pub fn init_log() -> Result<LogSender, VarError> {
    let log_path = env::var("LOG_PATH")?;
    let log_file_name = env::var("LOG_FILE_NAME")?;

    let config = LogConfig::new()
        .path(&log_path)
        .file_name(&log_file_name)
        .max_file_size(10 * (1 << 20))
        .max_buffer_size(6);

    let log = Logger::new(&config);
    let stream = log.stream();

    log.register();

    Ok(stream)
}
