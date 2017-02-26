#![feature(plugin)]
#![plugin(maud_macros)]
extern crate maud;

extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use libc::{c_char};
use std::ffi::CString;
use maud::PreEscaped;

mod project;

#[no_mangle]
pub extern "C" fn hello_world() -> *mut c_char {
    let current = project::current();
    let current_json = serde_json::to_string(&current).unwrap();
    let markup = html! {
        script { "
            PM = document.createElement('script');
            PM.current = " (PreEscaped(current_json)) ";
            PM.setAttribute('src', '/__pm.js');
            document.body.appendChild(PM);
            " }
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
