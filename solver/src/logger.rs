use crate::config;
use std::path::Path;

pub fn configure(config: &config::Log) -> anyhow::Result<()> {
    let mut dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{datetime} [{level}] {message}",
                datetime = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S.%3fZ"),
                level = level(record.level()),
                message = message
            ))
        })
        .level(config.level);

    dispatch = match &config.output {
        config::LogOutput::StdOut => dispatch.chain(std::io::stdout()),
        config::LogOutput::File(path) => {
            let log_file = match fern::log_reopen1(Path::new(&path), [libc::SIGHUP]) {
                Ok(log_file) => log_file,
                Err(error) => {
                    log::error!("Failed to open log file \"{path}\": {error}");
                    std::process::exit(1);
                }
            };
            dispatch.chain(log_file)
        }
    };

    dispatch.apply()?;

    Ok(())
}

#[inline]
fn level(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => "E",
        log::Level::Warn => "W",
        log::Level::Info => "I",
        log::Level::Debug => "D",
        log::Level::Trace => "T",
    }
}
