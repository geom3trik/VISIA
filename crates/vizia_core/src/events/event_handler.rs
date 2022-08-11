use crate::prelude::*;

use std::any::Any;

pub trait ViewHandler: Any {
    fn element(&self) -> Option<&'static str> {
        None
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event);

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas);

    fn as_any_ref(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn ViewHandler {
    /// Attempt to cast a view handler to an immutable reference to the specified type.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref()
    }

    /// Attempt to cast a view handler to a mutable reference to the specified type.
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }
}
