use crate::alloc::string::ToString;
use crate::renderer::css::cssom::ComponentValue;
use crate::renderer::css::cssom::Declaration;
use crate::renderer::css::cssom::Selector;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::layout::computed_style::Color;
use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::computed_style::DisplayType;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::vec::Vec;
use core::cell::RefCell;

/// HTML 要素は表示コンテンツの性質に基づいてブロック要素とインライン要素に分類される。
/// ブロック要素は Block, インライン要素は Inline で表す。
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    parent: Weak<RefCell<LayoutObject>>,
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

    /// ノードが選択されているかを判断する。
    /// 引数にセレクタを取り、そのノードがセレクタに選択されている場合 true を返す。
    pub fn is_node_selected(&self, selector: &Selector) -> bool {
        match &self.node_kind() {
            NodeKind::Element(e) => match selector {
                Selector::TypeSelector(type_name) => {
                    if e.kind().to_string() == *type_name {
                        return true;
                    }
                    false
                }
                Selector::ClassSelector(class_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "class" && attr.value() == *class_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::IdSelector(id_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "id" && attr.value() == *id_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::UnknownSelector => false,
            },
            _ => false,
        }
    }

    /// CSS の宣言リスト (declarations) を引数に取り、各宣言のプロパティをノードに適用する。
    /// 複数のスタイルシートや同じ要素に複数のスタイルを定義できるが、優先して適用するスタイルを決定する仕組みをカスケードと呼ぶ。
    /// 本ブラウザでは <style> タグに直接書く内部スタイルシートのみサポートする。
    /// background-color, color, display プロパティのみ変更できる。
    pub fn cascading_style(&mut self, declarations: Vec<Declaration>) {
        for declaration in declarations {
            match declaration.property.as_str() {
                "background-color" => {
                    if let ComponentValue::Ident(value) = &declaration.value {
                        let color = match Color::from_name(&value) {
                            Ok(color) => color,
                            Err(_) => Color::white(),
                        };
                        self.style.set_background_color(color);
                        continue;
                    }

                    if let ComponentValue::HashToken(color_code) = &declaration.value {
                        let color = match Color::from_code(&color_code) {
                            Ok(color) => color,
                            Err(_) => Color::white(),
                        };
                        self.style.set_background_color(color);
                        continue;
                    }
                }
                "color" => {
                    if let ComponentValue::Ident(value) = &declaration.value {
                        let color = match Color::from_name(&value) {
                            Ok(color) => color,
                            Err(_) => Color::black(),
                        };
                        self.style.set_color(color);
                    }

                    if let ComponentValue::HashToken(color_code) = &declaration.value {
                        let color = match Color::from_code(&color_code) {
                            Ok(color) => color,
                            Err(_) => Color::black(),
                        };
                        self.style.set_color(color);
                    }
                }
                "display" => {
                    if let ComponentValue::Ident(value) = declaration.value {
                        let display_type = match DisplayType::from_str(&value) {
                            Ok(display_type) => display_type,
                            Err(_) => DisplayType::DisplayNone,
                        };
                        self.style.set_display(display_type)
                    }
                }
                _ => {}
            }
        }
    }

    /// ノードに対して CSS の値が明示的に指定されていない場合、指定値を使用する。
    /// 指定値は、仕様書で定められている初期値、親要素の値の継承、CSS の inherit キーワードなどによる明示的な継承の設定により決定される。
    /// 1. CSS により明示的にプロパティに値を指定した場合はその値が使用される。
    /// 2. CSS により明示的な値の指定がない場合、可能であれば親要素から値を継承する。
    /// 3. 1と2のいずれも利用できない場合、要素の初期値が使用される。
    pub fn defaulting_style(
        &mut self,
        node: &Rc<RefCell<Node>>,
        parent_style: Option<ComputedStyle>,
    ) {
        self.style.defaulting(node, parent_style);
    }

    /// ブロック・インライン要素の最終決定
    /// カスケード、デフォルティングを経て CSS の値が最終的に決定した後、改めて LayoutObject のノードがブロック要素になるかインライン要素になるか決定する。
    pub fn update_kind(&mut self) {
        match self.node_kind() {
            NodeKind::Document => panic!("should not create a layout object for a Document node"),
            NodeKind::Element(_) => {
                let display = self.style.display();
                match display {
                    DisplayType::Block => self.kind = LayoutObjectKind::Block,
                    DisplayType::Inline => self.kind = LayoutObjectKind::Inline,
                    DisplayType::DisplayNone => {
                        panic!("should not create a layout object for display:none")
                    }
                }
            }
            NodeKind::Text(_) => self.kind = LayoutObjectKind::Text,
        }
    }
}

/// レイアウトオブジェクトを作成する。
///
pub fn create_layout_object(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    if let Some(n) = node {
        // LayoutObject を作成する。
        let layout_object = Rc::new(RefCell::new(LayoutObject::new(n.clone(), parent_obj)));

        // カスケード値を決める。
        // 複数存在する可能性のある宣言値の中から、実際に要素に適用する値を決定する。
        // これをカスケーディング処理と呼ぶ。
        for rule in &cssom.rules {
            if layout_object.borrow().is_node_selected(&rule.selector) {
                layout_object
                    .borrow_mut()
                    .cascading_style(rule.declarations.clone());
            }
        }

        // 指定値を決める。
        // プロパティがカスケード値を持たない場合、デフォルトの値または親ノードから継承した値を使用する。
        let parent_style = if let Some(parent) = parent_obj {
            Some(parent.borrow().style())
        } else {
            None
        };
        layout_object.borrow_mut().defaulting_style(n, parent_style);

        // display プロパティが none の場合、ノードを作成しない。
        if layout_object.borrow().style().display() == DisplayType::DisplayNone {
            return None;
        }

        // display プロパティの最終的な値を使用してノードの種類を決定する。
        layout_object.borrow_mut().update_kind();
        return Some(layout_object);
    }
    None
}
