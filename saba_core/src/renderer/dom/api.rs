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

/// DOM ツリーから特定の ID の要素を取得する。
/// ノードを再帰的にたどり、ノードの ID 名が id_name で指定されたものを返却する。
pub fn get_element_by_id(
    node: Option<Rc<RefCell<Node>>>,
    id_name: &String,
) -> Option<Rc<RefCell<Node>>> {
    match node {
        Some(n) => {
            if let NodeKind::Element(e) = n.borrow().kind() {
                for attr in &e.attributes() {
                    if attr.name() == "id" && attr.value() == *id_name {
                        return Some(n.clone());
                    }
                }
            }
            let result1 = get_element_by_id(n.borrow().first_child(), id_name);
            let result2 = get_element_by_id(n.borrow().next_sibling(), id_name);
            if result1.is_none() {
                return result2;
            }
            result1
        }
        None => None,
    }
}

/// JavaScript のコードを取得するため、<script> タグの関数を取得する関数。
pub fn get_js_content(root: Rc<RefCell<Node>>) -> String {
    let js_node = match get_target_element_node(Some(root), ElementKind::Script) {
        Some(node) => node,
        None => return "".to_string(),
    };
    let text_node = match js_node.borrow().first_child() {
        Some(node) => node,
        None => return "".to_string(),
    };
    let content = match &text_node.borrow().kind() {
        NodeKind::Text(ref s) => s.clone(),
        _ => "".to_string(),
    };
    content
}
