use vizia::*;

#[derive(Lens)]
pub struct AppData {
    title: String,
    inner_size: (u32, u32),
}

pub enum AppEvent {
    SetTitle(String),
    SetWidth(u32),
}

impl Model for AppData {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetTitle(title) => {
                self.title = title.clone();
            }

            AppEvent::SetWidth(width) => {
                self.inner_size.0 = *width;
            }
        });
    }
}

fn main() {
    Application::new(|cx| {
        AppData { title: "Window Modifiers".to_owned(), inner_size: (400, 400) }.build(cx);

        VStack::new(cx, |cx| {
            Textbox::new(cx, AppData::title).width(Pixels(200.0)).on_submit(|cx, txt| {
                println!("Do This: {}", txt);
                cx.emit(AppEvent::SetTitle(txt.clone()));
            });

            Slider::new(cx, AppData::inner_size.map(|size| size.0 as f32))
                .range(100.0..1000.0)
                .on_changing(|cx, val| cx.emit(AppEvent::SetWidth(val as u32)))
                .width(Pixels(200.0));
        })
        .row_between(Pixels(20.0))
        .child_space(Pixels(10.0));
    })
    .title(AppData::title)
    .inner_size(AppData::inner_size)
    .run();
}
