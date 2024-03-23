use skia_safe::gradient_shader::GradientShaderColors;
use skia_safe::image_filters::CropRect;
use skia_safe::{
    BlurStyle, ClipOp, FilterMode, IRect, ImageFilter, MaskFilter, Matrix, Paint, PaintStyle, Path,
    PathDirection, Point, RRect, Rect, SamplingOptions, Shader, TileMode,
};
use vizia_style::LengthPercentageOrAuto;
// use femtovg::{ImageId, Transform2D};
use std::any::{Any, TypeId};

use hashbrown::HashMap;

use crate::animation::Interpolator;
use crate::cache::CachedData;
use crate::events::ViewHandler;
use crate::model::ModelDataStore;
use crate::prelude::*;
use crate::resource::ResourceManager;
use crate::text::{TextConfig, TextContext};
use vizia_input::MouseState;

/// A context used when drawing a view.
///
/// The `DrawContext` is provided by the [`draw`](crate::view::View::draw) method in [`View`] and can be used to immutably access the
/// computed style and layout properties of the current view.
///
/// # Example
/// ```
/// # use vizia_core::prelude::*;
/// # use vizia_core::vg;
/// # let cx = &mut Context::default();
///
/// pub struct CustomView {}
///
/// impl CustomView {
///     pub fn new(cx: &mut Context) -> Handle<Self> {
///         Self{}.build(cx, |_|{})
///     }
/// }
///
/// impl View for CustomView {
///     fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
///         // Get the computed bounds after layout of the current view
///         let bounds = cx.bounds();
///         // Draw to the canvas using the bounds of the current view
///         let mut path = vg::Path::new();
///         path.rect(bounds.x, bounds.y, bounds.w, bounds.h);
///         canvas.fill_path(&mut path, &vg::Paint::color(vg::Color::rgb(200, 100, 100)));
///     }
/// }
/// ```
pub struct DrawContext<'a> {
    pub(crate) current: Entity,
    pub(crate) style: &'a Style,
    pub(crate) cache: &'a mut CachedData,
    pub(crate) tree: &'a Tree<Entity>,
    pub(crate) data: &'a HashMap<Entity, ModelDataStore>,
    pub(crate) views: &'a mut HashMap<Entity, Box<dyn ViewHandler>>,
    pub(crate) resource_manager: &'a ResourceManager,
    pub(crate) text_context: &'a mut TextContext,
    pub(crate) text_config: &'a TextConfig,
    pub(crate) modifiers: &'a Modifiers,
    pub(crate) mouse: &'a MouseState<Entity>,
    pub(crate) opacity: f32,
}

macro_rules! get_units_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> Units {
            let result = self.style.$name.get(self.current);
            if let Some(Units::Pixels(p)) = result {
                Units::Pixels(self.logical_to_physical(*p))
            } else {
                result.copied().unwrap_or_default()
            }
        }
    };
}

macro_rules! get_color_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> Color {
            if let Some(col) = self.style.$name.get(self.current) {
                Color::rgba(col.r(), col.g(), col.b(), col.a())
            } else {
                Color::rgba(0, 0, 0, 0)
            }
        }
    };
}

macro_rules! get_length_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> f32 {
            if let Some(length) = self.style.$name.get(self.current) {
                let bounds = self.bounds();

                let px = length.to_pixels(bounds.w.min(bounds.h), self.scale_factor());
                return px.round();
            }

            0.0
        }
    };
}

impl<'a> DrawContext<'a> {
    /// Returns the bounds of the current view.
    pub fn bounds(&self) -> BoundingBox {
        self.cache.get_bounds(self.current)
    }

