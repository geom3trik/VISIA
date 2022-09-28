use crate::prelude::*;

/// A simple push button with an action and views inside of it.
///
/// # Examples
///
/// ## Button with an action
///
/// A button can be used to call an action when pressed. Usually this is an
/// event that is being emitted.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # enum AppEvent {
/// #     Action,
/// # }
/// #
/// # let cx = &mut Context::new();
/// #
/// Button::new(cx, |cx| cx.emit(AppEvent::Action), |cx| Label::new(cx, "Text"));
/// ```
///
/// ## Button without an action
///
/// A button can be used without an action and therefore do nothing when pressed.
/// This is useful for prototyping and testing out the different styling options of
/// a button without having to add an action.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # let cx = &mut Context::new();
/// #
/// Button::new(cx, |_| {}, |cx| Label::new(cx, "Text"));
/// ```
///
/// ## Button containing multiple views
///
/// A button can contain more than just a single view or label inside of it. This can
/// for example be done by using a [`HStack`](crate::prelude::HStack) or [`VStack`](crate::prelude::VStack).
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # let cx = &mut Context::new();
/// #
/// Button::new(
///     cx,
///     |_| {},
///     |cx| {
///         HStack::new(cx, |cx| {
///             Label::new(cx, "Hello");
///             Label::new(cx, "World");
///         })
///     },
/// );
/// ```
pub struct Button {}

impl Button {
    /// Creates a new button.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vizia_core::prelude::*;
    /// #
    /// # let cx = &mut Context::new();
    /// #
    /// Button::new(cx, |_| {}, |cx| Label::new(cx, "Text"));
    /// ```
    pub fn new<F, V>(cx: &mut Context, content: F) -> Handle<Self>
    where
        F: FnOnce(&mut Context) -> Handle<V>,
        V: 'static + View,
    {
        Self {}
            .build(cx, move |cx| {
                (content)(cx).hoverable(false);
            })
            .cursor(CursorIcon::Hand)
            .keyboard_navigatable(true)
    }
}

impl View for Button {
    fn element(&self) -> Option<&'static str> {
        Some("button")
    }
}
