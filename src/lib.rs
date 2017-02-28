#![feature(plugin)]
#![plugin(maud_macros)]
extern crate maud;
extern crate websocket;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use libc::{c_char};
use std::ffi::CString;
use maud::PreEscaped;
use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use std::process::Command;

use std::io::prelude::*;
use std::fs::File;

mod project;

static XML: &'static str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";

#[no_mangle]
pub extern "C" fn start() {
    let server = Server::bind("127.0.0.1:2794").unwrap();
    thread::spawn(move || {
        for connection in server {
            let request = connection.unwrap().read_request().unwrap();
            request.validate().unwrap();
            let mut response = request.accept();
            println!("Connection");
        }
    });
}

#[no_mangle]
pub extern "C" fn panel_js() -> *mut c_char {
    let current = project::current();
    let current_json = serde_json::to_string(&current).unwrap();
    let markup = html! {
        script { "
            PM = document.createElement('script');
            PM.setAttribute('src', '/__pm.js');
            document.body.appendChild(PM);
            " }
    };
    CString::new(markup.into_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn panel_xslt() -> *mut c_char {
    let current = project::current();
    let current_json = serde_json::to_string(&current).unwrap();
    let scss = Command::new("/usr/local/bin/sassc").arg("assets/__pm.scss").output().unwrap();
    let css = String::from_utf8(scss.stdout).unwrap();
    
    let markup = html! {
        xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" {
            xsl:output method="html" indent="yes" {}
            xsl:template match="/" {
                div#__pm__panel {
                    style type="text/css" (css)
                    form#__pm__commit style="" {
                        ul#__pm__commits {
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "In Progress"
                                        span.after  "Blocked"
                                    }
                                    li {
                                        span.name "Estimate"
                                        span.before "3"
                                        span.after  "5"
                                    }
                                }
                            }
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                button.attachments {
                                    svg id="i-paperclip" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="24" height="24" fill="none" stroke="currentcolor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" {
                                        path d="M10 9 L10 24 C10 28 13 30 16 30 19 30 22 28 22 24 L22 6 C22 3 20 2 18 2 16 2 14 3 14 6 L14 23 C14 24 15 25 16 25 17 25 18 24 18 23 L18 9" {}
                                    }
                                }
                                blockquote { "Got it working!" }
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "Blocked"
                                        span.after  "Finished"
                                    }
                                }
                            }
                        }
                        textarea name="message" {}
                        input type="submit" value="Save Update" {}
                        details {
                            summary { "Include Changes" }
                            ul#__pm__commit__changes {
                                li {
                                    label {
                                        input type="checkbox" {}
                                        span "somefile.html"
                                    }
                                    button.button--tiny { " +10 -10" }
                                }
                                li {
                                    label {
                                        input type="checkbox" {}
                                        span "someotherfile.html"
                                    }
                                    button.button--tiny { " +1 -0" }
                                }
                            }
                        }
                    }
                    header "Project Management"
                    ul.tiles {
                        li draggable="true" {
                            strong {
                                xsl:value-of select="/state/ident" {}
                            }
                        }
                    }
                }
            }
        }
    };
    CString::new(format!("{}{}", XML, markup.into_string())).unwrap().into_raw()
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
