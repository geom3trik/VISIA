use crate::{
    convert::{winit_key_code_to_code, winit_key_to_key},
    window::Window,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
use accesskit::{Action, NodeBuilder, NodeId, TreeUpdate};
#[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
use accesskit_winit;
use std::cell::RefCell;
use vizia_core::backend::*;
#[cfg(not(target_arch = "wasm32"))]
use vizia_core::context::EventProxy;
use vizia_core::prelude::*;
// use vizia_input::KeyState;
#[cfg(all(
    feature = "clipboard",
    feature = "wayland",
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )
))]
use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle};
use vizia_window::Position;

use winit::{
    error::EventLoopError, event::ElementState, event_loop::EventLoopBuilder, keyboard::PhysicalKey,
};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    keyboard::NativeKeyCode,
};

#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum UserEvent {
    Event(Event),
    #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
    AccessKitActionRequest(accesskit_winit::ActionRequestEvent),
}

#[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
impl From<accesskit_winit::ActionRequestEvent> for UserEvent {
    fn from(action_request_event: accesskit_winit::ActionRequestEvent) -> Self {
        UserEvent::AccessKitActionRequest(action_request_event)
    }
}

impl From<vizia_core::events::Event> for UserEvent {
    fn from(event: vizia_core::events::Event) -> Self {
        UserEvent::Event(event)
    }
}

type IdleCallback = Option<Box<dyn Fn(&mut Context)>>;

#[derive(Debug)]
pub enum ApplicationError {
    EventLoopError(EventLoopError),
    LogError,
}

///Creating a new application creates a root `Window` and a `Context`. Views declared within the closure passed to `Application::new()` are added to the context and rendered into the root window.
///
/// # Example
/// ```no_run
/// # use vizia_core::prelude::*;
/// # use vizia_winit::application::Application;
/// Application::new(|cx|{
///    // Content goes here
/// })
/// .run();
///```
/// Calling `run()` on the `Application` causes the program to enter the event loop and for the main window to display.
pub struct Application {
    context: Context,
    event_loop: EventLoop<UserEvent>,
    on_idle: IdleCallback,
    window_description: WindowDescription,
    should_poll: bool,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WinitEventProxy(EventLoopProxy<UserEvent>);

#[cfg(not(target_arch = "wasm32"))]
impl EventProxy for WinitEventProxy {
    fn send(&self, event: Event) -> Result<(), ()> {
        self.0.send_event(UserEvent::Event(event)).map_err(|_| ())
    }

    fn make_clone(&self) -> Box<dyn EventProxy> {
        Box::new(WinitEventProxy(self.0.clone()))
    }
}

impl Application {
    pub fn new<F>(content: F) -> Self
    where
        F: 'static + FnOnce(&mut Context),
    {
        // wasm + debug: send panics to console
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        console_error_panic_hook::set_once();

        // TODO: User scale factors and window resizing has not been implement for winit
        // TODO: Changing the scale factor doesn't work for winit anyways since winit doesn't let
        //       you resize the window, so there's no mutator for that at he moment
        let mut context = Context::new(WindowSize::new(1, 1), 1.0);

        let event_loop =
            EventLoopBuilder::with_user_event().build().expect("Failed to create event loop");
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut cx = BackendContext::new(&mut context);
            let event_proxy_obj = event_loop.create_proxy();
            cx.set_event_proxy(Box::new(WinitEventProxy(event_proxy_obj)));
        }

        let mut cx = BackendContext::new(&mut context);
        cx.renegotiate_language();
        cx.0.remove_user_themes();
        (content)(cx.0);

        Self {
            context,
            event_loop,
            on_idle: None,
            window_description: WindowDescription::new(),
            should_poll: false,
        }
    }

    /// Sets the default built-in theming to be ignored.
    pub fn ignore_default_theme(mut self) -> Self {
        self.context.ignore_default_theme = true;
        self
    }

    pub fn set_text_config(mut self, text_config: TextConfig) -> Self {
        BackendContext::new(&mut self.context).set_text_config(text_config);
        self
    }

    pub fn should_poll(mut self) -> Self {
        self.should_poll = true;

        self
    }

