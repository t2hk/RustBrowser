use crate::renderer::html::attribute::Attribute;
use alloc::format;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;

/// ノードの種類を保持する列挙型。
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// https://dom.spec.whatwg.org/#interface-document
    Document,
    /// https://dom.spec.whatwg.org/#interface-element
    Element(Element),
    /// https://dom.spec.whatwg.org/#interface-text
    Text(String),
}

/// 1つのノードを表す構造体。
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,                      // ノードの種類
    window: Weak<RefCell<Window>>, // DOM ツリーを持つウィンドウ。1つのページに対して1つのウィンドウインスタンスが存在する。弱い参照として保持する（ウィークポインタ）。
    parent: Weak<RefCell<Node>>,   // ノードの親ノード。弱い参照として保持する（ウィークポインタ）。
    first_child: Option<Rc<RefCell<Node>>>, // ノードの一番初めの子ノード
    last_child: Weak<RefCell<Node>>, // ノードの最後の子ノード。弱い参照として保持する（ウィークポインタ）。
    previous_sibling: Weak<RefCell<Node>>, // ノードの前の兄弟ノード。弱い参照として保持する（ウィークポインタ）。
    next_sibling: Option<Rc<RefCell<Node>>>, // ノードの次の兄弟ノード。
}

impl Node {
    /// ノード構造体の作成。
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            window: Weak::new(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }
    }

    /// window オブジェクトのセッター。
    pub fn set_window(&mut self, window: Weak<RefCell<Window>>) {
        self.window = window;
    }

    /// 親ノードのセッター。
    pub fn set_parent(&mut self, parent: Weak<RefCell<Node>>) {
        self.parent = parent;
    }

    /// 親ノードのゲッター。
    pub fn parent(&self) -> Weak<RefCell<Node>> {
        self.parent.clone()
    }

    /// 最初の子ノードのセッター。
    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child;
    }

    /// 最初の子ノードのゲッター。
    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.as_ref().cloned()
    }

    /// 最後の子ノードのセッター。
    pub fn set_last_child(&mut self, last_child: Weak<RefCell<Node>>) {
        self.last_child = last_child;
    }

    /// 最後の子ノードのゲッター。
    pub fn last_child(&self) -> Weak<RefCell<Node>> {
        self.last_child.clone()
    }

    /// ノードの前の兄弟ノードのセッター。
    pub fn set_previous_sibling(&mut self, previous_sibling: Weak<RefCell<Node>>) {
        self.previous_sibling = previous_sibling;
    }

    /// ノードの前の兄弟ノードのゲッター。
    pub fn previous_sibling(&self) -> Weak<RefCell<Node>> {
        self.previous_sibling.clone()
    }

    /// ノードの次の兄弟ノードのセッター。
    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<Node>>>) {
        self.next_sibling = next_sibling;
    }

    /// ノードの次の兄弟ノードのゲッター。
    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().cloned()
    }

    /// ノードの種類を取得する。
    pub fn kind(&self) -> NodeKind {
        self.kind.clone()
    }

    /// 要素を取得する。
    pub fn get_element(&self) -> Option<Element> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.clone()),
        }
    }

    /// 要素の種類を取得する。
    pub fn get_element_kind(&self) -> Option<ElementKind> {
        match self.kind {
            NodeKind::Document | NodeKind::Text(_) => None,
            NodeKind::Element(ref e) => Some(e.kind()),
        }
    }
}

/// Window 構造体。
/// DOM ツリーのルートを持ち、1つの Web ページに対して1つのインスタンスが存在する。
/// 通常、window というグローバル変数で定義されるオブジェクトである。
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#window
#[derive(Debug, Clone)]
pub struct Window {
    document: Rc<RefCell<Node>>,
}

impl Window {
    /// window オブジェクトの生成。
    pub fn new() -> Self {
        let window = Self {
            document: Rc::new(RefCell::new(Node::new(NodeKind::Document))),
        };

        window
            .document
            .borrow_mut()
            .set_window(Rc::downgrade(&Rc::new(RefCell::new(window.clone()))));
        window
    }

    /// DOM ツリーのルートの document 要素のゲッター。
    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }
}

/// Element 構造体。
/// https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    kind: ElementKind,
    attributes: Vec<Attribute>,
}

impl Element {
    /// Element オブジェクトを生成する。
    pub fn new(element_name: &str, attributes: Vec<Attribute>) -> Self {
        Self {
            kind: ElementKind::from_str(element_name)
                .expect("failed to convert string to ElementKind"),
            attributes,
        }
    }

    /// Element オブジェクトのゲッター。
    pub fn kind(&self) -> ElementKind {
        self.kind
    }
}

/// 要素の種類を表す列挙型。
/// https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElementKind {
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-html-element
    Html,
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-head-element
    Head,
    /// https://html.spec.whatwg.org/multipage/semantics.html#the-style-element
    Style,
    /// https://html.spec.whatwg.org/multipage/scripting.html#the-script-element
    Script,
    /// https://html.spec.whatwg.org/multipage/sections.html#the-body-element
    Body,
    /// https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element
    P,
    /// https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
    H1,
    H2,
}

impl FromStr for ElementKind {
    type Err = String;

    /// 文字列から要素を作成する。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(ElementKind::Html),
            "head" => Ok(ElementKind::Head),
            "style" => Ok(ElementKind::Style),
            "script" => Ok(ElementKind::Script),
            "body" => Ok(ElementKind::Body),
            "p" => Ok(ElementKind::P),
            "h1" => Ok(ElementKind::H1),
            "h2" => Ok(ElementKind::H2),
            _ => Err(format!("unimplemented element name {:?}", s)),
        }
    }
}
