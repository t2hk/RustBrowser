use crate::alloc::string::ToString;
use crate::renderer::css::token::CssToken;
use crate::renderer::css::token::CssTokenizer;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::Peekable;

/// CSS のプロパティの値に対するノードを表す ComponentValue 構造体。
pub type ComponentValue = CssToken;

/// CSS の構文解析を行い、CSS オブジェクトモデル(CSSOM) を作成するための構造体。
/// CSSTokenizer 構造体を Peekable でラップして保持する。
#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
    }

    /// トークン列から CSSOM を構築する。
    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        // StyleSheet 構造体のインスタンスを作成する。
        let mut sheet = StyleSheet::new();
        // トークン列からルールのリストを作成し、StyleSheet のフィールドに設定する。
        sheet.set_rules(self.consume_list_of_rules());
        sheet
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        // 空のベクタを作成する。
        let mut rules = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return rules,
            };
            match token {
                // AtKeyword トークンが出てきた場合、他の CSS をインポートする @import, メディアクエリを表す @media などのルールが始まることを表す。
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_qualified_rule();
                    // しかし、このブラウザは @ から始まるルールはサポートしないので無視する。
                }
                _ => {
                    // AtKeyword トークン以外の場合、1つのルールを解釈し、ベクタに追加する。
                    // 1つのルールを解釈し、ベクタに追加する。
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    /// 1つのルールを解釈する。
    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                // 次のトークンが開き波括弧 ({) の場合、宣言ブロックの解釈を行う。
                CssToken::OpenCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                // 開き波括弧以外の場合、ルールのセレクタとして扱う。
                _ => {
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    /// セレクタを解釈する。
    fn consume_selector(&mut self) -> Selector {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            // 次のトークンがハッシュトークンの場合、ID セレクタを作成して返す。
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),

            // 次のトークンがピリオドの場合、クラスセレクタを作成して返す。
            CssToken::Delim(delim) => {
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            }

            // 次のトークンが識別子の場合、タイプセレクタを作成して返す。
            // ただし、a:hovert のようなセレクタは正しく解釈せず、タイプセレクタとして扱う。
            // コロンが出てきた場合、宣言ブロックの開始直前までトークンを無視する。
            CssToken::Ident(ident) => {
                if self.t.peek() == Some(&CssToken::Colon) {
                    while self.t.peek() != Some(&CssToken::OpenCurly) {
                        self.t.next();
                    }
                }
                Selector::TypeSelector(ident.to_string())
            }

            // アットキーワード @ の場合、宣言ブロックの開始直前までトークンを無視する。
            // 他の CSS をインポートする @import やメディアクエリを表す @media はサポートしない。
            CssToken::AtKeyword(_keyword) => {
                while self.t.peek() != Some(&CssToken::OpenCurly) {
                    self.t.next();
                }
                Selector::UnknownSelector
            }
            _ => {
                self.t.next();
                Selector::UnknownSelector
            }
        }
    }

    /// 複数の宣言を解釈する。
    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return declarations,
            };

            match token {
                // 閉じ波括弧の場合、今まで作成した宣言のベクタを返す。
                CssToken::CloseCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    return declarations;
                }

                // セミコロンの場合、1つの宣言が終了したことを表す。セミコロンのトークンを消費し、何もしない。
                CssToken::SemiColon => {
                    assert_eq!(self.t.next(), Some(CssToken::SemiColon));
                    // 1つの宣言が終了。何もしない。
                }

                // 識別子トークンの場合、1つの宣言を解釈し、ベクタに追加する。
                CssToken::Ident(ref _ident) => {
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                // 上記以外の場合、無視して次のトークンを処理する。
                _ => {
                    self.t.next();
                }
            }
        }
    }

    /// 1つの宣言を解釈する。
    fn consume_declaration(&mut self) -> Option<Declaration> {
        if self.t.peek().is_none() {
            return None;
        }

        // Declaration を初期化し、この構造体のプロパティに識別子を設定する。
        let mut declaration = Declaration::new();
        declaration.set_property(self.consume_ident());

        // もし次のトークンがコロンでない場合、パースエラーなので None を返す。
        match self.t.next() {
            Some(token) => match token {
                CssToken::Colon => {}
                _ => return None,
            },
            None => return None,
        }

        // Declaration 構造体の値にコンポーネント値を設定する。
        declaration.set_value(self.consume_component_value());

        Some(declaration)
    }

    /// 識別子トークンを消費し、文字列を取得する。
    fn consume_ident(&mut self) -> String {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::Ident(ref ident) => ident.to_string(),
            _ => {
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
        }
    }

    /// コンポーネント値を解釈する。
    /// https://www.w3.org/TR/css-syntax-3/#consume-component-value
    fn consume_component_value(&mut self) -> ComponentValue {
        self.t
            .next()
            .expect("should have a token in consume_component_value")
    }
}

/// CSSOM のルートノードである StyleSheet の構造体。
/// 複数のルールをベクタで保持する。
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    /// https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

/// ルールノード (QualifiedRule) 構造体。
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    /// セレクタを表す Selector 列挙体。
    /// https://www.w3.org/TR/selectors-4/#typedef-selector-list
    pub selector: Selector,

    /// 宣言を表す Declaration ベクタ。
    /// https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            selector: Selector::TypeSelector("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>) {
        self.declarations = declarations;
    }
}

/// セレクタノード
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// https://www.w3.org/TR/selectors-4/#type-selectors
    TypeSelector(String),
    /// https://www.w3.org/TR/selectors-4/#class-html
    ClassSelector(String),
    /// https://www.w3.org/TR/selectors-4/#id-selectors
    IdSelector(String),
    /// パース中にエラーが発生した場合に使用するセレクタ
    UnknownSelector,
}

/// 宣言ノード
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

impl Declaration {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue) {
        self.value = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// 空文字のテスト。何もない文字列が入力だった場合のケースについてのテスト。
    #[test]
    fn test_empty() {
        let style = "".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        assert_eq!(cssom.rules.len(), 0);
    }

    /// 1つのルールだけが存在する場合のテスト。
    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    /// ID セレクタのテスト。
    #[test]
    fn test_id_selector() {
        let style = "#id { color: red; }".to_string();

        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::IdSelector("id".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    /// クラスセレクタのテスト。
    #[test]
    fn test_class_selector() {
        let style = ".class { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::ClassSelector("class".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }

    /// 複数のルールのテスト。
    #[test]
    fn tset_multiple_rules() {
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule1 = QualifiedRule::new();
        rule1.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration1 = Declaration::new();
        declaration1.set_property("content".to_string());
        declaration1.set_value(ComponentValue::StringToken("Hey".to_string()));
        rule1.set_declarations(vec![declaration1]);

        let mut rule2 = QualifiedRule::new();
        rule2.set_selector(Selector::TypeSelector("h1".to_string()));
        let mut declaration2 = Declaration::new();
        declaration2.set_property("font-size".to_string());
        declaration2.set_value(ComponentValue::Number(40.0));
        let mut declaration3 = Declaration::new();
        declaration3.set_property("color".to_string());
        declaration3.set_value(ComponentValue::Ident("blue".to_string()));
        rule2.set_declarations(vec![declaration2, declaration3]);

        let expected = [rule1, rule2];
        assert_eq!(cssom.rules.len(), expected.len());

        let mut i = 0;
        for rule in &cssom.rules {
            assert_eq!(&expected[i], rule);
            i += 1;
        }
    }
}
