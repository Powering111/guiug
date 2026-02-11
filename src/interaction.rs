use std::collections::HashMap;

use winit::keyboard::PhysicalKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// Keyboard key is just pressed.
    KeyDown(PhysicalKey),

    /// Keyboard key is just released.
    KeyUp(PhysicalKey),

    /// Hitbox node is clicked.
    Click(crate::NodeId),
}

type EventHandler<'a> = dyn FnMut(&mut crate::Runtime) + 'a;

#[derive(Default)]
pub(crate) struct Interaction<'a> {
    inner: HashMap<Event, Vec<Box<EventHandler<'a>>>>,
}

impl<'a> Interaction<'a> {
    pub fn insert_handler(&mut self, event: Event, handler: impl FnMut(&mut crate::Runtime) + 'a) {
        let entry = self.inner.entry(event).or_default();
        entry.push(Box::new(handler));
    }

    pub fn get_handlers(&mut self, event: Event) -> &mut Vec<Box<EventHandler<'a>>> {
        self.inner.entry(event).or_default()
    }
}
