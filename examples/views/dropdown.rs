mod helpers;
use helpers::*;
use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    list: Vec<String>,
    choice: String,
}

pub enum AppEvent {
    SetChoice(String),
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetChoice(choice) => {
                self.choice = choice.clone();
            }
        })
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData {
            list: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
            choice: "Red".to_string(),
        }
        .build(cx);

        ExamplePage::new(cx, |cx| {
            Dropdown::new(
                cx,
                move |cx| {
                    Button::new(cx, |cx| Label::new(cx, AppData::choice))
                        .on_press(|cx| cx.emit(PopupEvent::Switch));
                },
                move |cx| {
                    List::new(cx, AppData::list, |cx, _, item| {
                        Label::new(cx, item)
                            .cursor(CursorIcon::Hand)
                            .bind(AppData::choice, move |handle, selected| {
                                if item.get(&handle) == selected.get(&handle) {
                                    handle.checked(true);
                                }
                            })
                            .on_press(move |cx| {
                                cx.emit(AppEvent::SetChoice(item.get(cx)));
                                cx.emit(PopupEvent::Close);
                            });
                    });
                },
            )
            .top(Pixels(40.0))
            .width(Pixels(100.0));
        });
    })
    .title("Dropdown")
    .inner_size((350, 300))
    .run()
}
