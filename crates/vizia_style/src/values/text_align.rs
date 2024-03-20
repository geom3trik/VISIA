use crate::{define_enum, Parse};

define_enum! {
    /// Determines how overflowed content that is not displayed should be signaled to the user.
    #[derive(Default)]
    pub enum TextAlign {
        /// The same as left if direction is left-to-right and right if direction is right-to-left.
        #[default]
        "start": Start,
        /// The same as right if direction is left-to-right and left if direction is right-to-left.
        "end": End,
        /// The inline contents are aligned to the left edge of the line box.
        "left": Left,
        /// The inline contents are aligned to the right edge of the line box.
        "right": Right,
        /// The inline contents are centered within the line box.
        "center": Center,
        /// The inline contents are justified. Text should be spaced to line up its left and right edges to the left and right edges of the line box, except for the last line.
        "justify": Justify,
    }
}

impl From<TextAlign> for skia_safe::textlayout::TextAlign {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Start => skia_safe::textlayout::TextAlign::Start,
            TextAlign::End => skia_safe::textlayout::TextAlign::End,
            TextAlign::Left => skia_safe::textlayout::TextAlign::Left,
            TextAlign::Right => skia_safe::textlayout::TextAlign::Right,
            TextAlign::Center => skia_safe::textlayout::TextAlign::Center,
            TextAlign::Justify => skia_safe::textlayout::TextAlign::Justify,
        }
    }
}
