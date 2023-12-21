use std::fmt::Display;

use self::{dequeue::Deque, stack::Stack};

use super::{DataPoint, Points};

mod dequeue;
mod stack;

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub struct UndoManager {
    max_history_size: Option<u16>,
    undo_events: Deque<Event>,
    redo_events: Stack<Event>,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self {
            max_history_size: Self::default_max_history(),
            undo_events: Default::default(),
            redo_events: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub enum Event {
    Add(AddEventData),
    Delete(DeleteEventData),
    Clear(ClearEventData),
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct AddEventData {
    pub point: DataPoint,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct DeleteEventData {
    pub index: usize,
    pub point: DataPoint,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct ClearEventData {
    pub points: Points,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Add(data) => data.fmt(f),
            Event::Delete(data) => data.fmt(f),
            Event::Clear(data) => data.fmt(f),
        }
    }
}

impl Display for AddEventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add Point: {}", self.point)
    }
}

impl Display for DeleteEventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete Point: {} at index: {}", self.point, self.index)
    }
}

impl Display for ClearEventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clear of Points")
    }
}

impl UndoManager {
    const DEFAULT_MAX_HISTORY: u16 = 200;
    pub fn max_history_size(&self) -> Option<u16> {
        self.max_history_size
    }

    pub fn set_max_history_size(&mut self, value: Option<u16>) {
        self.max_history_size = value;
        if let Some(max_size) = self.max_history_size {
            while self.undo_events.len() > max_size as usize {
                self.undo_events.remove_oldest();
            }
        }
    }

    pub fn clear_all(&mut self) {
        self.undo_events.clear();
        self.redo_events.clear();
    }

    pub fn is_undo_empty(&self) -> bool {
        self.undo_events.is_empty()
    }

    pub fn is_redo_empty(&self) -> bool {
        self.redo_events.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.is_undo_empty() && self.is_redo_empty()
    }

    pub fn add_undo(&mut self, event: Event) {
        self.redo_events.clear();
        self.undo_events.push(event);
        if let Some(max_size) = self.max_history_size {
            if self.undo_events.len() > max_size as usize {
                self.undo_events.remove_oldest();
            }
            debug_assert!(
                self.undo_events.len() <= max_size as usize,
                "at this point it should be withing the limit"
            );
        }
    }

    /// Moves the most recent item into redo and returns a reference to it
    ///
    /// PANICS: Panics if there is nothing to undo
    pub fn undo(&mut self) -> &mut Event {
        let event = self
            .undo_events
            .pop()
            .expect("should not be empty if called");
        self.redo_events.push(event);
        self.redo_events
            .peek()
            .unwrap_or_else(|| panic!("should not be empty we just put an item into it"))
    }

    /// Moves the most recent item into undo and returns a reference to it
    ///
    /// PANICS: Panics if there is nothing to redo
    pub fn redo(&mut self) -> &mut Event {
        let event = self
            .redo_events
            .pop()
            .expect("should not be empty if called");
        self.undo_events.push(event);
        self.undo_events
            .peek()
            .unwrap_or_else(|| panic!("should not be empty we just put an item into it"))
    }

    /// It is assumed in the UI that this is Some and not None
    pub fn default_max_history() -> Option<u16> {
        let result = Some(Self::DEFAULT_MAX_HISTORY);
        debug_assert!(result.is_some());
        result
    }
}
