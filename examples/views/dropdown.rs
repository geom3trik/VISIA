mod helpers;
use helpers::*;
use vizia::prelude::*;

#[derive(Lens, Model, Setter)]
pub struct AppData {
    list: Vec<String>,
    choice: String,
}

fn main() {
    Application::new(|cx| {
        AppData {
            list: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
            choice: "Red".to_string(),
        }
        .build(cx);

        view_controls(cx);

        VStack::new(cx, |cx| {
            // Dropdown List
            Dropdown::new(
                cx,
                move |cx| Label::new(cx, AppData::choice),
                move |cx| {
                    List::new(cx, AppData::list, |cx, _, item| {
                        Label::new(cx, item)
                            .width(Stretch(1.0))
                            //.child_top(Stretch(1.0))
                            //.child_bottom(Stretch(1.0))
                            .cursor(CursorIcon::Hand)
                            .bind(AppData::choice, move |handle, selected| {
                                if item.get(handle.cx) == selected.get(handle.cx) {
                                    handle.checked(true);
                                }
                            })
                            .on_press(move |cx| {
                                println!("Do this");
                                cx.emit(AppDataSetter::Choice(item.get(cx).clone()));
                                cx.emit(PopupEvent::Close);
                            });
                    });
                },
            )
            .top(Pixels(40.0))
            .width(Pixels(100.0));
        })
        .disabled(ControlsData::disabled)
        .class("container");
    })
    .title("Dropdown")
    .inner_size((350, 300))
    .run();
}
