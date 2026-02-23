#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub chat_jid: String,
    pub sender: String,
    pub sender_name: String,
    pub content: String,
    pub timestamp: String,
    pub is_from_me: bool,
    pub is_bot_message: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredGroup {
    pub name: String,
    pub folder: String,
    pub trigger: String,
    pub added_at: String,
    pub requires_trigger: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMetadata {
    pub jid: String,
    pub name: String,
    pub last_message_time: String,
    pub is_group: bool,
    pub channel: String,
}
