use log::error;

/// Encapsulates the message to show in the status bar
///
/// Provides a way to ensure the correct API is used and the string is not randomly edited
#[derive(Debug, PartialEq, Default)]
pub struct StatusMsg {
    msg: String,
}
impl StatusMsg {
    pub fn add_msg(&mut self, msg: &str) {
        if !self.msg.is_empty() {
            self.msg.push('\n');
        }
        self.msg.push_str(msg);
    }

    pub fn add_err(&mut self, msg: &str) {
        error!("{msg}");
        self.add_msg(msg);
    }

    pub fn get_msg(&self) -> &str {
        &self.msg
    }

    pub fn clear(&mut self) {
        self.msg.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.msg.is_empty()
    }
}
