use std::{
    any::Any,
    fmt::{
        Debug,
        Display,
    },
    ops::Div,
    sync::{
        Arc,
        Mutex,
    },
};

use accesskit::NodeId as AccessibilityId;
use bytes::Bytes;
use dioxus_core::AttributeValue;
use freya_engine::prelude::*;
use freya_native_core::node::FromAnyValue;
use tokio::sync::{
    mpsc::UnboundedSender,
    watch,
};
use torin::geometry::{
    Area,
    Size2D,
};

/// Image Reference
#[derive(Clone, Debug)]
pub struct ImageReference(pub Arc<Mutex<Option<Bytes>>>);

impl PartialEq for ImageReference {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Display for ImageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageReference").finish_non_exhaustive()
    }
}

/// Layout info of a certain Node, used by `use_node`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeReferenceLayout {
    pub area: Area,
    pub inner: Size2D,
}

impl NodeReferenceLayout {
    pub fn div(&mut self, rhs: f32) {
        self.area = self.area.div(rhs);
        self.inner = self.inner.div(rhs);
    }
}

/// Messages emitted from the layout library to the Nodes. Used in `use_editable`.
#[derive(Debug)]
pub enum CursorLayoutResponse {
    CursorPosition { position: usize, id: usize },
    TextSelection { from: usize, to: usize, id: usize },
}

/// Node Reference
#[derive(Debug, Clone)]
pub struct NodeReference(pub Arc<watch::Sender<NodeReferenceLayout>>);

impl PartialEq for NodeReference {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Display for NodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeReference").finish_non_exhaustive()
    }
}

pub struct CanvasRunnerContext<'a> {
    pub canvas: &'a Canvas,
    pub font_collection: &'a mut FontCollection,
    pub area: Area,
    pub scale_factor: f32,
}

pub type CanvasRunner = dyn FnMut(&mut CanvasRunnerContext) + Send + 'static;

/// Canvas Reference
#[derive(Clone)]
pub struct CanvasReference {
    pub runner: Arc<Mutex<CanvasRunner>>,
}

impl PartialEq for CanvasReference {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.runner, &other.runner)
    }
}

impl Debug for CanvasReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasReference").finish_non_exhaustive()
    }
}

/// Cursor reference
#[derive(Clone, Debug)]
pub struct CursorReference {
    pub text_id: usize,
    pub cursor_sender: UnboundedSender<CursorLayoutResponse>,
}

impl PartialEq for CursorReference {
    fn eq(&self, other: &Self) -> bool {
        self.text_id == other.text_id
    }
}

impl Display for CursorReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CursorReference").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributesBytes {
    Dynamic(Bytes),
    Static(&'static [u8]),
}

impl AttributesBytes {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Dynamic(bytes) => bytes.as_ref(),
            Self::Static(bytes) => bytes,
        }
    }
}

/// Group all the custom attribute types
#[derive(Clone, PartialEq)]
pub enum CustomAttributeValues {
    Reference(NodeReference),
    CursorReference(CursorReference),
    Bytes(AttributesBytes),
    ImageReference(ImageReference),
    AccessibilityId(AccessibilityId),
    TextHighlights(Vec<(usize, usize)>),
    Canvas(CanvasReference),
}

impl Debug for CustomAttributeValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(_) => f.debug_tuple("Reference").finish(),
            Self::CursorReference(_) => f.debug_tuple("CursorReference").finish(),
            Self::Bytes(_) => f.debug_tuple("Bytes").finish(),
            Self::ImageReference(_) => f.debug_tuple("ImageReference").finish(),
            Self::AccessibilityId(_) => f.debug_tuple("AccessibilityId").finish(),
            Self::TextHighlights(_) => f.debug_tuple("TextHighlights").finish(),
            Self::Canvas(_) => f.debug_tuple("Canvas").finish(),
        }
    }
}

impl FromAnyValue for CustomAttributeValues {
    fn from_any_value(b: &dyn Any) -> Self {
        b.downcast_ref::<CustomAttributeValues>().unwrap().clone()
    }
}

/// Transform some dynamic bytes (e.g: remote image fetched at runtime) into an attribute
pub fn dynamic_bytes(bytes: impl Into<Bytes>) -> AttributeValue {
    AttributeValue::any_value(CustomAttributeValues::Bytes(AttributesBytes::Dynamic(
        bytes.into(),
    )))
}

/// Transform some static bytes (e.g: statically linked images or SVGs) into an attribute
pub fn static_bytes(bytes: &'static [u8]) -> AttributeValue {
    AttributeValue::any_value(CustomAttributeValues::Bytes(AttributesBytes::Static(bytes)))
}
