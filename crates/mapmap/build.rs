#[cfg(windows)]
extern crate winres;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../resources/app_icons/mapflow.ico");
        res.compile().unwrap();
    }
}
