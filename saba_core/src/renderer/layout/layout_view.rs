use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::create_layout_object;
use crate::renderer::layout::layout_object::LayoutObject;
use alloc::rc::Rc;
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
