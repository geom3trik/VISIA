use crate::style::SystemFlags;
use crate::{prelude::*, style::Transform2D};
use vizia_id::GenerationalId;

pub fn transform_system(cx: &mut Context) {
    if cx.style.system_flags.contains(SystemFlags::RETRANSFORM) {
        for entity in cx.tree.into_iter() {
            if entity == Entity::root() {
                continue;
            }

            let parent = cx.tree.get_parent(entity).unwrap();
            //let parent_origin = state.data.get_origin(parent);
            let parent_transform = cx.cache.get_transform(parent);

            cx.cache.set_transform(entity, Transform2D::identity());

            cx.cache.set_transform(entity, parent_transform);

            let bounds = cx.cache.get_bounds(entity);

            if let Some((tx, ty)) = cx.style.translate.get(entity).copied() {
                let scale = cx.style.dpi_factor as f32;
                cx.cache.set_translate(entity, (tx * scale, ty * scale));
            }

            if let Some(rotate) = cx.style.rotate.get(entity).copied() {
                let x = bounds.x + (bounds.w / 2.0);
                let y = bounds.y + (bounds.h / 2.0);
                cx.cache.set_translate(entity, (x, y));
                cx.cache.set_rotate(entity, (rotate).to_radians());
                cx.cache.set_translate(entity, (-x, -y));
            }

            if let Some((scalex, scaley)) = cx.style.scale.get(entity).copied() {
                let x = bounds.x + (bounds.w / 2.0);
                let y = bounds.y + (bounds.h / 2.0);
                cx.cache.set_translate(entity, (x, y));
                cx.cache.set_scale(entity, (scalex, scaley));
                cx.cache.set_translate(entity, (-x, -y));
            }
        }
    }
}
