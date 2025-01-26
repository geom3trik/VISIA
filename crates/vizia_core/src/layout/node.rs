use morphorm::Node;
use skia_safe::wrapper::PointerWrapper;
use vizia_storage::MorphormChildIter;

use crate::prelude::*;
use crate::resource::{ImageOrSvg, ResourceManager};
use crate::text::TextContext;

pub struct SubLayout<'a> {
    pub text_context: &'a mut TextContext,
    pub resource_manager: &'a ResourceManager,
}

impl Node for Entity {
    type Store = Style;
    type Tree = Tree<Entity>;
    type CacheKey = Entity;
    type ChildIter<'t> = MorphormChildIter<'t, Entity>;
    type SubLayout<'a> = SubLayout<'a>;

    fn children<'t>(&'t self, tree: &'t Self::Tree) -> Self::ChildIter<'t> {
        MorphormChildIter::new(tree, *self)
    }

    fn key(&self) -> Self::CacheKey {
        *self
    }

    fn visible(&self, store: &Self::Store) -> bool {
        store.display.get(*self).copied().map(|display| display == Display::Flex).unwrap_or(true)
    }

    fn layout_type(&self, store: &Self::Store) -> Option<morphorm::LayoutType> {
        store.layout_type.get(*self).copied()
    }

    fn position_type(&self, store: &Self::Store) -> Option<morphorm::PositionType> {
        store.position_type.get(*self).copied()
    }

