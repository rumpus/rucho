use crate::utils::config::Config;
use std::fs;
use std::io::{Write};
use std::path::{Path, PathBuf};
use tracing_subscriber::fmt::MakeWriter;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::RollingFileAppender;

/// Enum to hold different types of MakeWriter implementations for access logging.
pub enum AccessLogMakeWriter {
    Stdout(()), // Changed to store unit type
    Stderr(()), // Changed to store unit type
    File(NonBlocking),
}

impl<'a> MakeWriter<'a> for AccessLogMakeWriter {
    type Writer = Box<dyn Write + Send + Sync + 'a>;

    fn make_writer(&'a self) -> Self::Writer {
        match self {
            // These still construct new instances or get global ones
            AccessLogMakeWriter::Stdout(_) => Box::new(std::io::stdout()),
            AccessLogMakeWriter::Stderr(_) => Box::new(std::io::stderr()),
            AccessLogMakeWriter::File(non_blocking_writer) => Box::new(non_blocking_writer.clone()),
        }
    }
}

// Custom Debug implementation
impl std::fmt::Debug for AccessLogMakeWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessLogMakeWriter::Stdout(_) => f.debug_tuple("Stdout").finish(), // Use debug_tuple for unit variant
            AccessLogMakeWriter::Stderr(_) => f.debug_tuple("Stderr").finish(), // Use debug_tuple for unit variant
            AccessLogMakeWriter::File(_) => f.debug_struct("File").finish(), // Keep as is or use debug_tuple if it holds no data for debug
        }
    }
}


