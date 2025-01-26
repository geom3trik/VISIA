//! Styling determines the appearance of a view.
//!
//! # Styling Views
//! Vizia provides two ways to style views:
//! - Inline
//! - Shared
//!
//! ## Inline Styling
//! Inline styling refers to setting the style and layout properties of a view using view [modifiers](crate::modifiers).
//! ```
//! # use vizia_core::prelude::*;
//! # let cx = &mut Context::default();
//! Element::new(cx).background_color(Color::red());
//! ```
//! Properties set inline affect only the modified view and override any shared styling for the same property.
//!
//! ## Shared Styling
//! Shared styling refers to setting the style and layout properties using css rules.
//! ```
//! # use vizia_core::prelude::*;
//! # let cx = &mut Context::default();
//! Element::new(cx).class("foo");
//! ```
//! ```css
//! .foo {
//!     background-color: red;
//! }
//! ```
//! Rules defined in css can apply to many views but are overridden by inline properties on a view.
//!
//! ### Adding Stylesheets
//! To add a css string to an application, use [`add_theme()`](crate::context::Context::add_theme()) on [`Context`].
//! This can be used with the `include_str!()` macro to embed an external stylesheet file into the application binary when compiled.
//! Alternatively a constant string literal can be used to embed the CSS in the application.
//!
//! ```
//! # use vizia_core::prelude::*;
//! # let cx = &mut Context::default();
//!
//! const STYLE: &str = r#"
//!     .foo {
//!         background-color: red;
//!     }
//! "#;
//!
//! cx.add_stylesheet(STYLE);
//!
//! Element::new(cx).class("foo");
//! ```
//!
//! To add an external css stylesheet which is read from a file at runtime, use [`add_stylesheet()`](crate::context::Context::add_stylesheet()) on [`Context`].
//! Stylesheets added this way can be hot-reloaded by pressing the F5 key in the application window.
//!
//! ```
//! # use vizia_core::prelude::*;
//! # let cx = &mut Context::default();
//!
//! cx.add_stylesheet("path/to/stylesheet.css");
//!
//! Element::new(cx).class("foo");
//! ```

use hashbrown::{HashMap, HashSet};
use indexmap::IndexMap;
use log::warn;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Range};

use crate::prelude::*;

pub use vizia_style::{
    Alignment, Angle, BackgroundImage, BackgroundSize, BorderStyleKeyword, ClipPath, Color,
    CornerShape, CssRule, CursorIcon, Display, Filter, FontFamily, FontSize, FontSlant,
    FontVariation, FontWeight, FontWeightKeyword, FontWidth, GenericFontFamily, Gradient,
    HorizontalPosition, HorizontalPositionKeyword, Length, LengthOrPercentage, LengthValue,
    LineClamp, LineDirection, LineHeight, LinearGradient, Matrix, Opacity, Overflow, PointerEvents,
    Position, Scale, Shadow, TextAlign, TextDecorationLine, TextDecorationStyle, TextOverflow,
    TextStroke, TextStrokeStyle, Transform, Transition, Translate, VerticalPosition,
    VerticalPositionKeyword, Visibility, RGBA,
};

use vizia_style::{
    BlendMode, EasingFunction, KeyframeSelector, ParserOptions, Property, SelectorList, Selectors,
    StyleSheet, TextShadow,
};

mod rule;
pub(crate) use rule::Rule;

mod pseudoclass;
pub(crate) use pseudoclass::*;

mod transform;
pub(crate) use transform::*;

use crate::animation::{AnimationState, Interpolator, Keyframe, TimingFunction};
use crate::storage::animatable_set::AnimatableSet;
use crate::storage::style_set::StyleSet;
use bitflags::bitflags;
use vizia_id::IdManager;
use vizia_storage::SparseSet;

bitflags! {
    /// Describes the capabilities of a view with respect to user interaction.
    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Abilities: u8 {
        // Whether a view will be included in hit tests and receive mouse input events.
        const HOVERABLE = 1 << 0;
        // Whether a view can be focused to receive keyboard events.
        const FOCUSABLE = 1 << 1;
        // Whether a view can be checked.
        const CHECKABLE = 1 << 2;
        // Whether a view can be focused via keyboard navigation.
        const NAVIGABLE = 1 << 3;
        // Whether a view can be dragged during a drag and drop.
        const DRAGGABLE = 1 << 4;
    }
}

impl Default for Abilities {
    fn default() -> Abilities {
        Abilities::HOVERABLE
    }
}

bitflags! {
    pub(crate) struct SystemFlags: u8 {
        /// Layout system flag.
        const RELAYOUT = 1;
        const RESTYLE = 1 << 1;
        const REFLOW = 1 << 2;
        const REDRAW = 1 << 3;
    }
}

impl Default for SystemFlags {
    fn default() -> Self {
        SystemFlags::all()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ImageOrGradient {
    #[default]
    None,
    Image(String),
    /// A gradient.
    Gradient(Gradient),
}

/// A font-family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyOwned {
    /// A generic font-family.
    Generic(GenericFontFamily),
    /// A named front-family.
    Named(String),
}

impl AsRef<str> for FamilyOwned {
    fn as_ref(&self) -> &str {
        match self {
            FamilyOwned::Generic(generic) => match generic {
                GenericFontFamily::Serif => "serif",
                GenericFontFamily::SansSerif => "sans-serif",
                GenericFontFamily::Cursive => todo!(),
                GenericFontFamily::Fantasy => todo!(),
                GenericFontFamily::Monospace => "Cascadia Mono",
            },
            FamilyOwned::Named(family) => family.as_str(),
        }
    }
}

pub(crate) struct Bloom(pub(crate) qfilter::Filter);

impl Default for Bloom {
    fn default() -> Self {
        Self(qfilter::Filter::new_resizeable(10000, 10000000, 0.01).unwrap())
    }
}

impl Deref for Bloom {
    type Target = qfilter::Filter;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bloom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Stores the style properties of all entities in the application.
#[derive(Default)]
pub struct Style {
    pub(crate) rule_manager: IdManager<Rule>,

    // Creates and destroys animation ids
    pub(crate) animation_manager: IdManager<Animation>,
    pub(crate) animations: HashMap<String, Animation>,
    // List of animations to be started on the next frame
    pub(crate) pending_animations: Vec<(Entity, Animation, Duration, Duration)>,

    // List of rules
    pub(crate) rules: IndexMap<Rule, SelectorList<Selectors>>,

    pub(crate) default_font: Vec<FamilyOwned>,

    // CSS Selector Properties
    pub(crate) ids: SparseSet<String>,
    pub(crate) classes: SparseSet<HashSet<String>>,
    pub(crate) pseudo_classes: SparseSet<PseudoClassFlags>,
    pub(crate) disabled: StyleSet<bool>,
    pub(crate) abilities: SparseSet<Abilities>,

    // Accessibility Properties
    pub(crate) name: StyleSet<String>,
    pub(crate) role: SparseSet<Role>,
    pub(crate) live: SparseSet<Live>,
    pub(crate) labelled_by: SparseSet<Entity>,
    pub(crate) hidden: SparseSet<bool>,
    pub(crate) text_value: SparseSet<String>,
    pub(crate) numeric_value: SparseSet<f64>,

    // Visibility
    pub(crate) visibility: StyleSet<Visibility>,

    // Opacity
    pub(crate) opacity: AnimatableSet<Opacity>,

    // Z Order
    pub(crate) z_index: StyleSet<i32>,

    // Clipping
    pub(crate) clip_path: AnimatableSet<ClipPath>,

    // Overflow
    pub(crate) overflowx: StyleSet<Overflow>,
    pub(crate) overflowy: StyleSet<Overflow>,

    // Filters
    pub(crate) backdrop_filter: AnimatableSet<Filter>,

    pub(crate) blend_mode: StyleSet<BlendMode>,

    // Transform
    pub(crate) transform: AnimatableSet<Vec<Transform>>,
    pub(crate) transform_origin: AnimatableSet<Translate>,
    pub(crate) translate: AnimatableSet<Translate>,
    pub(crate) rotate: AnimatableSet<Angle>,
    pub(crate) scale: AnimatableSet<Scale>,

    // Border
    pub(crate) border_width: AnimatableSet<LengthOrPercentage>,
    pub(crate) border_color: AnimatableSet<Color>,
    pub(crate) border_style: StyleSet<BorderStyleKeyword>,

