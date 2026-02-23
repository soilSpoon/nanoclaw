use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::runtime::Runtime;
use crate::types::Message;

fn parse_line(line: &str, idx: usize, file_tag: &str) -> Option<Message> {
    let parts: Vec<&str> = line.splitn(4, '\t').collect();
    if parts.len() != 4 {
        return None;
    }

    Some(Message {
        id: format!("{}-{}", file_tag, idx),
        timestamp: parts[0].trim().to_string(),
        chat_jid: parts[1].trim().to_string(),
        sender: format!("{}@local", parts[2].trim()),
        sender_name: parts[2].trim().to_string(),
        content: parts[3].to_string(),
        is_from_me: false,
        is_bot_message: false,
    })
}

fn list_tsv_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed reading ipc directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let path = entry
            .map_err(|e| format!("failed reading ipc directory entry: {}", e))?
            .path();
        if path.extension().and_then(|s| s.to_str()) == Some("tsv") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

pub fn run_ipc_once(
    runtime: &mut Runtime,
    ipc_dir: &Path,
    since_timestamp: &str,
    assistant_name: &str,
) -> Result<usize, String> {
    let messages_dir = ipc_dir.join("messages");
    let processed_dir = ipc_dir.join("processed");
    let output_dir = ipc_dir.join("output");
    let output_file = output_dir.join("responses.tsv");

    fs::create_dir_all(&messages_dir)
        .map_err(|e| format!("failed creating messages dir {}: {}", messages_dir.display(), e))?;
    fs::create_dir_all(&processed_dir)
        .map_err(|e| format!("failed creating processed dir {}: {}", processed_dir.display(), e))?;
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("failed creating output dir {}: {}", output_dir.display(), e))?;

    let files = list_tsv_files(&messages_dir)?;
    if files.is_empty() {
        return Ok(0);
    }

    let mut touched_groups = BTreeSet::new();

    for file in &files {
        let file_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let content = fs::read_to_string(file)
            .map_err(|e| format!("failed reading message file {}: {}", file.display(), e))?;

        for (idx, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim_end();
            if line.is_empty() {
                continue;
            }
            let Some(msg) = parse_line(line, idx + 1, file_name) else {
                continue;
            };

            touched_groups.insert(msg.chat_jid.clone());
            runtime.db.store_message(msg);
        }

        let dst = processed_dir.join(file.file_name().unwrap_or_default());
        fs::rename(file, &dst)
            .map_err(|e| format!("failed moving {} -> {}: {}", file.display(), dst.display(), e))?;
    }

    for g in &touched_groups {
        runtime.enqueue_group(g.clone());
    }

    let mut outputs = Vec::new();
    loop {
        let Some((chat_jid, prompt)) = runtime.process_next_group_prompt(since_timestamp) else {
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

    if !outputs.is_empty() {
        let mut existing = String::new();
        if output_file.exists() {
            existing = fs::read_to_string(&output_file).map_err(|e| {
                format!("failed reading output file {}: {}", output_file.display(), e)
            })?;
        }
        existing.push_str(&(outputs.join("\n") + "\n"));
        fs::write(&output_file, existing)
            .map_err(|e| format!("failed writing output file {}: {}", output_file.display(), e))?;
    }

    Ok(outputs.len())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::runtime::Runtime;

    use super::run_ipc_once;

    fn unique_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("nanoclaw-ipc-{}-{}", name, nanos))
    }

    #[test]
    fn processes_ipc_message_files_and_writes_responses() {
        let dir = unique_dir("run");
        let messages_dir = dir.join("messages");
        fs::create_dir_all(&messages_dir).expect("mkdir messages");

        fs::write(
            messages_dir.join("batch-1.tsv"),
            concat!(
                "2026-01-01T00:00:01Z\tg1\tAlice\tHello A\n",
                "2026-01-01T00:00:02Z\tg2\tBob\tHello B\n"
            ),
        )
        .expect("write batch");

        let mut runtime = Runtime::new("Andy", 4);
        let count = run_ipc_once(&mut runtime, &dir, "", "Andy").expect("run once");
        assert_eq!(count, 2);

        let out = fs::read_to_string(dir.join("output/responses.tsv")).expect("read responses");
        assert!(out.contains("g1\tAndy: processed"));
        assert!(out.contains("g2\tAndy: processed"));
        assert!(dir.join("processed/batch-1.tsv").exists());

        let _ = fs::remove_dir_all(dir);
    }
}
