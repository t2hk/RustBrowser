use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::layout::computed_style::computedStyle;
use alloc::rc::Rc;
use alloc::rc::Weak;
use core::cell::RefCell;

/// HTML 要素は表示コンテンツの性質に基づいてブロック要素とインライン要素に分類される。
/// ブロック要素は Block, インライン要素は Inline で表す。
pub enum LayoutObjectKind {
    Block,
    Inline,
    Text,
}

/// レイアウトオブジェクトの位置を表すデータ構造である。
/// レイアウトツリーを構築する際に、各要素の描画される位置を計算する。
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutPoint {
    x: i64,
    y: i64,
}

impl LayoutPoint {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn set_x(&self, x: i64) {
        self.x = x;
    }

    pub fn set_y(&self, y: i64) {
        self.y = y;
    }
}

/// レイアウトサイズ構造体
/// レイアウトオブジェクトのサイズを表すデータ構造。
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutSize {
    width: i64,
    height: i64,
}

impl LayoutSize {
    pub fn new(width: i64, height: i64) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> i64 {
        self.width
    }

    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn set_width(&mut self, width: i64) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: i64) {
        self.height = height;
    }
}

/// LayoutObject 構造体
/// レイアウトツリーの1つのノードであり、描画に必要な情報を全て持った構造体である。
#[derive(Debug, Clone)]
pub struct LayoutObject {
    kind: LayoutObjectKind,
    node: Rc<RefCell<Node>>,
    first_child: Option<Rc<RefCell<LayoutObject>>>,
    next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    prevent: Weak<RefCell<LayoutObject>>,
    style: ComputedStyle,
    point: LayoutPoint,
    size: LayoutSize,
}

impl LayoutObject {
    pub fn new(node: Rc<RefCell<Node>>, parent_obj: &Option<Rc<RefCell<LayoutObject>>>) -> Self {
        let parent = match parent_obj {
            Some(p) => Rc::downgrade(p),
            None => Weak::new(),
        };

        Self {
            kind: LayoutObjectKind::Block,
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            parent,
            style: ComputedStyle::new(),
            point: LayoutPoint::new(0, 0),
            size: LayoutSize::new(0, 0),
        }
    }

    pub fn kind(&self) -> LayoutObjectKind {
        self.kind
    }

    pub fn node_kind(&self) -> NodeKind {
        self.node.borrow().kind().clone()
    }

    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<LayoutObject>>>) {
        self.first_child = first_child;
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.first_child.as_ref().cloned()
    }

    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<LayoutObject>>>) {
        self.next_sibling = next_sibling;
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.next_sibling.as_ref().cloned()
    }

    pub fn parent(&self) -> Weak<RefCell<Self>> {
        self.parent.clone()
    }

    pub fn style(&self) -> ComputedStyle {
        self.style.clone()
    }

    pub fn point(&self) -> LayoutPoint {
        self.point
    }

    pub fn size(&self) -> LayoutSize {
        self.size
    }
}
