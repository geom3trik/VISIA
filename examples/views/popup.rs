use vizia::prelude::*;

fn main() {
    Application::new(|cx| {
        PopupData::default().build(cx);

        Button::new(cx, |cx| cx.emit(PopupEvent::Switch), |cx| Label::new(cx, "Open"));

        Popup::new(cx, PopupData::is_open, |_| {})
            .something(|cx| cx.emit(PopupEvent::Close))
            .space(Pixels(100.0))
            .size(Pixels(200.0))
            .background_color(Color::red());
    })
    .title("Popup")
    .run();
}
