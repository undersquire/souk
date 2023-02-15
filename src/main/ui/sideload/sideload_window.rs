// Souk - window.rs
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

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::{File, ListStore};
use glib::{clone, closure, subclass, ParamSpec, Properties};
use gtk::{gio, glib, CompositeTemplate};
use once_cell::sync::OnceCell;

use super::SkRemoteRow;
use crate::main::app::SkApplication;
use crate::main::context::SkContext;
use crate::main::error::Error;
use crate::main::flatpak::installation::SkRemote;
use crate::main::flatpak::package::{SkPackage, SkPackageExt, SkPackageKind};
use crate::main::flatpak::sideload::{SkSideloadKind, SkSideloadable};
use crate::main::flatpak::SkFlatpakOperationType;
use crate::main::i18n::{i18n, i18n_f};
use crate::main::task::{SkTask, SkTaskStatus};
use crate::main::ui::badge::SkBadge;
use crate::main::ui::context::{SkContextBox, SkContextDetailRow};
use crate::main::ui::installation::SkInstallationListBox;
use crate::main::ui::task::SkTaskProgressBar;
use crate::shared::{config, WorkerError};

mod imp {
    use super::*;

    enum LabelType {
        PackageInstall,
        PackageUpdate,
        Repo,
    }

    #[derive(Debug, Default, Properties, CompositeTemplate)]
    #[properties(wrapper_type = super::SkSideloadWindow)]
    #[template(resource = "/de/haeckerfelix/Souk/gtk/sideload_window.ui")]
    pub struct SkSideloadWindow {
        #[property(get, set, construct_only)]
        file: OnceCell<File>,
        #[property(get)]
        sideloadable: RefCell<Option<SkSideloadable>>,
        #[property(get)]
        task: RefCell<Option<SkTask>>,

        #[template_child]
        sideload_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        sideload_leaflet: TemplateChild<adw::Leaflet>,

        #[template_child]
        cancel_sideload_button: TemplateChild<gtk::Button>,
        #[template_child]
        start_button: TemplateChild<gtk::Button>,
        #[template_child]
        package_box: TemplateChild<gtk::Box>,

        #[template_child]
        details_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        package_icon_image: TemplateChild<gtk::Image>,
        #[template_child]
        package_name_label: TemplateChild<gtk::Label>,
        #[template_child]
        package_developer_label: TemplateChild<gtk::Label>,
        #[template_child]
        package_version_label: TemplateChild<gtk::Label>,
        #[template_child]
        package_branch_badge: TemplateChild<SkBadge>,
        #[template_child]
        package_repository_badge: TemplateChild<SkBadge>,
        #[template_child]
        package_context_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        warn_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        no_updates_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        replacing_remote_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        remotes_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        remotes_listbox: TemplateChild<gtk::ListBox>,

        #[template_child]
        context_box: TemplateChild<SkContextBox>,

        #[template_child]
        installation_listbox: TemplateChild<SkInstallationListBox>,

        #[template_child]
        progress_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        progress_bar: TemplateChild<SkTaskProgressBar>,
        #[template_child]
        progress_status_label: TemplateChild<gtk::Label>,
        #[template_child]
        progress_download_label: TemplateChild<gtk::Label>,

        #[template_child]
        done_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        done_spage: TemplateChild<adw::StatusPage>,
        #[template_child]
        launch_button: TemplateChild<gtk::Button>,

        #[template_child]
        error_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        error_spage: TemplateChild<adw::StatusPage>,

        #[template_child]
        already_done_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        already_done_spage: TemplateChild<adw::StatusPage>,

