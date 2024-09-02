use crate::icons::ICON_CHECK;
use crate::prelude::*;

/// A checkbox used to display and toggle a boolean state.
///
/// Pressing the checkbox triggers the [`on_toggle`](Checkbox::on_toggle) callback.
///
/// # Examples
///
/// ## Basic checkbox
///
/// The checkbox must bound to some boolean data.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # #[derive(Lens)]
/// # struct AppData {
/// #     value: bool,
/// # }
/// #
/// # impl Model for AppData {}
/// #
/// # let cx = &mut Context::default();
/// #
/// # AppData { value: false }.build(cx);
/// #
/// Checkbox::new(cx, AppData::value);
/// ```
///
/// ## Checkbox with an action
///
/// A checkbox can be used to trigger a callback when toggled. Usually this is emitting an
/// event responsible for changing the data the checkbox is bound to.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # #[derive(Lens)]
/// # struct AppData {
/// #     value: bool,
/// # }
/// #
/// # impl Model for AppData {}
/// #
/// # enum AppEvent {
/// #     ToggleValue,
/// # }
/// #
/// # let cx = &mut Context::default();
/// #
/// # AppData { value: false }.build(cx);
/// #
/// Checkbox::new(cx, AppData::value).on_toggle(|cx| cx.emit(AppEvent::ToggleValue));
/// ```
///
/// ## Checkbox with a label
///
/// A checkbox is usually used with a label next to it describing what data the checkbox
/// is bound to or what the checkbox does when pressed. This can be done, for example, by
/// wrapping the checkbox in an [`HStack`](crate::prelude::HStack) and adding a [`Label`](crate::prelude::Label)
/// to it.
///
/// The Label can be used to trigger the checkbox by assigning the checkbox an id name and using it with the `describing` modifier on the label.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # #[derive(Lens)]
/// # struct AppData {
/// #     value: bool,
/// # }
/// #
/// # impl Model for AppData {}
/// #
/// # let cx = &mut Context::default();
/// #
/// # AppData { value: false }.build(cx);
/// #
/// HStack::new(cx, |cx| {
///     Checkbox::new(cx, AppData::value).id("check1");
///     Label::new(cx, "Press me").describing("check1");
/// });
/// ```
///
/// ## Custom checkbox
///
/// The `with_icons` constructor can be used to create a checkbox with custom icons for both checked and unchecked states.
///
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # #[derive(Lens)]
/// # struct AppData {
/// #     value: bool,
/// # }
/// #
/// # impl Model for AppData {}
/// #
/// # enum AppEvent {
/// #     ToggleValue,
/// # }
/// #
/// # let cx = &mut Context::default();
/// #
/// # AppData { value: false }.build(cx);
/// # use vizia_core::icons::ICON_X;
///
/// Checkbox::with_icons(cx, AppData::value, None, Some(ICON_X))
///     .on_toggle(|cx| cx.emit(AppEvent::ToggleValue));
/// ```
pub struct Checkbox {
    on_toggle: Option<Box<dyn Fn(&mut EventContext)>>,
}

impl Checkbox {
    /// Creates a new checkbox.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vizia_core::prelude::*;
    /// #
    /// # #[derive(Lens)]
    /// # struct AppData {
    /// #     value: bool,
    /// # }
    /// #
    /// # impl Model for AppData {}
    /// #
    /// # let cx = &mut Context::default();
    /// #
    /// # AppData { value: false }.build(cx);
    /// #
    /// Checkbox::new(cx, AppData::value);
    /// ```
    pub fn new(cx: &mut Context, checked: impl Lens<Target = bool>) -> Handle<Self> {
        Self { on_toggle: None }
            .build(cx, |cx| {
                Binding::new(cx, checked, |cx, checked| {
                    if checked.get(cx) {
                        Svg::new(cx, ICON_CHECK);
                    }
                })
            })
            .checked(checked)
            .role(Role::CheckBox)
            .default_action_verb(DefaultActionVerb::Click)
            .navigable(true)
    }