/// Sets up the access log writer based on the configuration.
///
/// Returns an `AccessLogMakeWriter` enum instance and an optional `WorkerGuard`.
/// The `WorkerGuard` must be kept alive for the duration of logging for file-based appenders.
pub fn setup_access_log(
    config: &Config,
) -> (AccessLogMakeWriter, Option<WorkerGuard>) {
    match config.proxy_access_log.as_deref() {
        None => (AccessLogMakeWriter::Stderr(()), None), // Use unit variant
        Some("dev/stdout") => (AccessLogMakeWriter::Stdout(()), None), // Use unit variant
        Some("dev/stderr") => (AccessLogMakeWriter::Stderr(()), None), // Use unit variant
        Some(path_str) => {
            let log_directory: PathBuf;
            let log_file_name_prefix: String;

            if path_str.is_empty() {
                log_directory = PathBuf::from(&config.prefix);
                log_file_name_prefix = "access.log".to_string();
            } else {
                let path = Path::new(path_str);
                let final_path: PathBuf = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    Path::new(&config.prefix).join(path)
                };

                if path_str.ends_with('/')
                    || path_str.ends_with('.')
                {
                    log_directory = final_path;
                    log_file_name_prefix = "access.log".to_string();
                } else {
                    log_directory = final_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
                    log_file_name_prefix = final_path.file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new("access.log"))
                        .to_string_lossy().into_owned();
                }
            }

            // Note: The empty check for log_file_name_prefix was removed as it was deemed redundant.

            if let Err(e) = fs::create_dir_all(&log_directory) {
                eprintln!(
                    "Failed to create directory for access log at {:?}: {}. Defaulting to stderr.",
                    log_directory, e
                );
                return (AccessLogMakeWriter::Stderr(()), None); // Use unit variant
            }

            let file_appender: RollingFileAppender = tracing_appender::rolling::daily(&log_directory, &*log_file_name_prefix);
            let (non_blocking_writer, guard): (NonBlocking, WorkerGuard) = tracing_appender::non_blocking(file_appender);
            (AccessLogMakeWriter::File(non_blocking_writer), Some(guard))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::Config;

    #[test]
    fn test_setup_stdout() {
        let mut config = Config::default();
        config.proxy_access_log = Some("dev/stdout".to_string());
        let (writer, guard) = setup_access_log(&config);
        assert!(guard.is_none(), "Guard should be None for stdout");
        assert!(matches!(writer, AccessLogMakeWriter::Stdout(_)), "Writer should be Stdout variant");
        // Also can use: assert!(matches!(writer, AccessLogMakeWriter::Stdout(())));
    }

    #[test]
    fn test_setup_stderr() {
        let mut config = Config::default();
        config.proxy_access_log = Some("dev/stderr".to_string());
        let (writer, guard) = setup_access_log(&config);
        assert!(guard.is_none(), "Guard should be None for stderr");
        assert!(matches!(writer, AccessLogMakeWriter::Stderr(_)), "Writer should be Stderr variant");
        // Also can use: assert!(matches!(writer, AccessLogMakeWriter::Stderr(())));
    }

    #[test]
    fn test_setup_none_defaults_to_stderr() {
        let config = Config::default();
        let (writer, guard) = setup_access_log(&config);
        assert!(guard.is_none(), "Guard should be None for default (stderr)");
        assert!(matches!(writer, AccessLogMakeWriter::Stderr(_)), "Writer should be Stderr variant for None");
        // Also can use: assert!(matches!(writer, AccessLogMakeWriter::Stderr(())));
    }

    #[test]
    fn test_setup_file_path_returns_guard_and_creates_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir_name = "test_logs";
        let log_file_name = "access.log";

        let mut config = Config::default();
        config.prefix = temp_dir.path().to_string_lossy().to_string();
        config.proxy_access_log = Some(format!("{}/{}", log_dir_name, log_file_name));

        let (writer, guard) = setup_access_log(&config);

        assert!(guard.is_some(), "Guard should be Some for file logging");
        assert!(matches!(writer, AccessLogMakeWriter::File(_)), "Writer should be File variant");

        let expected_log_dir = temp_dir.path().join(log_dir_name);
        assert!(expected_log_dir.exists(), "Log directory should have been created at {:?}", expected_log_dir);

        drop(guard);
    }

    #[test]
    fn test_setup_file_path_absolute_creates_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_sub_dir = temp_dir.path().join("sub");
        let absolute_log_path = log_sub_dir.join("abs_access.log");

        let mut config = Config::default();
        config.prefix = "/some/other/prefix".to_string();
        config.proxy_access_log = Some(absolute_log_path.to_string_lossy().to_string());

        assert!(!log_sub_dir.exists(), "Log subdirectory should not exist before test");

        let (writer, guard) = setup_access_log(&config);

        assert!(guard.is_some(), "Guard should be Some for absolute file logging");
        assert!(matches!(writer, AccessLogMakeWriter::File(_)), "Writer should be File variant for absolute path");

        assert!(log_sub_dir.exists(), "Log subdirectory for absolute path should be created at {:?}", log_sub_dir);

        drop(guard);
    }

    #[test]
    fn test_setup_file_path_defaults_on_path_errors() {
        let mut config = Config::default();
        config.proxy_access_log = Some("".to_string());
        config.prefix = "test_prefix_for_empty_log_path".to_string();

        let (writer_empty_path, guard_empty_path) = setup_access_log(&config);
        assert!(guard_empty_path.is_some(), "Guard should be Some even for empty path string, using defaults");
        assert!(matches!(writer_empty_path, AccessLogMakeWriter::File(_)), "Writer should be File variant for empty path");

        let expected_dir_empty_path = Path::new(&config.prefix);
        assert!(expected_dir_empty_path.exists(), "Directory based on prefix should be created for empty log path");

        drop(guard_empty_path);
        if expected_dir_empty_path.exists() {
            fs::remove_dir_all(&expected_dir_empty_path).unwrap_or_else(|e| {
                eprintln!("Failed to clean up test directory {:?}: {}", expected_dir_empty_path, e);
            });
        }

        let base_temp_dir = tempfile::tempdir().unwrap();
        let file_as_dir_component = base_temp_dir.path().join("file.txt");
        fs::write(&file_as_dir_component, "i am a file").unwrap();

        config.prefix = file_as_dir_component.to_string_lossy().to_string();
        config.proxy_access_log = Some("some_log_dir/access.log".to_string());

        let (writer_dir_fail, guard_dir_fail) = setup_access_log(&config);
        assert!(guard_dir_fail.is_none(), "Guard should be None if directory creation fails");
        assert!(matches!(writer_dir_fail, AccessLogMakeWriter::Stderr(_)), "Writer should be Stderr variant if dir creation fails");
        // Also can use: assert!(matches!(writer_dir_fail, AccessLogMakeWriter::Stderr(())));
    }
}
