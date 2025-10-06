use flexi_logger::{
    Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, Naming, WriteMode,
};
use log::Record;

/// Pretty line format
fn line_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    let header = format!(
        "[{}] {:<5} ",
        now.format("%Y-%m-%d %H:%M:%S"),
        record.level(),
    );
    let msg = record.args().to_string();

    let indented = msg
        .lines()
        .enumerate()
        .map(|(i, line)| if i == 0 { format!("{}{}", header, line) }
                         else { format!("{:width$}{}", "", line, width = header.len()) })
        .collect::<Vec<_>>()
        .join("\n");

    writeln!(w, "{indented}")
}

/// Initialize logger 
pub fn init_logger(session_id: u64, dir: &str, level_spec: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir).ok();

    let file_name = format!("session_{}.log", session_id);
    let file_spec = FileSpec::default()
        .directory(dir)
        .basename(&file_name)
        .suppress_timestamp();

    Logger::try_with_str(level_spec)?
        .format(line_format)
        .log_to_file(file_spec)
        .write_mode(WriteMode::BufferAndFlush)
        .rotate(
            Criterion::Size(30_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(10),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()?;

    Ok(())
}

/// Redact + truncate JSON payloads for wire logs
pub fn trunc_json(v: &serde_json::Value, max_chars: usize, redact_tokens: bool) -> String {
    let mut s = serde_json::to_string(v).unwrap_or_default();

    if redact_tokens {
        for key in &[
            "token",
            "jwt",
            "authorization",
            "apiKey",
            "clientSecret",
            "password",
            "totp",
        ] {
            s = s.replace(&format!("\"{}\":\"", key), &format!("\"{}\":\"<REDACTED>", key));
        }
    }

    if s.len() > max_chars {
        let mut t = s;
        t.truncate(max_chars);
        t.push_str("…<truncated>");
        t
    } else {
        s
    }
}

/// Switch struct for config flags
#[derive(Clone)]
pub struct LogSwitches {
    pub wire_log: bool,
    pub redact: bool,
    pub max_body_chars: usize,
    pub log_vectors: bool,
    pub log_results: bool,
    pub log_uploads: bool,
    pub log_downloads: bool,
}

impl From<&crate::logging::config::AppConfig> for LogSwitches {
    fn from(c: &crate::logging::config::AppConfig) -> Self {
        Self {
            wire_log: c.wire_log,
            redact: c.redact,
            max_body_chars: c.max_body_chars,
            log_vectors: c.log_vectors,
            log_results: c.log_results,
            log_uploads: c.log_uploads,
            log_downloads: c.log_downloads,
        }
    }
}