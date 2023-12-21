#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self { data: vec![] }
    }
}

impl<T> Stack<T> {
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn peek(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }
}
