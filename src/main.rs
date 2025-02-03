#![no_std]
#![no_main]
// #![cfg_attr(not(target_os = "linux"), no_main)]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use noli::*;
use saba_core::browser::Browser;
use ui_wasabi::app::WasabiUI;

// use crate::alloc::string::ToString;
// use net_wasabi::http::HttpClient;
// use noli::prelude::*;
// use saba_core::http::HttpResponse;

// static TEST_HTTP_RESPONSE: &str = r#"HTTP/1.1 200 OK
// Data: xx xx xx

// <html>
// <head></head>
// <body>
//   <h1 id="title">H1 title</h1>
//   <h2 class="class">H2 title</h2>
//   <p>Test text.</p>
//   <p>
//     <a href="example.com">Link1</a>
//     <a href="example.com">Link2</a>
//   </p>
// </body>
// </html>
// "#;

fn main() -> u64 {
    // Browser 構造体と WasabiUI 構造体を初期化
    let browser = Browser::new();
    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    match ui.borrow_mut().start() {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start {:?}", e);
            return 1;
        }
    };

    // let response =
    //     HttpResponse::new(TEST_HTTP_RESPONSE.to_string()).expect("failed to parse http response");
    // let page = browser.borrow().current_page();
    // // let dom_string = page.borrow_mut().receive_response(response);
    // // for log in dom_string.lines() {
    // //     println!("{}", log);
    // // }
    // page.borrow_mut().receive_response(response);

    0

    /*   let client = HttpClient::new();
    // match client.get("example.com".to_string(), 80, "/".to_string()) {
    match client.get("host.test".to_string(), 8000, "/test.html".to_string()) {
        Ok(res) => {
            print!("response:\n{:#?}", res);
        }
        Err(e) => {
            print!("error:\n{:#?}", e);
        }
    }
    0 */
}

entry_point!(main);
