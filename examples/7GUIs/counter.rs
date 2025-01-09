use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    count: i32,
}

pub enum AppEvent {
    Increment,
}

impl Model for AppData {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::Increment => self.count += 1,
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData { count: 0 }.build(cx);

        HStack::new(cx, |cx| {
            Button::new(cx, |cx| Label::new(cx, "Increment"))
                .on_press(|cx| cx.emit(AppEvent::Increment));
            Label::new(cx, AppData::count).width(Pixels(50.0)).live(Live::Polite);
        })
        .alignment(Alignment::Center)
        .horizontal_gap(Pixels(50.0));
    })
    .title("Counter")
    .inner_size((400, 100))
    .run()
}
