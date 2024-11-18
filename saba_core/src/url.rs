use alloc::string::String;
use alloc::vec::Vec;

/// URL を表す構造体。
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl Url {
    /// URL 構造体のインスタンスを作成するための new　関数。
    pub fn new(url: String) => Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        }
    }

    /// URL を解析するメソッド。
    pub fn parse(&mut self) => Result<Self, String> {
        if !self.is_http() {
            return Err("Only HTTP scheme is supported.".to_string());
        }

        self.host = self.extract_host();
        self.port = self.extract_port();
        self.path = self.extract_path();
        self.searchpart = self.extract_searchpart();

        Ok(self.clone())
    }

    /// URL が http:// で開始されているかどうか。
    pub fn is_http(&mut self) -> bool {
        if self.url.contains("http://") {
            return true;
        }
        false
    }

    /// URL からホストを取得する。
    fn extract_host(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        if let some(index) = url_parts[0].find(':') {
            url_parts[0][..index].to_string()
        } else {
            url_parts[0].to_string()
        }
    }

    /// URL からポート番号を取得する。
    fn extract_port(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect();
        
        if let some(index) = url_parts[0].find(':') {
            url_parts[0][index + 1..].to_string()
        } else {
            "80".to_string()
        }
    }

    /// URL　からパス名を取得する。
    fn extract_path(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect()

        if url_parts.len() < 2 {
            return "".to_string()
        }
        let path_and_searchpart: Vec<&str> = url_parts[1]
            .splitn(2, "?")
            .collect();
        path_and_searchpart[0].to_string()
    }

    /// URL からクエリパラメータを取得する。
    fn extract_searchpart(&self) -> String {
        let url_parts: Vec<&str> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, "/")
            .collect()
        if url_parts.len() < 2 {
            return "".to_string()
        }

        let path_and_searchpart: Vec<&str> = url_parts[1]
            .splitn(2, "?")
            .collect();
        if path_and_searchpart.len() < 2 {
            "".to_string()
        } else {
            path_and_searchpart[1].to_string()
        }
    }

    /// URL　のホストを取得する。
    pub fn host(&self) -> String {
        self.host.clone()
    }
    ///　URL　のポート番号を取得する。
    pub fn port(&self) -> String {
        self.port.clone()
    }
    ///　URL　のパスを取得する。
    pub fn path(&self) -> String {
        self.path.clone()
    }
    ///　URL　のクエリパラメータを取得する。
    pub fn searchpart(&self) -> String {
        self.searchpart.clone()
    }        
}