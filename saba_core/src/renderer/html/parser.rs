use core::str::FromStr;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::Window;
use crate::renderer::html::token::HtmlToken;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

/// https://html.spec.whatwg.org/multipage/parsing.html#the-insertion-mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InsertionMode {
  Initial,
  BeforeHtml,
  BeforeHead,
  InHead,
  AfterHead,
  InBody,
  Text,
  AfterBody,
  AfterAfterBody,
}

#[derive(Debug, Clone)]
pub struct HtmlParser {
  /// DOM ツリーのルートノードを持つ Window オブジェクトを格納するフィールド。
  window: Rc<RefCell<Window>>,
  
  /// 状態遷移で使用する現在の状態を表す。
  mode: InsertionMode,
  
  /// ある状態に遷移したときに、以前の挿入モードを保存するために使用するフィールド。
  /// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
  original_insertion_mode: InsertionMode,
  
  /// HTML の構文解析中にブラウザが使用するスタックである。スタックはデータ構造の1つであり、最初に追加した要素が最後に取り出される(first-in-last-out)。
  /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
  stack_of_open_elements: Vec<Rc<RefCell<Node>>>,

  /// HtmlTokenizer の構造体を格納している。次のトークンは t.next メソッドで取得できる。
  t: HtmlTokenizer,
}

impl HtmlParser {
  /// HTML パーサーを作成する。
  pub fn new(t: HtmlTokenizer) -> Self {
    Self {
      window: Rc::new(RefCell::new(Window::new())),
      mode: InsertionMode::Initial,
      original_insertion_mode: InsertionMode::Initial,
      stack_of_open_elements: Vec::new(),
      t,
    }
  }

  /// DOM ツリーを構築する。
  pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
    let mut token = self.t.next();
  
