//! Clean twin: the composition root wires the layers and opens no console window.
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use billing_interfaces::Message;

fn main() -> ExitCode {
    let wired = billing_infrastructure::load("i-1", 1).is_some() && billing_interfaces::draft("i-1", 1) == Message::Drafted;
    if wired { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
