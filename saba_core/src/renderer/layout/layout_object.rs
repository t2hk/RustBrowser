use crate::alloc::string::ToString;
use crate::constants::CHAR_HEIGHT_WITH_PADDING;
use crate::constants::CHAR_WIDTH;
use crate::constants::CONTENT_AREA_WIDTH;
use crate::constants::WINDOW_PADDING;
use crate::constants::WINDOW_WIDTH;
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::ComponentValue;
use crate::renderer::css::cssom::Declaration;
use crate::renderer::css::cssom::Selector;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::layout::computed_style::Color;
use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::computed_style::DisplayType;
use crate::renderer::layout::computed_style::FontSize;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec;
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

/// 要素の幅を超えるテキストの場合、単語の境界であるホワイトスペースで改行する。
/// 改行位置を見つける関数である。
fn find_index_for_line_break(line: String, max_index: usize) -> usize {
    for i in (0..max_index).rev() {
        if line.chars().collect::<Vec<char>>()[i] == ' ' {
            return i;
        }
    }
    max_index
}

/// ウィンドウの大きさによってテキストを指定された幅内に収まるように単語の途中で折り返すことなく、スペースで区切られた部分ごとに分割する処理を行う。
/// この動作は、CSS の workd-break プロパティが normal の時と同じ動作である。word-break プロパティのデフォルトの挙動は、単語を途中で折り返さない。
fn split_text(line: String, char_width: i64) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    if line.len() as i64 * char_width > (WINDOW_WIDTH + WINDOW_PADDING) {
        let s = line.split_at(find_index_for_line_break(
            line.clone(),
            ((WINDOW_WIDTH + WINDOW_PADDING) / char_width) as usize,
        ));
        result.push(s.0.to_string());
        result.extend(split_text(s.1.trim().to_string(), char_width))
    } else {
        result.push(line);
    }
    result
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

    pub fn set_x(&mut self, x: i64) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i64) {
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

    /// 1つのノードのサイズを計算する。
    /// ノードがブロック要素の場合、親ノードの横幅がそのまま自身の横幅になる。
    /// ノードがインライン要素の場合、高さも横幅も子要素のサイズを足し合わせたものとなる。
    /// ノードがテキストの場合、まずはフォントのサイズによって文字の大きさの比率を決定する。
    pub fn compute_size(&mut self, parent_size: LayoutSize) {
        let mut size = LayoutSize::new(0, 0);

        match self.kind() {
            LayoutObjectKind::Block => {
                size.set_width(parent_size.width());

                let mut height = 0;
                let mut child = self.first_child();
                let mut previous_child_kind = LayoutObjectKind::Block;
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    if previous_child_kind == LayoutObjectKind::Block
                        || c.borrow().kind() == LayoutObjectKind::Block
                    {
                        height += c.borrow().size.height();
                    }

                    previous_child_kind = c.borrow().kind();
                    child = c.borrow().next_sibling();
                }
                size.set_height(height);
            }
            // ノードがインライン要素の場合、高さも横幅も子要素のサイズを足し合わせたものとする。
            // 本実装では、インライン要素の子ノードは常にテキストノードである。
            LayoutObjectKind::Inline => {
                // 全ての子ノードの高さと横幅を足し合わせた結果が現在のノードの高さと横幅になる。
                let mut width = 0;
                let mut height = 0;
                let mut child = self.first_child();
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    width += c.borrow().size.width();
                    height += c.borrow().size.height();
                    child = c.borrow().next_sibling();
                }
                size.set_width(width);
                size.set_height(height);
            }
            // ノードがテキストの倍、フォントのサイズによって文字の大きさの比率を決定する。
            LayoutObjectKind::Text => {
                if let NodeKind::Text(t) = self.node_kind() {
                    // フォントサイズによって文字の大きさの比率を決定する。
                    let ratio = match self.style.font_size() {
                        FontSize::Medium => 1,
                        FontSize::XLarge => 2,
                        FontSize::XXLarge => 3,
                    };
                    // 文字の幅、比率、文字列の長さからテキスト要素の幅を計算する。
                    let width = CHAR_WIDTH * ratio * t.len() as i64;
                    // もし文字列の長さが描画可能なエリアの横幅より長い場合、テキストを複数行に折り返す。
                    if width > CONTENT_AREA_WIDTH {
                        // テキストが複数行の場合
                        size.set_width(CONTENT_AREA_WIDTH);
                        // 文字列の長さを描画可能なエリアの横幅で割った結果の数値が行数になる。
                        let line_num = if width.wrapping_rem(CONTENT_AREA_WIDTH) == 0 {
                            width.wrapping_div(CONTENT_AREA_WIDTH)
                        } else {
                            // 割り切れない場合、最後の行が中途半端な位置で終わることになるため、1行追加する。
                            width.wrapping_div(CONTENT_AREA_WIDTH) + 1
                        };
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio * line_num);
                    }
                    // テキストが 1行に収まる場合
                    else {
                        size.set_width(width);
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio);
                    }
                }
            }
        }
        self.size = size;
    }

    /// 1つのノードの位置を計算する。
    /// ノードの位置は、現在のノードと親ノードの一、隣り合わせの兄弟ノードによって決定する。
    pub fn compute_position(
        &mut self,
        parent_point: LayoutPoint,
        previous_sibling_kind: LayoutObjectKind,
        previous_sibling_point: Option<LayoutPoint>,
        previous_sibling_size: Option<LayoutSize>,
    ) {
        let mut point = LayoutPoint::new(0, 0);

        match (self.kind(), previous_sibling_kind) {
            // 自分自身がブロック要素、または、兄弟ノードがブロック要素の場合、このノードは新しい行から描画されることになるため、ウィンドウの下方向 (Y 軸方向) に向かって位置を調整する。
            (LayoutObjectKind::Block, _) | (_, LayoutObjectKind::Block) => {
                if let (Some(size), Some(pos)) = (previous_sibling_size, previous_sibling_point) {
                    // 兄弟ノードが存在する場合、兄弟ノードの Y 位置と高さを足し合わせたものが次の位置になる。
                    point.set_y(pos.y() + size.height());
                } else {
                    // 兄弟ノードが存在しない場合、親ノードの Y 座標をセットする。
                    point.set_y(parent_point.y());
                }
                // 新しい行から始まるため、X 座標は常に親要素の X 座標と同じである。
                point.set_x(parent_point.x());
            }
            // もし自分自身と兄弟ノードがインライン要素の場合、同じ行に続いて配置されるため、ウィンドウの右方向に向かって位置を調整する。
            (LayoutObjectKind::Inline, LayoutObjectKind::Inline) => {
                // 兄弟ノードが存在する場合
                if let (Some(size), Some(pos)) = (previous_sibling_size, previous_sibling_point) {
                    point.set_x(pos.x() + size.width()); // 兄弟ノードの X 位置と横幅を足し合わせたものが次の位置になる。
                    point.set_y(pos.y()); // インライン要素は兄弟ノードと同じ行に並ぶため、兄弟ノードの Y 位置が自分の Y 位置になる。
                } else {
                    // 兄弟ノードが存在しない場合、親ノードの X と Y 位置をセットする。
                    point.set_x(parent_point.x());
                    point.set_y(parent_point.y());
                }
            }
            _ => {
                // ブロック要素やインライン要素ではない場合（テキストノードの場合）、親ノードの位置と同じ位置に描画する。
                point.set_x(parent_point.x());
                point.set_y(parent_point.y());
            }
        }
        self.point = point;
    }

    /// そのノードを DisplayItem に変換する。
    pub fn paint(&mut self) -> Vec<DisplayItem> {
        if self.style.display() == DisplayType::DisplayNone {
            return vec![];
        }

        match self.kind {
            // ノードがブロック要素の場合、ノードのスタイル、位置、サイズをそのまま使用して DisplayItem::Rect を作成して返す。
            LayoutObjectKind::Block => {
                if let NodeKind::Element(_e) = self.node_kind() {
                    return vec![DisplayItem::Rect {
                        style: self.style(),
                        layout_point: self.point(),
                        layout_size: self.size(),
                    }];
                }
            }
            // ノードがインライン要素の場合、本ブラウザでは描画するインライン要素がないため何も行わない。
            LayoutObjectKind::Inline => {
                // 本ブラウザでは、描画するインライン要素はない。
                // <img> タグなどをサポートした場合はこのアームの中で処理する。
            }
            // ノードがテキストノードの場合、ノードのサイズを計算したときと同じようにフォントのサイズと1文字の横幅の情報をもとに、改行すべき位置を探す。
            // テキストが複数行になる場合、複数の DisplayItem::Text オブジェクトを返す。
            LayoutObjectKind::Text => {
                if let NodeKind::Text(t) = self.node_kind() {
                    let mut v = vec![];
                    let ratio = match self.style.font_size() {
                        FontSize::Medium => 1,
                        FontSize::XLarge => 2,
                        FontSize::XXLarge => 3,
                    };
                    let plain_text = t
                        .replace("\n", " ")
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let lines = split_text(plain_text, CHAR_WIDTH * ratio);
                    let mut i = 0;
                    for line in lines {
                        let item = DisplayItem::Text {
                            text: line,
                            style: self.style(),
                            layout_point: LayoutPoint::new(
                                self.point().x(),
                                self.point().y() + CHAR_HEIGHT_WITH_PADDING * i,
                            ),
                        };
                        v.push(item);
                        i += 1;
                    }
                    return v;
                }
            }
        }
        vec![]
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

/// LayoutObject 構造体の PartialEq トレイトの実装
impl PartialEq for LayoutObject {
    /// LayoutObject 構造体の比較
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
