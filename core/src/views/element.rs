use crate::{Context, Handle, View};

/// A basic element with no interactivity.
///
///
pub struct Element {}

impl Element {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx)
    }
}

impl View for Element {
    fn element(&self) -> Option<String> {
        Some("element".to_string())
    }
}
