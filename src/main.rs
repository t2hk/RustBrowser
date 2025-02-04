#![no_std]
#![no_main]
// #![cfg_attr(not(target_os = "linux"), no_main)]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use net_wasabi::http::HttpClient;
use noli::*;
use saba_core::browser::Browser;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use saba_core::url::Url;
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

    match ui.borrow_mut().start(handle_url) {
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

/// 引数の URL にアクセスし、HttpResponse 構造体を返す。
fn handle_url(url: String) -> Result<HttpResponse, Error> {
    // URL を解釈する。
    let parsed_url = match Url::new(url.to_string()).parse() {
        Ok(url) => url,
        Err(e) => {
            return Err(Error::UnexpectedInput(format!(
                "input html is not supported {:?}",
                e
            )));
        }
    };

    // HTTP リクエストを送信する。
    let client = HttpClient::new();
    let response = match client.get(
        parsed_url.host(),
        parsed_url.port().parse::<u16>().expect(&format!(
            "port number should be u16 but got {}",
            parsed_url.port()
        )),
        parsed_url.path(),
    ) {
        Ok(res) => {
            // HTTP レスポンスのステータスコードが 302 の場合、転送する（リダイレクト）。
            // Location が示す転送先 URL を解釈し、HTTP アクセスする。
            if res.status_code() == 302 {
                let location = match res.header_value("Location") {
                    Ok(value) => value,
                    Err(_) => return Ok(res),
                };
                let redirect_parsed_url = Url::new(location);

                let redirect_res = match client.get(
                    redirect_parsed_url.host(),
                    redirect_parsed_url.port().parse::<u16>().expect(&format!(
                        "port number should be u16 but got {}",
                        parsed_url.port()
                    )),
                    redirect_parsed_url.path(),
                ) {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Network(format!("{:?}", e))),
                };
                redirect_res
            } else {
                // ステータスコードが 302 以外の場合、そのまま返す。
                res
            }
        }
        Err(e) => {
            return Err(Error::Network(format!(
                "failed to get http response: {:?}",
                e
            )))
        }
    };
    Ok(response)
}

entry_point!(main);