    /// Returns the scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.style.dpi_factor as f32
    }

    /// Returns a reference to the keyboard modifiers state.
    pub fn modifiers(&self) -> &Modifiers {
        self.modifiers
    }

    /// Returns a reference to the mouse state.
    pub fn mouse(&self) -> &MouseState<Entity> {
        self.mouse
    }

    /// Returns the clip path of the current view.
    pub fn clip_path(&self) -> Option<skia_safe::Path> {
        let bounds = self.bounds();
        let overflowx = self.style.overflowx.get(self.current).copied().unwrap_or_default();
        let overflowy = self.style.overflowy.get(self.current).copied().unwrap_or_default();

        let scale = self.scale_factor();

        let clip_bounds = self
            .style
            .clip_path
            .get(self.current)
            .map(|clip| match clip {
                ClipPath::Auto => bounds,
                ClipPath::Shape(rect) => bounds.shrink_sides(
                    rect.3.to_pixels(bounds.w, scale),
                    rect.0.to_pixels(bounds.h, scale),
                    rect.1.to_pixels(bounds.w, scale),
                    rect.2.to_pixels(bounds.h, scale),
                ),
            })
            .unwrap_or(bounds);

        let root_bounds = self.cache.get_bounds(Entity::root());

        let clip_bounds = match (overflowx, overflowy) {
            (Overflow::Visible, Overflow::Visible) => return None,
            (Overflow::Hidden, Overflow::Visible) => {
                let left = clip_bounds.left();
                let right = clip_bounds.right();
                let top = root_bounds.top();
                let bottom = root_bounds.bottom();
                BoundingBox::from_min_max(left, top, right, bottom)
            }
            (Overflow::Visible, Overflow::Hidden) => {
                let left = root_bounds.left();
                let right = root_bounds.right();
                let top = clip_bounds.top();
                let bottom = clip_bounds.bottom();
                BoundingBox::from_min_max(left, top, right, bottom)
            }
            (Overflow::Hidden, Overflow::Hidden) => clip_bounds,
        };

        Some(self.build_path(clip_bounds))
    }

    /// Returns the 2D transform of the current view.
    pub fn transform(&self) -> Matrix {
        let bounds = self.bounds();
        let scale_factor = self.scale_factor();

        // Apply transform origin.
        let mut origin = self
            .style
            .transform_origin
            .get(self.current)
            .map(|transform_origin| {
                let mut origin = Matrix::translate(bounds.top_left());
                let offset = transform_origin.as_transform(bounds, scale_factor);
                origin = offset * origin;
                origin
            })
            .unwrap_or(Matrix::translate(bounds.center()));
        // transform = origin * transform;
        let mut transform = origin;
        origin = origin.invert().unwrap();

        // Apply translation.
        if let Some(translate) = self.style.translate.get(self.current) {
            transform = transform * translate.as_transform(bounds, scale_factor);
        }

        // Apply rotation.
        if let Some(rotate) = self.style.rotate.get(self.current) {
            transform = transform * rotate.as_transform(bounds, scale_factor);
        }

        // Apply scaling.
        if let Some(scale) = self.style.scale.get(self.current) {
            transform = transform * scale.as_transform(bounds, scale_factor);
        }

        // Apply transform functions.
        if let Some(transforms) = self.style.transform.get(self.current) {
            // Check if the transform is currently animating
            // Get the animation state
            // Manually interpolate the value to get the overall transform for the current frame
            if let Some(animation_state) = self.style.transform.get_active_animation(self.current) {
                if let Some(start) = animation_state.keyframes.first() {
                    if let Some(end) = animation_state.keyframes.last() {
                        let start_transform = start.value.as_transform(bounds, scale_factor);
                        let end_transform = end.value.as_transform(bounds, scale_factor);
                        let t = animation_state.t;
                        let animated_transform =
                            Matrix::interpolate(&start_transform, &end_transform, t);
                        transform = transform * animated_transform;
                    }
                }
            } else {
                transform = transform * transforms.as_transform(bounds, scale_factor);
            }
        }

        transform = transform * origin;

        transform
    }

    /// Returns the visibility of the current view.
    pub fn visibility(&self) -> Option<Visibility> {
        self.style.visibility.get(self.current).copied()
    }

    /// Returns the display of the current view.
    pub fn display(&self) -> Display {
        self.style.display.get(self.current).copied().unwrap_or(Display::Flex)
    }

    /// Returns the opacity of the current view.
    pub fn opacity(&self) -> f32 {
        self.style.opacity.get(self.current).copied().unwrap_or(Opacity(1.0)).0
    }

    /// Returns the lookup pattern to pick the default font.
    pub fn default_font(&self) -> &[FamilyOwned] {
        &self.style.default_font
    }

    /// Returns the font-size of the current view in physical pixels.
    pub fn font_size(&self) -> f32 {
        self.logical_to_physical(
            self.style.font_size.get(self.current).copied().map(|f| f.0).unwrap_or(16.0),
        )
    }

    pub fn font_weight(&self) -> FontWeight {
        self.style.font_weight.get(self.current).copied().unwrap_or_default()
    }

    pub fn font_width(&self) -> FontWidth {
        self.style.font_width.get(self.current).copied().unwrap_or_default()
    }

    pub fn font_slant(&self) -> FontSlant {
        self.style.font_slant.get(self.current).copied().unwrap_or_default()
    }

    pub fn font_variation_settings(&self) -> &[FontVariation] {
        self.style.font_variation_settings.get(self.current).map(Vec::as_slice).unwrap_or_default()
    }

    /// Function to convert logical points to physical pixels.
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        self.style.logical_to_physical(logical)
    }

    /// Function to convert physical pixels to logical points.
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        self.style.physical_to_logical(physical)
    }

    get_length_property!(
        /// Returns the border width of the current view in physical pixels.
        border_width
    );

    get_color_property!(
        /// Returns the outline color of the current view.
        outline_color
    );

    get_length_property!(
        /// Returns the outline width of the current view in physical pixels.
        outline_width
    );

    get_length_property!(
        /// Returns the outline offset of the current view in physcial pixels.
        outline_offset
    );

    get_length_property!(
        /// Returns the border radius for the top-left corner of the current view.
        border_top_left_radius
    );

    get_length_property!(
        /// Returns the border radius for the top-right corner of the current view.
        border_top_right_radius
    );

    get_length_property!(
        /// Returns the border radius for the bottom-left corner of the current view.
        border_bottom_left_radius
    );

    get_length_property!(
        /// Returns the border radius for the bottom-right corner of the current view.
        border_bottom_right_radius
    );

    /// Returns the border corner shape for the top-left corner of the current view.
    pub fn border_top_left_shape(&self) -> BorderCornerShape {
        self.style.border_top_left_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the border corner shape for the top-left corner of the current view.
    pub fn border_top_right_shape(&self) -> BorderCornerShape {
        self.style.border_top_right_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the border corner shape for the top-left corner of the current view.
    pub fn border_bottom_left_shape(&self) -> BorderCornerShape {
        self.style.border_bottom_left_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the border corner shape for the top-left corner of the current view.
    pub fn border_bottom_right_shape(&self) -> BorderCornerShape {
        self.style.border_bottom_right_shape.get(self.current).copied().unwrap_or_default()
    }

    get_units_property!(
        /// Returns the child-left space of the current view.
        child_left
    );

    get_units_property!(
        /// Returns the child-right space of the current view.
        child_right
    );

    get_units_property!(
        /// Returns the child-top space of the current view.
        child_top
    );

    get_units_property!(
        /// Returns the child-bottom space of the current view.
        child_bottom
    );

    get_color_property!(background_color);
    get_color_property!(border_color);

    get_color_property!(selection_color);
    get_color_property!(caret_color);
    get_color_property!(font_color);

    /// Returns whether the current view should have its text wrapped.
    pub fn text_wrap(&self) -> bool {
        self.style.text_wrap.get(self.current).copied().unwrap_or(true)
    }

    pub fn text_align(&self) -> TextAlign {
        self.style.text_align.get(self.current).copied().unwrap_or_default()
    }

    pub fn text_overflow(&self) -> TextOverflow {
        self.style.text_overflow.get(self.current).copied().unwrap_or_default()
    }

    pub fn line_clamp(&self) -> Option<usize> {
        self.style.line_clamp.get(self.current).copied().map(|lc| lc.0 as usize)
    }

    pub fn box_shadows(&self) -> Option<&Vec<BoxShadow>> {
        self.style.box_shadow.get(self.current)
    }

    pub fn backdrop_filter(&self) -> Option<&Filter> {
        self.style.backdrop_filter.get(self.current)
    }

    pub fn background_images(&self) -> Option<&Vec<ImageOrGradient>> {
        self.style.background_image.get(self.current)
    }

    pub fn background_size(&self) -> Vec<BackgroundSize> {
        self.style.background_size.get(self.current).cloned().unwrap_or_default()
    }

    /// Get the vector path of the current view.
    pub fn build_path(&self, bounds: BoundingBox) -> Path {
        // Length proportional to radius of a cubic bezier handle for 90deg arcs.
        const KAPPA90: f32 = 0.552_284_8;

        let border_width = self.border_width();

        let border_top_left_radius = self.border_top_left_radius();
        let border_top_right_radius = self.border_top_right_radius();
        let border_bottom_right_radius = self.border_bottom_right_radius();
        let border_bottom_left_radius = self.border_bottom_left_radius();

        let border_top_left_shape = self.border_top_left_shape();
        let border_top_right_shape = self.border_top_right_shape();
        let border_bottom_right_shape = self.border_bottom_right_shape();
        let border_bottom_left_shape = self.border_bottom_left_shape();

        //TODO: Cache the path and regenerate if the bounds change
        let mut path = Path::new();

        if bounds.w == bounds.h
            && border_bottom_left_radius == bounds.w / 2.0
            && border_bottom_right_radius == bounds.w / 2.0
            && border_top_left_radius == bounds.h / 2.0
            && border_top_right_radius == bounds.h / 2.0
        {
            path.add_circle(
                bounds.center(),
                bounds.w / 2.0 - border_width / 2.0,
                PathDirection::CW,
            );
        } else {
            let x = bounds.x + border_width / 2.0;
            let y = bounds.y + border_width / 2.0;
            let w = bounds.w - border_width;
            let h = bounds.h - border_width;
            let halfw = w.abs() * 0.5;
            let halfh = h.abs() * 0.5;

            let rx_bl = border_bottom_left_radius.min(halfw) * w.signum();
            let ry_bl = border_bottom_left_radius.min(halfh) * h.signum();

            let rx_br = border_bottom_right_radius.min(halfw) * w.signum();
            let ry_br = border_bottom_right_radius.min(halfh) * h.signum();

            let rx_tr = border_top_right_radius.min(halfw) * w.signum();
            let ry_tr = border_top_right_radius.min(halfh) * h.signum();

            let rx_tl = border_top_left_radius.min(halfw) * w.signum();
            let ry_tl = border_top_left_radius.min(halfh) * h.signum();

            path.move_to((x, y + ry_tl));
            path.line_to((x, y + h - ry_bl));
            if border_bottom_left_radius != 0.0 {
                if border_bottom_left_shape == BorderCornerShape::Round {
                    path.cubic_to(
                        (x, y + h - ry_bl * (1.0 - KAPPA90)),
                        (x + rx_bl * (1.0 - KAPPA90), y + h),
                        (x + rx_bl, y + h),
                    );
                } else {
                    path.line_to((x + rx_bl, y + h));
                }
            }

            path.line_to((x + w - rx_br, y + h));

            if border_bottom_right_radius != 0.0 {
                if border_bottom_right_shape == BorderCornerShape::Round {
                    path.cubic_to(
                        (x + w - rx_br * (1.0 - KAPPA90), y + h),
                        (x + w, y + h - ry_br * (1.0 - KAPPA90)),
                        (x + w, y + h - ry_br),
                    );
                } else {
                    path.line_to((x + w, y + h - ry_br));
                }
            }

            path.line_to((x + w, y + ry_tr));

            if border_top_right_radius != 0.0 {
                if border_top_right_shape == BorderCornerShape::Round {
                    path.cubic_to(
                        (x + w, y + ry_tr * (1.0 - KAPPA90)),
                        (x + w - rx_tr * (1.0 - KAPPA90), y),
                        (x + w - rx_tr, y),
                    );
                } else {
                    path.line_to((x + w - rx_tr, y));
                }
            }

            path.line_to((x + rx_tl, y));

            if border_top_left_radius != 0.0 {
                if border_top_left_shape == BorderCornerShape::Round {
                    path.cubic_to(
                        (x + rx_tl * (1.0 - KAPPA90), y),
                        (x, y + ry_tl * (1.0 - KAPPA90)),
                        (x, y + ry_tl),
                    );
                } else {
                    path.line_to((x, y + ry_tl));
                }
            }

            path.close();
        }

        path
    }

    /// Draw background color or background image (including gradients) for the current view.
    pub fn draw_background(&mut self, canvas: &Canvas, path: &Path) {
        let background_color = self.background_color();
        if background_color.a() != 0 {
            let mut paint = Paint::default();
            paint.set_color(skia_safe::Color::from_argb(
                background_color.a(),
                background_color.r(),
                background_color.g(),
                background_color.b(),
            ));
            paint.set_anti_alias(true);
            canvas.draw_path(path, &paint);
        }

        self.draw_background_images(canvas, path);
    }

    // /// Draw backdrop filters for the current view.
    // pub fn draw_backdrop_filter(&mut self, canvas: &mut Canvas, path: &mut Path) {
    //     let window_width = self.cache.get_width(Entity::root());
    //     let window_height = self.cache.get_height(Entity::root());
    //     let bounds = self.bounds();

    //     let blur_radius = self.backdrop_filter().map(|filter| match filter {
    //         Filter::Blur(r) => r.to_px().unwrap_or_default(),
    //     });

    //     if let Some(blur_radius) = blur_radius {
    //         let sigma = blur_radius / 2.0;

    //         let filter_image =
    //             self.cache.filter_image.get(self.current).cloned().unwrap_or_default();

    //         fn create_images(canvas: &mut Canvas, w: usize, h: usize) -> (ImageId, ImageId) {
    //             (
    //                 canvas
    //                     .create_image_empty(
    //                         w,
    //                         h,
    //                         femtovg::PixelFormat::Rgba8,
    //                         femtovg::ImageFlags::FLIP_Y | femtovg::ImageFlags::PREMULTIPLIED,
    //                     )
    //                     .unwrap(),
    //                 canvas
    //                     .create_image_empty(
    //                         w,
    //                         h,
    //                         femtovg::PixelFormat::Rgba8,
    //                         femtovg::ImageFlags::FLIP_Y | femtovg::ImageFlags::PREMULTIPLIED,
    //                     )
    //                     .unwrap(),
    //             )
    //         }

    //         let (source, target) = match filter_image {
    //             Some((s, t)) => {
    //                 let image_size = canvas.image_size(s).unwrap();
    //                 if image_size.0 != bounds.w as usize || image_size.1 != bounds.h as usize {
    //                     canvas.delete_image(s);
    //                     canvas.delete_image(t);

    //                     create_images(canvas, bounds.w as usize, bounds.h as usize)
    //                 } else {
    //                     (s, t)
    //                 }
    //             }

    //             None => create_images(canvas, bounds.w as usize, bounds.h as usize),
    //         };

    //         self.cache.filter_image.insert(self.current, Some((source, target)));

    //         // TODO: Cache these
    //         let screenshot = canvas.screenshot().unwrap();

    //         let screenshot_image =
    //             self.cache.screenshot_image.get(self.current).cloned().unwrap_or_default();

    //         let screenshot_image_id = if let Some(s) = screenshot_image {
    //             let image_size = canvas.image_size(s).unwrap();
    //             if image_size.0 != screenshot.width() || image_size.1 != screenshot.height() {
    //                 canvas.delete_image(s);
    //                 canvas.create_image(screenshot.as_ref(), femtovg::ImageFlags::empty()).unwrap()
    //             } else {
    //                 canvas
    //                     .update_image(s, screenshot.as_ref(), 0, 0)
    //                     .expect("Failed to update image");
    //                 s
    //             }
    //         } else {
    //             canvas.create_image(screenshot.as_ref(), femtovg::ImageFlags::empty()).unwrap()
    //         };

    //         self.cache.screenshot_image.insert(self.current, Some(screenshot_image_id));

    //         // Draw canvas to source image
    //         canvas.save();
    //         canvas.set_render_target(femtovg::RenderTarget::Image(source));
    //         canvas.reset_scissor();
    //         canvas.reset_transform();
    //         canvas.clear_rect(
    //             0,
    //             0,
    //             bounds.w as u32,
    //             bounds.h as u32,
    //             femtovg::Color::rgba(0, 0, 0, 0),
    //         );
    //         let mut p = femtovg::Path::new();
    //         p.rect(0.0, 0.0, bounds.w, bounds.h);
    //         canvas.fill_path(
    //             &p,
    //             &Paint::image(
    //                 screenshot_image_id,
    //                 -bounds.x,
    //                 -bounds.y,
    //                 window_width,
    //                 window_height,
    //                 0.0,
    //                 1.0,
    //             ),
    //         );

    //         let blurred_image = if blur_radius > 0.0 {
    //             canvas.filter_image(target, femtovg::ImageFilter::GaussianBlur { sigma }, source);
    //             target
    //         } else {
    //             source
    //         };
    //         canvas.restore();
    //         canvas.set_render_target(femtovg::RenderTarget::Screen);

    //         canvas.fill_path(
    //             path,
    //             &Paint::image(blurred_image, bounds.x, bounds.y, bounds.w, bounds.h, 0.0, 1.0),
    //         );
    //     }
    // }

    // pub fn draw_text_and_selection(&mut self, canvas: &mut Canvas) {
    //     if self.text_context.has_buffer(self.current) {
    //         let mut bounds = self.bounds();
    //         let border_width = self.border_width();

    //         bounds = bounds.shrink(border_width);

    //         let child_left = self.child_left();
    //         let child_right = self.child_right();
    //         let child_top = self.child_top();
    //         let child_bottom = self.child_bottom();

    //         // shrink the bounding box based on pixel values
    //         let left = child_left.to_px(self.bounds().w, 0.0);
    //         let right = child_right.to_px(self.bounds().w, 0.0);
    //         let top = child_top.to_px(self.bounds().h, 0.0);
    //         let bottom = child_bottom.to_px(self.bounds().h, 0.0);

    //         bounds = bounds.shrink_sides(left, top, right, bottom);

    //         // Draw text

    //         let mut justify_x = match (child_left, child_right) {
    //             (Stretch(left), Stretch(right)) => {
    //                 if left + right == 0.0 {
    //                     0.5
    //                 } else {
    //                     left / (left + right)
    //                 }
    //             }
    //             (Stretch(_), _) => 1.0,
    //             _ => 0.0,
    //         };

    //         if let Some(text_align) = self.text_align() {
    //             justify_x = match text_align {
    //                 TextAlign::Left => 0.0,
    //                 TextAlign::Right => 1.0,
    //                 TextAlign::Center => 0.5,
    //                 _ => 0.0,
    //             };
    //         }

    //         let justify_y = match (child_top, child_bottom) {
    //             (Stretch(top), Stretch(bottom)) => {
    //                 if top + bottom == 0.0 {
    //                     0.5
    //                 } else {
    //                     top / (top + bottom)
    //                 }
    //             }
    //             (Stretch(_), _) => 1.0,
    //             _ => 0.0,
    //         };

    //         // let origin_x = box_x + box_w * justify_x;
    //         // let origin_y = box_y + (box_h * justify_y).round();

    //         // let justify_x = 0.0;
    //         // let justify_y = 0.0;
    //         // let origin_x = box_x;
    //         // let origin_y = box_y;

    //         self.text_context.sync_styles(self.current, self.style);

    //         self.draw_text_selection(canvas, bounds, (justify_x, justify_y));
    //         self.draw_text_caret(canvas, bounds, (justify_x, justify_y), 1.0);
    //         self.draw_text(canvas, bounds, (justify_x, justify_y));
    //     }
    // }

    /// Draw the border of the current view.
    pub fn draw_border(&mut self, canvas: &Canvas, path: &Path) {
        let border_color = self.border_color();
        let border_width = self.border_width();

        if border_width > 0.0 && border_color.a() > 0 {
            let mut paint = Paint::default();
            paint.set_style(PaintStyle::Stroke);
            paint.set_color(border_color);
            paint.set_stroke_width(border_width);
            paint.set_anti_alias(true);
            canvas.draw_path(path, &paint);
        }
    }

    /// Draw the outline of the current view.
    pub fn draw_outline(&mut self, canvas: &Canvas) {
        let outline_width = self.outline_width();
        let outline_color = self.outline_color();

        if outline_width >= 0.0 && outline_color.a() != 0 {
            let outline_offset = self.outline_offset();

            let bounds = self.bounds();

            let border_top_left_radius = self.border_top_left_radius();
            let border_top_right_radius = self.border_top_right_radius();
            let border_bottom_right_radius = self.border_bottom_right_radius();
            let border_bottom_left_radius = self.border_bottom_left_radius();

            let half_outline_width = outline_width / 2.0;
            let outline_path = RRect::new_rect_radii(
                Rect::new(
                    bounds.x - half_outline_width - outline_offset,
                    bounds.y - half_outline_width - outline_offset,
                    bounds.x - half_outline_width - outline_offset
                        + bounds.w
                        + outline_width
                        + 2.0 * outline_offset,
                    bounds.y - half_outline_width - outline_offset
                        + bounds.h
                        + outline_width
                        + 2.0 * outline_offset,
                ),
                &[
                    Point::new(border_top_left_radius * 1.5, border_top_left_radius * 1.5),
                    Point::new(border_top_right_radius * 1.5, border_top_left_radius * 1.5),
                    Point::new(border_bottom_right_radius * 1.5, border_bottom_right_radius * 1.5),
                    Point::new(border_bottom_left_radius * 1.5, border_bottom_left_radius * 1.5),
                ],
            );

            let mut outline_paint = Paint::default();
            outline_paint.set_color(outline_color);
            outline_paint.set_stroke_width(outline_width);
            outline_paint.set_style(PaintStyle::Stroke);
            outline_paint.set_anti_alias(true);
            canvas.draw_rrect(&outline_path, &outline_paint);
        }
    }

    // /// Draw inset box shadows for the current view.
    // pub fn draw_inset_box_shadows(&mut self, canvas: &mut Canvas, path: &mut Path) {
    //     if let Some(box_shadows) = self.box_shadows() {
    //         if box_shadows.is_empty() {
    //             return;
    //         }

    //         let mut shadow_images =
    //             self.cache.shadow_images.get(self.current).cloned().unwrap_or_default();

    //         if shadow_images.len() < box_shadows.len() {
    //             shadow_images.resize(box_shadows.len(), None);
    //         } else {
    //             let excess = shadow_images.split_off(box_shadows.len());
    //             for (s, t) in excess.into_iter().flatten() {
    //                 canvas.delete_image(s);
    //                 canvas.delete_image(t);
    //             }
    //         }

    //         for (index, box_shadow) in
    //             box_shadows.iter().enumerate().rev().filter(|(_, shadow)| shadow.inset)
    //         {
    //             let color = box_shadow.color.unwrap_or_default();
    //             let x_offset = box_shadow.x_offset.to_px().unwrap_or(0.0) * self.scale_factor();
    //             let y_offset = box_shadow.y_offset.to_px().unwrap_or(0.0) * self.scale_factor();
    //             let spread_radius =
    //                 box_shadow.spread_radius.as_ref().and_then(|l| l.to_px()).unwrap_or(0.0)
    //                     * self.scale_factor();

    //             let blur_radius =
    //                 box_shadow.blur_radius.as_ref().and_then(|br| br.to_px()).unwrap_or(0.0);
    //             let sigma = blur_radius / 2.0;
    //             let d = (sigma * 5.0).ceil() + 2.0 * spread_radius + 20.0;

    //             let bounds = self.bounds();

    //             let (source, target) =
    //                 shadow_images[index].map(|(s, t)| (Some(s), Some(t))).unwrap_or((None, None));

    //             fn create_images(canvas: &mut Canvas, w: usize, h: usize) -> (ImageId, ImageId) {
    //                 (
    //                     canvas
    //                         .create_image_empty(
    //                             w,
    //                             h,
    //                             femtovg::PixelFormat::Rgba8,
    //                             femtovg::ImageFlags::FLIP_Y | femtovg::ImageFlags::PREMULTIPLIED,
    //                         )
    //                         .unwrap(),
    //                     canvas
    //                         .create_image_empty(
    //                             w,
    //                             h,
    //                             femtovg::PixelFormat::Rgba8,
    //                             femtovg::ImageFlags::FLIP_Y | femtovg::ImageFlags::PREMULTIPLIED,
    //                         )
    //                         .unwrap(),
    //                 )
    //             }

    //             let (source, target) = match (source, target) {
    //                 (Some(s), Some(t)) => {
    //                     if canvas.image_size(s).unwrap().0 != (bounds.w + d) as usize {
    //                         canvas.delete_image(s);
    //                         canvas.delete_image(t);

    //                         create_images(canvas, (bounds.w + d) as usize, (bounds.h + d) as usize)
    //                     } else {
    //                         (s, t)
    //                     }
    //                 }

    //                 (None, None) => {
    //                     create_images(canvas, (bounds.w + d) as usize, (bounds.h + d) as usize)
    //                 }

    //                 _ => unreachable!(),
    //             };

    //             shadow_images[index] = Some((source, target));

    //             canvas.save();
    //             canvas.set_render_target(femtovg::RenderTarget::Image(source));
    //             canvas.reset_scissor();
    //             canvas.reset_transform();
    //             canvas.clear_rect(
    //                 0,
    //                 0,
    //                 (bounds.w + d) as u32,
    //                 (bounds.h + d) as u32,
    //                 femtovg::Color::rgba(0, 0, 0, 0),
    //             );

    //             let scalex = 1.0 - (2.0 * spread_radius / bounds.w);
    //             let scaley = 1.0 - (2.0 * spread_radius / bounds.h);
    //             canvas.translate(
    //                 (-bounds.x - bounds.w / 2.0) * scalex,
    //                 (-bounds.y - bounds.h / 2.0) * scaley,
    //             );
    //             canvas.scale(scalex, scaley);
    //             canvas.translate(
    //                 (bounds.w / 2.0 + d / 2.0) / scalex,
    //                 (bounds.h / 2.0 + d / 2.0) / scaley,
    //             );
    //             let paint = Paint::color(color.into());
    //             let mut shadow_path = path.clone();
    //             shadow_path.rect(
    //                 bounds.x - d / 2.0,
    //                 bounds.y - d / 2.0,
    //                 bounds.w + d,
    //                 bounds.h + d,
    //             );

    //             shadow_path.solidity(femtovg::Solidity::Hole);
    //             canvas.fill_path(&shadow_path, &paint);
    //             canvas.restore();

    //             let target_image = if blur_radius > 0.0 {
    //                 canvas.filter_image(
    //                     target,
    //                     femtovg::ImageFilter::GaussianBlur { sigma },
    //                     source,
    //                 );
    //                 target
    //             } else {
    //                 source
    //             };

    //             canvas.set_render_target(femtovg::RenderTarget::Screen);
    //             canvas.save();

    //             let paint = Paint::image(
    //                 target_image,
    //                 bounds.x - d / 2.0 + x_offset - 1.5,
    //                 bounds.y - d / 2.0 + y_offset - 1.5,
    //                 bounds.w + d + 3.0,
    //                 bounds.h + d + 3.0,
    //                 0f32,
    //                 1f32,
    //             );

    //             canvas.fill_path(path, &paint);

    //             canvas.restore();
    //         }
    //         self.cache.shadow_images.insert(self.current, shadow_images);
    //     }
    // }

    /// Draw non-inset box shadows for the current view.
    pub fn draw_shadows(&mut self, canvas: &Canvas, path: &Path) {
        if let Some(box_shadows) = self.box_shadows() {
            if box_shadows.is_empty() {
                return;
            }

            let bounds = self.bounds();

            for box_shadow in box_shadows.iter().rev() {
                let shadow_color = box_shadow.color.unwrap_or_default();

                let shadow_x_offset =
                    box_shadow.x_offset.to_px().unwrap_or(0.0) * self.scale_factor();
                let shadow_y_offset =
                    box_shadow.y_offset.to_px().unwrap_or(0.0) * self.scale_factor();
                let spread_radius =
                    box_shadow.spread_radius.as_ref().and_then(|l| l.to_px()).unwrap_or(0.0)
                        * self.scale_factor();

                let blur_radius =
                    box_shadow.blur_radius.as_ref().and_then(|br| br.to_px()).unwrap_or(0.0);

                if shadow_color.a() == 0
                    || (shadow_x_offset == 0.0
                        && shadow_y_offset == 0.0
                        && spread_radius == 0.0
                        && blur_radius == 0.0)
                {
                    continue;
                }

                let mut shadow_paint = Paint::default();

                let outset = if box_shadow.inset { -spread_radius } else { spread_radius };

                shadow_paint.set_style(PaintStyle::Fill);

                let mut shadow_path = self.build_path(bounds.expand(outset));

                shadow_paint.set_color(shadow_color);

                if blur_radius > 0.0 {
                    shadow_paint.set_mask_filter(MaskFilter::blur(
                        BlurStyle::Normal,
                        blur_radius / 2.0,
                        false,
                    ));
                }

                shadow_path.offset((shadow_x_offset, shadow_y_offset));

                if box_shadow.inset {
                    shadow_path = path.op(&shadow_path, skia_safe::PathOp::Difference).unwrap();
                }

                canvas.save();
                canvas.clip_path(
                    &path,
                    if box_shadow.inset { ClipOp::Intersect } else { ClipOp::Difference },
                    true,
                );
                canvas.draw_path(&shadow_path, &shadow_paint);
                canvas.restore();
            }
        }
    }

    /// Draw background images (including gradients) for the current view.
    fn draw_background_images(&self, canvas: &Canvas, path: &Path) {
        let bounds = self.bounds();

        if let Some(images) = self.background_images() {
            let image_sizes = self.background_size();

            for (index, image) in images.iter().enumerate() {
                match image {
                    ImageOrGradient::Gradient(gradient) => match gradient {
                        Gradient::Linear(linear_gradient) => {
                            let (start, end, parent_length) = match linear_gradient.direction {
                                LineDirection::Horizontal(horizontal_keyword) => {
                                    match horizontal_keyword {
                                        HorizontalPositionKeyword::Left => (
                                            bounds.center_right(),
                                            bounds.center_left(),
                                            bounds.width(),
                                        ),

                                        HorizontalPositionKeyword::Right => (
                                            bounds.center_left(),
                                            bounds.center_right(),
                                            bounds.width(),
                                        ),
                                    }
                                }

                                LineDirection::Vertical(vertical_keyword) => match vertical_keyword
                                {
                                    VerticalPositionKeyword::Top => (
                                        bounds.center_bottom(),
                                        bounds.center_top(),
                                        bounds.height(),
                                    ),

                                    VerticalPositionKeyword::Bottom => (
                                        bounds.center_top(),
                                        bounds.center_bottom(),
                                        bounds.height(),
                                    ),
                                },

                                LineDirection::Corner { horizontal, vertical } => {
                                    match (horizontal, vertical) {
                                        (
                                            HorizontalPositionKeyword::Right,
                                            VerticalPositionKeyword::Bottom,
                                        ) => (
                                            bounds.top_left(),
                                            bounds.bottom_right(),
                                            bounds.diagonal(),
                                        ),

                                        (
                                            HorizontalPositionKeyword::Right,
                                            VerticalPositionKeyword::Top,
                                        ) => (
                                            bounds.bottom_left(),
                                            bounds.top_right(),
                                            bounds.diagonal(),
                                        ),

                                        _ => (bounds.top_left(), bounds.bottom_right(), 0.0),
                                    }
                                }

                                LineDirection::Angle(angle) => {
                                    let angle_rad = angle.to_radians();
                                    let start_x =
                                        bounds.x + ((angle_rad.sin() * bounds.w) - bounds.w) / -2.0;
                                    let end_x =
                                        bounds.x + ((angle_rad.sin() * bounds.w) + bounds.w) / 2.0;
                                    let start_y =
                                        bounds.y + ((angle_rad.cos() * bounds.h) + bounds.h) / 2.0;
                                    let end_y =
                                        bounds.y + ((angle_rad.cos() * bounds.h) - bounds.h) / -2.0;

                                    let x = (end_x - start_x).abs();
                                    let y = (end_y - start_y).abs();

                                    let dist = (x * x + y * y).sqrt();

                                    ((start_x, start_y), (end_x, end_y), dist)
                                }
                            };

                            let num_stops = linear_gradient.stops.len();

                            let mut stops = linear_gradient
                                .stops
                                .iter()
                                .enumerate()
                                .map(|(index, stop)| {
                                    let pos = if let Some(pos) = &stop.position {
                                        pos.to_pixels(parent_length, self.scale_factor())
                                            / parent_length
                                    } else {
                                        index as f32 / (num_stops - 1) as f32
                                    };
                                    (pos, skia_safe::Color::from(stop.color))
                                })
                                .collect::<Vec<_>>();

                            // Insert a stop at the front if the first stop is not at 0.
                            if let Some(first) = stops.first() {
                                if first.0 != 0.0 {
                                    stops.insert(0, (0.0, first.1));
                                }
                            }

                            // Insert a stop at the end if the last stop is not at 1.0.
                            if let Some(last) = stops.last() {
                                if last.0 != 1.0 {
                                    stops.push((1.0, last.1));
                                }
                            }

                            let (offsets, colors): (Vec<f32>, Vec<skia_safe::Color>) =
                                stops.into_iter().unzip();

                            let shader = Shader::linear_gradient(
                                (Point::from(start), Point::from(end)),
                                GradientShaderColors::Colors(&colors[..]),
                                Some(&offsets[..]),
                                TileMode::Clamp,
                                None,
                                None,
                            );

                            let mut paint = Paint::default();
                            paint.set_shader(shader);

                            canvas.draw_path(path, &paint);
                        }

                        Gradient::Radial(radial_gradient) => {
                            let num_stops = radial_gradient.stops.len();

                            let mut stops = radial_gradient
                                .stops
                                .iter()
                                .enumerate()
                                .map(|(index, stop)| {
                                    let pos = if let Some(pos) = &stop.position {
                                        pos.to_pixels(bounds.width(), self.scale_factor())
                                            / bounds.width()
                                    } else {
                                        index as f32 / (num_stops - 1) as f32
                                    };

                                    (pos, skia_safe::Color::from(stop.color))
                                })
                                .collect::<Vec<_>>();

                            // Insert a stop at the front if the first stop is not at 0.
                            if let Some(first) = stops.first() {
                                if first.0 != 0.0 {
                                    stops.insert(0, (0.0, first.1));
                                }
                            }

                            // Insert a stop at the end if the last stop is not at 1.0.
                            if let Some(last) = stops.last() {
                                if last.0 != 1.0 {
                                    stops.push((1.0, last.1));
                                }
                            }

                            let (offsets, colors): (Vec<f32>, Vec<skia_safe::Color>) =
                                stops.into_iter().unzip();

                            let shader = Shader::radial_gradient(
                                Point::from(bounds.center()),
                                bounds.w.max(bounds.h),
                                GradientShaderColors::Colors(&colors[..]),
                                Some(&offsets[..]),
                                TileMode::Clamp,
                                None,
                                None,
                            );

                            let mut paint = Paint::default();
                            paint.set_shader(shader);
                            canvas.draw_path(path, &paint);
                        }

                        _ => {}
                    },

                    ImageOrGradient::Image(image_name) => {
                        if let Some(image) = self.resource_manager.images.get(image_name) {
                            let image_width = image.image.width();
                            let image_height = image.image.height();
                            let (width, height) = if let Some(background_size) =
                                image_sizes.get(index)
                            {
                                match background_size {
                                    BackgroundSize::Explicit { width, height } => {
                                        let w = match width {
                                            LengthPercentageOrAuto::LengthPercentage(length) => {
                                                length.to_pixels(bounds.w, self.scale_factor())
                                            }
                                            LengthPercentageOrAuto::Auto => image_width as f32,
                                        };

                                        let h = match height {
                                            LengthPercentageOrAuto::LengthPercentage(length) => {
                                                length.to_pixels(bounds.h, self.scale_factor())
                                            }
                                            LengthPercentageOrAuto::Auto => image_height as f32,
                                        };

                                        (w, h)
                                    }

                                    BackgroundSize::Contain => {
                                        let image_ratio = image_width as f32 / image_height as f32;
                                        let container_ratio = bounds.w / bounds.h;

                                        let (w, h) = if image_ratio > container_ratio {
                                            (bounds.w, bounds.w / image_ratio)
                                        } else {
                                            (bounds.h * image_ratio, bounds.h)
                                        };

                                        (w, h)
                                    }

                                    BackgroundSize::Cover => {
                                        let image_ratio = image_width as f32 / image_height as f32;
                                        let container_ratio = bounds.w / bounds.h;

                                        let (w, h) = if image_ratio < container_ratio {
                                            (bounds.w, bounds.w / image_ratio)
                                        } else {
                                            (bounds.h * image_ratio, bounds.h)
                                        };

                                        (w, h)
                                    }
                                }
                            } else {
                                (image_width as f32, image_height as f32)
                            };

                            let matrix = Matrix::rect_to_rect(
                                Rect::new(
                                    0.0,
                                    0.0,
                                    image.image.width() as f32,
                                    image.image.height() as f32,
                                ),
                                Rect::new(
                                    bounds.left(),
                                    bounds.top(),
                                    bounds.left() + width,
                                    bounds.top() + height,
                                ),
                                None,
                            );

                            let mut paint = Paint::default();
                            paint.set_anti_alias(true);
                            paint.set_shader(image.image.to_shader(
                                (TileMode::Repeat, TileMode::Repeat),
                                SamplingOptions::default(),
                                &matrix,
                            ));

                            canvas.draw_path(path, &paint);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw any text for the current view.
    pub fn draw_text(&mut self, canvas: &Canvas) {
        if let Some(paragraph) = self.text_context.text_paragraphs.get(self.current) {
            let bounds = self.bounds();
            // let padding_left = self.child_left().to_px(bounds.width(), 0.0);

            let mut vertical_flex_sum = 0.0;
            let mut horizontal_flex_sum = 0.0;

            let mut padding_top = match self.child_top() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    vertical_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let padding_bottom = match self.child_bottom() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    vertical_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let vertical_free_space =
                bounds.height() - paragraph.height() as f32 - padding_top - padding_bottom;

            if let Units::Stretch(val) = self.child_top() {
                padding_top = (vertical_free_space * val / vertical_flex_sum).round()
            }

            let mut padding_left = match self.child_left() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    horizontal_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let padding_right = match self.child_right() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    horizontal_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let horizontal_free_space =
                bounds.width() - paragraph.max_width() as f32 - padding_left - padding_right;

            if let Units::Stretch(val) = self.child_left() {
                padding_left = (horizontal_free_space * val / horizontal_flex_sum).round()
            }

            // let tb = paragraph
            //     .get_rects_for_range(
            //         paragraph.get_actual_text_range(0, true),
            //         RectHeightStyle::Tight,
            //         RectWidthStyle::Tight,
            //     )
            //     .first()
            //     .unwrap()
            //     .rect;

            // let mut paint = Paint::default();
            // paint.set_color(Color::green());
            // canvas.draw_rect(
            //     Rect::new(
            //         bounds.x + padding_left,
            //         bounds.y + padding_top,
            //         bounds.x + padding_left + tb.width(),
            //         bounds.y + padding_top + tb.height(),
            //     ),
            //     &paint,
            // );

            // println!("bounds.y {} padding_top: {}  {}", bounds.y, padding_top, paragraph.height());

            paragraph.paint(canvas, (bounds.x + padding_left, bounds.y + padding_top));
        }

        // println!("{:?}", bounds.top_left());
    }

    // /// Draw the selection box for the text of the current view.
    // pub fn draw_text_selection(
    //     &mut self,
    //     canvas: &mut Canvas,
    //     bounds: BoundingBox,
    //     justify: (f32, f32),
    // ) {
    //     let selections = self.text_context.layout_selection(self.current, bounds, justify);
    //     if !selections.is_empty() {
    //         let mut path = Path::new();
    //         for (x, y, w, h) in selections {
    //             path.rect(x, y, w, h);
    //         }
    //         let selection_color = self.selection_color();
    //         canvas.fill_path(&path, &Paint::color(selection_color.into()));
    //     }
    // }
}

impl<'a> DataContext for DrawContext<'a> {
    fn data<T: 'static>(&self) -> Option<&T> {
        // Return data for the static model.
        if let Some(t) = <dyn Any>::downcast_ref::<T>(&()) {
            return Some(t);
        }

        for entity in self.current.parent_iter(self.tree) {
            // Return model data.
            if let Some(model_data_store) = self.data.get(&entity) {
                if let Some(model) = model_data_store.models.get(&TypeId::of::<T>()) {
                    return model.downcast_ref::<T>();
                }
            }

            // Return view data.
            if let Some(view_handler) = self.views.get(&entity) {
                if let Some(data) = view_handler.downcast_ref::<T>() {
                    return Some(data);
                }
            }
        }

        None
    }
}