    while token.is_some() {
      match self.mode {
        // Initial 状態
        InsertionMode::Initial => {
          // DOCTYPE トークンをサポートしていないため、<!doctype html> のようなトークンは文字トークンとして表す。
          // 文字トークンは無視する。
          if let Some(HtmlToken::Char(_)) = token {
            token = self.t.next();
            continue;
          }

          self.mode = InsertionMode::BeforeHtml;
          continue;
        }
        // BeforeHtml 状態
        // 主に <html> の開始タグを扱う。
        InsertionMode::BeforeHtml => {
          match token {
            // 次のトークンがスペースや改行の場合、それを無視して次のトークンに移動する。
            Some(HtmlToken::Char(c)) => {
              if c == ' ' || c == '\n' {
                token = self.t.next();
                continue;
              }
            }
            // 次のトークンが HtmlToken::StartTag でタグ名が <html> の場合、DOM ツリーに新しいノードを追加する。
            Some(HtmlToken::StartTag {
              ref tag,
              self_closing: _,
              ref attributes,
            }) => {
              if tag == "html" {
                self.insert_element(tag, attributes.to_vec());
                self.mode = InsertionMode::BeforeHead;
                token = self.t.next();
                continue;
              }
            }
            // トークンの終了を表す EOF トークンの場合、それまで構築したツリーを返す。
            Some(HtmlToken::Eof) | None => {
              return self.window.clone();
            }
            _ => {}
          }
          // 上記以外の場合、自動的に HTML 要素を DOM ツリーに追加する。HTML タグを省略している場合もパースできる。
          self.insert_element("html", Vec::new());
          self.mode = InsertionMode::BeforeHead;
          continue;
        }
        // BeforeHead 状態
        // <head> の開始タグを扱う。
        InsertionMode::BeforeHead => {
          match token {
            // 次のトークンが空白文字や改行文字の場合、無視する。
            Some(HtmlToken::Char(c)) => {
              if c == ' ' || c == '\n' {
                token = self.t.next();
                continue;
              }
            }
            // 次のトークンが HtmlTOken::StartTag でタグの名前が head の場合、DOM ツリーに新しいノードを追加し、InHead 状態に遷移する。
            Some(HtmlToken::StartTag {
              ref tag,
              self_closing: _,
              ref attributes,              
            }) => {
              if tag == "head" {
                self.insert_element(tag, attributes.to_vec());
                self.mode = InsertionMode::InHead;
                token = self.t.next();
                continue;
              }
            }
            Some(HtmlToken::Eof) | None => {
              return self.window.clone();
            }
            _ => {}
          }
          // 上記以外の場合、自動的に HEAD 要素を DOM ツリーに追加する。
          // Head タグを省略している場合も正しくパースできる。
          self.insert_element("head", Vec::new());
          self.mode = InsertionMode::InHead;
          continue;
        }
        // InHead 状態
        // head 終了タグ, style 開始タグ, script 開始タグを扱う。
        InsertionMode::InHead => {
          match token {
            // 次のトークンはスペースや改行の場合、無視して次のトークンに移る。
            Some(HtmlToken::Char(c)) => {
              if c == ' ' || c == '\n' {
                self.insert_char(c);
                token = self.t.next();
                continue;
              }
            }
            // HtmlToken::StartTag でタグの名前が style や script の場合、DOM ツリーに新しいノードを追加し、Text 状態に遷移する。
            Some(HtmlToken::StartTag {
              ref tag,
              self_closing: _,
              ref attributes,
            }) => {
              if tag == "style" || tag == "script" {
                self.insert_element(tag, attributes.to_vec());
                self.original_insertion_mode = self.mode;
                self.mode = InsertionMode::Text;
                token = self.t.next();
                continue;
              }

              // head が省略されている HTML 文書を扱う絵で必要な処理。
              if tag == "body" {
                self.pop_until(ElementKind::Head);
                self.mode = InsertionMode::AfterHead;
                continue;
              }
              if let Ok(_element_kind) = ElementKind::from_str(tag) {
                self.pop_until(ElementKind::Head);
                self.mode = InsertionMode::AfterHead;
                continue;
              }
            }
            // head の終了タグの場合、スタックに保存されているノードを取得する(pop_until メソッド)。
            // 次の状態である AfterHead に遷移する。
            Some(HtmlToken::EndTag { ref tag }) => {
              if tag == "head" {
                self.mode = InsertionMode::AfterHead;
                token = self.t.next();
                self.opo_until(ElementKind::Head);
                continue;
              }
            }
            Some(HtmlToken::Eof) | None => {
              return self.window.clone();
            }
          }
          // <meta> や <title> などのサポートしていないタグは無視する。
          token = self.t.next();
          continue;
        }
        // AfterHead 状態
        // 主に Body 開始タグを扱う。
        InsertionMode::AfterHead => {
          match token {
            // 次のトークンが空白や改行の場合、無視捨て次のトークンに遷移する。
            Some(HtmlToken::Char(c)) => {
              if c == ' ' || c == '\n' {
                self.insert_char(c);
                token = self.t.next();
                continue;
              }
            }
            // 次のトークンが Body の開始タグの場合、DOM ツリーに新しいノードを追加し、InBody 状態に遷移する。
            Some(HtmlToken::StartTag {
              ref tag,
              self_closing: _,
              ref attributes,
            }) => {
              if tag == "body" {
                self.insert_element(tag, attributes.to_vec());
                token = self.t.next();
                self.mode = InsertionMode::InBody;
                continue;
              }
            }
            Some(HtmlToken::Eof) | None => {
              return self.window.clone();
            }
            _ => {}
          }
          // 上記以外の場合、自動的に body 要素を DOM ツリーに追加する。
          // これにより、body タグを省略している場合でもパースできる。
          self.insert_element("body", Vec::new());
          self.mode = InsertionMode::InBody;
          continue;  
        }
        InsertionMode::InBody => {}
        InsertionMode::Text => {}
        InsertionMode::AfterBody => {}
        InsertionMode::AfterAfterBody => {}
      }
    }

    self.window.clone()
  }  
}