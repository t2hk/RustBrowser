use crate::renderer::html::attribute::Attribute;
use alloc::string::String;
use alloc::vec::Vec;


/// Tokenization で定義されている状態を表す列挙型。
/// 正規には 80 の状態があるが、ここでは 17 とする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// https://html.spec.whatwg.org/multipage/parsing.html#data-state
    Data,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    TagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    EndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    TagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    BeforeAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    AfterAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    BeforeAttributeValue,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueDoubleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueSingleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AttributeValueUnquoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    AfterAttributeValueQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    SelfClosingStartTag,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptData,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
    ScriptDataLessThanSign,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    ScriptDataEndTagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
    TemporaryBuffer,
}

/// HTML トークンの列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
  // 開始タグ
  StartTag {
    tag: String,
    self_closing: bool,
    attributes: Vec<Attribute>,
  },
  // 終了タグ
  EndTag {
    tag: String,
  },
  // 文字
  Char(char),
  // ファイルの終了 (End of file)
  Eof,
}

/// 字句解析に必要な情報を保持する HtmlTokenizer 構造体。
/// ステートマシンの状態 (State)、HTML の文字列 (input)、現在処理している文字の位置 (pos) などを管理する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer  {
  state: State,
  pos: usize,
  reconsume: bool,
  latest_token: Option<HtmlToken>,
  input: Vec<char>,
  buf: String,
}

impl HtmlTokenizer {
  /// 初期化
  pub fn new(html: String) -> Self {
    Self {
      state: State::Data,
      pos: 0,
      reconsume: false, // 状態の変更だけ行い、現在の文字を再利用するかどうか
      latest_token: None,
      input: html.chars().collect(),
      buf: String::new(),
    }
  }

  /// input 文字列から現在の位置 pos の文字を1文字返却する。
  /// 現在位置 pos をカウントアップする。
  fn consume_next_input(&mut self) -> char {
    let c = self.input[self.pos];
    self.pos += 1;
    c
  }

  /// input 文字列から現在位置 pos の1文字前を返却する。
  /// reconsume を解除する。
  fn reconsume_input(&mut self) -> char {
    self.reconsume = false;
    self.input[self.pos - 1]
  }

  /// 開始タグまたは終了タグを作成する。
  fn create_tag(&mut self, start_tag_token: bool) {
    if start_tag_token {
      self.latest_token = Some(HtmlToken::StartTag {
        tag: String::new(),
        self_closing: false,
        attributes: Vec::new(),
      });
    } else {
      self.latest_token = Some(HtmlToken::EndTag {
        tag: String::new()        
      });
    }
  }

  /// 最後のトークンの名前に1文字追加する。
  fn append_tag_name(&mut self, c: char) {
    assert!(self.latest_token.is_some());
    if let Some(t) = self.latest_token.as_mut() {
      match t {
        HtmlToken::StartTag {
          ref mut tag,
          self_closing: _,
          attributes: _,
        }
        | HtmlToken::EndTag { ref mut tag } => tag.push(c),
        _ => panic!("`latest_token` should be either StartTag or EndTag"),
      }
    }
  }

  /// 最後のトークンに属性を追加する。
  fn start_new_attribute(&mut self) {
    assert!(self.latest_token.is_some());

    // 属性を追加するタグが開始タグの場合、属性を新規追加する。
    if let Some(t) = self.latest_token.as_mut() {
      match t {
        HtmlToken::StartTag {
          tag: _,
          self_closing: _,
          ref mut attributes,          
        } => {
          attributes.push(Attribute::new());
        }
        _ => panic!("`latest_token` should be either StartTag"),
      }
    }
  }

  /// 最後のトークンに属性の文字を追加する。
  fn append_attribute(&mut self, c: char, is_name: bool) {
    assert!(self.latest_token.is_some());

    if let Some(t) = self.latest_token.as_mut() {
      match t {
        HtmlToken::StartTag {
          tag: _,
          self_closing: _,
          ref mut attributes,
        } => {
          let len = attributes.len();
          assert!(len > 0);
          attributes[len - 1].add_char(c, is_name);
        }
        _ => panic!("`latest_token` should be either StartTag"),
      }
    }
  }

  /// 最後のトークンが開始タグの場合、self__closing フラグを true にする。
  fn set_self_closing_flag(&mut self) {
    assert!(self.latest_token.is_some());

    if let Some(t) = self.latest_token.as_mut() {
      match t {
        HtmlToken::StartTag {
          tag: _,
          ref mut self_closing,
          attributes: _,
        } => *self_closing = true,
        _ => panic!("`latest_token` should be either StartTag"),
      }
    }
  }

