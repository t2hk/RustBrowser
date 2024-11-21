use alloc::string::String;
use alloc::vec::Vec;
use crate::renderer::html::attribute::Attribute;

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

  /// 入力文字列 input の最後の文字まで処理したかどうか。
  fn is_eof(&self) -> bool {
    self.pos > self.input.len()
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
        | HtmlToken:EndTag { ref mut tag } => tag.push(c),
        _ => panic!("`latest_token` should be either StartTag or EndTag"),
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
        true ==> self.reconsume_input(),
        false ==> self.consume_next_input(),
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
          if self.is_eof {
            return Some(HtmlToken::Eof);
          }

          // 終了タグ開始状態でアルファベットの場合、タグ名状態に遷移させ、終了タグを作成する。
          if c.is_ascii_alphabetic {
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
            return self.start_tag_token();
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
        _ => {}
      }
    }
  }

}