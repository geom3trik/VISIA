//! # Layout
//! Layout determines the size and position of entities on the screen.
//!
//! All layout calculations are handled by the Morphorm crate.
pub(crate) mod cache;
pub(crate) mod node;
pub(crate) mod hierarchy;
pub(crate) mod iter;

use crate::prelude::*;
use morphorm::{Cache, Hierarchy};
pub use morphorm::GeometryChanged;
pub(crate) use iter::{LayoutChildIterator, LayoutTreeIterator};

pub(crate) fn geometry_changed(cx: &mut Context, tree: &Tree) {
    for node in tree.down_iter() {
        let geometry_changed = cx.cache.geometry_changed(node);
        if !geometry_changed.is_empty() {
            cx.event_queue.push_back(
                Event::new(WindowEvent::GeometryChanged(geometry_changed))
                    .target(node)
                    .propagate(Propagation::Up),
            );
        }

        cx.cache.set_geo_changed(node, morphorm::GeometryChanged::POSX_CHANGED, false);
        cx.cache.set_geo_changed(node, morphorm::GeometryChanged::POSY_CHANGED, false);
        cx.cache.set_geo_changed(node, morphorm::GeometryChanged::WIDTH_CHANGED, false);
        cx.cache.set_geo_changed(node, morphorm::GeometryChanged::HEIGHT_CHANGED, false);
    }
}
