// Souk - mod.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod dbus;
mod flatpak;
mod worker;

use std::fs;

use async_process::Command;
pub use worker::SkWorker;
use zbus::Result;

use crate::path;

/// Spawn souk-worker binary outside of the Flatpak sandbox
/// This method gets called from the main `souk` binary.
pub async fn spawn_process() {
    debug!("Start souk-worker process...");

    // First copy worker binary outside of sandbox
    let mut destination = path::BIN.clone();
    destination.push("souk-worker");
    fs::copy("/app/bin/souk-worker", destination).expect("Unable to copy souk-worker binary");

    // If we kill flatpak-spawn, we also want to kill the child process too.
    let mut args: Vec<String> = Vec::new();
    args.push("--watch-bus".into());

    // We cannot do stuff inside the Flatpak Sandbox,
    // so we have to spawn the worker process on host side
    args.push("--host".into());
    args.push("souk-worker".into());

    let mut _child = Command::new("flatpak-spawn").args(&args).spawn().unwrap();
}

/// Spawn the DBus server for worker, so it can send/receive requests.
/// This method gets called from the `souk-worker` binary.
pub async fn spawn_dbus_server() -> Result<()> {
    debug!("Start souk-worker dbus server...");
    dbus::server::start().await
}
