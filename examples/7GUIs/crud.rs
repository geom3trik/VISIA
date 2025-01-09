use vizia::prelude::*;

const STYLE: &str = r#"
    textbox {
        width: 1s;
    }

    hstack {
        height: auto;
        horizontal-gap: 10px;
        padding-top: 1s;
        padding-bottom: 1s;
    }

    vstack {
        height: 1s;
        vertical-gap: 10px;
    }

    button {
        width: 1s;
        padding: 1s;
    }

    list label {
        width: 1s;
        height: 32px;
        padding-left: 5px;
        padding-top: 1s;
        padding-bottom: 1s;
    }

    list label:checked {
        background-color: #5050AA40;
    }

    list {
        border-color: #d2d2d2;
        border-width: 1px;
        width: 1s;
        height: 1s;
    }
"#;

#[derive(Lens)]
pub struct AppData {
    filter_prefix: String,
    list: Vec<(String, String)>,
    selected: Option<usize>,
    name: String,
    surname: String,
}

pub enum AppEvent {
    SetSelected(usize),
    SetName(String),
    SetSurname(String),
    Create,
    Update,
    Delete,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetSelected(index) => {
                self.selected = Some(*index);
                self.name = self.list[*index].0.clone();
                self.surname = self.list[*index].1.clone();
            }

            AppEvent::SetName(name) => {
                self.name = name.clone();
            }

            AppEvent::SetSurname(surname) => {
                self.surname = surname.clone();
            }

            AppEvent::Create => {
                if !self.name.is_empty() && !self.surname.is_empty() {
                    self.list.push((self.name.clone(), self.surname.clone()));
                    self.selected = Some(self.list.len() - 1);
                }
            }

            AppEvent::Update => {
                if let Some(selected) = self.selected {
                    self.list[selected].0 = self.name.clone();
                    self.list[selected].1 = self.surname.clone();
                }
            }

            AppEvent::Delete => {
                if let Some(selected) = self.selected {
                    self.list.remove(selected);
                    if self.list.is_empty() {
                        self.selected = None;
                        self.name = String::new();
                        self.surname = String::new();
                    } else {
                        cx.emit(AppEvent::SetSelected(selected.saturating_sub(1)));
                    }
                }
            }
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(STYLE).expect("Failed to add stylesheet");

        AppData {
            filter_prefix: "".to_string(),
            list: vec![("John".to_string(), "Smith".to_string())],
            selected: None,
            name: "".to_string(),
            surname: "".to_string(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Filter prefix:");
                        Textbox::new(cx, AppData::filter_prefix);
                    });

                    List::new(cx, AppData::list, |cx, index, item| {
                        Label::new(
                            cx,
                            item.map(|(name, surname)| format!("{}, {}", surname, name)),
                        )
                        .on_press(move |cx| {
                            cx.emit(AppEvent::SetSelected(index));
                        })
                        .navigable(true)
                        .checked(AppData::selected.map(move |selected| *selected == Some(index)));
                    });
                });

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Name:").width(Pixels(80.0));

                        Textbox::new(cx, AppData::name).on_edit(move |cx, text| {
                            cx.emit(AppEvent::SetName(text.clone()));
                        });
                    });

                    HStack::new(cx, |cx| {
                        Label::new(cx, "Surname:").width(Pixels(80.0));

                        Textbox::new(cx, AppData::surname).on_edit(move |cx, text| {
                            cx.emit(AppEvent::SetSurname(text.clone()));
                        });
                    });
                });
            })
            .height(Stretch(1.0))
            .padding_top(Pixels(0.0))
            .padding_bottom(Pixels(0.0));

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "Create"))
                    .on_press(|cx| cx.emit(AppEvent::Create));
                Button::new(cx, |cx| Label::new(cx, "Update"))
                    .on_press(|cx| cx.emit(AppEvent::Update));
                Button::new(cx, |cx| Label::new(cx, "Delete"))
                    .on_press(|cx| cx.emit(AppEvent::Delete));
            })
            .horizontal_gap(Pixels(10.0));
        })
        .padding(Pixels(10.0));
    })
    .title("CRUD")
    .inner_size((450, 200))
    .run()
}
