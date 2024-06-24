//! Built-in views provided by vizia.

mod avatar;
mod badge;
mod button;
mod checkbox;
mod chip;
mod combobox;
mod datepicker;
mod dialog;
mod divider;
mod dropdown;
mod element;
mod form;
mod image;
// mod knob;
mod label;
mod list;
mod menu;
pub mod normalized_map;
mod notification;
mod picklist;
mod popup;
mod progressbar;
mod radio;
mod rating;
mod scrollbar;
mod scrollview;
mod slider;
mod spinbox;
mod stack;
mod switch;
mod tabview;
mod textbox;
mod toggle_button;
mod tooltip;
mod virtual_list;
mod xypad;

pub use self::image::Image;
pub use crate::binding::Binding;
pub use avatar::*;
pub use badge::*;
pub use button::{Button, ButtonGroup, ButtonModifiers, ButtonVariant, IconButton};
pub use checkbox::Checkbox;
pub use chip::*;
pub use combobox::*;
pub use datepicker::Datepicker;
pub use dialog::*;
pub use divider::*;
pub use dropdown::Dropdown;
pub use element::Element;
pub use form::{FormControl, FormGroup, FormPlacement};
// pub use knob::{ArcTrack, Knob, KnobMode, TickKnob, Ticks};
pub use label::{Icon, Label};
pub use list::*;
pub use menu::*;
pub use notification::Notification;
pub use picklist::{PickList, ScrollList};
pub use popup::*;
pub use progressbar::ProgressBar;
pub use radio::RadioButton;
pub use rating::Rating;
pub use scrollbar::Scrollbar;
pub use scrollview::{ScrollEvent, ScrollView};
pub use slider::{NamedSlider, Slider};
pub use spinbox::{Spinbox, SpinboxEvent, SpinboxIcons};
pub use stack::{HStack, VStack, ZStack};
pub use switch::Switch;
pub use tabview::{TabEvent, TabPair, TabView};
pub use textbox::{TextEvent, Textbox};
pub use toggle_button::{ToggleButton, ToggleButtonModifiers};
pub use tooltip::Tooltip;
pub use virtual_list::*;
pub use xypad::XYPad;

use crate::prelude::Data;

/// The orientation of a widget, such as a slider or scrollbar
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Data)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
