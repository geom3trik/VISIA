use crate::context::InternalEvent;
use crate::prelude::*;
use crate::tree::TreeExt;

/// Dispatches events to views and models.
///
/// The [EventManager] is responsible for taking the events in the event queue in context
/// and dispatching them to views and models based on the target and propagation metadata of the event.
#[doc(hidden)]
pub struct EventManager {
    // Queue of events to be processed
    event_queue: Vec<Event>,

    // A copy of the tree for iteration
    tree: Tree,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager { event_queue: Vec::new(), tree: Tree::new() }
    }

    /// Flush the event queue, dispatching events to their targets.
    /// Returns whether there are still more events to process, i.e. the event handlers sent events.
    pub fn flush_events(&mut self, context: &mut Context) -> bool {
        // Clear the event queue in the event manager
        self.event_queue.clear();

        //state.removed_entities.clear();

        // Move events from state to event manager
        self.event_queue.extend(context.event_queue.drain(0..));

        if context.tree().changed {
            self.tree = context.tree().clone();
        }

        // Loop over the events in the event queue
        'events: for event in self.event_queue.iter_mut() {
            // handle internal events
            event.map(|internal_event, _| match internal_event {
                InternalEvent::Redraw => context.need_redraw(),
                InternalEvent::LoadImage { path, image, policy } => {
                    if let Some(image) = image.lock().unwrap().take() {
                        context.load_image(path.clone(), image, *policy);
                    }
                }
            });

            // if event.trace {
            //     println!("Event: {:?}", event);
            // }

            // Send events to any listeners
            let listeners =
                context.listeners.iter().map(|(entity, _)| *entity).collect::<Vec<Entity>>();
            for entity in listeners {
                if let Some(listener) = context.listeners.remove(&entity) {
                    if let Some(mut event_handler) = context.views.remove(&entity) {
                        context.with_current(entity, |context| {
                            (listener)(event_handler.as_mut(), context, event);
                        });

                        context.views.insert(entity, event_handler);
                    }

                    context.listeners.insert(entity, listener);
                }

                if event.meta.consumed {
                    continue 'events;
                }
            }

            // Define the target to prevent multiple mutable borrows error
            let target = event.meta.target;

            // Send event to target
            visit_entity(context, target, event);

            if event.meta.consumed {
                continue 'events;
            }

            // if event.trace {
            //     println!("Target: {} Parents: {:?} Tree: {:?}", target, target.parent_iter(&self.tree).collect::<Vec<_>>(), self.tree.parent);
            // }

            // Propagate up from target to root (not including target)
            if event.meta.propagation == Propagation::Up {
                // Walk up the tree from parent to parent
                for entity in target.parent_iter(&self.tree) {
                    // Skip the target entity
                    if entity == event.meta.target {
                        continue;
                    }

                    // Send event to all entities before the target
                    visit_entity(context, entity, event);

                    // Skip to the next event if the current event is consumed
                    if event.meta.consumed {
                        continue 'events;
                    }
                }
            }

            if event.meta.propagation == Propagation::Subtree {
                for entity in target.branch_iter(&self.tree) {
                    // Skip the target entity
                    if entity == event.meta.target {
                        continue;
                    }

                    // Send event to all entities before the target
                    visit_entity(context, entity, event);

                    // Skip to the next event if the current event is consumed
                    if event.meta.consumed {
                        continue 'events;
                    }
                }
            }
        }

        !context.event_queue.is_empty()
    }
}

fn visit_entity(context: &mut Context, entity: Entity, event: &mut Event) {
    if let Some(mut view) = context.views.remove(&entity) {
        context.with_current(entity, |context| {
            view.event(context, event);
        });

        context.views.insert(entity, view);
    }

    if let Some(mut model_list) = context.data.remove(entity) {
        for (_, model) in model_list.data.iter_mut() {
            // if event.trace {
            //     println!("Event: {:?} -> Model {:?}", event, ty);
            // }
            context.with_current(entity, |cx| {
                model.event(cx, event);
            });
        }

        context.data.insert(entity, model_list).expect("Failed to insert data");
    }
}