        #[template_child]
        missing_runtime_spage: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkSideloadWindow {
        const NAME: &'static str = "SkSideloadWindow";
        type ParentType = adw::Window;
        type Type = super::SkSideloadWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            klass.install_action("window.go-back", None, move |w, _, _| w.imp().go_back());
            klass.install_action("window.select-installation", None, move |w, _, _| {
                w.imp().go_back()
            });
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SkSideloadWindow {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let worker = SkApplication::default().worker();

            // Add devel style class for development builds
            if config::PROFILE == "development" {
                self.obj().add_css_class("devel");
            }

            // Setup actions
            let actions = gio::SimpleActionGroup::new();
            self.obj().insert_action_group("sideload", Some(&actions));

            // sideload.select-installation
            let action = gio::SimpleAction::new("select-installation", None);
            action.connect_activate(clone!(@weak self as this => move |_, _| {
                let current = this.sideload_leaflet.visible_child_name().unwrap().to_string();
                let select_installation = this.sideload_leaflet.child_by_name("select-installation").unwrap();

                let page = match current.as_str() {
                    "details" => Some(this.sideload_leaflet.child_by_name("details").unwrap()),
                    "missing-runtime" => Some(this.sideload_leaflet.child_by_name("missing-runtime").unwrap()),
                    "already-done" => Some(this.sideload_leaflet.child_by_name("already-done").unwrap()),
                    _ => None,
                };

                // Reorder select-installation to the correct position, so that "navigate-back" works properly
                if let Some(page) = page {
                    this.sideload_leaflet.reorder_child_after(&select_installation, Some(&page));
                }

                this.sideload_leaflet.set_visible_child_name("select-installation");
            }));
            actions.add_action(&action);

            // Preselect preferred installation
            let preferred = worker.installations().preferred();
            self.installation_listbox
                .set_selected_installation(&preferred);

            // When the installation changes, we also have to update the sideloadable,
            // since it depends on the installation
            self.installation_listbox.connect_local(
                "notify::selected-installation",
                false,
                clone!(@weak self as this => @default-return None, move |_|{
                    let fut = async move {
                        this.update_sideloadable().await;
                    };
                    spawn!(fut);
                    None
                }),
            );

            // Load initial sideloadable
            let fut = clone!(@weak self as this => async move {
                this.update_sideloadable().await;
            });
            spawn!(fut);
        }
    }

    impl WidgetImpl for SkSideloadWindow {}

    impl WindowImpl for SkSideloadWindow {}

    impl AdwWindowImpl for SkSideloadWindow {}

    #[gtk::template_callbacks]
    impl SkSideloadWindow {
        fn set_sideloadable(&self, sideloadable: Option<&SkSideloadable>) {
            // The task can only be set once
            *self.sideloadable.borrow_mut() = sideloadable.cloned();
            self.obj().notify("sideloadable");

            let sideloadable = if let Some(sideloadable) = sideloadable {
                sideloadable
            } else {
                return;
            };

            self.sideload_stack.set_visible_child_name("leaflet");

            // Package
            if let Some(dry_run) = sideloadable.dry_run() {
                self.package_box.set_visible(true);
                let package = dry_run.package();

                // Set labels
                if package.operation_type() == SkFlatpakOperationType::Update {
                    self.set_labels(LabelType::PackageUpdate);
                } else {
                    self.set_labels(LabelType::PackageInstall);
                }

                // Only display launch button if it's an app
                let is_app = package.kind() == SkPackageKind::App;
                self.launch_button.set_visible(is_app);

                // Appstream
                let a = package.appstream();
                self.package_name_label.set_text(&a.name());
                self.package_icon_image.set_paintable(Some(&a.icon()));
                self.package_developer_label.set_text(&a.developer_name());
                self.package_version_label.set_text(&a.version_text(false));

                // Badges
                self.package_branch_badge.set_value(package.branch());
                self.package_repository_badge
                    .set_value(package.remote().name());

                // Context information
                let contexts = ListStore::new(SkContext::static_type());

                let download_size_context = dry_run.download_size_context();
                contexts.append(&download_size_context);

                let installed_size_context = dry_run.installed_size_context();
                contexts.append(&installed_size_context);

                let permissions_context = dry_run.package().permissions_context();
                contexts.append(&permissions_context);

                self.package_context_listbox.bind_model(
                    Some(&contexts),
                    clone!(@weak self as this => @default-panic, move |context|{
                        let context: &SkContext = context.downcast_ref().unwrap();
                        let row = SkContextDetailRow::new(&context.summary(), true);
                        row.set_activatable(true);
                        row.set_subtitle_lines(2);

                        row.connect_activated(clone!(@weak this, @weak context => move |_|{
                            this.context_box.set_context(&context);
                            this.sideload_leaflet.set_visible_child_name("context-information");
                        }));

                        row.upcast()
                    }),
                );

                // Show warning for packages without update source
                let has_update_source = dry_run.has_update_source();
                self.no_updates_row.set_visible(!has_update_source);

                // Show warning when package is already installed, but from a different remote
                if let Some(remote) = dry_run.is_replacing_remote().as_ref() {
                    self.replacing_remote_row.set_visible(true);
                    let msg = i18n_f("This package is already installed from \"{}\", during the installation the old version will be uninstalled first", &[&remote.name()]);
                    self.replacing_remote_row.set_subtitle(&msg);
                } else {
                    self.replacing_remote_row.set_visible(false);
                }

                // Show / hide warning preferences group
                self.warn_group.set_visible(
                    self.no_updates_row.is_visible() || self.replacing_remote_row.is_visible(),
                );

                // We don't support updating .flatpakrefs through sideloading, since the
                // installation would fail with "x is already installed". Only bundles can be
                // updated.
                if package.operation_type() == SkFlatpakOperationType::Update
                    && sideloadable.kind() == SkSideloadKind::Ref
                {
                    self.set_labels(LabelType::PackageInstall);
                    self.sideload_leaflet.set_visible_child_name("already-done");
                    return;
                }
            }

            // Remote(s)
            let has_remotes = sideloadable.remotes().n_items() != 0;
            self.remotes_group.set_visible(has_remotes);

            if sideloadable.remotes().n_items() != 0 {
                let remotes = sideloadable.remotes();

                if sideloadable.dry_run().is_none() {
                    self.package_box.set_visible(false);
                    self.set_labels(LabelType::Repo);

                    // Retrieve remote name
                    let remote: SkRemote = remotes.item(0).unwrap().downcast().unwrap();
                    let name = if !remote.title().is_empty() {
                        remote.title()
                    } else {
                        i18n("Software Source")
                    };

                    // The other widgets are bound to the package name / icon
                    self.package_name_label.set_text(&name);
                    self.package_icon_image.set_icon_name(Some("gear-symbolic"));

                    self.obj().set_default_height(430);

                    let msg = i18n("New applications can be obtained from this source. Only proceed if you trust this source.");
                    self.remotes_group.set_description(Some(&msg));
                } else {
                    let msg = i18n("This package adds a new software source. New applications can be obtained from it. Only proceed with the installation if you trust this source.");
                    self.remotes_group.set_description(Some(&msg));
                }

                self.remotes_listbox.bind_model(
                    Some(&remotes),
                    clone!(@weak self as this => @default-panic, move |remote|{
                        let remote: &SkRemote = remote.downcast_ref().unwrap();
                        let remote_row = SkRemoteRow::new(remote);
                        remote_row.upcast()
                    }),
                );
            }

            // Show "already done" page when there are no changes
            if sideloadable.no_changes() {
                self.sideload_leaflet.set_visible_child_name("already-done");
            } else {
                self.sideload_leaflet.set_visible_child_name("details");
            }
        }

