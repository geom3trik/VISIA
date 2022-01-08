use vizia::*;

// INCOMPLETE!

fn main() {
    Application::new(WindowDescription::new().with_title("Todo"), |cx| {
        TodoData {
            items: vec![
                TodoItem { text: "Item 1".to_string(), completed: false },
                TodoItem { text: "Item 2".to_string(), completed: true },
            ],
            selected: 0,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Enter a todo item...");
                Button::new(cx, |_| {}, |cx| Label::new(cx, "Add"));
            })
            .height(Auto)
            .child_space(Stretch(1.0));

            List::new(cx, TodoData::items, |cx, item| {
                //let item = item.clone();
                Binding::new(cx, TodoData::selected, move |cx, selected| {
                    let item = item.clone();
                    HStack::new(cx, move |cx| {
                        Label::new(cx, &item.value(cx).text.to_owned());
                        let item_index = item.index();
                        Checkbox::new(cx, item.value(cx).completed).on_toggle(move |cx| {
                            cx.emit(TodoEvent::ToggleCompleted(item_index))
                        });
                        Label::new(cx, &item.value(cx).completed.to_string());
                    })
                    .border_width(Pixels(1.0))
                    .border_color(Color::black());
                });
            })
            .size(Stretch(1.0))
            .child_space(Pixels(10.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    })
    .run();
}

#[derive(Clone, Data)]
pub struct TodoItem {
    text: String,
    completed: bool,
}

#[derive(Lens)]
pub struct TodoData {
    items: Vec<TodoItem>,
    selected: usize,
}

#[derive(Debug)]
pub enum TodoEvent {
    ToggleCompleted(usize),
}

impl Model for TodoData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(todo_event) = event.message.downcast() {
            match todo_event {
                TodoEvent::ToggleCompleted(index) => {
                    if let Some(item) = self.items.get_mut(*index) {
                        item.completed ^= true;
                    }
                }
            }
        }
    }
}
