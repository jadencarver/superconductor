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
extern crate markup;
extern crate git2;

pub static XML: &'static str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";


mod state;
mod view;
mod server;
mod task;

#[no_mangle]
pub extern "C" fn panel_xslt() -> *mut c_char {
    CString::new(view::panel_xslt()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn start() -> i32 {
    let mut port = 2794;
    let mut _server = server::start(port);
    if _server.is_err() {
        println!("Failed to connect on default websocket port");
        let mut rng = rand::thread_rng();
        port = port + rng.gen::<i32>() % 10;
        _server = server::start(port);
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
    port
}

#[no_mangle]
pub extern "C" fn panel_js(port: i32) -> *mut c_char {
    let markup = format!("<script>
        if (window === window.top) {}
            //var highlight = document.createElement('script');
            //highlight.setAttribute('src', '//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.10.0/highlight.min.js');
            //document.body.appendChild(highlight);
            PM = document.createElement('script');
            PM.setAttribute('src', '/assets/__pm.js');
            PM.port = {};
            document.body.appendChild(PM);
        {}
        </script>", "{", port, "}");
    CString::new(markup).unwrap().into_raw()
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
    use std::env;
    use std::process::Command;

    #[test]
    fn it_works() {
        env::set_var("TARGET",  "debug");
        env::set_var("GIT_DIR", "tmp/dummy/.git");

        let rspec = Command::new("rspec")
            .arg("spec/integration")
            .status().expect("Failed to execute rspec");

        assert!(rspec.success(), "rspec reported failing tests");
    }
}
