use alloc::string::String;
use alloc::vec::Vec;

/// CSS のトークンの種類を表す列挙型。
#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    /// https://www.w3.org/TR/css-syntax-3/#typedef-hash-token
    HashToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-delim-token
    Delim(char),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-number-token
    Number(f64),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-colon-token
    Colon,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-semicolon-token
    SemiColon,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-paren
    OpenParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-paren
    CloseParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-curly
    OpenCurly,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-curly
    CloseCurly,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-ident-token
    Ident(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-string-token
    StringToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-at-keyword-token
    AtKeyword(String),
}

/// トークナイザーを行う構造体。
#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>,
}

impl CssTokenizer {
    pub fn new(css: String) -> Self {
        Self {
            pos: 0,
            input: css.chars().collect(),
        }
    }

    /// 再びダブルクォートかシングルクォートが現れるまで、入力を文字として解釈する。
    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();

        loop {
            if self.pos >= self.input.len() {
                return s;
            }

            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                '"' | '\'' => break,
                _ => s.push(c),
            }
        }

        s
    }

    /// 数字またはピリオドがで続けている間、数字として解釈する。
    /// それ以外の文字が入力された場合、数字を返す。
    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            match c {
                '0'..='9' => {
                    if floating {
                        floating_digit *= 1f64 / 10f64;
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64);
                    }
                    self.pos += 1;
                }
                '.' => {
                    floating = true;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        num
    }

    /// 文字、数字、ハイフン、アンダースコアが出続ける間、識別子として扱う。
    /// それ以外の文字が登場した場合、今までの文字を返す。
    /// https://www.w3.org/TR/css-syntax-3/#consume-ident-like-token
    /// https://www.w3.org/TR/css-syntax-3/#consume-name
    fn consume_ident_token(&mut self) -> String {
        let mut s = String::new();
        s.push(self.input[self.pos]);

        loop {
            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    s.push(c);
                }
                _ => break,
            }
        }
        s
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    /// 次のトークンを返す。
    /// 入力の CSS 文字列を1文字ずつ参照し、現在の文字によって振る舞いを変える。
    /// https://www.w3c.org/TR/css-syntax-3/#consume-token
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input[self.pos];
            let token = match c {
                // 次のトークンを決定する。
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::SemiColon,
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                // ダブルクォートとシングルクォートが現れたら文字列トークンを返却する。
                '"' | '\'' => {
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                }
                '0'..='9' => {
                    let t = CssToken::Number(self.consume_numeric_token());
                    self.pos -= 1;
                    t
                }
                // ハッシュタグの場合、常に #ID 形式の ID セレクタとして扱う。
                '#' => {
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::HashToken(value)
                }
                // ハイフンの場合、常に識別子の1つとして扱う(負の値は扱わないこととする)。
                '-' => {
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                // アットマークの場合、次の3文字が識別子として有効な文字の場合、<at-keyword-token> トークンを返す。
                // それ以外の場合、<delim-token> を返す。
                '@' => {
                    if self.input[self.pos + 1].is_ascii_alphabetic()
                        && self.input[self.pos + 2].is_alphanumeric()
                        && self.input[self.pos + 3].is_alphanumeric()
                    {
                        // '@' をスキップする。
                        self.pos += 1;
                        let t = CssToken::AtKeyword(self.consume_ident_token());
                        self.pos -= 1;
                        t
                    } else {
                        CssToken::Delim('@')
                    }
                }
                // 小文字、大文字、アンダースコアの場合、識別子トークン(Ident) を作成して返す。
                'a'..='z' | 'A'..='Z' | '_' => {
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                _ => {
                    unimplemented!("char {} is not supported yet", c);
                }
            };

            self.pos += 1;
            return Some(token);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    /// 何もない文字列の場合、トークナイザーの next が None を返すことを確認する。
    fn test_empty() {
        let style = "".to_string();
        let mut t = CssTokenizer::new(style);
        assert!(t.next().is_none());
    }

    #[test]
    /// 1つのルールだけ存在する場合のテスト
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    /// クラスセレクタを持つルールが1つ存在する場合のテスト。
    fn test_class_selector() {
        let style = ".class { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Delim('.'),
            CssToken::Ident("class".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }

        assert!(t.next().is_none());
    }

    #[test]
    /// 複数のルールが存在する場合のテスト。
    fn test_multiple_rules() {
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("content".to_string()),
            CssToken::Colon,
            CssToken::StringToken("Hey".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
            CssToken::Ident("h1".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("font-size".to_string()),
            CssToken::Colon,
            CssToken::Number(40.0),
            CssToken::SemiColon,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("blue".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];

        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }
}