    // Corner Shape
    pub(crate) corner_top_left_shape: StyleSet<CornerShape>,
    pub(crate) corner_top_right_shape: StyleSet<CornerShape>,
    pub(crate) corner_bottom_left_shape: StyleSet<CornerShape>,
    pub(crate) corner_bottom_right_shape: StyleSet<CornerShape>,

    // Corner Radius
    pub(crate) corner_top_left_radius: AnimatableSet<LengthOrPercentage>,
    pub(crate) corner_top_right_radius: AnimatableSet<LengthOrPercentage>,
    pub(crate) corner_bottom_left_radius: AnimatableSet<LengthOrPercentage>,
    pub(crate) corner_bottom_right_radius: AnimatableSet<LengthOrPercentage>,

    // Corner Smoothing
    pub(crate) corner_top_left_smoothing: AnimatableSet<f32>,
    pub(crate) corner_top_right_smoothing: AnimatableSet<f32>,
    pub(crate) corner_bottom_left_smoothing: AnimatableSet<f32>,
    pub(crate) corner_bottom_right_smoothing: AnimatableSet<f32>,

    // Outline
    pub(crate) outline_width: AnimatableSet<LengthOrPercentage>,
    pub(crate) outline_color: AnimatableSet<Color>,
    pub(crate) outline_offset: AnimatableSet<LengthOrPercentage>,

    // Background
    pub(crate) background_color: AnimatableSet<Color>,
    pub(crate) background_image: AnimatableSet<Vec<ImageOrGradient>>,
    pub(crate) background_size: AnimatableSet<Vec<BackgroundSize>>,

    // Shadow
    pub(crate) shadow: AnimatableSet<Vec<Shadow>>,
    pub(crate) text_shadow: AnimatableSet<Vec<TextShadow>>,

    // Text
    pub(crate) text: SparseSet<String>,
    pub(crate) text_wrap: StyleSet<bool>,
    pub(crate) text_overflow: StyleSet<TextOverflow>,
    pub(crate) line_clamp: StyleSet<LineClamp>,
    pub(crate) text_align: StyleSet<TextAlign>,
    pub(crate) text_decoration_line: StyleSet<TextDecorationLine>,
    pub(crate) text_stroke_width: StyleSet<Length>,
    pub(crate) text_stroke_style: StyleSet<TextStrokeStyle>,
    pub(crate) underline_style: StyleSet<TextDecorationLine>,
    pub(crate) overline_style: StyleSet<TextDecorationStyle>,
    pub(crate) strikethrough_style: StyleSet<TextDecorationStyle>,
    pub(crate) underline_color: AnimatableSet<Color>,
    pub(crate) overline_color: AnimatableSet<Color>,
    pub(crate) strikethrough_color: AnimatableSet<Color>,
    pub(crate) font_family: StyleSet<Vec<FamilyOwned>>,
    pub(crate) font_color: AnimatableSet<Color>,
    pub(crate) font_size: AnimatableSet<FontSize>,
    pub(crate) font_weight: StyleSet<FontWeight>,
    pub(crate) font_slant: StyleSet<FontSlant>,
    pub(crate) font_width: StyleSet<FontWidth>,
    pub(crate) font_variation_settings: StyleSet<Vec<FontVariation>>,
    pub(crate) caret_color: AnimatableSet<Color>,
    pub(crate) selection_color: AnimatableSet<Color>,
    pub(crate) letter_spacing: AnimatableSet<Length>,
    pub(crate) word_spacing: AnimatableSet<Length>,
    pub(crate) line_height: AnimatableSet<LineHeight>,

    pub(crate) fill: AnimatableSet<Color>,

    // cursor Icon
    pub(crate) cursor: StyleSet<CursorIcon>,

    pub(crate) pointer_events: StyleSet<PointerEvents>,

    // LAYOUT

    // Display
    pub(crate) display: AnimatableSet<Display>,

    // Layout Type
    pub(crate) layout_type: StyleSet<LayoutType>,

    // Position
    pub(crate) position_type: StyleSet<PositionType>,

    pub(crate) alignment: StyleSet<Alignment>,

    // Spacing
    pub(crate) left: AnimatableSet<Units>,
    pub(crate) right: AnimatableSet<Units>,
    pub(crate) top: AnimatableSet<Units>,
    pub(crate) bottom: AnimatableSet<Units>,

    // Padding
    pub(crate) padding_left: AnimatableSet<Units>,
    pub(crate) padding_right: AnimatableSet<Units>,
    pub(crate) padding_top: AnimatableSet<Units>,
    pub(crate) padding_bottom: AnimatableSet<Units>,
    pub(crate) vertical_gap: AnimatableSet<Units>,
    pub(crate) horizontal_gap: AnimatableSet<Units>,

    // Scrolling
    pub(crate) vertical_scroll: AnimatableSet<f32>,
    pub(crate) horizontal_scroll: AnimatableSet<f32>,

    // Size
    pub(crate) width: AnimatableSet<Units>,
    pub(crate) height: AnimatableSet<Units>,

    // Size Constraints
    pub(crate) min_width: AnimatableSet<Units>,
    pub(crate) max_width: AnimatableSet<Units>,
    pub(crate) min_height: AnimatableSet<Units>,
    pub(crate) max_height: AnimatableSet<Units>,

    // Gap Constraints
    pub(crate) min_horizontal_gap: AnimatableSet<Units>,
    pub(crate) max_horizontal_gap: AnimatableSet<Units>,
    pub(crate) min_vertical_gap: AnimatableSet<Units>,
    pub(crate) max_vertical_gap: AnimatableSet<Units>,

    pub(crate) system_flags: SystemFlags,

    pub(crate) restyle: Bloom,
    pub(crate) text_construction: Bloom,
    pub(crate) text_layout: Bloom,
    pub(crate) reaccess: Bloom,

    pub(crate) text_range: SparseSet<Range<usize>>,
    pub(crate) text_span: SparseSet<bool>,

    /// This includes both the system's HiDPI scaling factor as well as `cx.user_scale_factor`.
    pub(crate) dpi_factor: f64,
}

impl Style {
    /// Returns the scale factor of the application.
    pub fn scale_factor(&self) -> f32 {
        self.dpi_factor as f32
    }

    /// Function to convert logical points to physical pixels.
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        (logical * self.dpi_factor as f32).round()
    }

    /// Function to convert physical pixels to logical points.
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        physical / self.dpi_factor as f32
    }

    pub(crate) fn remove_rules(&mut self) {
        self.rule_manager.reset();
        self.rules.clear();
    }

    pub(crate) fn get_animation(&self, name: &str) -> Option<&Animation> {
        self.animations.get(name)
    }

    pub(crate) fn add_keyframe(
        &mut self,
        animation_id: Animation,
        time: f32,
        properties: &[Property],
    ) {
        fn insert_keyframe<T: 'static + Interpolator + Debug + Clone + PartialEq + Default>(
            storage: &mut AnimatableSet<T>,
            animation_id: Animation,
            time: f32,
            value: T,
        ) {
            let keyframe = Keyframe { time, value, timing_function: TimingFunction::linear() };

            if let Some(anim_state) = storage.get_animation_mut(animation_id) {
                anim_state.keyframes.push(keyframe)
            } else {
                let anim_state = AnimationState::new(animation_id).with_keyframe(keyframe);
                storage.insert_animation(animation_id, anim_state);
            }
        }

