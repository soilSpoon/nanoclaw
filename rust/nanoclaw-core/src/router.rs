use crate::types::Message;

pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn format_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| {
            format!(
                "[{}] {}: {}",
                m.timestamp,
                m.sender_name,
                escape_xml(m.content.trim())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_outbound(assistant_name: &str, text: &str) -> String {
    format!("{}: {}", assistant_name, text.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;

    fn m(content: &str) -> Message {
        Message {
            id: "1".into(),
            chat_jid: "chat".into(),
            sender: "user@wa".into(),
            sender_name: "User".into(),
            content: content.into(),
            timestamp: "2026-01-01T00:00:00.000Z".into(),
            is_from_me: false,
            is_bot_message: false,
        }
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(escape_xml("<a&b>\"'"), "&lt;a&amp;b&gt;&quot;&apos;");
    }

    #[test]
    fn formats_message_batch() {
        let out = format_messages(&[m("hello"), m("<b>")]);
        assert!(out.contains("User: hello"));
        assert!(out.contains("&lt;b&gt;"));
    }

    #[test]
    fn formats_outbound_prefix() {
        assert_eq!(format_outbound("Andy", " hi "), "Andy: hi");
    }
}
