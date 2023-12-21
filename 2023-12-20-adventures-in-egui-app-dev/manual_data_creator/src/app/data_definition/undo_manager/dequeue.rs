use std::collections::VecDeque;

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Deque<T> {
    data: VecDeque<T>,
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }
}

impl<T> Deque<T> {
    pub fn push(&mut self, value: T) {
        self.data.push_back(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn peek(&mut self) -> Option<&mut T> {
        self.data.back_mut()
    }

    pub fn remove_oldest(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
