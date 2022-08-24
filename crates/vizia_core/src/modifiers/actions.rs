use crate::prelude::*;
use std::{any::TypeId, sync::Arc};

pub(crate) struct ActionsModel {
    pub(crate) on_press: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_release: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_hover: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_hover_out: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_over: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_over_out: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_mouse_move: Option<Arc<dyn Fn(&mut EventContext, f32, f32) + Send + Sync>>,
    pub(crate) on_mouse_down: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    pub(crate) on_mouse_up: Option<Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>>,
    pub(crate) on_focus_in: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_focus_out: Option<Arc<dyn Fn(&mut EventContext) + Send + Sync>>,
    pub(crate) on_geo_changed:
        Option<Arc<dyn Fn(&mut EventContext, GeometryChanged) + Send + Sync>>,
}

impl ActionsModel {
    pub(crate) fn new() -> Self {
        Self {
            on_press: None,
            on_release: None,
            on_hover: None,
            on_hover_out: None,
            on_over: None,
            on_over_out: None,
            on_mouse_move: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_focus_in: None,
            on_focus_out: None,
            on_geo_changed: None,
        }
    }
}

impl Model for ActionsModel {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|actions_event, _| match actions_event {
            ActionsEvent::OnPress(on_press) => {
                self.on_press = Some(on_press.clone());
            }

            ActionsEvent::OnRelease(on_release) => {
                self.on_release = Some(on_release.clone());
            }

            ActionsEvent::OnHover(on_hover) => {
                self.on_hover = Some(on_hover.clone());
            }

            ActionsEvent::OnHoverOut(on_hover_out) => {
                self.on_hover_out = Some(on_hover_out.clone());
            }

            ActionsEvent::OnOver(on_over) => {
                self.on_over = Some(on_over.clone());
            }

            ActionsEvent::OnOverOut(on_over_out) => {
                self.on_over_out = Some(on_over_out.clone());
            }

            ActionsEvent::OnMouseMove(on_move) => {
                self.on_mouse_move = Some(on_move.clone());
            }

            ActionsEvent::OnMouseDown(on_mouse_down) => {
                self.on_mouse_down = Some(on_mouse_down.clone());
            }

            ActionsEvent::OnMouseUp(on_mouse_up) => {
                self.on_mouse_up = Some(on_mouse_up.clone());
            }

            ActionsEvent::OnFocusIn(on_focus_in) => {
                self.on_focus_in = Some(on_focus_in.clone());
            }

            ActionsEvent::OnFocusOut(on_focus_out) => {
                self.on_focus_out = Some(on_focus_out.clone());
            }

            ActionsEvent::OnGeoChanged(on_geo_changed) => {
                self.on_geo_changed = Some(on_geo_changed.clone());
            }
        });

        event.map(|window_event, meta| match window_event {
            WindowEvent::TriggerDown { mouse } => {
                let over = if *mouse { cx.hovered() } else { cx.focused() };
                if cx.current() != over && !over.is_descendant_of(cx.tree, cx.current()) {
                    return;
                }
                if let Some(action) = &self.on_press {
                    (action)(cx);
                }
            }

            WindowEvent::TriggerUp { .. } => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_release {
                        (action)(cx);
                    }

                    cx.release();
                }
            }

            WindowEvent::MouseEnter => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_hover {
                        (action)(cx);
                    }
                }
            }

            WindowEvent::MouseLeave => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_hover_out {
                        (action)(cx);
                    }
                }
            }

            WindowEvent::MouseOver => {
                if let Some(action) = &self.on_over {
                    (action)(cx);
                }
            }

            WindowEvent::MouseOut => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_over_out {
                        (action)(cx);
                    }
                }
            }

            WindowEvent::MouseMove(x, y) => {
                if let Some(action) = &self.on_mouse_move {
                    (action)(cx, *x, *y);
                }
            }

            WindowEvent::MouseDown(mouse_button) => {
                if let Some(action) = &self.on_mouse_down {
                    (action)(cx, *mouse_button);
                }
            }

            WindowEvent::MouseUp(mouse_button) => {
                if let Some(action) = &self.on_mouse_up {
                    (action)(cx, *mouse_button);
                }
            }

            WindowEvent::FocusIn => {
                if let Some(action) = &self.on_focus_in {
                    (action)(cx);
                }
            }

            WindowEvent::FocusOut => {
                if let Some(action) = &self.on_focus_out {
                    (action)(cx);
                }
            }

            WindowEvent::GeometryChanged(geo) => {
                if meta.target == cx.current() {
                    if let Some(action) = &self.on_geo_changed {
                        (action)(cx, *geo);
                    }
                }
            }

            _ => {}
        });
    }
}

