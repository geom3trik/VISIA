//! Events
//!
//! Views communicate with each other and model data via events. An [Event] contains a message, as well as metadata to describe how events
//! should propagate through the tree. By default events will propagate up the tree from the target.
//!
//! A message can be any static thread-safe type but is usually an enum. For example:
//! ```
//! enum MyEvent {
//!     ReadDocs,
//!     CloseDocs,    
//! }
//! ```
//! Then, to send an event up the tree from the current entity:
//! ```compile_fail
//! cx.emit(MyEvent::ReadDocs);
//! ```
//! Or, to send an event from the current entity directly to a target:
//! ```compile_fail
//! cx.emit_to(target, MyEvent::ReadDocs);
//! ```
//!
//! Views and Models receive events through the `event()` method of the View or Model traits.
//! The event message must then be downcast to the right type:
//! ```compile_fail
//! fn on_event(&mut self, state: &mut EventContext, event: &mut Event) {
//!     event.map(|my_event, _| match my_event {
//!         MyEvent::ReadDocs => {
//!             // Do something
//!         }
//!
//!         MyEvent::CloseDocs => {
//!             // Do something else
//!         }
//!     });
//! }
//! ```

mod event_manager;
pub use event_manager::EventManager;

mod event;
pub use event::{Event, EventMeta, Propagation};

mod event_handler;
pub use event_handler::ViewHandler;
