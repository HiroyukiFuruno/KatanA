pub type ChatResponseReceiver = std::sync::mpsc::Receiver<Result<String, String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

pub struct ChatState {
    pub is_open: bool,
    pub is_pinned: bool,
    pub messages: Vec<ChatMessage>,
    pub draft: String,
    pub selected_model: String,
    pub error: Option<String>,
    pub is_pending: bool,
    pub response_rx: Option<ChatResponseReceiver>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            is_pinned: true,
            messages: Vec::new(),
            draft: String::new(),
            selected_model: String::new(),
            error: None,
            is_pending: false,
            response_rx: None,
        }
    }

    pub fn push_user(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content,
        });
    }

    pub fn push_assistant(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content,
        });
    }

    pub fn has_session_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    pub fn can_submit(&self) -> bool {
        !self.is_pending && !self.draft.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_state_starts_without_persisted_history() {
        let state = ChatState::new();
        assert!(!state.has_session_messages());
        assert!(state.response_rx.is_none());
    }

    #[test]
    fn chat_state_blocks_empty_or_pending_submit() {
        let mut state = ChatState::new();
        assert!(!state.can_submit());
        state.draft = "hello".to_string();
        assert!(state.can_submit());
        state.is_pending = true;
        assert!(!state.can_submit());
    }
}