    /// 最後のトークン(latest_token) を取得し、リセットする。
    fn take_latest_token(&mut self) -> Option<HtmlToken> {
      assert!(self.latest_token.is_some());
      let t = self.latest_token.as_ref().cloned();
      self.latest_token = None;
      assert!(self.latest_token.is_none());
  
      t
    }
    /// 入力文字列 input の最後の文字まで処理したかどうか。
    fn is_eof(&self) -> bool {
      self.pos > self.input.len()
    }
} 

/// HTML トークナイザ。
/// Iterator トレートを実装し、トークンを1つずつ処理する。
impl Iterator for HtmlTokenizer {
  type Item = HtmlToken;

  /// 入力文字列 input を1文字ずつ処理する。
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.input.len() {
      return None;
    }
    loop {
      // 次に処理する文字を取得する。
      // reconsume が true の場合、現在位置の1つ前の文字を読み込む。
      // false の場合、現在位置の文字を読み込む。
      let c = match self.reconsume {
        true => self.reconsume_input(),
        false => self.consume_next_input(),
      };

      // 読み込んだ文字とトークナイザーの状態から処理内容を振り分ける。
      match self.state {
        State::Data => {
          // タグの開始文字の場合、状態を TagOpen に設定し、次の文字の処理を継続する。
          if c == '<' {
            self.state = State::TagOpen;
            continue;
          }
          // 入力文字をすべて処理した場合、EOF を返す。
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          // 文字トークンを返す。
          return Some(HtmlToken::Char(c));
        }
        State::TagOpen => {
          // タグ開始状態で / が現れた場合、終了タグ開始状態に遷移する。
          if c == '/' {
            self.state = State::EndTagOpen;
            continue;
          }

          // タグ開始状態でアルファベットが現れた場合、タグ名状態に遷移させ、タグを作成する。
          if c.is_ascii_alphabetic() {
            self.reconsume = true;
            self.state = State::TagName;
            self.create_tag(true);
            continue;
          }

          // 入力文字列が最後に到達した場合、Eof トークンを返却する。
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }

          self.reconsume = true;
          self.state = State::Data;
        }
        State::EndTagOpen => {
          // 終了タグ開始状態で入力文字列が最後に到達した場合、Eof トークンを返却する。
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }

          // 終了タグ開始状態でアルファベットの場合、タグ名状態に遷移させ、終了タグを作成する。
          if c.is_ascii_alphabetic() {
            self.reconsume = true;
            self.state = State::TagName;
            self.create_tag(false);
            continue;
          }
        }
        State::TagName  => {
          // タグ名状態でスペースの場合、属性名開始前の状態に遷移させる。
          if c == ' ' {
            self.state = State::BeforeAttributeName;
            continue;
          }

          // タグ名状態で / の場合、現在の終了タグの開始状態に遷移させる。
          if c == '/' {
            self.state = State::SelfClosingStartTag;
            continue;
          }

          // タグ名状態で鵜 > の場合、データ状態に遷移させ、create_tag メソッドで作成した latest_token を返す。
          if c == '>' {
            self.state = State::Data;
            return self.take_latest_token();
          }

          // 次の文字がアルファベット大文字の場合、現在のタグの名前として追加する。
          if c.is_ascii_uppercase() {
            self.append_tag_name(c.to_ascii_lowercase());
            continue;
          }

          // 入力文字列の最後に到達した場合、Eof トークンを返す。
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }

