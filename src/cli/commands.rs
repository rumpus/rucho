//! CLI command definitions and handlers.

use clap::Parser;
use std::process;

use crate::utils::pid::{
    check_process_running, pid_file_path, read_pid_file, remove_pid_file, stop_process,
    write_pid_file, PidError, StopResult,
};

/// Represents the command line arguments passed to the application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: CliCommand,
}

/// Defines the available subcommands for the CLI.
#[derive(Parser, Debug)]
pub enum CliCommand {
    /// Starts the Rucho server.
    Start {},
    /// Stops the Rucho server.
    Stop {},
    /// Checks the status of the Rucho server.
    Status {},
    /// Displays the version of Rucho.
    Version {},
}

/// Handles the start command by writing the PID file.
///
/// # Returns
///
/// `true` if the PID file was written successfully, `false` otherwise.
pub fn handle_start_command() -> bool {
    println!("Starting server...");
    let pid = process::id();

    match write_pid_file(pid) {
        Ok(()) => {
            println!("Server PID {} written to {}", pid, pid_file_path());
            true
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            false
        }
    }
}

/// Handles the stop command.
pub fn handle_stop_command() {
    match read_pid_file() {
        Ok(pid_val) => {
            println!("Stopping server (PID: {})...", pid_val);
            match stop_process(pid_val) {
                StopResult::Stopped => {
                    println!("Server stopped successfully.");
                    if let Err(e) = remove_pid_file() {
                        eprintln!("Warning: {}", e);
                    }
                }
                StopResult::SignalSent => {
                    println!(
                        "Termination signal sent to process {}. It may still be shutting down.",
                        pid_val
                    );
                    println!(
                        "You might need to use kill -9 {} if it doesn't stop.",
                        pid_val
                    );
                }
                StopResult::NotFound => {
                    println!(
                        "Process {} not found. It might have already stopped.",
                        pid_val
                    );
                    if let Err(e) = remove_pid_file() {
                        eprintln!("Warning: {}", e);
                    }
                }
                StopResult::Failed => {
                    eprintln!(
                        "Error: Failed to send termination signal to process {}.",
                        pid_val
                    );
                }
            }
        }
        Err(e) => {
            if matches!(e, PidError::ReadFailed(_)) {
                println!(
                    "Server not running (PID file {} not found).",
                    pid_file_path()
                );
            } else {
                eprintln!("Error: {}", e);
            }
        }
    }
}

/// Handles the status command.
pub fn handle_status_command() {
    match read_pid_file() {
        Ok(pid_val) => {
            if check_process_running(pid_val) {
                println!("Server is running (PID: {}).", pid_val);
            } else {
                println!(
                    "Server is stopped (PID file {} found, but process {} not running).",
                    pid_file_path(),
                    pid_val
                );
                println!(
                    "Consider running 'rucho stop' to cleanup or manually delete {}.",
                    pid_file_path()
                );
            }
        }
        Err(e) => {
            if matches!(e, PidError::ReadFailed(_)) {
                println!(
                    "Server is stopped (PID file {} not found).",
                    pid_file_path()
                );
            } else {
                eprintln!("Error: {}", e);
            }
        }
    }
}

/// Handles the version command.
pub fn handle_version_command() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