        for property in properties.iter() {
            match property {
                // DISPLAY
                Property::Display(value) => {
                    insert_keyframe(&mut self.display, animation_id, time, *value);
                }

                Property::Opacity(value) => {
                    insert_keyframe(&mut self.opacity, animation_id, time, *value);
                }

                Property::ClipPath(value) => {
                    insert_keyframe(&mut self.clip_path, animation_id, time, value.clone());
                }

                // TRANSFORM
                Property::Transform(value) => {
                    insert_keyframe(&mut self.transform, animation_id, time, value.clone());
                }

                Property::TransformOrigin(transform_origin) => {
                    let x = transform_origin.x.to_length_or_percentage();
                    let y = transform_origin.y.to_length_or_percentage();
                    let value = Translate { x, y };
                    insert_keyframe(&mut self.transform_origin, animation_id, time, value);
                }

                Property::Translate(value) => {
                    insert_keyframe(&mut self.translate, animation_id, time, value.clone());
                }

                Property::Rotate(value) => {
                    insert_keyframe(&mut self.rotate, animation_id, time, *value);
                }

                Property::Scale(value) => {
                    insert_keyframe(&mut self.scale, animation_id, time, *value);
                }

                // BORDER
                Property::BorderWidth(value) => {
                    insert_keyframe(
                        &mut self.border_width,
                        animation_id,
                        time,
                        value.left.0.clone(),
                    );
                }

                Property::BorderColor(value) => {
                    insert_keyframe(&mut self.border_color, animation_id, time, *value);
                }

                Property::CornerTopLeftRadius(value) => {
                    insert_keyframe(
                        &mut self.corner_top_left_radius,
                        animation_id,
                        time,
                        value.clone(),
                    );
                }

                Property::CornerTopRightRadius(value) => {
                    insert_keyframe(
                        &mut self.corner_top_right_radius,
                        animation_id,
                        time,
                        value.clone(),
                    );
                }

                Property::CornerBottomLeftRadius(value) => {
                    insert_keyframe(
                        &mut self.corner_bottom_left_radius,
                        animation_id,
                        time,
                        value.clone(),
                    );
                }

                Property::CornerBottomRightRadius(value) => {
                    insert_keyframe(
                        &mut self.corner_bottom_right_radius,
                        animation_id,
                        time,
                        value.clone(),
                    );
                }

                // OUTLINE
                Property::OutlineWidth(value) => {
                    insert_keyframe(
                        &mut self.outline_width,
                        animation_id,
                        time,
                        value.left.0.clone(),
                    );
                }

                Property::OutlineColor(value) => {
                    insert_keyframe(&mut self.outline_color, animation_id, time, *value);
                }

                Property::OutlineOffset(value) => {
                    insert_keyframe(&mut self.outline_offset, animation_id, time, value.clone());
                }

                // BACKGROUND
                Property::BackgroundColor(value) => {
                    insert_keyframe(&mut self.background_color, animation_id, time, *value);
                }

                Property::BackgroundImage(images) => {
                    let images = images
                        .iter()
                        .filter_map(|img| match img {
                            BackgroundImage::None => None,
                            BackgroundImage::Gradient(gradient) => {
                                Some(ImageOrGradient::Gradient(*gradient.clone()))
                            }
                            BackgroundImage::Url(url) => {
                                Some(ImageOrGradient::Image(url.url.to_string()))
                            }
                        })
                        .collect::<Vec<_>>();
                    insert_keyframe(&mut self.background_image, animation_id, time, images);
                }

                Property::BackgroundSize(value) => {
                    insert_keyframe(&mut self.background_size, animation_id, time, value.clone());
                }

                // BOX SHADOW
                Property::Shadow(value) => {
                    insert_keyframe(&mut self.shadow, animation_id, time, value.clone());
                }

                Property::TextShadow(value) => {
                    insert_keyframe(&mut self.text_shadow, animation_id, time, value.clone());
                }

                // TEXT
                Property::FontColor(value) => {
                    insert_keyframe(&mut self.font_color, animation_id, time, *value);
                }

                Property::FontSize(value) => {
                    insert_keyframe(&mut self.font_size, animation_id, time, *value);
                }

                Property::CaretColor(value) => {
                    insert_keyframe(&mut self.caret_color, animation_id, time, *value);
                }

                Property::SelectionColor(value) => {
                    insert_keyframe(&mut self.selection_color, animation_id, time, *value);
                }

                // SPACE
                Property::Left(value) => {
                    insert_keyframe(&mut self.left, animation_id, time, *value);
                }

                Property::Right(value) => {
                    insert_keyframe(&mut self.right, animation_id, time, *value);
                }

                Property::Top(value) => {
                    insert_keyframe(&mut self.top, animation_id, time, *value);
                }

                Property::Bottom(value) => {
                    insert_keyframe(&mut self.bottom, animation_id, time, *value);
                }

                // Padding
                Property::PaddingLeft(value) => {
                    insert_keyframe(&mut self.padding_left, animation_id, time, *value);
                }

                Property::PaddingRight(value) => {
                    insert_keyframe(&mut self.padding_right, animation_id, time, *value);
                }

                Property::PaddingTop(value) => {
                    insert_keyframe(&mut self.padding_top, animation_id, time, *value);
                }

                Property::PaddingBottom(value) => {
                    insert_keyframe(&mut self.padding_bottom, animation_id, time, *value);
                }

                Property::HorizontalGap(value) => {
                    insert_keyframe(&mut self.horizontal_gap, animation_id, time, *value);
                }

                Property::VerticalGap(value) => {
                    insert_keyframe(&mut self.vertical_gap, animation_id, time, *value);
                }

                Property::Gap(value) => {
                    insert_keyframe(&mut self.horizontal_gap, animation_id, time, *value);
                    insert_keyframe(&mut self.vertical_gap, animation_id, time, *value);
                }

                // GAP CONSSTRAINTS
                Property::MinGap(value) => {
                    insert_keyframe(&mut self.min_horizontal_gap, animation_id, time, *value);
                    insert_keyframe(&mut self.min_vertical_gap, animation_id, time, *value);
                }

                Property::MaxGap(value) => {
                    insert_keyframe(&mut self.max_horizontal_gap, animation_id, time, *value);
                    insert_keyframe(&mut self.max_vertical_gap, animation_id, time, *value);
                }

                Property::MinHorizontalGap(value) => {
                    insert_keyframe(&mut self.min_horizontal_gap, animation_id, time, *value);
                }

                Property::MaxHorizontalGap(value) => {
                    insert_keyframe(&mut self.max_horizontal_gap, animation_id, time, *value);
                }

                Property::MinVerticalGap(value) => {
                    insert_keyframe(&mut self.min_vertical_gap, animation_id, time, *value);
                }

                Property::MaxVerticalGap(value) => {
                    insert_keyframe(&mut self.max_vertical_gap, animation_id, time, *value);
                }

                // SIZE
                Property::Width(value) => {
                    insert_keyframe(&mut self.width, animation_id, time, *value);
                }

                Property::Height(value) => {
                    insert_keyframe(&mut self.height, animation_id, time, *value);
                }

                // SIZE CONSTRAINTS
                Property::MinWidth(value) => {
                    insert_keyframe(&mut self.min_width, animation_id, time, *value);
                }

                Property::MaxWidth(value) => {
                    insert_keyframe(&mut self.max_width, animation_id, time, *value);
                }

                Property::MinHeight(value) => {
                    insert_keyframe(&mut self.min_height, animation_id, time, *value);
                }

                Property::MaxHeight(value) => {
                    insert_keyframe(&mut self.max_height, animation_id, time, *value);
                }

                Property::UnderlineColor(value) => {
                    insert_keyframe(&mut self.underline_color, animation_id, time, *value);
                }

                Property::Fill(value) => {
                    insert_keyframe(&mut self.fill, animation_id, time, *value);
                }

                Property::LetterSpacing(value) => {
                    insert_keyframe(&mut self.letter_spacing, animation_id, time, value.0.clone());
                }

                Property::WordSpacing(value) => {
                    insert_keyframe(&mut self.word_spacing, animation_id, time, value.0.clone());
                }

                Property::LineHeight(value) => {
                    insert_keyframe(&mut self.line_height, animation_id, time, value.clone());
                }

                _ => {}
            }
        }
    }

    pub(crate) fn add_animation(&mut self, animation: AnimationBuilder) -> Animation {
        let animation_id = self.animation_manager.create();
        for keyframe in animation.keyframes.iter() {
            self.add_keyframe(animation_id, keyframe.time, &keyframe.properties);
        }

        animation_id
    }

    pub(crate) fn enqueue_animation(
        &mut self,
        entity: Entity,
        animation: Animation,
        duration: Duration,
        delay: Duration,
    ) {
        self.pending_animations.push((entity, animation, duration, delay));
    }

    pub(crate) fn play_pending_animations(&mut self) {
        let start_time = Instant::now();

        let pending_animations = self.pending_animations.drain(..).collect::<Vec<_>>();

        for (entity, animation, duration, delay) in pending_animations {
            self.play_animation(entity, animation, start_time + delay, duration, delay)
        }
    }

    pub(crate) fn play_animation(
        &mut self,
        entity: Entity,
        animation: Animation,
        start_time: Instant,
        duration: Duration,
        delay: Duration,
    ) {
        self.display.play_animation(entity, animation, start_time, duration, delay);
        self.opacity.play_animation(entity, animation, start_time, duration, delay);
        self.clip_path.play_animation(entity, animation, start_time, duration, delay);

        self.transform.play_animation(entity, animation, start_time, duration, delay);
        self.transform_origin.play_animation(entity, animation, start_time, duration, delay);
        self.translate.play_animation(entity, animation, start_time, duration, delay);
        self.rotate.play_animation(entity, animation, start_time, duration, delay);
        self.scale.play_animation(entity, animation, start_time, duration, delay);

        self.border_width.play_animation(entity, animation, start_time, duration, delay);
        self.border_color.play_animation(entity, animation, start_time, duration, delay);

        self.corner_top_left_radius.play_animation(entity, animation, start_time, duration, delay);
        self.corner_top_right_radius.play_animation(entity, animation, start_time, duration, delay);
        self.corner_bottom_left_radius
            .play_animation(entity, animation, start_time, duration, delay);
        self.corner_bottom_right_radius
            .play_animation(entity, animation, start_time, duration, delay);

        self.outline_width.play_animation(entity, animation, start_time, duration, delay);
        self.outline_color.play_animation(entity, animation, start_time, duration, delay);
        self.outline_offset.play_animation(entity, animation, start_time, duration, delay);

        self.background_color.play_animation(entity, animation, start_time, duration, delay);
        self.background_image.play_animation(entity, animation, start_time, duration, delay);
        self.background_size.play_animation(entity, animation, start_time, duration, delay);

        self.shadow.play_animation(entity, animation, start_time, duration, delay);
        self.text_shadow.play_animation(entity, animation, start_time, duration, delay);

        self.font_color.play_animation(entity, animation, start_time, duration, delay);
        self.font_size.play_animation(entity, animation, start_time, duration, delay);
        self.caret_color.play_animation(entity, animation, start_time, duration, delay);
        self.selection_color.play_animation(entity, animation, start_time, duration, delay);

        self.left.play_animation(entity, animation, start_time, duration, delay);
        self.right.play_animation(entity, animation, start_time, duration, delay);
        self.top.play_animation(entity, animation, start_time, duration, delay);
        self.bottom.play_animation(entity, animation, start_time, duration, delay);

        self.padding_left.play_animation(entity, animation, start_time, duration, delay);
        self.padding_right.play_animation(entity, animation, start_time, duration, delay);
        self.padding_top.play_animation(entity, animation, start_time, duration, delay);
        self.padding_bottom.play_animation(entity, animation, start_time, duration, delay);
        self.horizontal_gap.play_animation(entity, animation, start_time, duration, delay);
        self.vertical_gap.play_animation(entity, animation, start_time, duration, delay);

        self.width.play_animation(entity, animation, start_time, duration, delay);
        self.height.play_animation(entity, animation, start_time, duration, delay);

        self.min_width.play_animation(entity, animation, start_time, duration, delay);
        self.max_width.play_animation(entity, animation, start_time, duration, delay);
        self.min_height.play_animation(entity, animation, start_time, duration, delay);
        self.max_height.play_animation(entity, animation, start_time, duration, delay);

        self.min_horizontal_gap.play_animation(entity, animation, start_time, duration, delay);
        self.max_horizontal_gap.play_animation(entity, animation, start_time, duration, delay);
        self.min_vertical_gap.play_animation(entity, animation, start_time, duration, delay);
        self.max_vertical_gap.play_animation(entity, animation, start_time, duration, delay);

        self.underline_color.play_animation(entity, animation, start_time, duration, delay);

        self.fill.play_animation(entity, animation, start_time, duration, delay);

        self.letter_spacing.play_animation(entity, animation, start_time, duration, delay);
        self.word_spacing.play_animation(entity, animation, start_time, duration, delay);

        self.line_height.play_animation(entity, animation, start_time, duration, delay);
    }

    pub(crate) fn is_animating(&self, entity: Entity, animation: Animation) -> bool {
        self.display.has_active_animation(entity, animation)
            | self.opacity.has_active_animation(entity, animation)
            | self.clip_path.has_active_animation(entity, animation)
            | self.transform.has_active_animation(entity, animation)
            | self.transform_origin.has_active_animation(entity, animation)
            | self.translate.has_active_animation(entity, animation)
            | self.rotate.has_active_animation(entity, animation)
            | self.scale.has_active_animation(entity, animation)
            | self.border_width.has_active_animation(entity, animation)
            | self.border_color.has_active_animation(entity, animation)
            | self.corner_top_left_radius.has_active_animation(entity, animation)
            | self.corner_top_right_radius.has_active_animation(entity, animation)
            | self.corner_bottom_left_radius.has_active_animation(entity, animation)
            | self.corner_bottom_right_radius.has_active_animation(entity, animation)
            | self.outline_width.has_active_animation(entity, animation)
            | self.outline_color.has_active_animation(entity, animation)
            | self.outline_offset.has_active_animation(entity, animation)
            | self.background_color.has_active_animation(entity, animation)
            | self.background_image.has_active_animation(entity, animation)
            | self.background_size.has_active_animation(entity, animation)
            | self.shadow.has_active_animation(entity, animation)
            | self.text_shadow.has_active_animation(entity, animation)
            | self.font_color.has_active_animation(entity, animation)
            | self.font_size.has_active_animation(entity, animation)
            | self.caret_color.has_active_animation(entity, animation)
            | self.selection_color.has_active_animation(entity, animation)
            | self.left.has_active_animation(entity, animation)
            | self.right.has_active_animation(entity, animation)
            | self.top.has_active_animation(entity, animation)
            | self.bottom.has_active_animation(entity, animation)
            | self.padding_left.has_active_animation(entity, animation)
            | self.padding_right.has_active_animation(entity, animation)
            | self.padding_top.has_active_animation(entity, animation)
            | self.padding_bottom.has_active_animation(entity, animation)
            | self.horizontal_gap.has_active_animation(entity, animation)
            | self.vertical_gap.has_active_animation(entity, animation)
            | self.width.has_active_animation(entity, animation)
            | self.height.has_active_animation(entity, animation)
            | self.min_width.has_active_animation(entity, animation)
            | self.max_width.has_active_animation(entity, animation)
            | self.min_height.has_active_animation(entity, animation)
            | self.max_height.has_active_animation(entity, animation)
            | self.min_horizontal_gap.has_active_animation(entity, animation)
            | self.max_horizontal_gap.has_active_animation(entity, animation)
            | self.min_vertical_gap.has_active_animation(entity, animation)
            | self.max_vertical_gap.has_active_animation(entity, animation)
            | self.underline_color.has_active_animation(entity, animation)
            | self.fill.has_active_animation(entity, animation)
            | self.letter_spacing.has_active_animation(entity, animation)
            | self.word_spacing.has_active_animation(entity, animation)
            | self.line_height.has_active_animation(entity, animation)
    }

    pub(crate) fn parse_theme(&mut self, stylesheet: &str) {
        if let Ok(stylesheet) = StyleSheet::parse(stylesheet, ParserOptions::new()) {
            let rules = stylesheet.rules.0;

            for rule in rules {
                match rule {
                    CssRule::Style(style_rule) => {
                        let rule_id = self.rule_manager.create();

                        let selectors = style_rule.selectors;

                        self.rules.insert(rule_id, selectors);

                        for property in style_rule.declarations.declarations {
                            match property {
                                Property::Transition(transitions) => {
                                    for transition in transitions.iter() {
                                        self.insert_transition(rule_id, transition);
                                    }
                                }

                                _ => {
                                    self.insert_property(rule_id, property);
                                }
                            }
                        }
                    }

                    CssRule::Keyframes(keyframes_rule) => {
                        let name = keyframes_rule.name.as_string();

                        let animation_id = self.animation_manager.create();

                        for keyframes in keyframes_rule.keyframes {
                            for selector in keyframes.selectors.iter() {
                                let time = match selector {
                                    KeyframeSelector::From => 0.0,
                                    KeyframeSelector::To => 1.0,
                                    KeyframeSelector::Percentage(percentage) => {
                                        percentage.0 / 100.0
                                    }
                                };

                                self.add_keyframe(
                                    animation_id,
                                    time,
                                    &keyframes.declarations.declarations,
                                );
                            }
                        }

                        self.animations.insert(name, animation_id);
                    }

                    _ => {}
                }
            }
        } else {
            println!("Failed to parse stylesheet");
        }
    }

    fn insert_transition(&mut self, rule_id: Rule, transition: &Transition) {
        let animation = self.animation_manager.create();
        match transition.property.as_ref() {
            "display" => {
                self.display.insert_animation(animation, self.add_transition(transition));
                self.display.insert_transition(rule_id, animation);
            }

            "opacity" => {
                self.opacity.insert_animation(animation, self.add_transition(transition));
                self.opacity.insert_transition(rule_id, animation);
            }

            "clip-path" => {
                self.clip_path.insert_animation(animation, self.add_transition(transition));
                self.clip_path.insert_transition(rule_id, animation);
            }

            "transform" => {
                self.transform.insert_animation(animation, self.add_transition(transition));
                self.transform.insert_transition(rule_id, animation);
            }

            "transform-origin" => {
                self.transform_origin.insert_animation(animation, self.add_transition(transition));
                self.transform_origin.insert_transition(rule_id, animation);
            }

            "translate" => {
                self.translate.insert_animation(animation, self.add_transition(transition));
                self.translate.insert_transition(rule_id, animation);
            }

            "rotate" => {
                self.rotate.insert_animation(animation, self.add_transition(transition));
                self.rotate.insert_transition(rule_id, animation);
            }

            "scale" => {
                self.scale.insert_animation(animation, self.add_transition(transition));
                self.scale.insert_transition(rule_id, animation);
            }

            "border" => {
                self.border_width.insert_animation(animation, self.add_transition(transition));
                self.border_width.insert_transition(rule_id, animation);
                self.border_color.insert_animation(animation, self.add_transition(transition));
                self.border_color.insert_transition(rule_id, animation);
            }

            "border-width" => {
                self.border_width.insert_animation(animation, self.add_transition(transition));
                self.border_width.insert_transition(rule_id, animation);
            }

            "border-color" => {
                self.border_color.insert_animation(animation, self.add_transition(transition));
                self.border_color.insert_transition(rule_id, animation);
            }

            "corner-radius" => {
                self.corner_bottom_left_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_bottom_left_radius.insert_transition(rule_id, animation);
                self.corner_bottom_right_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_bottom_right_radius.insert_transition(rule_id, animation);
                self.corner_top_left_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_top_left_radius.insert_transition(rule_id, animation);
                self.corner_top_right_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_top_right_radius.insert_transition(rule_id, animation);
            }

            "corner-top-left-radius" => {
                self.corner_top_left_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_top_left_radius.insert_transition(rule_id, animation);
            }

            "corner-top-right-radius" => {
                self.corner_top_right_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_top_right_radius.insert_transition(rule_id, animation);
            }

            "corner-bottom-left-radius" => {
                self.corner_bottom_left_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_bottom_left_radius.insert_transition(rule_id, animation);
            }

            "corner-bottom-right-radius" => {
                self.corner_bottom_right_radius
                    .insert_animation(animation, self.add_transition(transition));
                self.corner_bottom_right_radius.insert_transition(rule_id, animation);
            }

            "outline" => {
                self.outline_width.insert_animation(animation, self.add_transition(transition));
                self.outline_width.insert_transition(rule_id, animation);
                self.outline_color.insert_animation(animation, self.add_transition(transition));
                self.outline_color.insert_transition(rule_id, animation);
            }

            "outline-width" => {
                self.outline_width.insert_animation(animation, self.add_transition(transition));
                self.outline_width.insert_transition(rule_id, animation);
            }

            "outline-color" => {
                self.outline_color.insert_animation(animation, self.add_transition(transition));
                self.outline_color.insert_transition(rule_id, animation);
            }

            "outline-offset" => {
                self.outline_offset.insert_animation(animation, self.add_transition(transition));
                self.outline_offset.insert_transition(rule_id, animation);
            }

            "background-color" => {
                self.background_color.insert_animation(animation, self.add_transition(transition));
                self.background_color.insert_transition(rule_id, animation);
            }

            "background-image" => {
                self.background_image.insert_animation(animation, self.add_transition(transition));
                self.background_image.insert_transition(rule_id, animation);
            }

            "background-size" => {
                self.background_size.insert_animation(animation, self.add_transition(transition));
                self.background_size.insert_transition(rule_id, animation);
            }

            "shadow" => {
                self.shadow.insert_animation(animation, self.add_transition(transition));
                self.shadow.insert_transition(rule_id, animation);
            }

            "text-shadow" => {
                self.text_shadow.insert_animation(animation, self.add_transition(transition));
                self.text_shadow.insert_transition(rule_id, animation);
            }

            "color" => {
                self.font_color.insert_animation(animation, self.add_transition(transition));
                self.font_color.insert_transition(rule_id, animation);
            }

            "font-size" => {
                self.font_size.insert_animation(animation, self.add_transition(transition));
                self.font_size.insert_transition(rule_id, animation);
            }

            "caret-color" => {
                self.caret_color.insert_animation(animation, self.add_transition(transition));
                self.caret_color.insert_transition(rule_id, animation);
            }

            "selection-color" => {
                self.selection_color.insert_animation(animation, self.add_transition(transition));
                self.selection_color.insert_transition(rule_id, animation);
            }

            "left" => {
                self.left.insert_animation(animation, self.add_transition(transition));
                self.left.insert_transition(rule_id, animation);
            }

            "right" => {
                self.right.insert_animation(animation, self.add_transition(transition));
                self.right.insert_transition(rule_id, animation);
            }

            "top" => {
                self.top.insert_animation(animation, self.add_transition(transition));
                self.top.insert_transition(rule_id, animation);
            }

            "bottom" => {
                self.bottom.insert_animation(animation, self.add_transition(transition));
                self.bottom.insert_transition(rule_id, animation);
            }

            "padding-left" => {
                self.padding_left.insert_animation(animation, self.add_transition(transition));
                self.padding_left.insert_transition(rule_id, animation);
            }

            "padding-right" => {
                self.padding_right.insert_animation(animation, self.add_transition(transition));
                self.padding_right.insert_transition(rule_id, animation);
            }

            "padding-top" => {
                self.padding_top.insert_animation(animation, self.add_transition(transition));
                self.padding_top.insert_transition(rule_id, animation);
            }

            "padding-bottom" => {
                self.padding_bottom.insert_animation(animation, self.add_transition(transition));
                self.padding_bottom.insert_transition(rule_id, animation);
            }

            "horizontal-gap" => {
                self.horizontal_gap.insert_animation(animation, self.add_transition(transition));
                self.horizontal_gap.insert_transition(rule_id, animation);
            }

            "vertical-gap" => {
                self.vertical_gap.insert_animation(animation, self.add_transition(transition));
                self.vertical_gap.insert_transition(rule_id, animation);
            }

            "gap" => {
                self.horizontal_gap.insert_animation(animation, self.add_transition(transition));
                self.horizontal_gap.insert_transition(rule_id, animation);
                self.vertical_gap.insert_animation(animation, self.add_transition(transition));
                self.vertical_gap.insert_transition(rule_id, animation);
            }

            "width" => {
                self.width.insert_animation(animation, self.add_transition(transition));
                self.width.insert_transition(rule_id, animation);
            }

            "height" => {
                self.height.insert_animation(animation, self.add_transition(transition));
                self.height.insert_transition(rule_id, animation);
            }

            "min-width" => {
                self.min_width.insert_animation(animation, self.add_transition(transition));
                self.min_width.insert_transition(rule_id, animation);
            }

            "max-width" => {
                self.max_width.insert_animation(animation, self.add_transition(transition));
                self.max_width.insert_transition(rule_id, animation);
            }

            "min-height" => {
                self.min_height.insert_animation(animation, self.add_transition(transition));
                self.min_height.insert_transition(rule_id, animation);
            }

            "max-height" => {
                self.max_height.insert_animation(animation, self.add_transition(transition));
                self.max_height.insert_transition(rule_id, animation);
            }

            "min-horizontal-gap" => {
                self.min_horizontal_gap
                    .insert_animation(animation, self.add_transition(transition));
                self.min_horizontal_gap.insert_transition(rule_id, animation);
            }

            "max-horizontal-gap" => {
                self.max_horizontal_gap
                    .insert_animation(animation, self.add_transition(transition));
                self.max_horizontal_gap.insert_transition(rule_id, animation);
            }

            "min-vertical-gap" => {
                self.min_vertical_gap.insert_animation(animation, self.add_transition(transition));
                self.min_vertical_gap.insert_transition(rule_id, animation);
            }

            "max-vertical-gap" => {
                self.max_vertical_gap.insert_animation(animation, self.add_transition(transition));
                self.max_vertical_gap.insert_transition(rule_id, animation);
            }

            "underline-color" => {
                self.underline_color.insert_animation(animation, self.add_transition(transition));
                self.underline_color.insert_transition(rule_id, animation);
            }

            "fill" => {
                self.fill.insert_animation(animation, self.add_transition(transition));
                self.fill.insert_transition(rule_id, animation);
            }

            "letter-spacing" => {
                self.letter_spacing.insert_animation(animation, self.add_transition(transition));
                self.letter_spacing.insert_transition(rule_id, animation);
            }

            "word-spacing" => {
                self.word_spacing.insert_animation(animation, self.add_transition(transition));
                self.word_spacing.insert_transition(rule_id, animation);
            }

            "line-height" => {
                self.line_height.insert_animation(animation, self.add_transition(transition));
                self.line_height.insert_transition(rule_id, animation);
            }

            _ => {}
        }
    }

    fn insert_property(&mut self, rule_id: Rule, property: Property) {
        match property {
            // Display
            Property::Display(display) => {
                self.display.insert_rule(rule_id, display);
            }

            // Visibility
            Property::Visibility(visibility) => {
                self.visibility.insert_rule(rule_id, visibility);
            }

            // Opacity
            Property::Opacity(opacity) => {
                self.opacity.insert_rule(rule_id, opacity);
            }

            // Clipping
            Property::ClipPath(clip) => {
                self.clip_path.insert_rule(rule_id, clip);
            }

            // Filters
            Property::BackdropFilter(filter) => {
                self.backdrop_filter.insert_rule(rule_id, filter);
            }

            // Blend Mode
            Property::BlendMode(blend_mode) => {
                self.blend_mode.insert_rule(rule_id, blend_mode);
            }

            // Layout Type
            Property::LayoutType(layout_type) => {
                self.layout_type.insert_rule(rule_id, layout_type);
            }

            // Position Type
            Property::PositionType(position) => {
                self.position_type.insert_rule(rule_id, position);
            }

            Property::Alignment(alignment) => {
                self.alignment.insert_rule(rule_id, alignment);
            }

            // Space
            Property::Space(space) => {
                self.left.insert_rule(rule_id, space);
                self.right.insert_rule(rule_id, space);
                self.top.insert_rule(rule_id, space);
                self.bottom.insert_rule(rule_id, space);
            }

            Property::Left(left) => {
                self.left.insert_rule(rule_id, left);
            }

            Property::Right(right) => {
                self.right.insert_rule(rule_id, right);
            }

            Property::Top(top) => {
                self.top.insert_rule(rule_id, top);
            }

            Property::Bottom(bottom) => {
                self.bottom.insert_rule(rule_id, bottom);
            }

            // Size
            Property::Size(size) => {
                self.width.insert_rule(rule_id, size);
                self.height.insert_rule(rule_id, size);
            }

            Property::Width(width) => {
                self.width.insert_rule(rule_id, width);
            }

            Property::Height(height) => {
                self.height.insert_rule(rule_id, height);
            }

            // Padding
            Property::Padding(padding) => {
                self.padding_left.insert_rule(rule_id, padding);
                self.padding_right.insert_rule(rule_id, padding);
                self.padding_top.insert_rule(rule_id, padding);
                self.padding_bottom.insert_rule(rule_id, padding);
            }

            Property::PaddingLeft(padding_left) => {
                self.padding_left.insert_rule(rule_id, padding_left);
            }

            Property::PaddingRight(padding_right) => {
                self.padding_right.insert_rule(rule_id, padding_right);
            }

            Property::PaddingTop(padding_top) => {
                self.padding_top.insert_rule(rule_id, padding_top);
            }

            Property::PaddingBottom(padding_bottom) => {
                self.padding_bottom.insert_rule(rule_id, padding_bottom);
            }

            Property::VerticalGap(vertical_gap) => {
                self.vertical_gap.insert_rule(rule_id, vertical_gap);
            }

            Property::HorizontalGap(horizontal_gap) => {
                self.horizontal_gap.insert_rule(rule_id, horizontal_gap);
            }

            Property::Gap(gap) => {
                self.horizontal_gap.insert_rule(rule_id, gap);
                self.vertical_gap.insert_rule(rule_id, gap);
            }

            // Size Constraints
            Property::MinSize(min_size) => {
                self.min_width.insert_rule(rule_id, min_size);
                self.min_height.insert_rule(rule_id, min_size);
            }

            Property::MinWidth(min_width) => {
                self.min_width.insert_rule(rule_id, min_width);
            }

            Property::MinHeight(min_height) => {
                self.min_height.insert_rule(rule_id, min_height);
            }

            Property::MaxSize(max_size) => {
                self.max_width.insert_rule(rule_id, max_size);
                self.max_height.insert_rule(rule_id, max_size);
            }

            Property::MaxWidth(max_width) => {
                self.max_width.insert_rule(rule_id, max_width);
            }

            Property::MaxHeight(max_height) => {
                self.max_height.insert_rule(rule_id, max_height);
            }

            // Gap Constraints
            Property::MinGap(min_gap) => {
                self.min_horizontal_gap.insert_rule(rule_id, min_gap);
                self.min_vertical_gap.insert_rule(rule_id, min_gap);
            }

            Property::MinHorizontalGap(min_gap) => {
                self.min_horizontal_gap.insert_rule(rule_id, min_gap);
            }

            Property::MinVerticalGap(min_gap) => {
                self.min_vertical_gap.insert_rule(rule_id, min_gap);
            }

            Property::MaxGap(max_gap) => {
                self.max_horizontal_gap.insert_rule(rule_id, max_gap);
                self.max_vertical_gap.insert_rule(rule_id, max_gap);
            }

            Property::MaxHorizontalGap(max_gap) => {
                self.max_horizontal_gap.insert_rule(rule_id, max_gap);
            }

            Property::MaxVerticalGap(max_gap) => {
                self.max_vertical_gap.insert_rule(rule_id, max_gap);
            }

            // Background Colour
            Property::BackgroundColor(color) => {
                self.background_color.insert_rule(rule_id, color);
            }

            // Border
            Property::Border(border) => {
                if let Some(border_color) = border.color {
                    self.border_color.insert_rule(rule_id, border_color);
                }

                if let Some(border_width) = border.width {
                    self.border_width.insert_rule(rule_id, border_width.into());
                }

                if let Some(border_style) = border.style {
                    self.border_style.insert_rule(rule_id, border_style.top);
                }
            }

            // Border
            Property::BorderWidth(border_width) => {
                self.border_width.insert_rule(rule_id, border_width.top.0);
            }

            Property::BorderColor(color) => {
                self.border_color.insert_rule(rule_id, color);
            }

            Property::BorderStyle(style) => {
                self.border_style.insert_rule(rule_id, style.top);
            }

            // Border Radius
            Property::CornerRadius(corner_radius) => {
                self.corner_bottom_left_radius.insert_rule(rule_id, corner_radius.bottom_left);
                self.corner_bottom_right_radius.insert_rule(rule_id, corner_radius.bottom_right);
                self.corner_top_left_radius.insert_rule(rule_id, corner_radius.top_left);
                self.corner_top_right_radius.insert_rule(rule_id, corner_radius.top_right);
            }

            Property::CornerBottomLeftRadius(corner_radius) => {
                self.corner_bottom_left_radius.insert_rule(rule_id, corner_radius);
            }

            Property::CornerTopLeftRadius(corner_radius) => {
                self.corner_top_left_radius.insert_rule(rule_id, corner_radius);
            }

            Property::CornerBottomRightRadius(corner_radius) => {
                self.corner_bottom_right_radius.insert_rule(rule_id, corner_radius);
            }

            Property::CornerTopRightRadius(corner_radius) => {
                self.corner_top_right_radius.insert_rule(rule_id, corner_radius);
            }

            // Corner Shape
            Property::CornerShape(corner_shape) => {
                self.corner_top_left_shape.insert_rule(rule_id, corner_shape.0);
                self.corner_top_right_shape.insert_rule(rule_id, corner_shape.1);
                self.corner_bottom_right_shape.insert_rule(rule_id, corner_shape.2);
                self.corner_bottom_left_shape.insert_rule(rule_id, corner_shape.3);
            }

            Property::CornerTopLeftShape(corner_shape) => {
                self.corner_top_left_shape.insert_rule(rule_id, corner_shape);
            }

            Property::CornerTopRightShape(corner_shape) => {
                self.corner_top_right_shape.insert_rule(rule_id, corner_shape);
            }

            Property::CornerBottomLeftShape(corner_shape) => {
                self.corner_bottom_left_shape.insert_rule(rule_id, corner_shape);
            }

            Property::CornerBottomRightShape(corner_shape) => {
                self.corner_bottom_right_shape.insert_rule(rule_id, corner_shape);
            }

            // Font Family
            Property::FontFamily(font_family) => {
                self.font_family.insert_rule(
                    rule_id,
                    font_family
                        .iter()
                        .map(|family| match family {
                            FontFamily::Named(name) => FamilyOwned::Named(name.to_string()),
                            FontFamily::Generic(generic) => FamilyOwned::Generic(*generic),
                        })
                        .collect::<Vec<_>>(),
                );
            }

            // Font Color
            Property::FontColor(font_color) => {
                self.font_color.insert_rule(rule_id, font_color);
            }

            // Font Size
            Property::FontSize(font_size) => {
                self.font_size.insert_rule(rule_id, font_size);
            }

            // Font Weight
            Property::FontWeight(font_weight) => {
                self.font_weight.insert_rule(rule_id, font_weight);
            }

            // Font Slant
            Property::FontSlant(font_slant) => {
                self.font_slant.insert_rule(rule_id, font_slant);
            }

            // Font Width
            Property::FontWidth(font_width) => {
                self.font_width.insert_rule(rule_id, font_width);
            }

            // Font Variation Settings
            Property::FontVariationSettings(font_variation_settings) => {
                self.font_variation_settings.insert_rule(rule_id, font_variation_settings);
            }

            Property::LetterSpacing(letter_spacing) => {
                self.letter_spacing.insert_rule(rule_id, letter_spacing.0);
            }

            Property::WordSpacing(word_spacing) => {
                self.word_spacing.insert_rule(rule_id, word_spacing.0);
            }

            Property::LineHeight(line_height) => {
                self.line_height.insert_rule(rule_id, line_height);
            }

            // Caret Color
            Property::CaretColor(caret_color) => {
                self.caret_color.insert_rule(rule_id, caret_color);
            }

            // Selection Color
            Property::SelectionColor(selection_color) => {
                self.selection_color.insert_rule(rule_id, selection_color);
            }

            // Transform
            Property::Transform(transforms) => {
                self.transform.insert_rule(rule_id, transforms);
            }

            Property::TransformOrigin(transform_origin) => {
                let x = transform_origin.x.to_length_or_percentage();
                let y = transform_origin.y.to_length_or_percentage();
                self.transform_origin.insert_rule(rule_id, Translate { x, y });
            }

            Property::Translate(translate) => {
                self.translate.insert_rule(rule_id, translate);
            }

            Property::Rotate(rotate) => {
                self.rotate.insert_rule(rule_id, rotate);
            }

            Property::Scale(scale) => {
                self.scale.insert_rule(rule_id, scale);
            }

            // Overflow
            Property::Overflow(overflow) => {
                self.overflowx.insert_rule(rule_id, overflow);
                self.overflowy.insert_rule(rule_id, overflow);
            }

            Property::OverflowX(overflow) => {
                self.overflowx.insert_rule(rule_id, overflow);
            }

            Property::OverflowY(overflow) => {
                self.overflowy.insert_rule(rule_id, overflow);
            }

            // Z Index
            Property::ZIndex(z_index) => self.z_index.insert_rule(rule_id, z_index),

            // Outline
            Property::Outline(outline) => {
                if let Some(outline_color) = outline.color {
                    self.outline_color.insert_rule(rule_id, outline_color);
                }

                if let Some(outline_width) = outline.width {
                    self.outline_width.insert_rule(rule_id, outline_width.into());
                }
            }

            Property::OutlineColor(outline_color) => {
                self.outline_color.insert_rule(rule_id, outline_color);
            }

            Property::OutlineWidth(outline_width) => {
                self.outline_width.insert_rule(rule_id, outline_width.left.0);
            }

            Property::OutlineOffset(outline_offset) => {
                self.outline_offset.insert_rule(rule_id, outline_offset);
            }

            // Background Images & Gradients
            Property::BackgroundImage(images) => {
                let images = images
                    .into_iter()
                    .filter_map(|img| match img {
                        BackgroundImage::None => None,
                        BackgroundImage::Gradient(gradient) => {
                            Some(ImageOrGradient::Gradient(*gradient))
                        }
                        BackgroundImage::Url(url) => {
                            Some(ImageOrGradient::Image(url.url.to_string()))
                        }
                    })
                    .collect::<Vec<_>>();

                self.background_image.insert_rule(rule_id, images);
            }

            // Background Size
            Property::BackgroundSize(sizes) => {
                self.background_size.insert_rule(rule_id, sizes);
            }

            // Text Wrapping
            Property::TextWrap(text_wrap) => {
                self.text_wrap.insert_rule(rule_id, text_wrap);
            }

            // Text Alignment
            Property::TextAlign(text_align) => {
                self.text_align.insert_rule(rule_id, text_align);
            }

            // Box Shadows
            Property::Shadow(shadows) => {
                self.shadow.insert_rule(rule_id, shadows);
            }

            Property::TextShadow(shadows) => {
                self.text_shadow.insert_rule(rule_id, shadows);
            }

            // Cursor Icon
            Property::Cursor(cursor) => {
                self.cursor.insert_rule(rule_id, cursor);
            }

            Property::PointerEvents(pointer_events) => {
                self.pointer_events.insert_rule(rule_id, pointer_events);
            }

            // Unparsed. TODO: Log the error.
            Property::Unparsed(unparsed) => {
                warn!("Unparsed: {}", unparsed.name);
            }

            // TODO: Custom property support
            Property::Custom(custom) => {
                warn!("Custom Property: {}", custom.name);
            }
            Property::TextOverflow(text_overflow) => {
                self.text_overflow.insert_rule(rule_id, text_overflow);
            }
            Property::LineClamp(line_clamp) => {
                self.line_clamp.insert_rule(rule_id, line_clamp);
            }
            Property::TextDecorationLine(line) => {
                self.text_decoration_line.insert_rule(rule_id, line);
            }
            Property::TextStroke(stroke) => {
                self.text_stroke_width.insert_rule(rule_id, stroke.width);
                self.text_stroke_style.insert_rule(rule_id, stroke.style);
            }
            Property::TextStrokeWidth(stroke_width) => {
                self.text_stroke_width.insert_rule(rule_id, stroke_width);
            }
            Property::TextStrokeStyle(stroke_style) => {
                self.text_stroke_style.insert_rule(rule_id, stroke_style);
            }
            Property::Fill(fill) => {
                self.fill.insert_rule(rule_id, fill);
            }
            _ => {}
        }
    }

    // Helper function for generating AnimationState from a transition definition.
    fn add_transition<T: Default + Interpolator>(
        &self,
        transition: &Transition,
    ) -> AnimationState<T> {
        let timing_function = transition
            .timing_function
            .map(|easing| match easing {
                EasingFunction::Linear => TimingFunction::linear(),
                EasingFunction::Ease => TimingFunction::ease(),
                EasingFunction::EaseIn => TimingFunction::ease_in(),
                EasingFunction::EaseOut => TimingFunction::ease_out(),
                EasingFunction::EaseInOut => TimingFunction::ease_in_out(),
                EasingFunction::CubicBezier(x1, y1, x2, y2) => TimingFunction::new(x1, y1, x2, y2),
            })
            .unwrap_or_default();

        AnimationState::new(Animation::null())
            .with_duration(transition.duration)
            .with_delay(transition.delay.unwrap_or_default())
            .with_keyframe(Keyframe { time: 0.0, value: Default::default(), timing_function })
            .with_keyframe(Keyframe { time: 1.0, value: Default::default(), timing_function })
    }

    // Add style data for the given entity.
    pub(crate) fn add(&mut self, entity: Entity) {
        self.pseudo_classes.insert(entity, PseudoClassFlags::VALID);
        self.classes.insert(entity, HashSet::new());
        self.abilities.insert(entity, Abilities::default());
        self.system_flags = SystemFlags::RELAYOUT;
        self.restyle.0.insert(entity).unwrap();
        self.reaccess.0.insert(entity).unwrap();
    }

    // Remove style data for the given entity.
    pub(crate) fn remove(&mut self, entity: Entity) {
        self.ids.remove(entity);
        self.classes.remove(entity);
        self.pseudo_classes.remove(entity);
        self.disabled.remove(entity);
        self.abilities.remove(entity);

        self.name.remove(entity);
        self.role.remove(entity);
        // self.default_action_verb.remove(entity);
        self.live.remove(entity);
        self.labelled_by.remove(entity);
        self.hidden.remove(entity);
        self.text_value.remove(entity);
        self.numeric_value.remove(entity);

        // Display
        self.display.remove(entity);
        // Visibility
        self.visibility.remove(entity);
        // Opacity
        self.opacity.remove(entity);
        // Z Order
        self.z_index.remove(entity);
        // Clipping
        self.clip_path.remove(entity);

        self.overflowx.remove(entity);
        self.overflowy.remove(entity);

        // Backdrop Filter
        self.backdrop_filter.remove(entity);

        // Blend Mode
        self.blend_mode.remove(entity);

        // Transform
        self.transform.remove(entity);
        self.transform_origin.remove(entity);
        self.translate.remove(entity);
        self.rotate.remove(entity);
        self.scale.remove(entity);

        // Border
        self.border_width.remove(entity);
        self.border_color.remove(entity);
        self.border_style.remove(entity);

        // Corner Shape
        self.corner_bottom_left_shape.remove(entity);
        self.corner_bottom_right_shape.remove(entity);
        self.corner_top_left_shape.remove(entity);
        self.corner_top_right_shape.remove(entity);

        // Corner Radius
        self.corner_bottom_left_radius.remove(entity);
        self.corner_bottom_right_radius.remove(entity);
        self.corner_top_left_radius.remove(entity);
        self.corner_top_right_radius.remove(entity);

        // Corner Smoothing
        self.corner_bottom_left_smoothing.remove(entity);
        self.corner_bottom_right_smoothing.remove(entity);
        self.corner_top_left_smoothing.remove(entity);
        self.corner_top_right_smoothing.remove(entity);

        // Outline
        self.outline_width.remove(entity);
        self.outline_color.remove(entity);
        self.outline_offset.remove(entity);

        // Background
        self.background_color.remove(entity);
        self.background_image.remove(entity);
        self.background_size.remove(entity);

        // Box Shadow
        self.shadow.remove(entity);
        self.text_shadow.remove(entity);

        // Text and Font
        self.text.remove(entity);
        self.text_wrap.remove(entity);
        self.text_overflow.remove(entity);
        self.line_clamp.remove(entity);
        self.text_align.remove(entity);
        self.font_family.remove(entity);
        self.font_color.remove(entity);
        self.font_size.remove(entity);
        self.font_weight.remove(entity);
        self.font_slant.remove(entity);
        self.font_width.remove(entity);
        self.font_variation_settings.remove(entity);
        self.caret_color.remove(entity);
        self.selection_color.remove(entity);
        self.text_decoration_line.remove(entity);
        self.text_stroke_width.remove(entity);
        self.text_stroke_style.remove(entity);
        self.letter_spacing.remove(entity);
        self.word_spacing.remove(entity);
        self.line_height.remove(entity);

        // Cursor
        self.cursor.remove(entity);

        self.pointer_events.remove(entity);

        // Layout Type
        self.layout_type.remove(entity);

        // Position Type
        self.position_type.remove(entity);

        self.alignment.remove(entity);

        // Space
        self.left.remove(entity);
        self.right.remove(entity);
        self.top.remove(entity);
        self.bottom.remove(entity);

        // Padding
        self.padding_left.remove(entity);
        self.padding_right.remove(entity);
        self.padding_top.remove(entity);
        self.padding_bottom.remove(entity);
        self.vertical_gap.remove(entity);
        self.horizontal_gap.remove(entity);

        // Scrolling
        self.vertical_scroll.remove(entity);
        self.horizontal_scroll.remove(entity);

        // Size
        self.width.remove(entity);
        self.height.remove(entity);

        // Size Constraints
        self.min_width.remove(entity);
        self.max_width.remove(entity);
        self.min_height.remove(entity);
        self.max_height.remove(entity);

        self.min_horizontal_gap.remove(entity);
        self.max_horizontal_gap.remove(entity);
        self.min_vertical_gap.remove(entity);
        self.max_vertical_gap.remove(entity);

        self.text_range.remove(entity);
        self.text_span.remove(entity);

        self.fill.remove(entity);
    }

    pub(crate) fn needs_restyle(&mut self, entity: Entity) {
        self.restyle.0.insert(entity).unwrap();
    }

    pub(crate) fn needs_relayout(&mut self) {
        self.system_flags.set(SystemFlags::RELAYOUT, true);
    }

    pub(crate) fn needs_access_update(&mut self, entity: Entity) {
        self.reaccess.0.insert(entity).unwrap();
    }

    pub(crate) fn needs_text_update(&mut self, entity: Entity) {
        self.text_construction.0.insert(entity).unwrap();
        self.text_layout.0.insert(entity).unwrap();
    }

    pub(crate) fn needs_text_layout(&mut self, entity: Entity) {
        self.text_layout.0.insert(entity).unwrap();
    }

    // pub fn should_redraw<F: FnOnce()>(&mut self, f: F) {
    //     if !self.redraw_list.is_empty() {
    //         f();
    //     }
    // }

    // Remove all shared style data.
    pub(crate) fn clear_style_rules(&mut self) {
        self.disabled.clear_rules();
        // Display
        self.display.clear_rules();
        // Visibility
        self.visibility.clear_rules();
        // Opacity
        self.opacity.clear_rules();
        // Z Order
        self.z_index.clear_rules();

        // Clipping
        self.clip_path.clear_rules();

        // Backdrop Filer
        self.backdrop_filter.clear_rules();

        // Blend Mode
        self.blend_mode.clear_rules();

        // Transform
        self.transform.clear_rules();
        self.transform_origin.clear_rules();
        self.translate.clear_rules();
        self.rotate.clear_rules();
        self.scale.clear_rules();

        self.overflowx.clear_rules();
        self.overflowy.clear_rules();

        // Border
        self.border_width.clear_rules();
        self.border_color.clear_rules();
        self.border_style.clear_rules();

        // Corner Shape
        self.corner_bottom_left_shape.clear_rules();
        self.corner_bottom_right_shape.clear_rules();
        self.corner_top_left_shape.clear_rules();
        self.corner_top_right_shape.clear_rules();

        // Corner Radius
        self.corner_bottom_left_radius.clear_rules();
        self.corner_bottom_right_radius.clear_rules();
        self.corner_top_left_radius.clear_rules();
        self.corner_top_right_radius.clear_rules();

        // Corner Smoothing
        self.corner_bottom_left_smoothing.clear_rules();
        self.corner_bottom_right_smoothing.clear_rules();
        self.corner_top_left_smoothing.clear_rules();
        self.corner_top_right_smoothing.clear_rules();

        // Outline
        self.outline_width.clear_rules();
        self.outline_color.clear_rules();
        self.outline_offset.clear_rules();

        // Background
        self.background_color.clear_rules();
        self.background_image.clear_rules();
        self.background_size.clear_rules();

        self.shadow.clear_rules();
        self.text_shadow.clear_rules();

        self.layout_type.clear_rules();
        self.position_type.clear_rules();
        self.alignment.clear_rules();

        // Space
        self.left.clear_rules();
        self.right.clear_rules();
        self.top.clear_rules();
        self.bottom.clear_rules();

        // Size
        self.width.clear_rules();
        self.height.clear_rules();

        // Size Constraints
        self.min_width.clear_rules();
        self.max_width.clear_rules();
        self.min_height.clear_rules();
        self.max_height.clear_rules();

        self.min_horizontal_gap.clear_rules();
        self.max_horizontal_gap.clear_rules();
        self.min_vertical_gap.clear_rules();
        self.max_vertical_gap.clear_rules();

        // Padding
        self.padding_left.clear_rules();
        self.padding_right.clear_rules();
        self.padding_top.clear_rules();
        self.padding_bottom.clear_rules();
        self.horizontal_gap.clear_rules();
        self.vertical_gap.clear_rules();

        // Scrolling
        self.horizontal_scroll.clear_rules();
        self.vertical_scroll.clear_rules();

        // Text and Font
        self.text_wrap.clear_rules();
        self.text_overflow.clear_rules();
        self.line_clamp.clear_rules();
        self.text_align.clear_rules();
        self.font_family.clear_rules();
        self.font_weight.clear_rules();
        self.font_slant.clear_rules();
        self.font_color.clear_rules();
        self.font_size.clear_rules();
        self.font_variation_settings.clear_rules();
        self.selection_color.clear_rules();
        self.caret_color.clear_rules();
        self.text_decoration_line.clear_rules();
        self.text_stroke_width.clear_rules();
        self.text_stroke_style.clear_rules();
        self.letter_spacing.clear_rules();
        self.word_spacing.clear_rules();
        self.line_height.clear_rules();

        self.cursor.clear_rules();

        self.pointer_events.clear_rules();

        self.name.clear_rules();

        self.fill.clear_rules();
    }
}
