use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::{BoxImpl, WidgetImpl};

use std::cell::Cell;
use std::cell::RefCell;

use crate::backend::SoukPackage;
use crate::ui::utils;

pub struct SoukPackageRowPrivate {
    package: RefCell<Option<SoukPackage>>,
    installed_view: Cell<bool>,
    builder: gtk::Builder,
}

static PROPERTIES: [subclass::Property; 1] = [subclass::Property("package", |package| {
    glib::ParamSpec::object(
        package,
        "Package",
        "Package",
        SoukPackage::static_type(),
        glib::ParamFlags::READWRITE,
    )
})];

impl ObjectSubclass for SoukPackageRowPrivate {
    const NAME: &'static str = "SoukPackageRow";
    type Type = SoukPackageRow;
    type ParentType = gtk::Box;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    glib_object_subclass!();

    fn new() -> Self {
        let package = RefCell::new(None);
        let builder = gtk::Builder::from_resource("/de/haeckerfelix/Souk/gtk/package_row.ui");
        let installed_view = Cell::default();

        Self {
            package,
            installed_view,
            builder,
        }
    }
}

impl ObjectImpl for SoukPackageRowPrivate {
    fn set_property(&self, _obj: &SoukPackageRow, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => {
                let package = value.get().unwrap();
                *self.package.borrow_mut() = package;
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &SoukPackageRow, id: usize) -> glib::Value {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("package", ..) => self.package.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for SoukPackageRowPrivate {}

impl BoxImpl for SoukPackageRowPrivate {}

glib_wrapper! {
    pub struct SoukPackageRow(ObjectSubclass<SoukPackageRowPrivate>)
    @extends gtk::Widget, gtk::Box;
}

impl SoukPackageRow {
    pub fn new(installed_view: bool) -> Self {
        let row = glib::Object::new(SoukPackageRow::static_type(), &[])
            .unwrap()
            .downcast::<SoukPackageRow>()
            .unwrap();

        let self_ = SoukPackageRowPrivate::from_instance(&row);
        self_.installed_view.set(installed_view);

        get_widget!(self_.builder, gtk::Box, package_row);
        row.append(&package_row);

        row.setup_signals();
        row
    }

    fn setup_signals(&self) {
        self.connect_notify(Some("package"), |this, _| {
            let self_ = SoukPackageRowPrivate::from_instance(this);
            let package = self_.package.borrow().as_ref().unwrap().clone();

            get_widget!(self_.builder, gtk::Label, title_label);
            get_widget!(self_.builder, gtk::Label, summary_label);
            get_widget!(self_.builder, gtk::Image, icon_image);
            get_widget!(self_.builder, gtk::Label, branch_label);
            get_widget!(self_.builder, gtk::Image, installed_check);
            get_widget!(self_.builder, gtk::Box, uninstall_box);
            get_widget!(self_.builder, gtk::Button, uninstall_button);
            get_widget!(self_.builder, gtk::Label, installed_size_label);

            // Icon
            utils::set_icon(&package, &icon_image, 64);

            match package.get_appdata() {
                Some(appdata) => {
                    // Title
                    utils::set_label_translatable_string(&title_label, Some(appdata.name.clone()));
                    // Summary
                    utils::set_label_translatable_string(&summary_label, appdata.summary.clone());
                }
                None => {
                    // Fallback to basic information when no appdata available
                    title_label.set_text(&package.get_name());
                    summary_label.set_text(&package.get_branch());
                }
            };

            // Installed indicator
            if !self_.installed_view.get() {
                package
                    .bind_property("is_installed", &installed_check, "visible")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build()
                    .unwrap();
            }

            // Branch label / tag
            let branch = package.get_branch();
            if branch != "stable" {
                branch_label.set_text(&branch.to_uppercase());
                branch_label.set_visible(true);

                let ctx = branch_label.get_style_context();
                ctx.remove_class("branch-label-orange");
                ctx.remove_class("branch-label-red");

                if branch == "beta" {
                    ctx.add_class("branch-label-orange");
                }

                if branch == "master" {
                    ctx.add_class("branch-label-red");
                }
            } else {
                branch_label.set_visible(false);
            }

            // Uninstall button
            uninstall_button.set_sensitive(true);
            if self_.installed_view.get() {
                uninstall_box.set_visible(true);

                let bytes = package
                    .get_installed_info()
                    .as_ref()
                    .unwrap()
                    .get_installed_size();
                let size = glib::format_size(bytes);
                installed_size_label.set_text(&size);

                uninstall_button.connect_clicked(clone!(@weak package => move|btn|{
                    btn.set_sensitive(false);
                    package.uninstall();
                }));
            }
        });
    }

    pub fn set_package(&self, package: &SoukPackage) {
        self.set_property("package", package).unwrap();
    }

    pub fn get_package(&self) -> Option<SoukPackage> {
        self.get_property("package").unwrap().get().unwrap()
    }
}
