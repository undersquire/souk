// Souk - task.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use gtk::glib;
use serde::{Deserialize, Serialize};

use crate::shared::task::{AppstreamTask, FlatpakTask};

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash, glib::Boxed)]
#[boxed_type(name = "Task", nullable)]
pub struct Task {
    /// Each task has a unique UUID that can be used for identification. This is
    /// required, for example, if a running task should be cancelled.
    pub uuid: String,
    /// Whether the task can be cancelled
    pub cancellable: bool,
    /// The actual task data
    pub kind: TaskKind,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone, Hash)]
pub enum TaskKind {
    Flatpak(Box<FlatpakTask>),
    Appstream(Box<AppstreamTask>),
}
