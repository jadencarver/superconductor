#![feature(plugin)]
#![plugin(maud_macros)]
extern crate maud;
extern crate libc;

use libc::{c_char};
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn hello_world() -> *mut c_char {
    let name = "Lyra";
    let markup = html! {
        p { "Hi, " (name) "!" }
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
