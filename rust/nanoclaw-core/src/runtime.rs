use crate::db::Database;
use crate::group_queue::GroupQueue;
use crate::router::format_messages;

pub struct Runtime {
    pub db: Database,
    queue: GroupQueue,
    assistant_name: String,
}

impl Runtime {
    pub fn new(assistant_name: impl Into<String>, max_concurrency: usize) -> Self {
        Self {
            db: Database::new(),
            queue: GroupQueue::new(max_concurrency),
            assistant_name: assistant_name.into(),
        }
    }

    pub fn enqueue_group(&mut self, chat_jid: impl Into<String>) {
        self.queue.enqueue(chat_jid);
    }

    pub fn process_next_group_prompt(&mut self, since_timestamp: &str) -> Option<(String, Option<String>)> {
        let chat_jid = self.queue.next()?;
        let messages = self
            .db
            .get_messages_since(&chat_jid, since_timestamp, &self.assistant_name);

        let prompt = if messages.is_empty() {
            None
        } else {
            Some(format_messages(&messages))
        };

        self.queue.complete(&chat_jid);
        Some((chat_jid, prompt))
    }
}

#[cfg(test)]
mod tests {
    use super::Runtime;
    use crate::types::Message;

    #[test]
    fn builds_prompt_from_queued_group_messages() {
        let mut rt = Runtime::new("Andy", 1);
        rt.db.store_message(Message {
            id: "1".into(),
            chat_jid: "g1".into(),
            sender: "u".into(),
            sender_name: "User".into(),
            content: "hello".into(),
            timestamp: "2026-01-01T00:00:01Z".into(),
            is_from_me: false,
            is_bot_message: false,
        });

        rt.enqueue_group("g1");
        let (jid, prompt) = rt
            .process_next_group_prompt("2026-01-01T00:00:00Z")
            .expect("prompt expected");

        assert_eq!(jid, "g1");
        assert!(prompt.expect("non-empty prompt").contains("User: hello"));
    }
}
