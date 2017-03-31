#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;
extern crate websocket;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_xml;

use libc::{c_char};
use std::ffi::CString;

pub static XML: &'static str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    focus: String,
    message: String,
    include: Vec<String>,
    save_update: Option<String>
}

impl Commit {
}

mod project;
mod xslt;
mod server;
mod payload;

#[no_mangle]
pub extern "C" fn panel_xslt() -> *mut c_char {
    CString::new(xslt::panel_xslt()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn start() {
    server::start()
}

#[no_mangle]
pub extern "C" fn panel_js() -> *mut c_char {
    let markup = html! {
        script { "
        if (window === window.top) {
            PM = document.createElement('script');
            PM.setAttribute('src', '/__pm.js');
            document.body.appendChild(PM);
        }" }
    };
    CString::new(markup.into_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern fn cleanup(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        CString::from_raw(s)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
