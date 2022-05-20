use crate::{
    convert::{scan_code_to_code, virtual_key_code_to_code, virtual_key_code_to_key},
    window::Window,
};
use std::cell::RefCell;
use vizia_core::cache::BoundingBox;
#[cfg(not(target_arch = "wasm32"))]
use vizia_core::context::EventProxy;
use vizia_core::events::EventManager;
use vizia_core::fonts;
use vizia_core::prelude::*;
use vizia_core::window::Position;
use winit::{
    dpi::LogicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

pub struct Application {
    context: Context,
    event_loop: EventLoop<Event>,
    builder: Option<Box<dyn Fn(&mut Context)>>,
    on_idle: Option<Box<dyn Fn(&mut Context)>>,
    window_description: WindowDescription,
    should_poll: bool,
}

// TODO uhhhhhhhhhhhhhhhhhhhhhh I think it's a winit bug that EventLoopProxy isn't Send on web
#[cfg(not(target_arch = "wasm32"))]
pub struct WinitEventProxy(EventLoopProxy<Event>);

#[cfg(not(target_arch = "wasm32"))]
impl EventProxy for WinitEventProxy {
    fn send(&self, event: Event) -> Result<(), ()> {
        self.0.send_event(event).map_err(|_| ())
    }

    fn make_clone(&self) -> Box<dyn EventProxy> {
        Box::new(WinitEventProxy(self.0.clone()))
    }
}

impl Application {
    pub fn new<F>(content: F) -> Self
    where
        F: 'static + Fn(&mut Context),
    {
        // wasm + debug: send panics to console
        #[cfg(all(debug_assertions, target_arch = "wasm32"))]
        console_error_panic_hook::set_once();

        #[allow(unused_mut)]
        let mut context = Context::new();

        let event_loop = EventLoop::with_user_event();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let event_proxy_obj = event_loop.create_proxy();
            context.set_event_proxy(Box::new(WinitEventProxy(event_proxy_obj)));
        }

        context.set_current(Entity::root());

        (content)(&mut context);

        Self {
            context,
            event_loop,
            builder: Some(Box::new(content)),
            on_idle: None,
            window_description: WindowDescription::new(),
            should_poll: false,
        }
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

    // TODO - Rename this
    pub fn get_proxy(&self) -> EventLoopProxy<Event> {
        self.event_loop.create_proxy()
    }

    /// Sets the background color of the window.
    pub fn background_color(mut self, color: Color) -> Self {
        self.context.style().background_color.insert(Entity::root(), color);

        self
    }

    /// Starts the application and enters the main event loop.
    pub fn run(mut self) {
        let mut context = self.context;
        let event_loop = self.event_loop;

        // let handle = ContextBuilder::new()
        //     .with_vsync(true)
        //     .build_windowed(WindowBuilder::new(), &event_loop)
        //     .expect("Failed to build windowed context");

        // let handle = unsafe { handle.make_current().unwrap() };

        // let renderer = OpenGl::new(|s| handle.context().get_proc_address(s) as *const _)
        //     .expect("Cannot create renderer");
        // let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        let mut window = Window::new(&event_loop, &self.window_description);

        // let font = canvas.add_font_mem(FONT).expect("Failed to load font");

        // context.fonts = vec![font];

        let regular_font = fonts::ROBOTO_REGULAR;
        let bold_font = fonts::ROBOTO_BOLD;
        let icon_font = fonts::ENTYPO;
        let emoji_font = fonts::OPEN_SANS_EMOJI;
        let arabic_font = fonts::AMIRI_REGULAR;
        let material_font = fonts::MATERIAL_ICONS_REGULAR;

        context.add_font_mem("roboto", regular_font);
        context.add_font_mem("roboto-bold", bold_font);
        context.add_font_mem("icons", icon_font);
        context.add_font_mem("emoji", emoji_font);
        context.add_font_mem("arabic", arabic_font);
        context.add_font_mem("material", material_font);

        context.style().default_font = "roboto".to_string();

        // Load resources
        context.synchronize_fonts(&mut window.canvas);

        let dpi_factor = window.window().scale_factor();

        let physical_size = window.window().inner_size();

        let clear_color =
            context.style().background_color.get(Entity::root()).cloned().unwrap_or_default();

        window.canvas.set_size(physical_size.width as u32, physical_size.height as u32, 1.0);
        window.canvas.clear_rect(
            0,
            0,
            physical_size.width as u32,
            physical_size.height as u32,
            clear_color.into(),
        );

        context.style().dpi_factor = window.window().scale_factor();

        context.views.insert(Entity::root(), Box::new(window));

        let logical_size: LogicalSize<f32> = physical_size.to_logical(dpi_factor);

        context.cache().set_width(Entity::root(), physical_size.width as f32);
        context.cache().set_height(Entity::root(), physical_size.height as f32);

        context.style().width.insert(Entity::root(), Units::Pixels(logical_size.width));
        context.style().height.insert(Entity::root(), Units::Pixels(logical_size.height));

        context.style().pseudo_classes.insert(Entity::root(), PseudoClass::default()).unwrap();
        context.style().disabled.insert(Entity::root(), false);

        let bounding_box = BoundingBox {
            w: physical_size.width as f32,
            h: physical_size.height as f32,
            ..Default::default()
        };

        context.cache().set_clip_region(Entity::root(), bounding_box);

        let mut event_manager = EventManager::new();

        // if let Some(builder) = self.builder.take() {
        //     (builder)(&mut context);

        //     self.builder = Some(builder);
        // }

        let builder = self.builder.take();

        let on_idle = self.on_idle.take();

        let event_loop_proxy = event_loop.create_proxy();

        let default_should_poll = self.should_poll;
        let stored_control_flow = RefCell::new(ControlFlow::Poll);

        event_loop.run(move |event, _, control_flow| {
            match event {
                winit::event::Event::UserEvent(event) => {
                    context.emit_custom(event);
                }

                winit::event::Event::MainEventsCleared => {
                    *stored_control_flow.borrow_mut() =
                        if default_should_poll { ControlFlow::Poll } else { ControlFlow::Wait };

                    // Rebuild application if required
                    if context.environment().needs_rebuild {
                        context.set_current(Entity::root());
                        context.remove_children(Entity::root());
                        if let Some(builder) = &builder {
                            (builder)(&mut context);
                        }
                        context.environment().needs_rebuild = false;
                    }

                    if let Some(mut window_view) = context.views.remove(&Entity::root()) {
                        if let Some(window) = window_view.downcast_mut::<Window>() {
                            context.synchronize_fonts(&mut window.canvas);
                        }

                        context.views.insert(Entity::root(), window_view);
                    }

                    // Events
                    while event_manager.flush_events(&mut context) {}

                    context.process_data_updates();
                    context.process_style_updates();

                    if context.has_animations() {
                        *stored_control_flow.borrow_mut() = ControlFlow::Poll;

                        //context.insert_event(Event::new(WindowEvent::Relayout).target(Entity::root()));
                        event_loop_proxy.send_event(Event::new(WindowEvent::Redraw)).unwrap();
                        //window.handle.window().request_redraw();
                        if let Some(window_event_handler) = context.views.remove(&Entity::root()) {
                            if let Some(window) = window_event_handler.downcast_ref::<Window>() {
                                window.window().request_redraw();
                            }

                            context.views.insert(Entity::root(), window_event_handler);
                        }
                    }

                    context.apply_animations();

                    context.process_visual_updates();

                    if let Some(window_view) = context.views.remove(&Entity::root()) {
                        if let Some(window) = window_view.downcast_ref::<Window>() {
                            if context.style().needs_redraw {
                                window.window().request_redraw();
                                context.style().needs_redraw = false;
                            }
                        }

                        context.views.insert(Entity::root(), window_view);
                    }

                    if let Some(idle_callback) = &on_idle {
                        context.set_current(Entity::root());
                        (idle_callback)(&mut context);
                    }

                    if context.has_queued_events() {
                        *stored_control_flow.borrow_mut() = ControlFlow::Poll;
                        event_loop_proxy.send_event(Event::new(())).expect("Failed to send event");
                    }
                }

                winit::event::Event::RedrawRequested(_) => {
                    // Redraw here
                    context_draw(&mut context);
                }

                winit::event::Event::WindowEvent { window_id: _, event } => {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            *stored_control_flow.borrow_mut() = ControlFlow::Exit;
                        }

                        winit::event::WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            new_inner_size,
                        } => {
                            context.style().dpi_factor = scale_factor;
                            context.cache().set_width(Entity::root(), new_inner_size.width as f32);
                            context
                                .cache()
                                .set_height(Entity::root(), new_inner_size.height as f32);

                            let logical_size: LogicalSize<f32> =
                                new_inner_size.to_logical(context.style().dpi_factor);

                            context
                                .style()
                                .width
                                .insert(Entity::root(), Units::Pixels(logical_size.width as f32));

                            context
                                .style()
                                .height
                                .insert(Entity::root(), Units::Pixels(logical_size.height as f32));
                        }

                        #[allow(deprecated)]
                        winit::event::WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                            modifiers: _,
                        } => {
                            context.dispatch_system_event(WindowEvent::MouseMove(
                                position.x as f32,
                                position.y as f32,
                            ));
                        }

                        #[allow(deprecated)]
                        winit::event::WindowEvent::MouseInput {
                            device_id: _,
                            button,
                            state,
                            modifiers: _,
                        } => {
                            let button = match button {
                                winit::event::MouseButton::Left => MouseButton::Left,
                                winit::event::MouseButton::Right => MouseButton::Right,
                                winit::event::MouseButton::Middle => MouseButton::Middle,
                                winit::event::MouseButton::Other(val) => MouseButton::Other(val),
                            };

                            let event = match state {
                                winit::event::ElementState::Pressed => {
                                    WindowEvent::MouseDown(button)
                                }
                                winit::event::ElementState::Released => {
                                    WindowEvent::MouseUp(button)
                                }
                            };

                            context.dispatch_system_event(event);
                        }

                        winit::event::WindowEvent::MouseWheel { delta, phase: _, .. } => {
                            let out_event = match delta {
                                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                                    WindowEvent::MouseScroll(x, y)
                                }
                                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                    WindowEvent::MouseScroll(
                                        pos.x as f32 / 20.0,
                                        pos.y as f32 / 114.0,
                                    )
                                }
                            };

                            context.dispatch_system_event(out_event);
                        }

                        winit::event::WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                            is_synthetic: _,
                        } => {
                            // Prefer virtual keycodes to scancodes, as scancodes aren't uniform between platforms
                            let code = if let Some(vkey) = input.virtual_keycode {
                                virtual_key_code_to_code(vkey)
                            } else {
                                scan_code_to_code(input.scancode)
                            };

                            let key = virtual_key_code_to_key(
                                input.virtual_keycode.unwrap_or(VirtualKeyCode::NoConvert),
                            );
                            let event = match input.state {
                                winit::event::ElementState::Pressed => {
                                    WindowEvent::KeyDown(code, key)
                                }
                                winit::event::ElementState::Released => {
                                    WindowEvent::KeyUp(code, key)
                                }
                            };

                            context.dispatch_system_event(event);
                        }

                        winit::event::WindowEvent::ReceivedCharacter(character) => {
                            context.dispatch_system_event(WindowEvent::CharInput(character));
                        }

                        winit::event::WindowEvent::Resized(physical_size) => {
                            if let Some(mut window_view) = context.views.remove(&Entity::root()) {
                                if let Some(window) = window_view.downcast_mut::<Window>() {
                                    window.resize(physical_size);
                                }

                                context.views.insert(Entity::root(), window_view);
                            }

                            let logical_size: LogicalSize<f32> =
                                physical_size.to_logical(context.style().dpi_factor);

                            context
                                .style()
                                .width
                                .insert(Entity::root(), Units::Pixels(logical_size.width as f32));

                            context
                                .style()
                                .height
                                .insert(Entity::root(), Units::Pixels(logical_size.height as f32));

                            context.cache().set_width(Entity::root(), physical_size.width as f32);
                            context.cache().set_height(Entity::root(), physical_size.height as f32);

                            let bounding_box = BoundingBox {
                                w: physical_size.width as f32,
                                h: physical_size.height as f32,
                                ..Default::default()
                            };

                            context.cache().set_clip_region(Entity::root(), bounding_box);

                            context.need_restyle();
                            context.need_relayout();
                            context.need_redraw();

                            // let mut bounding_box = BoundingBox::default();
                            // bounding_box.w = size.width as f32;
                            // bounding_box.h = size.height as f32;

                            // context.cache.set_clip_region(Entity::root(), bounding_box);
                        }

                        winit::event::WindowEvent::ModifiersChanged(modifiers_state) => {
                            context.modifiers().set(Modifiers::SHIFT, modifiers_state.shift());
                            context.modifiers().set(Modifiers::ALT, modifiers_state.alt());
                            context.modifiers().set(Modifiers::CTRL, modifiers_state.ctrl());
                            context.modifiers().set(Modifiers::LOGO, modifiers_state.logo());
                        }

                        _ => {}
                    }
                }

                _ => {}
            }

            *control_flow = *stored_control_flow.borrow();
        });
    }
}

