use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// トークン列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// 記号を表す。
    /// https://262.ecma-international.org/#sec-punctuators
    Punctuator(char),
    /// 数字を表す。
    /// https://262.ecma-international.org/#sec-literals-numeric-literals
    Number(u64),
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

        let c = self.input[self.pos];

        let token = match c {
            // 記号トークン
            '+' | '-' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                t
            }
            // 数字トークン
            '0'..='9' => Token::Number(self.consume_number()),
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
}
