use freya_native_core::{
    prelude::NodeType,
    real_dom::NodeImmutable,
    tree::TreeRef,
    NodeId,
};
use rustc_hash::FxHashMap;
use torin::prelude::*;

use crate::{
    dom::DioxusDOM,
    states::LayoutState,
};

/// RealDOM adapter for Torin.
pub struct DioxusDOMAdapter<'a> {
    pub rdom: &'a DioxusDOM,
    pub scale_factor: f32,
    cache: FxHashMap<NodeId, bool>,
}

impl<'a> DioxusDOMAdapter<'a> {
    pub fn new(rdom: &'a DioxusDOM, scale_factor: f32) -> Self {
        Self {
            rdom,
            scale_factor,
            cache: FxHashMap::default(),
        }
    }
}

impl DOMAdapter<NodeId> for DioxusDOMAdapter<'_> {
    fn root_id(&self) -> NodeId {
        self.rdom.root_id()
    }

    fn get_node(&self, node_id: &NodeId) -> Option<Node> {
        let node = self.rdom.get(*node_id)?;
        let contains_text = node
            .node_type()
            .tag()
            .map(|t| t.contains_text())
            .unwrap_or_default();

        let mut layout = node.get::<LayoutState>()?.clone();

        // The root node expands by default
        if *node_id == self.rdom.root_id() {
            layout.width = Size::Percentage(Length::new(100.0));
            layout.height = Size::Percentage(Length::new(100.0));
        }

        let mut node = Node {
            width: layout.width,
            height: layout.height,
            minimum_width: layout.minimum_width,
            minimum_height: layout.minimum_height,
            maximum_width: layout.maximum_width,
            maximum_height: layout.maximum_height,
            visible_width: layout.visible_width,
            visible_height: layout.visible_height,
            direction: layout.direction,
            padding: layout.padding,
            margin: layout.margin,
            main_alignment: layout.main_alignment,
            cross_alignment: layout.cross_alignment,
            offset_x: layout.offset_x,
            offset_y: layout.offset_y,
            has_layout_references: layout.node_ref.is_some(),
            position: layout.position,
            content: layout.content,
            wrap_content: layout.wrap_content,
            contains_text,
            spacing: layout.spacing,
        };

        node.scale_if_needed(self.scale_factor);

        Some(node)
    }

    fn height(&self, node_id: &NodeId) -> Option<u16> {
        self.rdom.tree_ref().height(*node_id)
    }

    fn parent_of(&self, node_id: &NodeId) -> Option<NodeId> {
        self.rdom.tree_ref().parent_id(*node_id)
    }

    fn children_of(&mut self, node_id: &NodeId) -> Vec<NodeId> {
        let mut children = self.rdom.tree_ref().children_ids(*node_id);
        children.retain(|id| is_node_valid(self.rdom, &mut self.cache, id));
        children
    }

    fn is_node_valid(&mut self, node_id: &NodeId) -> bool {
        is_node_valid(self.rdom, &mut self.cache, node_id)
    }
}

/// Check is the given Node is valid or not, this means not being a placeholder or an unconnected Node.
fn is_node_valid(rdom: &DioxusDOM, cache: &mut FxHashMap<NodeId, bool>, node_id: &NodeId) -> bool {
    // Check if Node was valid from cache
    if let Some(is_valid) = cache.get(node_id) {
        return *is_valid;
    }

    let node = rdom.get(*node_id);

    let is_valid = 'validation: {
        if let Some(node) = node {
            let is_placeholder = matches!(*node.node_type(), NodeType::Placeholder);

            // Placeholders can't be measured
            if is_placeholder {
                break 'validation false;
            }

            // Make sure this Node isn't part of an unconnected Node
            // This walkes up to the ancestor that has a height of 0 and checks if it has the same ID as the root Node
            // If it has the same ID, it means that is not an unconnected ID, otherwise, it is and should be skipped.
            let tree = rdom.tree_ref();
            let mut current = *node_id;
            loop {
                let height = tree.height(current);
                if let Some(height) = height {
                    if height == 0 {
                        break;
                    }
                }

                let parent_current = tree.parent_id(current);
                if let Some(parent_current) = parent_current {
                    current = parent_current;
                }
            }

            current == rdom.root_id()
        } else {
            false
        }
    };

    // Save the validation result in the cache
    cache.insert(*node_id, is_valid);

    is_valid
}