          self.append_tag_name(c);
        }        
        State::BeforeAttributeName => {
          // タグ属性名の開始前状態の場合に、/ や > 終了時は属性名終了状態に遷移させる。
          if c == '/' || c == '>' || self.is_eof() {
            self.state = State::AfterAttributeName;
            self.reconsume = true;
            continue;            
          }
          // それ以外の場合、属性名状態に遷移させ、新たな属性を作成する。
          self.reconsume = true;
          self.state = State::AttributeName;
          self.start_new_attribute();
        }
        State::AttributeName => {
          // 属性名状態の場合に、スペース, / , > または文字列の最後の場合、ステータスを属性名終了状態に遷移させる。
          if c == ' ' || c == '/' || c == '>' || self.is_eof() {
            self.reconsume = true;
            self.state = State::AfterAttributeName;
            continue;
          }
          // = の場合、属性値前の状態に遷移させる。
          if c == '=' {
            self.state = State::BeforeAttributeValue;
            continue;
          }
          // アスキー文字の場合、属性名に一文字追加する。
          if c.is_ascii_uppercase() {
            self.append_attribute(c.to_ascii_lowercase(), /*is_name*/ true);
            continue;
          }
          self.append_attribute(c, /*is_name*/ true);
        }
        // タグの属性名の処理中の場合
        State::AfterAttributeName => {
          // スペースは無視する。
          if c == ' ' {
            continue;
          }
          if c == '/' { // 自己終了タグの場合
            self.state = State::SelfClosingStartTag;
            continue;
          }
          if c == '=' { // 属性の値の読み込み開始前
            self.state = State::BeforeAttributeValue;
            continue;
          }
          if c == '>' { // タグが終了した場合
            self.state = State::Data;
            return self.take_latest_token();
          }
          if self.is_eof() { // 文字の最後の場合
            return Some(HtmlToken::Eof);
          }
          self.reconsume = true;
          self.state = State::AttributeName;
          self.start_new_attribute();
        }
        // タグの属性値を処理する前の状態
        // ダブルクォートやシングルクォートが登場した場合、それぞれ該当するステータスに遷移させる。
        State::BeforeAttributeValue => {        
          if c == ' ' {
            continue;
          } // 空白は無視する
          if c == '"' { 
            self.state = State::AttributeValueDoubleQuoted;
            continue;
          }
          if c == '\'' {
            self.state = State::AttributeValueSingleQuoted;
            continue;
          }
          self.reconsume = true;
          self.state = State::AttributeValueUnquoted;
        }
        // ダブルクォートで囲まれた属性値を処理する状態
        State::AttributeValueDoubleQuoted => {
          if c == '"' { // ダブルクォートが登場した場合、属性値の終了状態に遷移する。
            self.state = State::AfterAttributeValueQuoted;
            continue;
          }
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          self.append_attribute(c, /*is_name*/ false);
        }
        // シングルクォートで囲まれた属性値を処理する状態
        State::AttributeValueSingleQuoted => {
          if c == '\'' { // シングルクォートが登場した場合、属性値の終了状態に遷移する。
            self.state = State::AfterAttributeValueQuoted;
            continue;
          }
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          self.append_attribute(c, /*is_name*/ false);
        }
        // シングルクォートで囲まれたタグの属性値を処理する。
        State::AttributeValueUnquoted => {
          if c == ' ' {
            self.state = State::BeforeAttributeName;
            continue;
          }

          if c == '>' {
            self.state = State::Data;
            return self.take_latest_token();
          }

          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          self.append_attribute(c, /*is_name*/ false);
        }


        // 属性値を処理した後の状態の場合
        State::AfterAttributeValueQuoted => {
          if c == ' ' { // スペースの場合、次の属性名の開始前に遷移する。
            self.state = State::BeforeAttributeName;
            continue;
          }
          if c == '/' { //自己終了タグの場合
            self.state = State::SelfClosingStartTag;
            continue;
          }
          if c == '>' { // タグの終了の場合、データ処理状態に遷移し、トークンを返却する。
            self.state = State::Data;
            return self.take_latest_token();
          }
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          self.reconsume = true;
          self.state = State::BeforeAttributeValue;
        }
        // 自己終了タグを処理する状態の場合
        State::SelfClosingStartTag => {
          if c == '>' {  // タグの終了の場合、データ状態に遷移する。
            self.set_self_closing_flag();
            self.state = State::Data;
            return self.take_latest_token();
          }
          if self.is_eof() {
            // invalid parser error.
            return Some(HtmlToken::Eof);
          }
        }

        // <script> タグに記述されている Javascript を処理する状態
        State::ScriptData => {
          if c == '<' { // 文字が < の場合、ScriptDataLessThanSign 状態に遷移させる。この状態では、< がただの文字なのか、次以降に /script> が来る終了タグの一部なのか判断することになる。
            self.state = State::ScriptDataLessThanSign;
            continue;
          }
          if self.is_eof() {
            return Some(HtmlToken::Eof);
          }
          return Some(HtmlToken::Char(c));
        }

        // スクリプトデータ処理中に < 文字が現れた場合の処理。</script> の終了タグなのかどうか判断する。
        State::ScriptDataLessThanSign => {
          if c == '/' {
            self.buf = String::new(); // 一時的なバッファを用意する。
            self.state = State::ScriptDataEndTagOpen;
            continue;
          }
          self.reconsume = true;
          self.state = State::ScriptData;
          return Some(HtmlToken::Char('<'));
        }
        // Javascript の終了タグの処理を開始する前の状態
        State::ScriptDataEndTagOpen => {
          if c.is_ascii_alphabetic() {
            self.reconsume = true;
            self.state = State::ScriptDataEndTagName;
            self.create_tag(false);
            continue;
          }

          self.reconsume = true;
          self.state = State::ScriptData;
          return Some(HtmlToken::Char('<'));
        }
        // Javascript の終了タグのタグ名部分を処理する状態
        State::ScriptDataEndTagName => {
          if c == '>' { // スクリプトの終了タグが閉じられた場合、データ状態に遷移させ、最後のトークンを返却する。
            self.state = State::Data;
            return self.take_latest_token();
          }
          if c.is_ascii_alphabetic() { // アルファベットの場合、一時的なバッファ buf に文字を追加し、append_tag_name で文字をトークンに追加する。
            self.buf.push(c);
            self.append_tag_name(c.to_ascii_lowercase());
            continue;
          }
          self.state = State::TemporaryBuffer;
          self.buf = String::from("</") + &self.buf;
          self.buf.push(c);
          continue;
        }
        // 一次的なバッファの管理
        State::TemporaryBuffer => {
          self.reconsume = true;
          if self.buf.chars().count() == 0 {
            self.state = State::ScriptData;
            continue;
          }
          // 最初の一文字を削除する。
          let c = self
          .buf
          .chars()
          .nth(0)
          .expect("self.buf should have at least 1 char");
          self.buf.remove(0);
          return Some(HtmlToken::Char(c));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::alloc::string::ToString;
  use alloc::vec;

  /// 空文字の場合のテスト。
  #[test]
  fn test_empty() {
    let html = "".to_string();
    let mut tokenizer = HtmlTokenizer::new(html);
    assert!(tokenizer.next().is_none());
  }

  /// 開始タグと終了タグのテスト
  #[test]
  fn test_start_and_end_tag() {
    let html = "<body></body>".to_string();
    let mut tokenizer = HtmlTokenizer::new(html);
    let expected = [
      HtmlToken::StartTag {
        tag: "body".to_string(),
        self_closing: false,
        attributes: Vec::new(),
      },
      HtmlToken::EndTag {
        tag: "body".to_string(),
      },
    ];
    for e in expected {
      assert_eq!(Some(e), tokenizer.next());
    }
  }

  /// 属性のテスト
  #[test]
  fn test_attributes() {
    let html = "<p class=\"A\" id='B' foo=bar></p>".to_string();
    let mut tokenizer = HtmlTokenizer::new(html);
    let mut attr1 = Attribute::new();
    attr1.add_char('c', true);
    attr1.add_char('l', true);
    attr1.add_char('a', true);
    attr1.add_char('s', true);
    attr1.add_char('s', true);
    attr1.add_char('A', false);

    let mut attr2 = Attribute::new();
    attr2.add_char('i', true);
    attr2.add_char('d', true);
    attr2.add_char('B', false);

    let mut attr3 = Attribute::new();
    attr3.add_char('f', true);
    attr3.add_char('o', true);
    attr3.add_char('o', true);
    attr3.add_char('b', false);
    attr3.add_char('a', false);
    attr3.add_char('r', false);

    let expected = [
      HtmlToken::StartTag {
        tag: "p".to_string(),
        self_closing: false,
        attributes: vec![attr1, attr2, attr3],
      },
      HtmlToken::EndTag {
        tag: "p".to_string(),
      },
    ];
    for e in expected {
      assert_eq!(Some(e), tokenizer.next());
    }
  }

  // 空要素のテスト
  #[test]
  fn test_self_closing_tag() {
    let html = "<img />".to_string();
    let mut tokenizer = HtmlTokenizer::new(html);

    let expected = [HtmlToken::StartTag {
      tag: "img".to_string(),
      self_closing: true,
      attributes: Vec::new(),
    }];
    for e in expected {
      assert_eq!(Some(e), tokenizer.next());
    }
  }

  // スクリプトタグのテスト
  #[test]
  fn test_script_tag() {
    let html = "<script>js code;</script>".to_string();
    let mut tokenizer = HtmlTokenizer::new(html);
    let expected = [
      HtmlToken::StartTag {
        tag: "script".to_string(),
        self_closing: false,
        attributes: Vec::new(),
      },
      HtmlToken::Char('j'),
      HtmlToken::Char('s'),
      HtmlToken::Char(' '),
      HtmlToken::Char('c'),
      HtmlToken::Char('o'),
      HtmlToken::Char('d'),
      HtmlToken::Char('e'),
      HtmlToken::Char(';'),
      HtmlToken::EndTag {
        tag: "script".to_string(),
      },
    ];
    for e in expected {
      assert_eq!(Some(e), tokenizer.next());
    }
  }
}