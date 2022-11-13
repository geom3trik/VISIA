use chrono::{NaiveDateTime, Utc};
use vizia::prelude::*;

const ICON_CALENDAR: &str = "\u{1f4c5}";

#[derive(Clone, Lens)]
struct AppState {
    datetime: NaiveDateTime,
    show_popup: bool,
}

pub enum AppEvent {
    SetDateTime(NaiveDateTime),
    ToggleDatetimePicker,
}

impl Model for AppState {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetDateTime(datetime) => {
                println!("Datetime changed to: {}", datetime);
                self.datetime = *datetime;
            }

            AppEvent::ToggleDatetimePicker => {
                self.show_popup ^= true;
            }
        });
    }
}

const CENTER_LAYOUT: &str = "crates/vizia_core/resources/themes/center_layout.css";
#[allow(dead_code)]
const DARK_THEME: &str = "crates/vizia_core/resources/themes/dark_theme.css";
#[allow(dead_code)]
const LIGHT_THEME: &str = "crates/vizia_core/resources/themes/light_theme.css";

fn main() {
    Application::new(|cx| {
        AppState { datetime: Utc::now().naive_utc(), show_popup: false }.build(cx);

        PopupData::default().build(cx);

        cx.add_stylesheet(CENTER_LAYOUT).expect("Failed to find stylesheet");
        cx.add_stylesheet(DARK_THEME).expect("Failed to find stylesheet");

        VStack::new(cx, |cx| {
            ZStack::new(cx, |cx| {
                Textbox::new(
                    cx,
                    AppState::datetime
                        .map(|datetime| format!("{}", datetime.format("%d/%m/%Y  %H:%M:%S"))),
                )
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .width(Pixels(252.0))
                .height(Pixels(32.0));

                Label::new(cx, ICON_CALENDAR)
                    .height(Pixels(32.0))
                    .width(Pixels(32.0))
                    .left(Stretch(1.0))
                    .right(Pixels(0.0))
                    .child_space(Stretch(1.0))
                    .font("icons")
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(PopupEvent::Switch));
            })
            .width(Pixels(252.0))
            .height(Pixels(32.0));

            Popup::new(cx, PopupData::is_open, false, |cx| {
                DatetimePicker::new(cx, AppState::datetime)
                    .on_change(|cx, datetime| cx.emit(AppEvent::SetDateTime(datetime)));
            })
            .on_blur(|cx| cx.emit(PopupEvent::Close))
            .top(Pixels(36.0));
        })
        .row_between(Pixels(8.0));
    })
    .ignore_default_theme()
    .title("Datepicker")
    .run();
}