        fn set_task(&self, task: &SkTask) {
            // The task can only be set once
            *self.task.borrow_mut() = Some(task.clone());
            self.obj().notify("task");

            task.connect_local(
                "done",
                false,
                clone!(@weak self as this => @default-return None, move |_|{
                    this.sideload_leaflet.set_visible_child_name("done");
                    None
                }),
            );

            task.connect_local(
                "cancelled",
                false,
                clone!(@weak self as this => @default-return None, move |_|{
                    this.obj().close();
                    None
                }),
            );

            task.connect_local(
                "error",
                false,
                clone!(@weak self as this => @default-return None, move |error|{
                    let msg: String = error[1].get().unwrap();
                    this.show_error_message(&msg);
                    None
                }),
            );

            // Setup progress view
            self.sideload_leaflet.set_visible_child_name("progress");
            self.progress_bar.set_task(task);

            task.property_expression("status")
                .chain_closure::<String>(closure!(
                    |_: Option<glib::Object>, status: SkTaskStatus| { status.to_string() }
                ))
                .bind(&self.progress_status_label.get(), "label", None::<&SkTask>);

            task.property_expression("download-rate")
                .chain_closure::<String>(closure!(|_: Option<glib::Object>, download_rate: u64| {
                    if download_rate != 0 {
                        i18n_f(
                            "Downloading data {}/s",
                            &[&glib::format_size(download_rate)],
                        )
                    } else {
                        String::new()
                    }
                }))
                .bind(
                    &self.progress_download_label.get(),
                    "label",
                    None::<&SkTask>,
                );
        }

        fn set_labels(&self, type_: LabelType) {
            // Label of the button which starts the sideloading
            let start_button = vec![i18n("Install"), i18n("Update"), i18n("Add")];

            // Dialog title
            let details_title = vec![
                i18n("Install Package"),
                i18n("Update Package"),
                i18n("Add Software Source"),
            ];
            let progress_title = vec![
                i18n("Installing Package"),
                i18n("Updating Package"),
                i18n("Adding Software Source"),
            ];
            let already_done_title = vec![
                i18n("Already Installed"),
                String::new(),
                i18n("Already Added Source"),
            ];
            let done_title = vec![
                i18n("Installation Complete"),
                i18n("Update Complete"),
                i18n("Added Software Source"),
            ];
            let error_title = vec![
                i18n("Installation Failed"),
                i18n("Update Failed"),
                i18n("Adding Source Failed"),
            ];

            // Descriptions which get displayed in status pages
            let already_done_desc = vec![
                i18n("This application or runtime is already installed."),
                String::new(),
                i18n("This software source is already added."),
            ];
            let done_desc = vec![
                i18n("Successfully installed!"),
                i18n("Successfully updated!"),
                i18n("Successfully added!"),
            ];

            let i = match type_ {
                LabelType::PackageInstall => 0,
                LabelType::PackageUpdate => 1,
                LabelType::Repo => 2,
            };

            self.start_button.set_label(&start_button[i]);
            self.details_title.set_title(&details_title[i]);
            self.progress_title.set_title(&progress_title[i]);
            self.already_done_title.set_title(&already_done_title[i]);
            self.done_title.set_title(&done_title[i]);
            self.error_title.set_title(&error_title[i]);

            self.already_done_spage
                .set_description(Some(&already_done_desc[i]));
            self.done_spage.set_description(Some(&done_desc[i]));
        }

