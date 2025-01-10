use vizia::prelude::*;

use crate::components::DemoRegion;

#[derive(Clone, Lens)]
struct ComboBoxState {
    options: Vec<&'static str>,
    selected_option: usize,
}

pub enum ComboBoxEvent {
    SetOption(usize),
}

impl Model for ComboBoxState {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            ComboBoxEvent::SetOption(index) => {
                self.selected_option = *index;
            }
        });
    }
}

pub fn combobox(cx: &mut Context) {
    VStack::new(cx, |cx| {
        ComboBoxState {
            options: vec![
                "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
            ],

            selected_option: 0,
        }
        .build(cx);

        Markdown::new(cx, "# Combobox");

        Divider::new(cx);

        Markdown::new(cx, "### Basic combobox");

        DemoRegion::new(
            cx,
            |cx| {
                ComboBox::new(cx, ComboBoxState::options, ComboBoxState::selected_option)
                    .on_select(|cx, index| cx.emit(ComboBoxEvent::SetOption(index)))
                    .width(Pixels(100.0));
            },
            r#"ComboBox::new(cx, ComboBoxState::options, ComboBoxState::selected_option)
    .on_select(|cx, index| cx.emit(ComboBoxEvent::SetOption(index)))
    .width(Pixels(100.0));"#,
        );
    })
    .class("panel");
}
