// Souk - task_status.rs
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

use glib::Enum;
use gtk::glib;

#[derive(Copy, Debug, Clone, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SkTaskStatus")]
pub enum SkTaskStatus {
    None,
    Pending,
    Preparing,
    Installing,
    InstallingBundle,
    Uninstalling,
    Updating,
    Done,
    Cancelled,
    Error,
}

impl SkTaskStatus {
    pub fn is_completed(&self) -> bool {
        self == &Self::Done || self == &Self::Cancelled || self == &Self::Error
    }

    pub fn has_no_detailed_progress(&self) -> bool {
        self == &Self::InstallingBundle
    }
}

impl From<String> for SkTaskStatus {
    fn from(string: String) -> Self {
        match string.as_str() {
            "pending" => Self::Pending,
            "preparing" => Self::Preparing,
            "install" => Self::Installing,
            "install-bundle" => Self::InstallingBundle,
            "uninstall" => Self::Uninstalling,
            "update" => Self::Updating,
            "done" => Self::Done,
            "cancelled" => Self::Cancelled,
            "error" => Self::Error,
            _ => {
                error!("Unable to parse string as SkTaskStatus: {}", string);
                Self::default()
            }
        }
    }
}

impl Default for SkTaskStatus {
    fn default() -> Self {
        SkTaskStatus::None
    }
}
