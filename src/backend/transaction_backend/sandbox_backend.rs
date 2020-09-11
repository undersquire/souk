use async_process::Command;
use async_process::Stdio;
use futures_util::io::BufReader;
use futures_util::AsyncBufReadExt;
use futures_util::StreamExt;
use regex::Regex;

use std::sync::Arc;

use crate::backend::transaction_backend::TransactionBackend;
use crate::backend::{PackageAction, PackageTransaction, TransactionMode, TransactionState};

pub struct SandboxBackend {}

impl TransactionBackend for SandboxBackend {
    fn new() -> Self {
        Self {}
    }

    fn add_package_transaction(&self, transaction: Arc<PackageTransaction>) {
        debug!("New transaction: {:#?}", transaction);
        spawn!(Self::execute_package_transacton(transaction));
    }
}

impl SandboxBackend {
    async fn execute_package_transacton(transaction: Arc<PackageTransaction>) {
        // Set initial transaction state
        let mut state = TransactionState::default();
        state.percentage = 0.0;
        transaction.set_state(state);

        let args = Self::get_flatpak_args(&transaction);
        let mut child = Command::new("flatpak-spawn")
            .args(&args)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

        while let Some(line) = lines.next().await {
            // Check if transaction got cancelled
            if transaction.is_cancelled(){
                match child.kill(){
                    Ok(_) => debug!("Cancelled transaction successfully."),
                    Err(err) => warn!("Unable to cancel transaction: {}", err),
                };

                let mut state = TransactionState::default();
                state.mode = TransactionMode::Cancelled;

                break;
            }

            println!("{}", line.as_ref().unwrap());
            let state = Self::parse_line(line.unwrap());
            transaction.set_state(state);
        }

        // Finish transaction
        let mut state = TransactionState::default();
        state.percentage = 1.0;
        state.mode = TransactionMode::Finished;
        transaction.set_state(state);

        debug!("Finished package transaction.");
    }

    fn get_flatpak_args(transaction: &PackageTransaction) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        args.push("--host".into());
        args.push("flatpak".into());

        match transaction.action {
            PackageAction::Install => {
                args.push("install".into());
                args.push("--system".into());
                args.push(transaction.package.remote.clone());
                args.push(transaction.package.app_id.clone());
                args.push("-y".into());
            }
            PackageAction::Uninstall => {
                args.push("uninstall".into());
                args.push("--system".into());
                args.push(transaction.package.app_id.clone());
                args.push("-y".into());
            }
            _ => (),
        };

        args
    }

    fn parse_line(line: String) -> TransactionState {
        let mut state = TransactionState::default();
        state.mode = TransactionMode::Running;
        state.message = line.clone();

        // Regex to get percentage value
        let regex = Regex::new(r"(\d{1,3})%").unwrap();

        if let Some(percentage) = regex.captures(&line) {
            let value = percentage.get(1).unwrap().as_str();
            let percentage: f32 = value.parse().unwrap();
            state.percentage = percentage / 100.0;
        }

        state
    }
}
