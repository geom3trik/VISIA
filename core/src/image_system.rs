use crate::{prelude::*, resource::ImageOrId};

pub fn image_system(cx: &mut Context) {
    println!("Image System");
    for entity in cx.tree.clone().into_iter() {
        // Check if the entity is using a background image
        if let Some(background_image) = cx.style.background_image.get(entity).cloned() {
            println!("Entity background image: {}", background_image);
            // Check if the background image is already loaded
            if let Some(image_store) = cx.resource_manager.images.get_mut(&background_image) {
                println!("Found an image for: {}", background_image);
                match &image_store.image {
                    // Image exists and is already loaded so just add this entity as an observer
                    ImageOrId::Id(_, _) => {
                        // TODO: check if the image is actually the same?
                        image_store.observers.insert(entity);
                    }

                    // Image exists but isn't loaded yet
                    ImageOrId::Image(_, _) => {
                        if let Some(canvas) = cx.canvases.get_mut(&Entity::root()) {
                            // This loads the image and sets the image id
                            image_store.image.id(canvas);
                        }
                    }
                }
            } else {
                // Image doesn't exists yet so call the image loader
                if let Some(callback) = cx.resource_manager.image_loader.take() {
                    (callback)(cx, &background_image);

                    cx.resource_manager.image_loader = Some(callback);
                }
            }
        }
    }
}