    /// Takes a closure which will be called at the end of every loop of the application.
    ///
    /// The callback provides a place to run 'idle' processing and happens at the end of each loop but before drawing.
    /// If the callback pushes events into the queue in state then the event loop will re-run. Care must be taken not to
    /// push events into the queue every time the callback runs unless this is intended.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use vizia_core::prelude::*;
    /// # use vizia_winit::application::Application;
    /// #
    /// Application::new(|cx| {
    ///     // Build application here
    /// })
    /// .on_idle(|cx| {
    ///     // Code here runs at the end of every event loop after OS and vizia events have been handled
    /// })
    /// .run();
    /// ```
    pub fn on_idle<F: 'static + Fn(&mut Context)>(mut self, callback: F) -> Self {
        self.on_idle = Some(Box::new(callback));

        self
    }

    /// Returns a `ContextProxy` which can be used to send events from another thread.
    pub fn get_proxy(&self) -> ContextProxy {
        self.context.get_proxy()
    }

    /// Starts the application and enters the main event loop.
    pub fn run(mut self) -> Result<(), ApplicationError> {
        let mut context = self.context;

        let event_loop = self.event_loop;

        let (window, canvas) = Window::new(&event_loop, &self.window_description);

        // On windows cloak (hide) the window initially, we later reveal it after the first draw.
        // This is a workaround to hide the "white flash" that occurs during application startup.
        #[cfg(target_os = "windows")]
        let mut is_initially_cloaked = window.set_cloak(true);

        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
        let event_loop_proxy = event_loop.create_proxy();

        let mut cx = BackendContext::new(&mut context);

        // update the sys theme if any
        if let Some(theme) = window.window().theme() {
            let theme = match theme {
                winit::window::Theme::Light => ThemeMode::LightMode,
                winit::window::Theme::Dark => ThemeMode::DarkMode,
            };
            cx.emit_origin(WindowEvent::ThemeChanged(theme));
        }

        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
        let root_node = NodeBuilder::new(Role::Window).build(cx.accesskit_node_classes());
        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
        let accesskit = accesskit_winit::Adapter::new(
            window.window(),
            move || {
                // TODO: set a flag to signify that a screen reader has been attached
                use accesskit::Tree;

                let root_id = Entity::root().accesskit_id();

                // Build initial tree here

                TreeUpdate {
                    nodes: vec![(root_id, root_node)],
                    tree: Some(Tree::new(root_id)),
                    focus: Entity::root().accesskit_id(),
                }
            },
            event_loop_proxy,
        );

        // Accesskit requires that the window starts invisible until accesskit has been initialised.
        // At this point we can set the visibility based on the desired visibility from the window description.
        window.window().set_visible(self.window_description.visible);

        #[cfg(all(
            feature = "clipboard",
            feature = "wayland",
            any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        ))]
        unsafe {
            let display = window.window().raw_display_handle();
            if let RawDisplayHandle::Wayland(display_handle) = display {
                let (_, clipboard) = copypasta::wayland_clipboard::create_clipboards_from_external(
                    display_handle.display,
                );
                cx.set_clipboard_provider(Box::new(clipboard));
            }
        }

        let scale_factor = window.window().scale_factor() as f32;
        cx.add_main_window(&self.window_description, canvas, scale_factor);
        cx.add_window(window);

        cx.0.remove_user_themes();

        let on_idle = self.on_idle.take();

        let event_loop_proxy = event_loop.create_proxy();

        let default_should_poll = self.should_poll;
        let stored_control_flow = RefCell::new(ControlFlow::Poll);

        // #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
        // cx.process_tree_updates(|tree_updates| {
        //     for update in tree_updates.iter_mut() {
        //         accesskit.update_if_active(|| update.take().unwrap());
        //     }
        // });

        let mut cursor_moved = false;
        let mut cursor = (0.0f32, 0.0f32);

        // cx.process_events();

        cx.process_data_updates();
        cx.process_style_updates();
        cx.process_visual_updates();

        let mut main_events = false;
        event_loop
            .run(move |event, elwt| {
                let mut cx = BackendContext::new_with_event_manager(&mut context);

                match event {
                    winit::event::Event::NewEvents(_) => {
                        cx.process_timers();
                        cx.emit_scheduled_events();
                    }

                    winit::event::Event::UserEvent(user_event) => match user_event {
                        UserEvent::Event(event) => {
                            cx.send_event(event);
                        }

                        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
                        UserEvent::AccessKitActionRequest(action_request_event) => {
                            let node_id = action_request_event.request.target;

                            if action_request_event.request.action != Action::ScrollIntoView {
                                let entity = Entity::new(node_id.0 as u64, 0);

                                // Handle focus action from screen reader
                                if action_request_event.request.action == Action::Focus {
                                    cx.0.with_current(entity, |cx| {
                                        cx.focus();
                                    });
                                }

                                cx.send_event(
                                    Event::new(WindowEvent::ActionRequest(
                                        action_request_event.request,
                                    ))
                                    .direct(entity),
                                );
                            }
                        }
                    },

                    winit::event::Event::AboutToWait => {
                        main_events = true;

                        *stored_control_flow.borrow_mut() =
                            if default_should_poll { ControlFlow::Poll } else { ControlFlow::Wait };

                        if cursor_moved {
                            cx.emit_origin(WindowEvent::MouseMove(cursor.0, cursor.1));
                            cursor_moved = false;
                        }

                        cx.process_events();

                        cx.process_style_updates();

                        if cx.process_animations() {
                            *stored_control_flow.borrow_mut() = ControlFlow::Poll;

                            event_loop_proxy
                                .send_event(UserEvent::Event(Event::new(WindowEvent::Redraw)))
                                .expect("Failed to send redraw event");

                            cx.mutate_window(|_, window: &Window| {
                                window.window().request_redraw();
                            });
                        }

                        cx.process_visual_updates();

                        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
                        cx.process_tree_updates(|tree_updates| {
                            for update in tree_updates.iter_mut() {
                                accesskit.update_if_active(|| update.take().unwrap());
                            }
                        });

                        cx.mutate_window(|cx, window: &Window| {
                            cx.style().should_redraw(|| {
                                window.window().request_redraw();
                            });
                        });

                        if let Some(idle_callback) = &on_idle {
                            cx.set_current(Entity::root());
                            (idle_callback)(cx.context());
                        }

                        if cx.has_queued_events() {
                            *stored_control_flow.borrow_mut() = ControlFlow::Poll;
                            event_loop_proxy
                                .send_event(UserEvent::Event(Event::new(())))
                                .expect("Failed to send event");
                        }

                        cx.mutate_window(|_, window: &Window| {
                            if window.should_close {
                                elwt.exit();
                            }
                        });
                    }

                    winit::event::Event::WindowEvent { window_id: _, event } => {
                        #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
                        cx.mutate_window(|_, window: &Window| {
                            accesskit.process_event(window.window(), &event);
                        });

                        match event {
                            winit::event::WindowEvent::RedrawRequested => {
                                if main_events {
                                    // Redraw
                                    cx.draw();
                                    cx.mutate_window(|_, window: &Window| {
                                        // window.window().pre_present_notify();
                                        window.swap_buffers();
                                    });

                                    // Un-cloak
                                    #[cfg(target_os = "windows")]
                                    if is_initially_cloaked {
                                        is_initially_cloaked = false;
                                        cx.draw();
                                        cx.mutate_window(|_, window: &Window| {
                                            window.swap_buffers();
                                            window.set_cloak(false);
                                        });
                                    }
                                }
                            }

                            winit::event::WindowEvent::CloseRequested => {
                                cx.emit_origin(WindowEvent::WindowClose);
                            }

                            winit::event::WindowEvent::Focused(is_focused) => {
                                cx.0.window_has_focus = is_focused;
                                #[cfg(all(not(target_arch = "wasm32"), feature = "accesskit"))]
                                accesskit.update_if_active(|| TreeUpdate {
                                    nodes: vec![],
                                    tree: None,
                                    focus: is_focused
                                        .then_some(cx.focused().accesskit_id())
                                        .unwrap_or(NodeId(0)),
                                });
                            }

                            winit::event::WindowEvent::ScaleFactorChanged {
                                scale_factor,
                                inner_size_writer: _,
                            } => {
                                cx.set_scale_factor(scale_factor);
                                // cx.set_window_size(
                                //     new_inner_size.width as f32,
                                //     new_inner_size.height as f32,
                                // );
                                cx.needs_refresh();
                            }

                            winit::event::WindowEvent::DroppedFile(path) => {
                                cx.emit_origin(WindowEvent::Drop(DropData::File(path)));
                            }

                            #[allow(deprecated)]
                            winit::event::WindowEvent::CursorMoved { device_id: _, position } => {
                                // To avoid calling the hover system multiple times in one frame when multiple cursor moved
                                // events are received, instead we set a flag here and emit the MouseMove event during MainEventsCleared.
                                if !cursor_moved {
                                    cursor_moved = true;
                                    cursor.0 = position.x as f32;
                                    cursor.1 = position.y as f32;
                                }
                            }

                            #[allow(deprecated)]
                            winit::event::WindowEvent::MouseInput {
                                device_id: _,
                                button,
                                state,
                            } => {
                                let button = match button {
                                    winit::event::MouseButton::Left => MouseButton::Left,
                                    winit::event::MouseButton::Right => MouseButton::Right,
                                    winit::event::MouseButton::Middle => MouseButton::Middle,
                                    winit::event::MouseButton::Other(val) => {
                                        MouseButton::Other(val)
                                    }
                                    winit::event::MouseButton::Back => MouseButton::Back,
                                    winit::event::MouseButton::Forward => MouseButton::Forward,
                                };

                                let event = match state {
                                    winit::event::ElementState::Pressed => {
                                        WindowEvent::MouseDown(button)
                                    }
                                    winit::event::ElementState::Released => {
                                        WindowEvent::MouseUp(button)
                                    }
                                };

                                cx.emit_origin(event);
                            }

                            winit::event::WindowEvent::MouseWheel { delta, phase: _, .. } => {
                                let out_event = match delta {
                                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                        WindowEvent::MouseScroll(x, y)
                                    }
                                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                        WindowEvent::MouseScroll(
                                            pos.x as f32 / 20.0,
                                            pos.y as f32 / 20.0, // this number calibrated for wayland
                                        )
                                    }
                                };

                                cx.emit_origin(out_event);
                            }

                            winit::event::WindowEvent::KeyboardInput {
                                device_id: _,
                                event,
                                is_synthetic: _,
                            } => {
                                let code = match event.physical_key {
                                    PhysicalKey::Code(code) => winit_key_code_to_code(code),
                                    PhysicalKey::Unidentified(native) => match native {
                                        NativeKeyCode::Windows(_scancode) => return,
                                        _ => return,
                                    },
                                };

                                let key = match event.logical_key {
                                    winit::keyboard::Key::Named(named_key) => {
                                        winit_key_to_key(named_key)
                                    }
                                    _ => None,
                                };

                                if let winit::keyboard::Key::Character(character) =
                                    event.logical_key
                                {
                                    if event.state == ElementState::Pressed {
                                        cx.emit_origin(WindowEvent::CharInput(
                                            character.as_str().chars().next().unwrap(),
                                        ));
                                    }
                                }

                                let event = match event.state {
                                    winit::event::ElementState::Pressed => {
                                        WindowEvent::KeyDown(code, key)
                                    }
                                    winit::event::ElementState::Released => {
                                        WindowEvent::KeyUp(code, key)
                                    }
                                };

                                cx.emit_origin(event);
                            }

                            winit::event::WindowEvent::Resized(physical_size) => {
                                cx.mutate_window(|_, window: &Window| {
                                    window.resize(physical_size);
                                });

                                cx.set_window_size(
                                    physical_size.width as f32,
                                    physical_size.height as f32,
                                );

                                cx.needs_refresh();

                                #[cfg(target_os = "windows")]
                                {
                                    cx.process_events();

                                    cx.process_style_updates();

                                    if cx.process_animations() {
                                        *stored_control_flow.borrow_mut() = ControlFlow::Poll;

                                        event_loop_proxy
                                            .send_event(UserEvent::Event(Event::new(
                                                WindowEvent::Redraw,
                                            )))
                                            .expect("Failed to send redraw event");

                                        cx.mutate_window(|_, window: &Window| {
                                            window.window().request_redraw();
                                        });
                                    }

                                    cx.process_visual_updates();

                                    #[cfg(all(
                                        not(target_arch = "wasm32"),
                                        feature = "accesskit"
                                    ))]
                                    cx.process_tree_updates(|tree_updates| {
                                        for update in tree_updates.iter_mut() {
                                            accesskit.update_if_active(|| update.take().unwrap());
                                        }
                                    });

                                    cx.mutate_window(|_, window: &Window| {
                                        window.window().request_redraw();
                                    });
                                }
                            }

                            winit::event::WindowEvent::ThemeChanged(theme) => {
                                let theme = match theme {
                                    winit::window::Theme::Light => ThemeMode::LightMode,
                                    winit::window::Theme::Dark => ThemeMode::DarkMode,
                                };
                                cx.emit_origin(WindowEvent::ThemeChanged(theme));
                            }

                            winit::event::WindowEvent::ModifiersChanged(modifiers_state) => {
                                cx.modifiers()
                                    .set(Modifiers::SHIFT, modifiers_state.state().shift_key());

                                cx.modifiers()
                                    .set(Modifiers::ALT, modifiers_state.state().alt_key());

                                cx.modifiers()
                                    .set(Modifiers::CTRL, modifiers_state.state().control_key());

                                cx.modifiers()
                                    .set(Modifiers::SUPER, modifiers_state.state().super_key());
                            }

                            winit::event::WindowEvent::CursorEntered { device_id: _ } => {
                                cx.emit_origin(WindowEvent::MouseEnter);
                            }

                            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                                cx.emit_origin(WindowEvent::MouseLeave);
                            }

                            _ => {}
                        }
                    }

                    _ => {}
                }

                if let Some(timer_time) = cx.get_next_timer_time() {
                    elwt.set_control_flow(ControlFlow::WaitUntil(timer_time));
                } else {
                    elwt.set_control_flow(*stored_control_flow.borrow());
                }
            })
            .map_err(ApplicationError::EventLoopError)?;

        Ok(())
    }
}

