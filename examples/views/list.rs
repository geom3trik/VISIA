mod helpers;
use helpers::*;
use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    list: Vec<u32>,
}

impl Model for AppData {}

fn main() {
    Application::new(|cx| {
        view_controls(cx);

        cx.add_theme(include_str!("../resources/list_style.css"));

        let list: Vec<u32> = (10..14u32).collect();
        AppData { list }.build(cx);

        VStack::new(cx, |cx| {
            List::new(cx, AppData::list, |cx, _, item| {
                Label::new(cx, item);
            });
        })
        .disabled(ControlsData::disabled)
        .class("container");
    })
    .title("List")
    .run();
}
