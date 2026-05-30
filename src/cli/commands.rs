//! CLI command definitions and handlers.

use clap::Parser;
use std::process;

use crate::utils::pid::{
    check_process_running, read_pid_file, remove_pid_file, stop_process, write_pid_file, PidError,
    StopResult,
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

/// Handles the start command by writing the PID file at `pid_path`.
///
/// A write failure (read-only filesystem, missing parent directory, …) is
/// **non-fatal**: it logs a warning and the server still starts. The PID file
/// only backs `rucho stop`/`status`; a containerized server is stopped with a
/// signal (SIGTERM / Ctrl+C), so a missing PID file is acceptable there.
pub fn handle_start_command(pid_path: &str) {
    println!("Starting server...");
    let pid = process::id();

    match write_pid_file(pid_path, pid) {
        Ok(()) => println!("Server PID {} written to {}", pid, pid_path),
        Err(e) => eprintln!(
            "Warning: could not write PID file at {}: {}. Starting anyway — \
             stop the server with a signal (SIGTERM / Ctrl+C) rather than `rucho stop`.",
            pid_path, e
        ),
    }
}

/// Handles the stop command, reading the PID from `pid_path`.
pub fn handle_stop_command(pid_path: &str) {
    match read_pid_file(pid_path) {
        Ok(pid_val) => {
            println!("Stopping server (PID: {})...", pid_val);
            match stop_process(pid_val) {
                StopResult::Stopped => {
                    println!("Server stopped successfully.");
                    if let Err(e) = remove_pid_file(pid_path) {
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
                    if let Err(e) = remove_pid_file(pid_path) {
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
                println!("Server not running (PID file {} not found).", pid_path);
            } else {
                eprintln!("Error: {}", e);
            }
        }
    }
}

/// Handles the status command, reading the PID from `pid_path`.
pub fn handle_status_command(pid_path: &str) {
    match read_pid_file(pid_path) {
        Ok(pid_val) => {
            if check_process_running(pid_val) {
                println!("Server is running (PID: {}).", pid_val);
            } else {
                println!(
                    "Server is stopped (PID file {} found, but process {} not running).",
                    pid_path, pid_val
                );
                println!(
                    "Consider running 'rucho stop' to cleanup or manually delete {}.",
                    pid_path
                );
            }
        }
        Err(e) => {
            if matches!(e, PidError::ReadFailed(_)) {
                println!("Server is stopped (PID file {} not found).", pid_path);
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
