use vizia::*;

#[derive(Lens)]
pub struct AppData {
    list: Vec<u32>,
}

impl Model for AppData {}

fn main() {
    Application::new(|cx| {
        cx.add_theme(include_str!("../resources/list_style.css"));

        let list: Vec<u32> = (10..14u32).collect();
        AppData { list }.build(cx);

        List::new(cx, AppData::list, |cx, _, item| {
            Label::new(cx, item);
        })
        .space(Stretch(1.0));
    })
    .title("List")
    .run();
}
