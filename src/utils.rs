use appstream_rs::{Bundle, Component};

pub fn get_flatpak_ref (component: &Component) -> String {
    match &component.bundle[0]{
        Bundle::Flatpak{id, runtime, sdk} => {
            return id.clone();
        },
        _ => return "".to_string(),
    };
}
