extern crate alloc;
use alloc::string::String;
use alloc::format;
use alloc::string::ToString;
use noli::net::lookup_host;
use saba_core::error::Error;
use saba_core::http::HttpResponse;
use noli::net::SocketAddr;
use noli::net::TcpStream;
use alloc::vec::Vec;

/// HTTP リクエストと HTTP レスポンスを扱う HTTPClient 構造体
pub struct HttpClient {}

impl HttpClient {
  pub fn new() -> Self {
    Self {}
  }

  pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
    // wasabiOS の lookup_host 関数を使ってホスト名を IP アドレスに変換する。
    // 戻り値は IP アドレスのベクタである。
    let ips = match lookup_host(&host) {
      Ok(ips) => ips,
      Err(e) => {
        return Err(Error::Network(format!(
          "Failed to find IP addresses: {:#?}",e
        )))
      }
    };
    // 1つも IP アドレスを取得できなかった場合はエラーとする。
    if ips.len() < 1 {
      return Err(Error::Network("Failed to find IP addresses".to_string()))
    }

    // TCP ストリームを構築する。noli ライブラリが提供する TcpStream 構造体とデータ書き込み API を使用する。
    // connect メソッドを使ってコネクションを確立し、成功の場合は TcpStream 構造体を返す。
    let socket_addr: SocketAddr = (ips[0], port).into();
    let mut stream = match TcpStream::connect(socket_addr) {
      Ok(stream) => stream,
      Err(_) => {
        return Err(Error::Network(
          "Faild to connect to TCP stream".to_string(),
        ))
      }
    };

    // TCP ストリームに送信する HTTP のリクエストラインを構築する。
    // メソッド名、パス名、HTTP バージョンをホワイトスペースで結合する。
    let mut request = String::from("GET /");
    request.push_str(&path);
    request.push_str(" HTTP/1.1\n");

    // ヘッダの追加
    request.push_str("Host: ");
    request.push_str(&host);
    request.push('\n');
    request.push_str("Accept: text:html\n");
    request.push_str("Connection: close\n");
    request.push('\n');

    // リクエストの送信
    // TcpStream 構造体の write メソッドで行う。write メソッドの戻り値は送信したバイト数である。
    let _bytes_written = match stream.write(request.as_bytes()) {
      Ok(bytes) => bytes,
      Err(_) => {
        return Err(Error::Network(
          "Failed to send a request to TCP stream".to_string(),
        ))
      }
    };

    // レスポンスの受信
    // レスポンスの受信は TcpStream 構造体の read メソッドで行う。read メソッドの引数に HTTP レスポンスを格納するバッファを渡す。
    // read メソッドは読み込んだバイト数を返却する。読み込むバイト数が 0 になるまで繰り返す。
    // 分割されたレスポンスは Vec の extend_from_slice で結合する。    
    let mut received = Vec::new();
    loop {
        let mut buf = [0u8; 4096];
        let bytes_read = match stream.read(&mut buf) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::Network(
                    "Failed to receive a request from TCP stream".to_string(),
                ))
            }
        };
        if bytes_read == 0 {
            break;
        }
        received.extend_from_slice(&buf[..bytes_read]);
    }

    match core::str::from_utf8(&received) {
      Ok(response) => HttpResponse::new(response.to_string()),
      Err(e) => Err(Error::Network(format!("Invalid ceveived response: {}", e))),
    }

  }
}