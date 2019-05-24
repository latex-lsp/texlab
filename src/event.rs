use std::sync::Mutex;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    Initialized,
    WorkspaceChanged,
}

#[derive(Debug, Default)]
pub struct EventManager {
    events: Mutex<Vec<Event>>,
}

impl EventManager {
    pub fn push(&self, event: Event) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn take(&self) -> Vec<Event> {
        let mut events = self.events.lock().unwrap();
        std::mem::replace(&mut *events, Vec::new())
    }
}
