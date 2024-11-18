use alloc::string::String;
use alloc::string::ToString;
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
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        }
    }

    /// URL を解析するメソッド。
    pub fn parse(&mut self) -> Result<Self, String> {
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
        if let Some(index) = url_parts[0].find(':') {
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
        
        if let Some(index) = url_parts[0].find(':') {
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
            .collect();

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
            .collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// ポート番号 80 の URL 構造体のテスト。
    fn test_url_host() {
        let url = "http://example.com".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    ///　ポート番号 8888 の URL 構造体のテスト。
    fn test_url_host_port() {
        let url = "http://example.com:8888".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    /// ポート番号 8888 かつ パスを保持する URL 構造体のテスト。
    fn test_url_host_port_path() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse())
    }

    #[test]
    /// ポート番号 80 かつパスを保持する URL 構造体のテスト。
    fn test_url_host_path() {
        let url = "http://example.com/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "index.html".to_string(),
            searchpart: "".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    /// ポート8888　かつクエリパラメータを保持する URL 構造体のテスト。
    fn test_url_host_port_path_searchquery() {
        let url = "http://example.com:8888/index.html?a=123&b=xyz".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            searchpart: "a=123&b=xyz".to_string(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    /// HTTP スキーマのない URL　の場合のエラーテスト。
    fn test_no_scheme() {
        let url = "example.com".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    /// 対応していない FTP　スキーマの場合のエラーテスト。
    fn test_unsupported_schema() {
        let url = "ftp://example.com:8888/index.html".to_string();
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::new(url).parse());
    }
}