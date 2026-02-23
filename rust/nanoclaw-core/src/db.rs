use std::collections::HashMap;

use crate::types::{ChatMetadata, Message, RegisteredGroup};

#[derive(Default)]
pub struct Database {
    router_state: HashMap<String, String>,
    sessions: HashMap<String, String>,
    registered_groups: HashMap<String, RegisteredGroup>,
    chats: HashMap<String, ChatMetadata>,
    messages: Vec<Message>,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_router_state(&mut self, key: &str, value: &str) {
        self.router_state.insert(key.to_string(), value.to_string());
    }

    pub fn get_router_state(&self, key: &str) -> Option<String> {
        self.router_state.get(key).cloned()
    }

    pub fn set_session(&mut self, chat_jid: &str, session_id: &str) {
        self.sessions
            .insert(chat_jid.to_string(), session_id.to_string());
    }

    pub fn get_all_sessions(&self) -> HashMap<String, String> {
        self.sessions.clone()
    }

    pub fn set_registered_group(&mut self, jid: &str, group: RegisteredGroup) {
        self.registered_groups.insert(jid.to_string(), group);
    }

    pub fn get_all_registered_groups(&self) -> HashMap<String, RegisteredGroup> {
        self.registered_groups.clone()
    }

    pub fn store_chat_metadata(&mut self, meta: ChatMetadata) {
        self.chats.insert(meta.jid.clone(), meta);
    }

    pub fn get_all_chats(&self) -> Vec<ChatMetadata> {
        let mut chats = self.chats.values().cloned().collect::<Vec<_>>();
        chats.sort_by(|a, b| b.last_message_time.cmp(&a.last_message_time));
        chats
    }

    pub fn store_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn get_new_messages(&self, last_timestamp: &str) -> Vec<Message> {
        let mut out = self
            .messages
            .iter()
            .filter(|m| m.timestamp.as_str() > last_timestamp)
            .cloned()
            .collect::<Vec<_>>();
        out.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        out
    }

    pub fn get_messages_since(
        &self,
        chat_jid: &str,
        since_timestamp: &str,
        assistant_name: &str,
    ) -> Vec<Message> {
        let prefix = format!("{}:", assistant_name);
        let mut out = self
            .messages
            .iter()
            .filter(|m| {
                m.chat_jid == chat_jid
                    && m.timestamp.as_str() > since_timestamp
                    && !m.is_bot_message
                    && !m.content.starts_with(&prefix)
            })
            .cloned()
            .collect::<Vec<_>>();
        out.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::types::{ChatMetadata, Message, RegisteredGroup};

    fn msg(ts: &str, bot: bool, content: &str) -> Message {
        Message {
            id: ts.into(),
            chat_jid: "group@g.us".into(),
            sender: "alice@wa".into(),
            sender_name: "Alice".into(),
            content: content.into(),
            timestamp: ts.into(),
            is_from_me: false,
            is_bot_message: bot,
        }
    }

    #[test]
    fn router_state_roundtrip() {
        let mut db = Database::new();
        db.set_router_state("k", "v");
        assert_eq!(db.get_router_state("k").as_deref(), Some("v"));
    }

    #[test]
    fn session_and_group_storage() {
        let mut db = Database::new();
        db.set_session("group@g.us", "s1");
        db.set_registered_group(
            "group@g.us",
            RegisteredGroup {
                name: "Team".into(),
                folder: "team".into(),
                trigger: "@Andy".into(),
                added_at: "2026".into(),
                requires_trigger: true,
            },
        );
        assert_eq!(db.get_all_sessions().get("group@g.us").map(|s| s.as_str()), Some("s1"));
        assert_eq!(db.get_all_registered_groups().len(), 1);
    }

    #[test]
    fn message_filters_match_runtime_rules() {
        let mut db = Database::new();
        db.store_message(msg("2026-01-01T00:00:01Z", false, "hello"));
        db.store_message(msg("2026-01-01T00:00:02Z", true, "bot says"));
        db.store_message(msg("2026-01-01T00:00:03Z", false, "Andy: prefixed"));

        let new_msgs = db.get_new_messages("2026-01-01T00:00:00Z");
        assert_eq!(new_msgs.len(), 3);

        let user_msgs = db.get_messages_since("group@g.us", "2026-01-01T00:00:00Z", "Andy");
        assert_eq!(user_msgs.len(), 1);
        assert_eq!(user_msgs[0].content, "hello");
    }

    #[test]
    fn chat_metadata_sorted_desc_by_activity() {
        let mut db = Database::new();
        db.store_chat_metadata(ChatMetadata {
            jid: "a".into(),
            name: "A".into(),
            last_message_time: "2026-01-01T00:00:00Z".into(),
            is_group: true,
            channel: "whatsapp".into(),
        });
        db.store_chat_metadata(ChatMetadata {
            jid: "b".into(),
            name: "B".into(),
            last_message_time: "2026-01-01T00:00:10Z".into(),
            is_group: true,
            channel: "whatsapp".into(),
        });

        let chats = db.get_all_chats();
        assert_eq!(chats[0].jid, "b");
    }
}
