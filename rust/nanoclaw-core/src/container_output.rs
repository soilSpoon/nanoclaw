const OUTPUT_START_MARKER: &str = "---NANOCLAW_OUTPUT_START---";
const OUTPUT_END_MARKER: &str = "---NANOCLAW_OUTPUT_END---";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerOutput {
    pub status: String,
    pub result: Option<String>,
    pub new_session_id: Option<String>,
    pub error: Option<String>,
}

fn json_string_value(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let idx = json.find(&needle)? + needle.len();
    let rest = json[idx..].trim_start();

    if let Some(stripped) = rest.strip_prefix("null") {
        let _ = stripped;
        return None;
    }

    let after_quote = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut escaped = false;

    for ch in after_quote.chars() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }

    None
}

fn parse_single_output(json: &str) -> Option<ContainerOutput> {
    let status = json_string_value(json, "status")?;
    Some(ContainerOutput {
        status,
        result: json_string_value(json, "result"),
        new_session_id: json_string_value(json, "newSessionId"),
        error: json_string_value(json, "error"),
    })
}

pub fn parse_streamed_outputs(stdout: &str) -> Vec<ContainerOutput> {
    let mut out = Vec::new();
    let mut cursor = 0;

    while let Some(start_rel) = stdout[cursor..].find(OUTPUT_START_MARKER) {
        let start = cursor + start_rel + OUTPUT_START_MARKER.len();
        let rest = &stdout[start..];
        let Some(end_rel) = rest.find(OUTPUT_END_MARKER) else {
            break;
        };

        let json_blob = rest[..end_rel].trim();
        if let Some(parsed) = parse_single_output(json_blob) {
            out.push(parsed);
        }

        cursor = start + end_rel + OUTPUT_END_MARKER.len();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::parse_streamed_outputs;

    #[test]
    fn parses_single_marker_pair() {
        let input = r#"
noise
---NANOCLAW_OUTPUT_START---
{"status":"success","result":"hello","newSessionId":"s1"}
---NANOCLAW_OUTPUT_END---
"#;

        let outputs = parse_streamed_outputs(input);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].status, "success");
        assert_eq!(outputs[0].result.as_deref(), Some("hello"));
        assert_eq!(outputs[0].new_session_id.as_deref(), Some("s1"));
    }

    #[test]
    fn parses_multiple_marker_pairs() {
        let input = r#"
---NANOCLAW_OUTPUT_START---
{"status":"success","result":"first"}
---NANOCLAW_OUTPUT_END---
---NANOCLAW_OUTPUT_START---
{"status":"error","result":null,"error":"boom"}
---NANOCLAW_OUTPUT_END---
"#;

        let outputs = parse_streamed_outputs(input);
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].result.as_deref(), Some("first"));
        assert_eq!(outputs[1].status, "error");
        assert_eq!(outputs[1].error.as_deref(), Some("boom"));
    }

    #[test]
    fn ignores_invalid_json_blobs() {
        let input = r#"
---NANOCLAW_OUTPUT_START---
not json
---NANOCLAW_OUTPUT_END---
---NANOCLAW_OUTPUT_START---
{"status":"success","result":"ok"}
---NANOCLAW_OUTPUT_END---
"#;

        let outputs = parse_streamed_outputs(input);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].result.as_deref(), Some("ok"));
    }
}
