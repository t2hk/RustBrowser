use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// 予約語
static RESERVED_WEORDS: [&str; 3] = ["var", "function", "return"];

/// トークン列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// 記号を表す。
    /// https://262.ecma-international.org/#sec-punctuators
    Punctuator(char),
    /// 数字を表す。
    /// https://262.ecma-international.org/#sec-literals-numeric-literals
    Number(u64),
    /// 変数を表す。
    /// https://262.ecma-international.org/#sec-identifier-names
    Identifier(String),
    /// 予約語を表す。
    /// https://262.ecma-international.org/#sec-keywords-and-reserved-words
    Keyword(String),
    /// 文字列を表す。
    /// https://262.ecma-international.org/#sec-literals-string-literals
    StringLiteral(String),
}

/// JsLexer 構造体
/// 読み込んだ位置を保持する pos, 入力文字列を保持する input を持つ。
pub struct JsLexer {
    pos: usize,
    input: Vec<char>,
}

/// 字句解析を行うレキサーである。
impl JsLexer {
    pub fn new(js: String) -> Self {
        Self {
            pos: 0,
            input: js.chars().collect(),
        }
    }

    /// 数字を読み込む。
    /// 0 から 9 が出続けている間、文字を消費し、数字として解釈する。
    fn consume_number(&mut self) -> u64 {
        let mut num = 0;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];
            match c {
                '0'..='9' => {
                    num = num * 10 + (c.to_digit(10).unwrap() as u64);
                    self.pos += 1;
                }
                _ => break,
            }
        }
        return num;
    }

    /// 現在の位置 (pos) から始まる文字が予約語と一致する場合、true を返す。
    fn contains(&self, keyword: &str) -> bool {
        for i in 0..keyword.len() {
            if keyword
                .chars()
                .nth(i)
                .expect("failed to access to i-th char")
                != self.input[self.pos + i]
            {
                return false;
            }
        }
        true
    }

    /// 予約語の場合、Token::Keyword トークンを返す。
    fn check_reserved_word(&self) -> Option<String> {
        for word in RESERVED_WEORDS {
            if self.contains(word) {
                return Some(word.to_string());
            }
        }
        None
    }

    /// 数字でも記号でもなく、かつ、変数として受け入れ可能な文字列で始まった場合、変数が終了するまで入力の文字列を進める。
    fn consume_identifier(&mut self) -> String {
        let mut result = String::new();

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            if self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == '$' {
                result.push(self.input[self.pos]);
                self.pos += 1;
            } else {
                return result;
            }
        }
    }

    /// ダブルクォートの場合、文字列として解釈する。
    fn consume_string(&mut self) -> String {
        let mut result = String::new();
        self.pos += 1;

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            if self.input[self.pos] == '"' {
                self.pos += 1;
                return result;
            }

            result.push(self.input[self.pos]);
            self.pos += 1;
        }
    }
}

impl Iterator for JsLexer {
    type Item = Token;

    /// 次のトークンを返す。
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        // ホワイトスペースまたは改行文字が続く限り、次の位置に進める。
        while self.input[self.pos] == ' ' || self.input[self.pos] == '\n' {
            self.pos += 1;

            if self.pos >= self.input.len() {
                return None;
            }
        }

        // 予約語の場合、Keyword トークンを返す。
        if let Some(keyword) = self.check_reserved_word() {
            self.pos += keyword.len();
            let token = Some(Token::Keyword(keyword));
            return token;
        }

        let c = self.input[self.pos];

        let token = match c {
            // 記号トークン
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                t
            }
            // 数字トークン
            '0'..='9' => Token::Number(self.consume_number()),
            // 変数として受け入れ可能な文字
            'a'..='z' | 'A'..='Z' | '_' | '$' => Token::Identifier(self.consume_identifier()),
            // 文字列の場合
            '"' => Token::StringLiteral(self.consume_string()),

            _ => unimplemented!("char {:?} is not supported yet", c),
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 空文字のテスト
    #[test]
    fn test_empty() {
        let input = "".to_string();
        let mut lexer = JsLexer::new(input).peekable();
        assert!(lexer.peek().is_none());
    }

    /// 1つの数字トークンのみのテスト
    #[test]
    fn test_num() {
        let input = "42".to_string();
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [Token::Number(42)].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
        assert_eq!(1, i);
    }

    /// 足し算のテスト
    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [Token::Number(1), Token::Punctuator('+'), Token::Number(2)].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    /// 変数の定義のテスト
    #[test]
    fn test_assign_variable() {
        let input = "var foo=\"bar\";".to_string();
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [
            Token::Keyword("var".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('='),
            Token::StringLiteral("bar".to_string()),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;

        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    /// 複雑な文のトークン化
    #[test]
    fn test_add_local_variable_and_num() {
        let input = "function foo() { var a=42; return a; } var result = foo() + 1;".to_string();
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [
            Token::Keyword("function".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('{'),
            Token::Keyword("var".to_string()),
            Token::Identifier("a".to_string()),
            Token::Punctuator('='),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Keyword("return".to_string()),
            Token::Identifier("a".to_string()),
            Token::Punctuator(';'),
            Token::Punctuator('}'),
            Token::Keyword("var".to_string()),
            Token::Identifier("result".to_string()),
            Token::Punctuator('='),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('+'),
            Token::Number(1),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }
}