impl WindowModifiers for Application {
    fn title<T: ToString>(mut self, title: impl Res<T>) -> Self {
        title.set_or_bind(&mut self.context, Entity::root(), |cx, title| {
            cx.emit(WindowEvent::SetTitle(title.get(cx).to_string()));
        });

        self
    }

    fn inner_size<S: Into<WindowSize>>(mut self, size: impl Res<S>) -> Self {
        self.window_description.inner_size = size.get(&self.context).into();

        size.set_or_bind(&mut self.context, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetSize(size.get(cx).into()));
        });

        self
    }

    fn min_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.min_inner_size = size.get(&self.context).map(|s| s.into());

        size.set_or_bind(&mut self.context, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetMinSize(size.get(cx).map(|s| s.into())));
        });

        self
    }

    fn max_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.max_inner_size = size.get(&self.context).map(|s| s.into());

        size.set_or_bind(&mut self.context, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetMaxSize(size.get(cx).map(|s| s.into())));
        });
        self
    }

    fn position<P: Into<Position>>(mut self, position: impl Res<P>) -> Self {
        self.window_description.position = Some(position.get(&self.context).into());

        position.set_or_bind(&mut self.context, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetPosition(size.get(cx).into()));
        });

        self
    }

    fn resizable(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.resizable = flag.get(&self.context);

        flag.set_or_bind(&mut self.context, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetResizable(flag.get(cx)));
        });

        self
    }

    fn minimized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.minimized = flag.get(&self.context);

        flag.set_or_bind(&mut self.context, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetMinimized(flag.get(cx)));
        });
        self
    }

    fn maximized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.maximized = flag.get(&self.context);

        flag.set_or_bind(&mut self.context, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetMaximized(flag.get(cx)));
        });

        self
    }

    fn visible(mut self, flag: bool) -> Self {
        self.window_description.visible = flag;

        self
    }

    fn transparent(mut self, flag: bool) -> Self {
        self.window_description.transparent = flag;

        self
    }

    fn decorations(mut self, flag: bool) -> Self {
        self.window_description.decorations = flag;

        self
    }

    fn always_on_top(mut self, flag: bool) -> Self {
        self.window_description.always_on_top = flag;
        self
    }

    fn vsync(mut self, flag: bool) -> Self {
        self.window_description.vsync = flag;

        self
    }

    fn icon(mut self, width: u32, height: u32, image: Vec<u8>) -> Self {
        self.window_description.icon = Some(image);
        self.window_description.icon_width = width;
        self.window_description.icon_height = height;

        self
    }

    #[cfg(target_arch = "wasm32")]
    fn canvas(mut self, canvas: &str) -> Self {
        self.window_description.target_canvas = Some(canvas.to_owned());

        self
    }
}