        /// Load the sideloadable for the selected file asynchronously
        async fn update_sideloadable(&self) {
            self.sideload_stack.set_visible_child_name("loading");

            let installation = self.installation_listbox.selected_installation().unwrap();
            let file = self.obj().file();

            let worker = SkApplication::default().worker();
            let sideloadable = worker.load_sideloadable(&file, &installation).await;

            match sideloadable {
                Ok(sideloadable) => self.set_sideloadable(Some(&sideloadable)),
                Err(err) => {
                    if let Error::Worker(worker_error) = &err {
                        match worker_error {
                            WorkerError::DryRunRuntimeNotFound(runtime) => {
                                self.show_missing_runtime_message(runtime)
                            }
                            _ => self.show_error_message(&err.message()),
                        }
                    }

                    match err {
                        Error::Worker(_) => (),
                        Error::UnsupportedSideloadType => {
                            let message = i18n("Unknown or unsupported file format.");
                            self.show_error_message(&message);
                        }
                        Error::GLib(err) => self.show_error_message(err.message()),
                        Error::ZBus(err) => {
                            let message = i18n_f(
                                "Unable to communicate with worker process: {}",
                                &[&err.to_string()],
                            );
                            self.show_error_message(&message);
                        }
                    }
                }
            }
        }

        async fn start_sideload(&self) {
            let worker = SkApplication::default().worker();
            let sideloadable = self.obj().sideloadable().unwrap();

            match sideloadable.kind() {
                SkSideloadKind::Bundle | SkSideloadKind::Ref => {
                    let task = match sideloadable.sideload(&worker).await {
                        Ok(task) => task.unwrap(),
                        Err(err) => {
                            self.show_error_message(&err.to_string());
                            return;
                        }
                    };

                    self.set_task(&task);
                }
                SkSideloadKind::Repo => {
                    match sideloadable.sideload(&worker).await {
                        Ok(_) => self.sideload_leaflet.set_visible_child_name("done"),
                        Err(err) => self.show_error_message(&err.message()),
                    };
                }
                _ => (),
            }
        }

        fn show_error_message(&self, message: &str) {
            self.sideload_leaflet.set_visible_child_name("error");
            self.sideload_stack.set_visible_child_name("leaflet");

            self.error_spage.set_description(Some(message));
        }

        fn show_missing_runtime_message(&self, runtime: &str) {
            self.sideload_leaflet
                .set_visible_child_name("missing-runtime");
            self.sideload_stack.set_visible_child_name("leaflet");

            let message = i18n_f(
                "The required runtime <tt>{}</tt> could not be found. Possibly the runtime is available in a different installation.",
                &[runtime],
            );
            self.missing_runtime_spage.set_description(Some(&message));
        }

        fn go_back(&self) {
            self.sideload_leaflet
                .navigate(adw::NavigationDirection::Back);
        }

        #[template_callback]
        fn start_sideload_clicked(&self) {
            let fut = clone!(@weak self as this => async move{
                this.start_sideload().await;
            });
            spawn!(fut);
        }

        #[template_callback]
        fn cancel_sideload_clicked(&self) {
            let fut = clone!(@weak self as this => async move{
                let task = this.obj().task().unwrap();
                let worker = SkApplication::default().worker();

                this.cancel_sideload_button.set_sensitive(false);
                if let Err(err) = worker.cancel_task(&task).await {
                    this.show_error_message(&err.to_string());
                }
            });
            spawn!(fut);
        }

        #[template_callback]
        fn launch_app_clicked(&self) {
            let fut = clone!(@weak self as this => async move{
                let sideloadable = this.obj().sideloadable().unwrap();
                let installation = sideloadable.installation();

                let package: SkPackage = sideloadable.dry_run().unwrap().package().upcast();
                installation.launch_app(&package);

                this.obj().close();
            });
            spawn!(fut);
        }
    }
}

glib::wrapper! {
    pub struct SkSideloadWindow(
        ObjectSubclass<imp::SkSideloadWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SkSideloadWindow {
    pub fn new(file: &File) -> Self {
        glib::Object::builder().property("file", file).build()
    }
}