impl WindowModifiers for Application {
    fn title<T: ToString>(mut self, title: impl Res<T>) -> Self {
        self.window_description.title = title.get_val(&self.context).to_string();
        title.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetTitle(val.to_string()));
        });

        self
    }

    fn inner_size<S: Into<WindowSize>>(mut self, size: impl Res<S>) -> Self {
        self.window_description.inner_size = size.get_val(&self.context).into();
        size.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetSize(val.into()));
        });

        self
    }

    fn min_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.min_inner_size =
            size.get_val(&self.context).map(|size| size.into());
        size.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetMinSize(val.map(|size| size.into())));
        });

        self
    }

    fn max_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.max_inner_size =
            size.get_val(&self.context).map(|size| size.into());
        size.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetMaxSize(val.map(|size| size.into())));
        });

        self
    }

    fn position<P: Into<Position>>(mut self, position: impl Res<P>) -> Self {
        self.window_description.position = Some(position.get_val(&self.context).into());
        position.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetPosition(val.into()));
        });

        self
    }

    fn resizable(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.resizable = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetResizable(val));
        });

        self
    }

    fn minimized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.minimized = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetMinimized(val));
        });

        self
    }

    fn maximized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.maximized = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetMaximized(val));
        });

        self
    }

    fn visible(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.visible = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetVisible(val));
        });

        self
    }

    fn transparent(mut self, flag: bool) -> Self {
        self.window_description.transparent = flag;

        self
    }

    fn decorations(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.decorations = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetDecorations(val));
        });

        self
    }

    fn always_on_top(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.always_on_top = flag.get_val(&self.context);
        flag.set_or_bind(&mut self.context, Entity::root(), |cx, _, val| {
            cx.emit(WindowEvent::SetAlwaysOnTop(val));
        });

        self
    }

    fn vsync(mut self, flag: bool) -> Self {
        self.window_description.vsync = flag;

        self
    }

    fn icon(mut self, image: Vec<u8>, width: u32, height: u32) -> Self {
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

impl Env for Application {
    fn ignore_default_styles(mut self) -> Self {
        if self.context.environment().include_default_theme {
            self.context.environment().include_default_theme = false;
            self.context.environment().needs_rebuild = true;
            self.context.reload_styles().expect("Failed to reload styles");
        }

        self
    }
}

// fn debug(cx: &mut Context, entity: Entity) -> String {
//     if let Some(view) = cx.views.get(&entity) {
//         view.debug(entity)
//     } else {
//         "None".to_string()
//     }
// }

fn context_draw(cx: &mut Context) {
    if let Some(mut window_view) = cx.views.remove(&Entity::root()) {
        if let Some(window) = window_view.downcast_mut::<Window>() {
            cx.draw(&mut window.canvas);
            window.swap_buffers();
        }

        cx.views.insert(Entity::root(), window_view);
    }
}