    /// Creates a new checkbox with custom icons for both checked and unchecked states.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vizia_core::prelude::*;
    /// #
    /// # #[derive(Lens)]
    /// # struct AppData {
    /// #     value: bool,
    /// # }
    /// #
    /// # impl Model for AppData {}
    /// #
    /// # enum AppEvent {
    /// #     ToggleValue,
    /// # }
    /// #
    /// # let cx = &mut Context::default();
    /// #
    /// # AppData { value: false }.build(cx);
    /// # use vizia_core::icons::ICON_X;
    ///
    /// Checkbox::with_icons(cx, AppData::value, None, Some(ICON_X))
    ///     .on_toggle(|cx| cx.emit(AppEvent::ToggleValue));
    /// ```
    pub fn with_icons<T>(
        cx: &mut Context,
        checked: impl Lens<Target = bool>,
        icon_default: Option<impl Res<T> + 'static + Clone>,
        icon_checked: Option<impl Res<T> + 'static + Clone>,
    ) -> Handle<Self>
    where
        T: AsRef<[u8]> + 'static,
    {
        Self { on_toggle: None }
            .build(cx, |cx| {
                Binding::new(cx, checked, move |cx, checked| {
                    let icon_default = icon_default.clone();
                    let icon_checked = icon_checked.clone();
                    if checked.get(cx) {
                        if let Some(icon) = icon_checked {
                            Svg::new(cx, icon);
                        }
                    } else if let Some(icon) = icon_default {
                        Svg::new(cx, icon);
                    }
                })
            })
            .checked(checked)
            .role(Role::CheckBox)
            .default_action_verb(DefaultActionVerb::Click)
            .navigable(true)
    }

    pub fn intermediate(
        cx: &mut Context,
        checked: impl Lens<Target = bool>,
        intermediate: impl Lens<Target = bool>,
    ) -> Handle<Self> {
        Self { on_toggle: None }
            .build(cx, |_| {})
            .bind(checked, move |handle, c| {
                handle.bind(intermediate, move |handle, i| {
                    if c.get(&handle) {
                        handle.text(ICON_CHECK).toggle_class("intermediate", false);
                    } else if i.get(&handle) {
                        handle.text("-").toggle_class("intermediate", true);
                    } else {
                        handle.text("").toggle_class("intermediate", false);
                    }
                });
            })
            .checked(checked)
            .navigable(true)
    }
}

impl Handle<'_, Checkbox> {
    /// Set the callback triggered when the checkbox is pressed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use vizia_core::prelude::*;
    /// #
    /// # #[derive(Lens)]
    /// # struct AppData {
    /// #     value: bool,
    /// # }
    /// #
    /// # impl Model for AppData {}
    /// #
    /// # enum AppEvent {
    /// #     ToggleValue,
    /// # }
    /// #
    /// # let cx = &mut Context::default();
    /// #
    /// # AppData { value: false }.build(cx);
    /// #
    /// Checkbox::new(cx, AppData::value).on_toggle(|cx| cx.emit(AppEvent::ToggleValue));
    /// ```
    pub fn on_toggle<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext),
    {
        self.modify(|checkbox| checkbox.on_toggle = Some(Box::new(callback)))
    }
}

impl View for Checkbox {
    fn element(&self) -> Option<&'static str> {
        Some("checkbox")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            WindowEvent::PressDown { mouse: _ } => {
                if meta.target == cx.current {
                    cx.focus();
                }
            }

            WindowEvent::Press { mouse: _ } => {
                if meta.target == cx.current {
                    if let Some(callback) = &self.on_toggle {
                        (callback)(cx);
                    }
                }
            }

            WindowEvent::ActionRequest(action) => match action.action {
                Action::Default => {
                    if let Some(callback) = &self.on_toggle {
                        (callback)(cx);
                    }
                }

                _ => {}
            },

            _ => {}
        });
    }
}
