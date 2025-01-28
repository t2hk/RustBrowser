use crate::renderer::dom::node::Element;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;

/// 引数の要素の種類 (element_kind) と一致した最初のノードを返す。返すノードは１つのみである。
/// 引数のノード (node) から再帰的に要素の種類をチェックする。
pub fn get_target_element_node(
    node: Option<Rc<RefCell<Node>>>,
    element_kind: ElementKind,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            // 現在のノードの要素が element_kind と同じであれば、そのノードを返却する。
            if n.borrow().kind()
                == NodeKind::Element(Element::new(&element_kind.to_string(), Vec::new()))
            {
                return Some(n.clone());
            }

            // 現在のノードの要素が element_kind と異なる場合、子ノードと兄弟ノードに対して再帰的に呼び出す。
            let result1 = get_target_element_node(n.borrow().first_child(), element_kind);
            let result2 = get_target_element_node(n.borrow().next_sibling(), element_kind);

            if result1.is_none() && result2.is_none() {
                return None;
            }

            if result1.is_none() {
                return result2;
            }
            result1
        }
        None => None,
    }
}

/// <style>　タグのコンテンツを取得できる関数
pub fn get_style_content(root: Rc<RefCell<Node>>) -> String {
    let style_node = match get_target_element_node(Some(root), ElementKind::Style) {
        Some(node) => node,
        None => return "".to_string(),
    };
    let text_node = match style_node.borrow().first_child() {
        Some(node) => node,
        None => return "".to_string(),
    };
    let content = match &text_node.borrow().kind() {
        NodeKind::Text(ref s) => s.clone(),
        _ => "".to_string(),
    };
    content
}
