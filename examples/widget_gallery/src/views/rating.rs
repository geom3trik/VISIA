use vizia::prelude::*;

use crate::DemoRegion;

#[derive(Clone, Lens)]
struct RatingData {
    rating: u32,
}

impl Model for RatingData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            RatingEvent::SetRating(val) => self.rating = *val,
        })
    }
}

enum RatingEvent {
    SetRating(u32),
}

pub fn rating(cx: &mut Context) {
    RatingData { rating: 3 }.build(cx);

    VStack::new(cx, |cx| {
        Markdown::new(cx, "# Rating");

        Divider::new(cx);

        Markdown::new(cx, "### Basic rating");

        DemoRegion::new(
            cx,
            |cx| {
                Rating::new(cx, 5, RatingData::rating)
                    .on_change(|ex, rating| ex.emit(RatingEvent::SetRating(rating)));
            },
            r#"Rating::new(cx, 5, RatingData::rating)
    .on_change(|ex, rating| ex.emit(RatingEvent::SetRating(rating)));"#,
        );
    })
    .class("panel");
}
