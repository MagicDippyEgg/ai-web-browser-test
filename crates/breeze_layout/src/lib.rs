pub mod box_model;
pub mod layout_tree;

pub use box_model::{Dimensions, EdgeSizes, Rect};
pub use layout_tree::{BoxType, LayoutBox};
use breeze_css::style::{ComputedStyle, Display, StyleEngine};
use breeze_html::dom::{NodeData, NodeRef};

pub fn build_layout_tree(node: &NodeRef, style_engine: &StyleEngine, parent_style: Option<&ComputedStyle>) -> Option<LayoutBox> {
    let computed_style = style_engine.compute_style(node, parent_style);

    if computed_style.display == Display::None {
        return None;
    }

    let node_borrow = node.borrow();
    let box_type = match &node_borrow.data {
        NodeData::Element(_) => match computed_style.display {
            Display::Block => BoxType::Block,
            Display::Flex => BoxType::Flex,
            Display::Grid => BoxType::Grid,
            Display::Inline | Display::InlineBlock => BoxType::Inline,
            Display::None => unreachable!(),
        },
        NodeData::Text { content } => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return None;
            }
            BoxType::Text(content.clone())
        }
        NodeData::Document => BoxType::Block,
        _ => return None,
    };

    drop(node_borrow);

    let mut layout_box = LayoutBox::new(box_type, computed_style.clone(), Some(std::rc::Rc::clone(node)));

    for child in &node.borrow().children {
        if let Some(child_box) = build_layout_tree(child, style_engine, Some(&computed_style)) {
            layout_box.children.push(child_box);
        }
    }

    Some(layout_box)
}
