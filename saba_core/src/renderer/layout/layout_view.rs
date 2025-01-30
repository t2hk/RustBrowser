use crate::constants::CONTENT_AREA_WIDTH;
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::create_layout_object;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::layout::layout_object::LayoutObjectKind;
use crate::renderer::layout::layout_object::LayoutPoint;
use crate::renderer::layout::layout_object::LayoutSize;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

/// レイアウトツリーを管理する LayoutView 構造体。
#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        // レイアウトツリーは描画される要素だけを持つツリーなので、<body> タグを取得し、その子要素以下をレイアウトツリーのノードに変換する。
        let body_root = get_target_element_node(Some(root), ElementKind::Body);

        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom),
        };

        tree.update_layout();
        tree
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    /// ノードの位置、サイズ情報の更新
    /// レイアウトツリーのノードをどこに描画するかを決定するため、位置とサイズを計算する必要がある。
    /// 本メソッドは構築し終えたレイアウトツリーに対して、各ノードのサイズと位置を計算する。    
    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));

        Self::calculate_node_position(
            &self.root,
            LayoutPoint::new(0, 0),
            LayoutObjectKind::Block,
            None,
            None,
        )
    }

    /// サイズの計算
    /// レイアウトツリーの各ノードのサイズを再帰的に計算する。
    /// 第1引数: ターゲットのノード
    /// 第2引数: 親ノードのサイズ
    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        if let Some(n) = node {
            // ノードがブロック要素の場合、子ノードのレイアウトを計算する前に横幅を決める。
            if n.borrow().kind() == LayoutObjectKind::Block {
                n.borrow_mut().compute_size(parent_size);
            }

            let first_child = n.borrow().first_child();
            Self::calculate_node_size(&first_child, n.borrow().size());

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_size(&next_sibling, parent_size);

            // 子ノードのサイズが決まった後にサイズを計算する。
            // ブロック要素の時、高さは子ノードの高さに依存する。
            // インライン要素の時、高さも横幅も子ノードに依存する。
            n.borrow_mut().compute_size(parent_size);
        }
    }

    /// 位置の計算
    /// レイアウトツリーのノードの位置を再帰的に計算する。
    /// 第1引数: 計算ターゲットのノード
    /// 第2引数: 親ノードの位置
    /// 第3引数: 自分より前の兄弟ノードの種類
    /// 第4引数: 自分より前の兄弟ノードの①
    /// 第5引数: 自分より前の兄弟ノードのサイズ
    fn calculate_node_position(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        parent_point: LayoutPoint,
        previous_sibling_kind: LayoutObjectKind,
        previous_sibling_point: Option<LayoutPoint>,
        previous_sibling_size: Option<LayoutSize>,
    ) {
        if let Some(n) = node {
            // 現在のノードの位置を計算する。
            n.borrow_mut().compute_position(
                parent_point,
                previous_sibling_kind,
                previous_sibling_point,
                previous_sibling_size,
            );

            // ノードの子ノードの位置を計算する。
            let first_child = n.borrow().first_child();
            Self::calculate_node_position(
                &first_child,
                n.borrow().point(),
                LayoutObjectKind::Block,
                None, // 子ノードには自分より前の兄弟ノードが存在しないため、None を渡す。
                None, // 子ノードには自分より前の兄弟ノードが存在しないため、None を渡す。
            );

            // ノードの兄弟ノードの位置を計算する。
            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_position(
                &next_sibling,
                parent_point,
                n.borrow().kind(),
                Some(n.borrow().point()),
                Some(n.borrow().size()),
            );
        }
    }

    /// 現在のノードを DisplayItem 列挙型のベクタに変換する。
    /// また、子ノードや兄弟ノードに対して再帰的に呼び出し、各ノードの結果を DisplayItem 列挙型のベクタに extend で結合することで、描画に必要な情報のベクタを作成する。
    fn paint_node(node: &Option<Rc<RefCell<LayoutObject>>>, display_items: &mut Vec<DisplayItem>) {
        match node {
            Some(n) => {
                display_items.extend(n.borrow_mut().paint());
                let first_child = n.borrow().first_child();
                Self::paint_node(&first_child, display_items);
                let next_sibling = n.borrow().next_sibling();
                Self::paint_node(&next_sibling, display_items);
            }
            None => (),
        }
    }

    // paint_node を呼び出し、レイアウトツリーを走査する。
    pub fn paint(&self) -> Vec<DisplayItem> {
        let mut display_items = Vec::new();
        Self::paint_node(&self.root, &mut display_items);
        display_items
    }
}

