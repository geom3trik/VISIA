mod helpers;
pub use helpers::*;
use vizia::prelude::*;

const STYLE: &str = r#"
    element.test {
        background-color: rgb(100, 100, 100);
        size: 100px;
        padding: 1s;
    }
"#;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(STYLE).expect("Failed to add stylesheet");

        ExamplePage::vertical(cx, |cx| {
            HStack::new(cx, |cx| {
                Element::new(cx)
                    .text("Top Start")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::TopStart)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Top")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Top)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Top End")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::TopEnd)
                    })
                    .class("test");
            })
            .size(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Element::new(cx)
                    .text("LeftStart")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::LeftStart)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Left")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Left)
                    })
                    .class("test");

                Element::new(cx)
                    .text("LeftEnd")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::LeftEnd)
                    })
                    .class("test");
            })
            .size(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Element::new(cx)
                    .text("RightStart")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::RightStart)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Right")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Right)
                    })
                    .class("test");

                Element::new(cx)
                    .text("RightEnd")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::RightEnd)
                    })
                    .class("test");
            })
            .size(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Element::new(cx)
                    .text("BottomStart")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::BottomStart)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Bottom")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Bottom)
                    })
                    .class("test");

                Element::new(cx)
                    .text("BottomEnd")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::BottomEnd)
                    })
                    .class("test");
            })
            .size(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Element::new(cx)
                    .text("Over")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Over)
                    })
                    .class("test");

                Element::new(cx)
                    .text("Cursor")
                    .tooltip(|cx| {
                        Tooltip::new(cx, |cx| {
                            Label::new(cx, "This is a tooltip").padding(Pixels(4.0));
                        })
                        .padding(Pixels(4.0))
                        .size(Auto)
                        .placement(Placement::Cursor)
                    })
                    .class("test");
            })
            .size(Auto)
            .horizontal_gap(Pixels(8.0));
        });
    })
    .title("Tooltip")
    .inner_size((800, 800))
    .run()
}
