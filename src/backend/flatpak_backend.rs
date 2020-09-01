use bus::{Bus, BusReader};
use flatpak::prelude::*;
use flatpak::{Installation, InstallationExt, RefKind};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::backend::Package;
use crate::database::package_database;
use crate::database::queries;

pub enum BackendMessage {
    Installed,
    Removed,
}

pub struct FlatpakBackend {
    pub system_installation: Installation,
    pub user_installation: Installation,

    message_bus: RefCell<Bus<BackendMessage>>,

    packages: RefCell<HashMap<String, Package>>,
}

impl FlatpakBackend {
    pub fn new() -> Rc<Self> {
        let system_installation =
            flatpak::Installation::new_system(Some(&gio::Cancellable::new())).unwrap();

        let mut user_path = glib::get_home_dir().unwrap();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        let user_installation = flatpak::Installation::new_for_path(
            &gio::File::new_for_path(user_path),
            true,
            Some(&gio::Cancellable::new()),
        )
        .unwrap();

        let message_bus = RefCell::new(Bus::new(10));

        let packages = RefCell::new(HashMap::new());

        let backend = Rc::new(Self {
            system_installation,
            user_installation,
            message_bus,
            packages,
        });

        package_database::init(backend.clone());

        backend
    }

    /// Returns receiver which can be used to subscribe to backend messages.
    /// Receives message when something happens on Flatpak side (e.g. install/uninstall/update/...)
    pub fn get_message_receiver(self: Rc<Self>) -> BusReader<BackendMessage> {
        self.message_bus.borrow_mut().add_rx()
    }

    pub fn get_installed_packages(self: Rc<Self>) -> Vec<Package> {
        let mut installed_packages = Vec::new();

        let mut system_refs = self
            .system_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();
        let mut user_refs = self
            .user_installation
            .list_installed_refs(Some(&gio::Cancellable::new()))
            .unwrap();

        let mut installed_refs = Vec::new();
        installed_refs.append(&mut system_refs);
        installed_refs.append(&mut user_refs);

        for ref_ in installed_refs {
            let kind = match ref_.get_kind() {
                RefKind::App => "app".to_string(),
                RefKind::Runtime => "runtime".to_string(),
                _ => "unknown".to_string(),
            };
            let name = ref_.get_name().unwrap().to_string();
            let branch = ref_.get_branch().unwrap().to_string();

            match queries::get_package(name, branch, "flathub".to_string()).unwrap() {
                Some(package) => installed_packages.insert(0, package.clone()),
                None => (), //warn!("Unable to get package for flatpak ref {} ({})", name, origin),
            }
        }

        installed_packages
    }

    pub fn is_package_installed(self: Rc<Self>, package: &Package) -> bool {
        let mut result = false;

        let installed_packages = self.clone().get_installed_packages();
        let mut iter = installed_packages.into_iter();
        iter.find(|p| package == p).map(|_| {
            result = true;
            result
        });

        result
    }

    pub fn install_package(self: Rc<Self>, _package: Package) {}
}