/// レイアウトツリーの作成
/// レイアウトツリーはレイアウトオブジェクトをノードとして持つ木構造である。
/// レイアウトツリーを構築するには、DOM ツリーをルートノードから走査しながら DOM ノードからレイアウトオブジェクトを作成する。
/// 本関数を再帰的に呼び出してレイアウトオブジェクトを構築する。
fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>, // 現在の DOM ツリーのノード
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>, // 親のレイアウトオブジェクト
    cssom: &StyleSheet,               // CSS スタイルシート
) -> Option<Rc<RefCell<LayoutObject>>> {
    // create_layout_object 関数によって、ノードとなる LayoutObject の作成を試みる。
    // CSS によって "display:none" が指定されていた場合、ノードは作成されない。
    let mut target_node = node.clone();
    let mut layout_object = create_layout_object(node, parent_obj, cssom);

    // もしノードが作成されなかった場合、DOM ノードの兄弟ノードを使用して LayoutObject の作成を試みる。
    // LyaoutObject が作成されるまで、兄弟ノードをたどり続ける。
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = create_layout_object(&target_node, parent_obj, cssom);
        } else {
            // もし兄弟ノードがない場合、処理すべき DOM ツリーは終了したので、今まで作成したレイアウトツリーを返却する。
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        // 現在処理している DMO ノードの子ノードと兄弟ノードに対して、再帰的に本関数を呼び出し、子と兄弟のレイアウトツリーを構築する。
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(&original_first_child, &layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        // もし子ノードに "display:none" が指定されていた場合、LayoutObject は作成されないため、
        // 子ノードの兄弟ノードを使用して LayoutObject の作成を試みる。
        // LayoutObject が作成されるか、たどるべき兄弟ノードがなくなるまで処理を繰り返す。
        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);

                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }
                break;
            }
        }

        // もし兄弟ノードに "display:node" が指定されていた場合、LayoutObject は作成されないため、兄弟ノードの兄弟ノードを使用して LayoutObject の作成を試みる。
        // LayoutObject が作成されるか、たどるべき兄弟ノードがなくなるまで処理を繰り返す。
        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);

                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exitst")
                        .borrow()
                        .next_sibling();
                    continue;
                }
                break;
            }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use crate::renderer::css::cssom::CssParser;
    use crate::renderer::css::token::CssTokenizer;
    use crate::renderer::dom::api::get_style_content;
    use crate::renderer::dom::node::Element;
    use crate::renderer::dom::node::NodeKind;
    use crate::renderer::html::parser::HtmlParser;
    use crate::renderer::html::token::HtmlTokenizer;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// 引数の HTML 文字列からレイアウトツリーを作成する関数。
    fn create_layout_view(html: String) -> LayoutView {
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let dom = window.borrow().document();
        let style = get_style_content(dom.clone());
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(css_tokenizer).parse_stylesheet();
        LayoutView::new(dom, &cssom)
    }

    /// 空文字のテスト
    #[test]
    fn test_empty() {
        let layout_view = create_layout_view("".to_string());
        assert_eq!(None, layout_view.root());
    }

    /// <body> タグのみのテスト
    /// LayoutView 構造体の root ノードは LayoutObjectKind::Block であり、かつ、body の NodeKind::Element であることを確認する。
    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );
    }

    /// テキスト要素のテスト
    /// <body> タグにテキストを持つ HTML のテスト。
    /// LayoutView 構造体の root ノードは LayoutObjectKind::Block であり、かつ、
    /// body の NodeKind::Element であることを確認する。
    /// root ノードの子ノードは LayoutObjectKind::Text であり、かつ NodeKind::Text であることを確認する。
    #[test]
    fn test_text() {
        let html = "<html<head></head><body>text</body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let text = root.expect("root should exist").borrow().first_child();
        assert!(text.is_some());
        assert_eq!(
            LayoutObjectKind::Text,
            text.clone()
                .expect("text node should exist")
                .borrow()
                .kind()
        );
        assert_eq!(
            NodeKind::Text("text".to_string()),
            text.clone()
                .expect("text node should exist")
                .borrow()
                .node_kind()
        );
    }

    /// body が display:none のテスト
    /// CSS によって <body> タグに対して {display: none;} が指定されている場合のテスト。
    /// レイアウトツリーは、描画されない要素をノードとして持たないため、root ノードは None となる。
    #[test]
    fn test_display_none() {
        let html = "<html><head><style>body{display:none;}</style></head><body>text</body></html>"
            .to_string();
        let layout_view = create_layout_view(html);
        assert_eq!(None, layout_view.root());
    }

    /// 複数の要素が hidden:none のテスト
    /// .hidden クラスに対して {display: none;} が指定されている場合のテストである。
    /// <body> タグに3つの子要素が存在するが、レイアウトツリーに存在するのはそのうち1つのみである。
    #[test]
    fn test_hidden_class() {
        let html = r#"<html>
      <head>
        <style>
          .hidden {
            display: none;
          }
        </style>
      </head>
      <body>
        <a class="hidden">link1</a>
        <p></p>
        <p class="hidden"><a>link2</a></p>
      </body>
      </html>"#
            .to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let p = root.expect("root should exist").borrow().first_child();
        assert!(p.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            p.clone().expect("p node should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("p", Vec::new())),
            p.clone().expect("p node should exist").borrow().node_kind()
        );
        assert!(p
            .clone()
            .expect("p node should exist")
            .borrow()
            .first_child()
            .is_none());
        assert!(p
            .clone()
            .expect("p node should exist")
            .borrow()
            .next_sibling()
            .is_none());
    }
}
