use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::dom::node::Window;
use crate::renderer::html::attribute::Attribute;
use crate::renderer::html::token::HtmlToken;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

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

    /// 要素ノードを作成する。
    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    /// HTML の構造を解析して要素ノードを正しい位置に挿入する。
    /// 指定されたタグと属性を持つ要素ノードw作成し、挿入先の位置を決定する。
    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let window = self.window.borrow();

        // 現在の開いている要素スタック (stack_of_open_elements) の最後のノードを取得する。
        // スタックが空の場合はルート要素が現在参照しているノードになる。
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };
        // 新しい要素ノードを作成する。
        // 変更可能な査証カウンタである Rc<RefCell<Node>> 形式とする。
        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        // 現在参照しているノードに子要素が存在する場合、最後の兄弟ノードを探索し、新しいノードをその直後に挿入する。
        if current.borrow().first_child().is_some() {
            let mut last_sibling = current.borrow().first_child();
            loop {
                last_sibling = match last_sibling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
                        } else {
                            break;
                        }
                    }
                    None => unimplemented!("last_sibling should be Some"),
                };
            }

            // 新しいノードを最後の兄弟ノードの直後に挿入する。
            last_sibling
                .unwrap()
                .borrow_mut()
                .set_next_sibling(Some(node.clone()));
            // 最後の兄弟ノードを新しいノードの直前の兄弟ノードとして設定する。
            node.borrow_mut().set_previous_sibling(Rc::downgrade(
                &current
                    .borrow()
                    .first_child()
                    .expect("failed to get a first child"),
            ))
        } else {
            // 現在参照しているノードに兄弟ノードが存在しな場合、現在参照しているノードの最初の子ノードとして新しいノードを設定する。
            current.borrow_mut().set_first_child(Some(node.clone()));
        }

        // 挿入の完了後、親子関係と兄弟関係のリンクを適切に設定する。
        current.borrow_mut().set_last_child(Rc::downgrade(&node)); // 現在のノードの最後の子ノードを新しいノードに設定する。
        node.borrow_mut().set_parent(Rc::downgrade(&current)); // 新しいノードの親を現在参照しているノードに設定する。

        // 新しいノードを開いている要素スタックに追加する。
        self.stack_of_open_elements.push(node);
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
                // InBody 状態の場合に <body> タグのコンテンツを処理する。
                // 具体的には <div>, <h1>, <p> のようなタグである。
                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::EndTag { ref tag }) => {
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    token = self.t.next();
                                    // パースに失敗した場合、トークンを無視する。
                                    if !self.contain.in_stack(ElementKind::Body) {
                                        continue;
                                    }
                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.t.next();
                                    }
                                    continue;
                                }
                                _ => {
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                }
                // Text 状態は <style> や <script> タグが開始した後の状態である。
                // 終了タグが現れるまで文字をテキストノードとして DOM ツリーに追加する。
                // 終了タグが現れたら元の状態の "original_insertion_mode" に戻る。
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        _ => {}
                    }
                    self.mode = self.original_insertion_mode;
                }
                // AfterBody 状態の場合、主に <html> 終了タグを扱う。
                // 次のトークンが文字トークンの場合、無視して次のトークンを処理する。
                // 次のトークンが HtmlToken::EndTag でタグの名前が <html> の場合、AfterAfterBody 状態に遷移する。
                InsertionMode::AfterBody => {
                    match token {
                        // 次が文字トークンの場合、無視して次のトークンを処理する。
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                }
                // AfterAfterBody 状態の場合、トークンが終了することを確認してパースを終了する。
                // 次のトークンが文字トークンの場合、無視して次のトークンに移動する。
                // 次のトークンが Eof か存在しない場合、トークン列をすべて消費したことを表し、構築した DOM ツリーを返却する。
                // それ以外の場合はパースエラーだが、ブラウザは間違った HTML でもできる限り解釈しようとするため、
                // すぐにエラーとせずに、InBody 状態に遷移して再度トークンの処理を試みる。
                InsertionMode::AfterAfterBody => {
                    match token {
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // パースに失敗した場合、InBody 状態に遷移する。
                    self.mode = InsertionMode::InBody;
                }
            }
        }

        self.window.clone()
    }
}
