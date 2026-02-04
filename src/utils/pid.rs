//! PID file management utilities for the Rucho daemon.
//!
//! This module provides functions for managing the PID file used to track
//! the running server process. The PID file allows the CLI to check status
//! and stop the server gracefully.

use std::fs;
use std::io::Write;
use sysinfo::{Pid, Signal, System};

use crate::utils::constants::PID_FILE_PATH;

/// Errors that can occur during PID file operations.
#[derive(Debug)]
pub enum PidError {
    /// Failed to create the PID file
    CreateFailed(std::io::Error),
    /// Failed to write to the PID file
    WriteFailed(std::io::Error),
    /// Failed to read the PID file
    ReadFailed(std::io::Error),
    /// Failed to remove the PID file
    RemoveFailed(std::io::Error),
    /// Invalid PID format in the file
    InvalidFormat,
    /// Process not found
    ProcessNotFound(usize),
    /// Failed to send signal to process
    SignalFailed(usize),
}

impl std::fmt::Display for PidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PidError::CreateFailed(e) => write!(f, "Failed to create PID file: {}", e),
            PidError::WriteFailed(e) => write!(f, "Failed to write PID file: {}", e),
            PidError::ReadFailed(e) => write!(f, "Failed to read PID file: {}", e),
            PidError::RemoveFailed(e) => write!(f, "Failed to remove PID file: {}", e),
            PidError::InvalidFormat => write!(f, "Invalid PID format in file"),
            PidError::ProcessNotFound(pid) => write!(f, "Process {} not found", pid),
            PidError::SignalFailed(pid) => write!(f, "Failed to send signal to process {}", pid),
        }
    }
}

impl std::error::Error for PidError {}

/// Writes the current process PID to the PID file.
///
/// # Arguments
///
/// * `pid` - The process ID to write
///
/// # Returns
///
/// `Ok(())` on success, or a `PidError` if the operation fails.
pub fn write_pid_file(pid: u32) -> Result<(), PidError> {
    let mut file = fs::File::create(PID_FILE_PATH).map_err(PidError::CreateFailed)?;
    writeln!(file, "{}", pid).map_err(PidError::WriteFailed)?;
    Ok(())
}

/// Reads the PID from the PID file.
///
/// # Returns
///
/// `Ok(pid)` if successful, or `Err(PidError)` if the file doesn't exist,
/// can't be read, or contains an invalid format.
pub fn read_pid_file() -> Result<usize, PidError> {
    let contents = fs::read_to_string(PID_FILE_PATH).map_err(PidError::ReadFailed)?;
    contents
        .trim()
        .parse::<usize>()
        .map_err(|_| PidError::InvalidFormat)
}

/// Removes the PID file.
///
/// # Returns
///
/// `Ok(())` on success, or a `PidError` if removal fails.
pub fn remove_pid_file() -> Result<(), PidError> {
    fs::remove_file(PID_FILE_PATH).map_err(PidError::RemoveFailed)
}

/// Checks if a process with the given PID is running.
///
/// # Arguments
///
/// * `pid_val` - The process ID to check
///
/// # Returns
///
/// `true` if the process exists, `false` otherwise.
pub fn check_process_running(pid_val: usize) -> bool {
    let mut system = System::new_all();
    system.refresh_processes();
    system.process(Pid::from(pid_val)).is_some()
}

/// Result of attempting to stop a process.
#[derive(Debug, PartialEq)]
pub enum StopResult {
    /// Process was stopped successfully
    Stopped,
    /// Signal was sent but process may still be running
    SignalSent,
    /// Process was not found (already stopped)
    NotFound,
    /// Failed to send the termination signal
    Failed,
}

/// Attempts to stop a process by sending SIGTERM.
///
/// # Arguments
///
/// * `pid_val` - The process ID to stop
///
/// # Returns
///
/// A `StopResult` indicating the outcome of the stop attempt.
pub fn stop_process(pid_val: usize) -> StopResult {
    let pid = Pid::from(pid_val);
    let mut system = System::new_all();
    system.refresh_processes();

    match system.process(pid) {
        Some(process) => match process.kill_with(Signal::Term) {
            Some(true) => {
                // Wait a moment and check if process stopped
                std::thread::sleep(std::time::Duration::from_secs(1));
                system.refresh_processes();
                if system.process(pid).is_none() {
                    StopResult::Stopped
                } else {
                    StopResult::SignalSent
                }
            }
            Some(false) | None => {
                // Signal failed, but check if process is gone anyway
                system.refresh_processes();
                if system.process(pid).is_none() {
                    StopResult::NotFound
                } else {
                    StopResult::Failed
                }
            }
        },
        None => StopResult::NotFound,
    }
}

/// Returns the path to the PID file.
pub fn pid_file_path() -> &'static str {
    PID_FILE_PATH
}
