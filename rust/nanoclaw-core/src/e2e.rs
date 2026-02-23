use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::runtime::Runtime;
use crate::types::Message;

fn parse_line(line: &str, idx: usize) -> Option<Message> {
    let parts: Vec<&str> = line.splitn(4, '\t').collect();
    if parts.len() != 4 {
        return None;
    }

    Some(Message {
        id: format!("line-{}", idx),
        timestamp: parts[0].trim().to_string(),
        chat_jid: parts[1].trim().to_string(),
        sender: format!("{}@local", parts[2].trim()),
        sender_name: parts[2].trim().to_string(),
        content: parts[3].to_string(),
        is_from_me: false,
        is_bot_message: false,
    })
}

pub fn run_file_e2e(
    assistant_name: &str,
    input_path: &Path,
    output_path: &Path,
) -> Result<usize, String> {
    let content = fs::read_to_string(input_path)
        .map_err(|e| format!("failed reading input file {}: {}", input_path.display(), e))?;

    let mut runtime = Runtime::new(assistant_name, 4);
    let mut groups = BTreeSet::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        let Some(msg) = parse_line(line, idx + 1) else {
            continue;
        };

        groups.insert(msg.chat_jid.clone());
        runtime.db.store_message(msg);
    }

    for g in &groups {
        runtime.enqueue_group(g.clone());
    }

    let mut outputs = Vec::new();
    loop {
        let Some((chat_jid, prompt)) = runtime.process_next_group_prompt("") else {
            break;
        };

        if let Some(prompt_text) = prompt {
            outputs.push(format!(
                "{}\t{}: processed\t{}",
                chat_jid,
                assistant_name,
                prompt_text.replace('\n', "\\n")
            ));
        }
    }

    fs::write(output_path, outputs.join("\n") + if outputs.is_empty() { "" } else { "\n" })
        .map_err(|e| format!("failed writing output file {}: {}", output_path.display(), e))?;

    Ok(outputs.len())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::run_file_e2e;

    fn unique_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("nanoclaw-{}-{}", name, nanos))
    }


    #[test]
    fn returns_clear_error_for_missing_input_file() {
        let input = unique_path("missing-input.tsv");
        let output = unique_path("unused-output.tsv");

        let err = run_file_e2e("Andy", &input, &output).expect_err("missing input should error");
        assert!(err.contains("failed reading input file"));
    }

    #[test]
    fn processes_input_file_and_writes_group_outputs() {
        let input = unique_path("in.txt");
        let output = unique_path("out.txt");

        fs::write(
            &input,
            concat!(
                "2026-01-01T00:00:01Z\tg1\tAlice\tHello there\n",
                "2026-01-01T00:00:02Z\tg2\tBob\tSecond group\n"
            ),
        )
        .expect("write input");

        let count = run_file_e2e("Andy", &input, &output).expect("run e2e");
        assert_eq!(count, 2);

        let out = fs::read_to_string(&output).expect("read output");
        assert!(out.contains("g1\tAndy: processed"));
        assert!(out.contains("g2\tAndy: processed"));

        let _ = fs::remove_file(input);
        let _ = fs::remove_file(output);
    }
}
