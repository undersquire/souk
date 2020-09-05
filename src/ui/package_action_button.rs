use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{BackendMessage, FlatpakBackend, Package, PackageTransaction};

pub struct PackageActionButton {
    pub widget: gtk::Box,
    package: Package,

    flatpak_backend: Rc<FlatpakBackend>,
    builder: gtk::Builder,
}

impl PackageActionButton {
    pub fn new(flatpak_backend: Rc<FlatpakBackend>, package: Package) -> Rc<Self> {
        let builder = gtk::Builder::from_resource(
            "/de/haeckerfelix/FlatpakFrontend/gtk/package_action_button.ui",
        );
        get_widget!(builder, gtk::Box, package_action_button);

        let package_action_button = Rc::new(Self {
            widget: package_action_button,
            package,
            flatpak_backend,
            builder,
        });

        package_action_button.clone().update_stack();
        package_action_button.clone().setup_signals();
        package_action_button
    }

    fn setup_signals(self: Rc<Self>) {
        // install
        get_widget!(self.builder, gtk::Button, install_button);
        install_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().install_package(package.clone());
        }));

        // uninstall
        get_widget!(self.builder, gtk::Button, uninstall_button);
        uninstall_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().uninstall_package(package.clone());
        }));

        // open
        get_widget!(self.builder, gtk::Button, open_button);
        open_button.connect_clicked(clone!(@strong self.flatpak_backend as flatpak_backend, @strong self.package as package => move |_|{
            flatpak_backend.clone().launch_package(package.clone());
        }));

        spawn!(self.clone().backend_message_receiver());
    }

    async fn backend_message_receiver(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        let mut backend_channel = self.flatpak_backend.clone().get_channel();

        while let Some(backend_message) = backend_channel.recv().await {
            match backend_message {
                BackendMessage::NewPackageTransaction(transaction) => {
                    // We only care about this package
                    if transaction.package == self.package {
                        let mut transaction_channel = transaction.clone().get_channel();

                        // We have a transaction which affects this package, so display progressbar
                        button_stack.set_visible_child_name("processing");

                        // Listen to transaction state changes
                        while let Some(state) = transaction_channel.recv().await {
                            if state.is_finished {
                                self.clone().update_stack();
                                break;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn update_stack(self: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, button_stack);

        match self
            .flatpak_backend
            .clone()
            .is_package_installed(&self.package)
        {
            true => {
                button_stack.set_visible_child_name("installed");
            }
            false => {
                button_stack.set_visible_child_name("install");
            }
        };
    }
}
