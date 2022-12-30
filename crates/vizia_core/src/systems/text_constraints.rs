use crate::prelude::*;
use vizia_id::GenerationalId;

// Apply this before layout
// THE GOAL OF THIS FUNCTION: set content-width and content-height
pub fn text_constraints_system(cx: &mut Context, tree: &Tree<Entity>) {
    let mut draw_tree: Vec<Entity> = tree.into_iter().collect();
    draw_tree.sort_by_cached_key(|entity| cx.cache.get_z_index(*entity));

    for entity in draw_tree.into_iter() {
        if entity == Entity::root() {
            continue;
        }

        if cx.cache.display.get(entity) == Some(&Display::None) {
            continue;
        }

        if tree.is_ignored(entity) {
            continue;
        }

        // content-size is only used if any dimension is auto
        if cx.style.min_width.get(entity).copied().unwrap_or_default() != Units::Auto
            && cx.style.min_height.get(entity).copied().unwrap_or_default() != Units::Auto
            && cx.style.width.get(entity).copied().unwrap_or_default() != Units::Auto
            && cx.style.height.get(entity).copied().unwrap_or_default() != Units::Auto
            && cx.style.max_width.get(entity).map_or(true, |w| w != &Units::Auto)
            && cx.style.max_height.get(entity).map_or(true, |h| h != &Units::Auto)
        {
            continue;
        }

        let desired_width = cx.style.width.get(entity).cloned().unwrap_or_default();
        let desired_height = cx.style.height.get(entity).cloned().unwrap_or_default();
        let style = &cx.style;
        let image = style.image.get(entity);

        if (cx.cosmic_context.has_buffer(entity) || image.is_some())
            && (desired_width == Auto || desired_height == Auto)
        {
            let child_left = cx.style.child_left.get(entity).cloned().unwrap_or_default();
            let child_right = cx.style.child_right.get(entity).cloned().unwrap_or_default();
            let child_top = cx.style.child_top.get(entity).cloned().unwrap_or_default();
            let child_bottom = cx.style.child_bottom.get(entity).cloned().unwrap_or_default();

            let mut child_space_x = 0.0;
            let mut child_space_y = 0.0;

            // shrink the bounding box based on pixel values
            if let Pixels(val) = child_left {
                let val = val * cx.style.dpi_factor as f32;
                child_space_x += val;
            }
            if let Pixels(val) = child_right {
                let val = val * cx.style.dpi_factor as f32;
                child_space_x += val;
            }
            if let Pixels(val) = child_top {
                let val = val * cx.style.dpi_factor as f32;
                child_space_y += val;
            }
            if let Pixels(val) = child_bottom {
                let val = val * cx.style.dpi_factor as f32;
                child_space_y += val;
            }

            let mut content_width = 0.0;
            let mut content_height = 0.0;

            if cx.cosmic_context.has_buffer(entity) {
                cx.cosmic_context.sync_styles(entity, &cx.style);
                let (text_width, text_height) = cx.cosmic_context.with_buffer(entity, |buf| {
                    buf.set_size(i32::MAX, i32::MAX);
                    let w = buf
                        .layout_runs()
                        .filter_map(|r| (!r.line_w.is_nan()).then_some(r.line_w))
                        .max_by(|f1, f2| f1.partial_cmp(f2).unwrap())
                        .unwrap_or_default();
                    let h = buf.layout_runs().len() as f32 * buf.metrics().line_height as f32;
                    (w, h)
                });

                // Add an extra pixel to account for AA
                let text_width = text_width.round() + 1.0 + child_space_x;
                let text_height = text_height.round() + 1.0 + child_space_y;

                if content_width < text_width {
                    content_width = text_width;
                }
                if content_height < text_height {
                    content_height = text_height;
                }
            }

            if let Some(image_name) = cx.style.image.get(entity) {
                if let Some(img) = cx.resource_manager.images.get(image_name) {
                    let (image_width, image_height) = img.image.dimensions();
                    let image_width = image_width as f32;
                    let image_height = image_height as f32;

                    if content_width < image_width {
                        content_width = image_width;
                    }
                    if content_height < image_height {
                        content_height = image_height;
                    }
                }
            }

            cx.style.content_width.insert(entity, content_width / cx.style.dpi_factor as f32);
            cx.style.content_height.insert(entity, content_height / cx.style.dpi_factor as f32);
        }
    }
}
