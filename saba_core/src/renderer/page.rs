// use crate::alloc::string::ToString;
use crate::browser::Browser;
use crate::display_item::DisplayItem;
use crate::http::HttpResponse;
use crate::renderer::css::cssom::CssParser;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::css::token::CssTokenizer;
use crate::renderer::dom::api::get_style_content;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::dom::node::Window;
use crate::renderer::html::parser::HtmlParser;
use crate::renderer::html::token::HtmlTokenizer;
use crate::renderer::layout::layout_view::LayoutView;
// use crate::utils::convert_dom_to_string;
use crate::renderer::dom::api::get_js_content;
use crate::renderer::js::ast::JsParser;
use crate::renderer::js::runtime::JsRuntime;
use crate::renderer::js::token::JsLexer;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

/// Page 構造体
/// Browser 構造体へのウィークポインタと DOM ツリーを保持する Window 構造体、および、描画に関する情報を保持する DisplayItem 構造体を持つ。
#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
    style: Option<StyleSheet>,
    layout_view: Option<LayoutView>,
    display_items: Vec<DisplayItem>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            browser: Weak::new(),
            frame: None,
            style: None,
            layout_view: None,
            display_items: Vec::new(),
        }
    }

    pub fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    /// HttpResponse を受け取り、DOM ツリーを文字列として返す。
    pub fn receive_response(&mut self, response: HttpResponse) {
        self.create_frame(response.body());

        // HTML を受信した時に Javascript を実行する。
        self.execute_js();

        self.set_layout_view();
        self.paint_tree();
    }

    fn execute_js(&mut self) {
        let dom = match &self.frame {
            Some(frame) => frame.borrow().document(),
            None => return,
        };

        let js = get_js_content(dom.clone());
        let lexer = JsLexer::new(js);

        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();

        let mut runtime = JsRuntime::new(dom);
        runtime.execute(&ast);
    }

    //   pub fn receive_response(&mut self, response: HttpResponse) -> String {
    //     // デバッグ用に DOM ツリーを文字列として返す。
    //     if let Some(frame) = &self.frame {
    //         let dom = frame.borrow().document().clone();
    //         let debug = convert_dom_to_string(&Some(dom));
    //         return debug;
    //     }

    //     "".to_string()
    // }

    fn create_frame(&mut self, html: String) {
        // HTML 文字列から DOM を構築する。
        let html_tokenizer = HtmlTokenizer::new(html);
        let frame = HtmlParser::new(html_tokenizer).construct_tree();
        // self.frame = Some(frame);
        let dom = frame.borrow().document();

        // CSS を解釈する。
        let style = get_style_content(dom);
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(css_tokenizer).parse_stylesheet();

        self.frame = Some(frame);
        self.style = Some(cssom);
    }

    /// LayoutView 構造体を作成して Page 構造体に設定する。
    fn set_layout_view(&mut self) {
        let dom = match &self.frame {
            Some(frame) => frame.borrow().document(),
            None => return,
        };

        let style = match self.style.clone() {
            Some(style) => style,
            None => return,
        };

        let layout_view = LayoutView::new(dom, &style);
        self.layout_view = Some(layout_view);
    }

    /// 作成したレイアウトツリーの paint メソッドを使って DisplayItem のベクタを取得し、フィールドにセットする。
    fn paint_tree(&mut self) {
        if let Some(layout_view) = &self.layout_view {
            self.display_items = layout_view.paint();
        }
    }

    /// DisplayItems 構造体のベクタを取得する。
    pub fn display_items(&self) -> Vec<DisplayItem> {
        self.display_items.clone()
    }

    /// DisplayItem 構造体のベクタをクリアする。
    pub fn clear_display_items(&mut self) {
        self.display_items = Vec::new();
    }

    /// マウスの位置から度のノードがクリックされたか取得し、そのノードの親が href 属性を持ってる場合、その値を返す。
    pub fn clicked(&self, position: (i64, i64)) -> Option<String> {
        let view = match &self.layout_view {
            Some(v) => v,
            None => return None,
        };

        if let Some(n) = view.find_node_by_position(position) {
            if let Some(parent) = n.borrow().parent().upgrade() {
                if let NodeKind::Element(e) = parent.borrow().node_kind() {
                    if e.kind() == ElementKind::A {
                        return e.get_attribute("href");
                    }
                }
            }
        }
        None
    }
}
