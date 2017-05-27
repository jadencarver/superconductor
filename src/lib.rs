#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;
extern crate websocket;
extern crate yaml_rust;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_xml;

use libc::{c_char};
use std::ffi::CString;
use std::thread;

extern crate rand;
use rand::Rng;

extern crate termion;

extern crate git2;

pub static XML: &'static str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";

mod state;
mod xslt;
mod server;
mod task;

#[no_mangle]
pub extern "C" fn panel_xslt() -> *mut c_char {
    CString::new(xslt::panel_xslt()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn start() {
    let mut _server = server::start(None);
    if _server.is_err() {
        println!("Failed to connect on default websocket port");
        let mut rng = rand::thread_rng();
        let port = 2794 + rng.gen::<i32>() % 10;
        _server = server::start(Some(port));
    }
    match _server {
        Ok(server) => {
            thread::spawn(move || {
                for connection in server {
                    thread::spawn(move || server::connect(connection.unwrap()));
                }
            });
        },
        _ => println!("Unable to start websocket server")
    };
}

#[no_mangle]
pub extern "C" fn panel_js() -> *mut c_char {
    let markup = html! {
        script { "
        if (window === window.top) {
            //var highlight = document.createElement('script');
            //highlight.setAttribute('src', '//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.10.0/highlight.min.js');
            //document.body.appendChild(highlight);
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