pub(crate) enum ActionsEvent {
    OnPress(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnRelease(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnHover(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnHoverOut(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnOver(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnOverOut(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnMouseMove(Arc<dyn Fn(&mut EventContext, f32, f32) + Send + Sync>),
    OnMouseDown(Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>),
    OnMouseUp(Arc<dyn Fn(&mut EventContext, MouseButton) + Send + Sync>),
    OnFocusIn(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnFocusOut(Arc<dyn Fn(&mut EventContext) + Send + Sync>),
    OnGeoChanged(Arc<dyn Fn(&mut EventContext, GeometryChanged) + Send + Sync>),
}

/// Modifiers which add an action callback to a view.
pub trait ActionModifiers {
    /// Adds a callback which is performed when the the view receives the [`TriggerDown`](crate::prelude::WindowEvent::TriggerDown) event.
    /// By default a view receives the [`TriggerDown`](crate::prelude::WindowEvent::TriggerDown) event when the left mouse button is pressed on the view,
    /// or when the space or enter keys are pressed while the view is focused.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_press(|_| println!("View was pressed!"));
    /// ```
    fn on_press<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the the view receives the [`TriggerUp`](crate::prelude::WindowEvent::TriggerUp) event.
    /// By default a view receives the [`TriggerUp`](crate::prelude::WindowEvent::TriggerUp) event when the left mouse button is released on the view,
    /// or when the space or enter keys are released while the view is focused.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_release(|_| println!("View was released!"));
    /// ```
    fn on_release<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves over a view.
    /// This callback is not triggered when the mouse pointer moves over an overlapping child of the view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_hover(|_| println!("Mouse cursor entered the view!"));
    /// ```
    fn on_hover<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves away from a view.
    /// This callback is not triggered when the mouse pointer moves away from an overlapping child of the view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_hover_out(|_| println!("Mouse cursor left the view!"));
    /// ```
    fn on_hover_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves over the bounds of a view,
    /// including any overlapping children.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_over(|_| println!("Mouse cursor entered the view bounds!"));
    /// ```
    fn on_over<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves away from the bounds of a view,
    /// including any overlapping children.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_over_out(|_| println!("Mouse cursor left the view bounds!"));
    /// ```
    fn on_over_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the mouse pointer moves within the bounds of a view.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_mouse_move(|_, x, y| println!("Cursor moving: {} {}", x, y));
    /// ```
    fn on_mouse_move<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, f32, f32) + Send + Sync;

    /// Adds a callback which is performed when a mouse button is pressed on the view.
    /// Unlike the `on_press` callback, this callback is triggered for all mouse buttons and not for any keyboard keys.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_mouse_down(|_, button| println!("Mouse button, {:?}, was pressed!", button));
    /// ```
    fn on_mouse_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, MouseButton) + Send + Sync;

    /// Adds a callback which is performed when a mouse button is released on the view.
    /// Unlike the `on_release` callback, this callback is triggered for all mouse buttons and not for any keyboard keys.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_mouse_up(|_, button| println!("Mouse button, {:?}, was released!", button));
    /// ```
    fn on_mouse_up<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, MouseButton) + Send + Sync;

    /// Adds a callback which is performed when the view gains keyboard focus.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_focus_in(|_| println!("View gained keyboard focus!"));
    /// ```
    fn on_focus_in<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the view loses keyboard focus.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_focus_out(|_| println!("View lost keyboard focus!"));
    /// ```
    fn on_focus_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync;

    /// Adds a callback which is performed when the the view changes size or position after layout.
    ///
    /// # Example
    /// ```rust
    /// # use vizia_core::prelude::*;
    /// # let mut cx = &mut Context::new();
    /// Element::new(cx).on_geo_changed(|_, _| println!("View geometry changed!"));
    /// ```
    fn on_geo_changed<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, GeometryChanged) + Send + Sync;
}

// If the entity doesn't have an `ActionsModel` then add one to the entity
fn build_action_model(cx: &mut Context, entity: Entity) {
    if cx
        .data
        .get(entity)
        .and_then(|model_data_store| model_data_store.models.get(&TypeId::of::<ActionsModel>()))
        .is_none()
    {
        cx.with_current(entity, |cx| {
            ActionsModel::new().build(cx);
        });
    }
}

impl<'a, V: View> ActionModifiers for Handle<'a, V> {
    fn on_press<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnPress(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_release<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnRelease(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_hover<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnHover(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_hover_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnHoverOut(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_over<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnOver(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_over_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnOverOut(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_move<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, f32, f32) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseMove(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_down<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, MouseButton) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseDown(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_mouse_up<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, MouseButton) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnMouseUp(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_focus_in<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnFocusIn(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_focus_out<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnFocusOut(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }

    fn on_geo_changed<F>(self, action: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, GeometryChanged) + Send + Sync,
    {
        build_action_model(self.cx, self.entity);

        self.cx.emit_custom(
            Event::new(ActionsEvent::OnGeoChanged(Arc::new(action)))
                .target(self.entity)
                .origin(self.entity),
        );

        self
    }
}