    fn left(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.left.get(*self).cloned().map(|l| match l {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn right(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.right.get(*self).cloned().map(|r| match r {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn top(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.top.get(*self).cloned().map(|t| match t {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn bottom(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.bottom.get(*self).cloned().map(|b| match b {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn width(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.width.get(*self).cloned().map(|w| match w {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn min_width(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.min_width.get(*self).cloned().map(|w| match w {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn max_width(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.max_width.get(*self).cloned().map(|w| match w {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn content_size(
        &self,
        store: &Self::Store,
        sublayout: &mut Self::SubLayout<'_>,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Option<(f32, f32)> {
        if let Some(paragraph) = sublayout.text_context.text_paragraphs.get_mut(*self) {
            // // If the width is known use that, else use 0 for wrapping text or 999999 for non-wrapping text.
            // let max_width = if let Some(width) = width {
            //     let padding_left =
            //         store.padding_left.get(*self).cloned().unwrap_or_default().to_px(width, 0.0)
            //             * store.scale_factor();
            //     let padding_right =
            //         store.padding_right.get(*self).cloned().unwrap_or_default().to_px(width, 0.0)
            //             * store.scale_factor();
            //     let border_width = store
            //         .border_width
            //         .get(*self)
            //         .cloned()
            //         .unwrap_or_default()
            //         .to_pixels(0.0, store.scale_factor());

            //     width.ceil() - padding_left - padding_right - border_width - border_width
            // } else {
            //     f32::MAX
            // };

            paragraph.layout(f32::MAX);

            let padding_left = store.padding_left.get(*self).copied().unwrap_or_default();
            let padding_right = store.padding_right.get(*self).copied().unwrap_or_default();
            let padding_top = store.padding_top.get(*self).copied().unwrap_or_default();
            let padding_bottom = store.padding_bottom.get(*self).copied().unwrap_or_default();

            let mut child_space_x = 0.0;
            let mut child_space_y = 0.0;

            let mut p_left = 0.0;
            let mut p_top = 0.0;

            // shrink the bounding box based on pixel values
            if let Pixels(val) = padding_left {
                let val = val * store.scale_factor();
                child_space_x += val;
                p_left += val;
            }
            if let Pixels(val) = padding_right {
                let val = val * store.scale_factor();
                child_space_x += val;
            }
            if let Pixels(val) = padding_top {
                let val = val * store.scale_factor();
                child_space_y += val;
                p_top += val;
            }
            if let Pixels(val) = padding_bottom {
                let val = val * store.scale_factor();
                child_space_y += val;
            }

            let border_width = store
                .border_width
                .get(*self)
                .cloned()
                .unwrap_or_default()
                .to_pixels(0.0, store.scale_factor());

            child_space_x += 2.0 * border_width;
            child_space_y += 2.0 * border_width;

            p_left += border_width;
            p_top += border_width;

            let text_wrap = store.text_wrap.get(*self).copied().unwrap_or(true);
            let text_overflow = store.text_overflow.get(*self).copied();

            let text_width = match (text_wrap, text_overflow) {
                (true, _) => {
                    if let Some(width) = width {
                        width - child_space_x
                    } else {
                        paragraph.min_intrinsic_width().ceil()
                    }
                }
                (false, Some(TextOverflow::Ellipsis)) => {
                    if let Some(width) = width {
                        width - child_space_x
                    } else {
                        paragraph.max_intrinsic_width().ceil()
                    }
                }
                _ => {
                    if let Some(width) = width {
                        (width - child_space_x).max(paragraph.min_intrinsic_width().ceil())
                    } else {
                        paragraph.max_intrinsic_width().ceil()
                    }
                }
            };

            if text_wrap || text_overflow.is_some() {
                paragraph.layout(text_width);
            }

            let text_height = if let Some(height) = height { height } else { paragraph.height() };

            let width =
                if let Some(width) = width { width } else { text_width.round() + child_space_x };

            let height = if let Some(height) = height {
                height
            } else {
                text_height.round() + child_space_y
            };

            // Cache the text_width/ text_height in the text context so we can use it to compute transforms later
            sublayout.text_context.set_text_bounds(
                *self,
                BoundingBox { x: p_left, y: p_top, w: text_width, h: text_height },
            );

            Some((width, height))
        } else if let Some(images) = store.background_image.get(*self) {
            let mut max_width = 0.0f32;
            let mut max_height = 0.0f32;
            for image in images.iter() {
                match image {
                    ImageOrGradient::Image(image_name) => {
                        if let Some(image_id) = sublayout.resource_manager.image_ids.get(image_name)
                        {
                            match sublayout
                                .resource_manager
                                .images
                                .get(image_id)
                                .map(|stored_img| &stored_img.image)
                            {
                                Some(ImageOrSvg::Image(image)) => {
                                    max_width =
                                        max_width.max(image.width() as f32 * store.scale_factor());
                                    max_height = max_height
                                        .max(image.height() as f32 * store.scale_factor());
                                }

                                Some(ImageOrSvg::Svg(svg)) => {
                                    max_width = max_width.max(
                                        svg.inner().fContainerSize.fWidth * store.scale_factor(),
                                    );
                                    max_height = max_height.max(
                                        svg.inner().fContainerSize.fWidth * store.scale_factor(),
                                    );
                                }

                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            let width = if let Some(width) = width { width } else { max_width };
            let height = if let Some(height) = height { height } else { max_height };
            Some((width, height))
        } else {
            None
        }
    }

    fn height(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.height.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn min_height(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.min_height.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn max_height(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.max_height.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn padding_left(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.padding_left.get(*self).cloned().map(|l| match l {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn padding_right(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.padding_right.get(*self).cloned().map(|r| match r {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn padding_top(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.padding_top.get(*self).cloned().map(|t| match t {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn padding_bottom(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.padding_bottom.get(*self).cloned().map(|b| match b {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn vertical_gap(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.vertical_gap.get(*self).cloned().map(|v| match v {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn horizontal_gap(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.horizontal_gap.get(*self).cloned().map(|v| match v {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn border_left(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.border_width.get(*self).map(|border_width| match border_width {
            LengthOrPercentage::Length(val) => {
                Units::Pixels(store.logical_to_physical(val.to_px().unwrap_or_default()))
            }
            LengthOrPercentage::Percentage(val) => Units::Percentage(*val),
        })
    }

    fn border_right(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.border_width.get(*self).map(|border_width| match border_width {
            LengthOrPercentage::Length(val) => {
                Units::Pixels(store.logical_to_physical(val.to_px().unwrap_or_default()))
            }
            LengthOrPercentage::Percentage(val) => Units::Percentage(*val),
        })
    }

    fn border_top(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.border_width.get(*self).map(|border_width| match border_width {
            LengthOrPercentage::Length(val) => {
                Units::Pixels(store.logical_to_physical(val.to_px().unwrap_or_default()))
            }
            LengthOrPercentage::Percentage(val) => Units::Percentage(*val),
        })
    }

    fn border_bottom(&self, store: &Self::Store) -> Option<morphorm::Units> {
        store.border_width.get(*self).map(|border_width| match border_width {
            LengthOrPercentage::Length(val) => {
                Units::Pixels(store.logical_to_physical(val.to_px().unwrap_or_default()))
            }
            LengthOrPercentage::Percentage(val) => Units::Percentage(*val),
        })
    }

    fn alignment(&self, store: &Self::Store) -> Option<morphorm::Alignment> {
        store.alignment.get(*self).copied()
    }

    fn vertical_scroll(&self, store: &Self::Store) -> Option<f32> {
        store.vertical_scroll.get(*self).copied()
    }

    fn horizontal_scroll(&self, store: &Self::Store) -> Option<f32> {
        store.horizontal_scroll.get(*self).copied()
    }

    fn min_vertical_gap(&self, store: &Self::Store) -> Option<Units> {
        store.min_vertical_gap.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn min_horizontal_gap(&self, store: &Self::Store) -> Option<Units> {
        store.min_horizontal_gap.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn max_vertical_gap(&self, store: &Self::Store) -> Option<Units> {
        store.max_vertical_gap.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }

    fn max_horizontal_gap(&self, store: &Self::Store) -> Option<Units> {
        store.max_horizontal_gap.get(*self).cloned().map(|h| match h {
            Units::Pixels(val) => Units::Pixels(store.logical_to_physical(val)),
            t => t,
        })
    }
}
