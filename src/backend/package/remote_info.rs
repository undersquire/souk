use appstream::Component;
use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;

use crate::database::DbPackage;

#[derive(Default)]
pub struct SoukRemoteInfoPrivate {
    appdata: RefCell<String>,
    commit: RefCell<String>,
    installed_size: RefCell<u64>,
    download_size: RefCell<u64>,
}

static PROPERTIES: [subclass::Property; 4] = [
    subclass::Property("appdata", |appdata| {
        glib::ParamSpec::string(
            appdata,
            "AppData",
            "AppData",
            None,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("commit", |commit| {
        glib::ParamSpec::string(commit, "Commit", "Commit", None, glib::ParamFlags::READABLE)
    }),
    subclass::Property("installed_size", |installed_size| {
        glib::ParamSpec::uint64(
            installed_size,
            "Installed Size",
            "Installed Size",
            0,
            std::u64::MAX,
            0,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("download_size", |download_size| {
        glib::ParamSpec::uint64(
            download_size,
            "Download Size",
            "Download Size",
            0,
            std::u64::MAX,
            0,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukRemoteInfoPrivate {
    const NAME: &'static str = "SoukRemoteInfo";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        Self::default()
    }
}

impl ObjectImpl for SoukRemoteInfoPrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("appdata", ..) => Ok(self.appdata.borrow().to_value()),
            subclass::Property("commit", ..) => Ok(self.commit.borrow().to_value()),
            subclass::Property("installed_size", ..) => Ok(self.installed_size.borrow().to_value()),
            subclass::Property("download_size", ..) => Ok(self.download_size.borrow().to_value()),
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukRemoteInfo(
        Object<subclass::simple::InstanceStruct<SoukRemoteInfoPrivate>,
        subclass::simple::ClassStruct<SoukRemoteInfoPrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukRemoteInfoPrivate::get_type().to_glib(),
    }
}

impl SoukRemoteInfo {
    pub fn new(db_package: &DbPackage) -> Self {
        let info = glib::Object::new(SoukRemoteInfo::static_type(), &[])
            .unwrap()
            .downcast::<SoukRemoteInfo>()
            .unwrap();

        let info_priv = SoukRemoteInfoPrivate::from_instance(&info);
        *info_priv.commit.borrow_mut() = db_package.commit.clone();
        *info_priv.installed_size.borrow_mut() = db_package.installed_size.clone() as u64;
        *info_priv.download_size.borrow_mut() = db_package.download_size.clone() as u64;

        info
    }

    pub fn appdata(&self) -> Option<Component> {
        let xml: String = self
            .get_property("appdata")
            .unwrap()
            .get()
            .unwrap()
            .unwrap();
        serde_json::from_str(&xml).ok()
    }
}
