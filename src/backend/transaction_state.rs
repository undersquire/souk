use gio::prelude::*;
use glib::subclass;
use glib::subclass::prelude::*;
use glib::translate::*;

use std::cell::RefCell;

#[derive(Debug, Eq, PartialEq, Clone, Copy, GEnum)]
#[repr(u32)]
#[genum(type_name = "SoukTransactionStateKind")]
pub enum SoukTransactionMode {
    Waiting = 0,
    Running = 1,
    Finished = 2,
    Cancelled = 3,
    Error = 4, // TODO: Store error message somewhere else...
}

impl Default for SoukTransactionMode {
    fn default() -> Self {
        SoukTransactionMode::Waiting
    }
}

pub struct SoukTransactionStatePrivate {
    message: RefCell<String>,
    percentage: RefCell<f32>,
    mode: RefCell<SoukTransactionMode>,
}

static PROPERTIES: [subclass::Property; 3] = [
    subclass::Property("message", |message| {
        glib::ParamSpec::string(
            message,
            "Message",
            "Message",
            None,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("percentage", |percentage| {
        glib::ParamSpec::float(
            percentage,
            "Percentage",
            "Percentage",
            0.0,
            1.0,
            0.0,
            glib::ParamFlags::READABLE,
        )
    }),
    subclass::Property("mode", |mode| {
        glib::ParamSpec::enum_(
            mode,
            "Mode",
            "Mode",
            SoukTransactionMode::static_type(),
            SoukTransactionMode::default() as i32,
            glib::ParamFlags::READABLE,
        )
    }),
];

impl ObjectSubclass for SoukTransactionStatePrivate {
    const NAME: &'static str = "SoukTransactionState";
    type ParentType = glib::Object;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        SoukTransactionStatePrivate {
            message: RefCell::default(),
            percentage: RefCell::default(),
            mode: RefCell::default(),
        }
    }
}

impl ObjectImpl for SoukTransactionStatePrivate {
    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("message", ..) => Ok(self.message.borrow().to_value()),
            subclass::Property("percentage", ..) => Ok(self.percentage.borrow().to_value()),
            subclass::Property("mode", ..) => Ok(self.mode.borrow().to_value()),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("message", ..) => {
                let message = value.get().unwrap().unwrap();
                *self.message.borrow_mut() = message;
            }
            subclass::Property("percentage", ..) => {
                let percentage = value.get().unwrap().unwrap();
                *self.percentage.borrow_mut() = percentage;
            }
            subclass::Property("mode", ..) => {
                let mode = value.get().unwrap().unwrap();
                *self.mode.borrow_mut() = mode;
            }
            _ => unimplemented!(),
        }
    }
}

glib_wrapper! {
    pub struct SoukTransactionState(
        Object<subclass::simple::InstanceStruct<SoukTransactionStatePrivate>,
        subclass::simple::ClassStruct<SoukTransactionStatePrivate>,
        GsApplicationWindowClass>);

    match fn {
        get_type => || SoukTransactionStatePrivate::get_type().to_glib(),
    }
}

impl SoukTransactionState {
    pub fn new() -> Self {
        let state = glib::Object::new(SoukTransactionState::static_type(), &[])
            .unwrap()
            .downcast::<SoukTransactionState>()
            .unwrap();

        state
    }

    pub fn get_message(&self) -> String {
        self.get_property("message")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_percentage(&self) -> f32 {
        self.get_property("percentage")
            .unwrap()
            .get()
            .unwrap()
            .unwrap()
    }

    pub fn get_mode(&self) -> SoukTransactionMode {
        self.get_property("mode").unwrap().get().unwrap().unwrap()
    }
}

impl Default for SoukTransactionState {
    fn default() -> Self {
        SoukTransactionState::new()
    }
}
