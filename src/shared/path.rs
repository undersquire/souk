// Souk - path.rs
// Copyright (C) 2021-2023  Felix Häcker <haeckerfelix@gnome.org>
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

use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use gtk::glib;

use crate::shared::config;

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = glib::user_data_dir();
    path.push(config::NAME);
    path
});

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = glib::user_config_dir();
    path.push(config::NAME);
    path
});

pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = glib::user_cache_dir();
    path.push(config::NAME);
    path
});

pub static APPSTREAM_CACHE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut file = CACHE_DIR.clone();
    file.push("appstream.xmlb");
    file
});

pub static BIN_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = glib::home_dir();
    path.push(".local");
    path.push("bin");
    path
});

pub fn init() -> std::io::Result<()> {
    fs::create_dir_all(DATA_DIR.to_owned())?;
    fs::create_dir_all(CONFIG_DIR.to_owned())?;
    fs::create_dir_all(CACHE_DIR.to_owned())?;
    fs::create_dir_all(BIN_DIR.to_owned())?;
    Ok(())
}
