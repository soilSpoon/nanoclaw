use std::path::{Path, PathBuf};

use nanoclaw_core::config::RuntimeConfig;
use nanoclaw_core::container_output::parse_streamed_outputs;
use nanoclaw_core::e2e::run_file_e2e;

fn parse_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn parse_opt(args: &[String], name: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == name)?;
    args.get(idx + 1).cloned()
}

fn print_usage() {
    println!("nanoclawd usage:");
    println!("  --dry-run");
    println!("  --e2e --input <path> --output <path> [--assistant-name <name>]");
    println!("  --parse-container-output --input <path> --output <path>");
}

fn print_missing_input_hint(input: &Path) {
    eprintln!(
        "input file does not exist: {}\n\
Create one first (tab-separated: timestamp\\tchat_jid\\tsender_name\\tcontent):\n\
  cat > {} <<'EOF'\n\
  2026-01-01T00:00:01Z\tgroup-a\tAlice\tHello from A\n\
  2026-01-01T00:00:02Z\tgroup-b\tBob\tHello from B\n\
  EOF",
        input.display(),
        input.display()
    );
}

fn require_io_paths(args: &[String], mode: &str) -> (PathBuf, PathBuf) {
    let input = match parse_opt(args, "--input") {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("missing --input for {}", mode);
            print_usage();
            std::process::exit(2);
        }
    };

    let output = match parse_opt(args, "--output") {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("missing --output for {}", mode);
            print_usage();
            std::process::exit(2);
        }
    };

    (input, output)
}

fn run_parse_container_output(input: &Path, output: &Path) {
    if !input.exists() {
        eprintln!("input file does not exist: {}", input.display());
        std::process::exit(2);
    }

    let content = match std::fs::read_to_string(input) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("failed reading {}: {}", input.display(), err);
            std::process::exit(1);
        }
    };

    let parsed = parse_streamed_outputs(&content);
    let mut lines = Vec::new();
    for o in parsed {
        lines.push(format!(
            "status={}\tresult={}\tnew_session_id={}\terror={}",
            o.status,
            o.result.unwrap_or_default().replace('\n', "\\n"),
            o.new_session_id.unwrap_or_default(),
            o.error.unwrap_or_default()
        ));
    }

    if let Err(err) = std::fs::write(output, lines.join("\n") + if lines.is_empty() { "" } else { "\n" }) {
        eprintln!("failed writing {}: {}", output.display(), err);
        std::process::exit(1);
    }

    println!(
        "nanoclawd parsed container output: records={} input={} output={}",
        lines.len(),
        input.display(),
        output.display()
    );
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let cfg = RuntimeConfig::from_env();

    if parse_flag(&args, "--dry-run") {
        println!(
            "nanoclawd dry-run assistant_name={} poll_interval_ms={} idle_timeout_ms={}",
            cfg.assistant_name, cfg.poll_interval_ms, cfg.idle_timeout_ms
        );
        return;
    }

    if parse_flag(&args, "--parse-container-output") {
        let (input, output) = require_io_paths(&args, "--parse-container-output");
        run_parse_container_output(&input, &output);
        return;
    }

    if parse_flag(&args, "--e2e") {
        let (input, output) = require_io_paths(&args, "--e2e");

        if !input.exists() {
            print_missing_input_hint(&input);
            std::process::exit(2);
        }

        let assistant_name = parse_opt(&args, "--assistant-name").unwrap_or(cfg.assistant_name);

        match run_file_e2e(&assistant_name, &input, &output) {
            Ok(processed_groups) => {
                println!(
                    "nanoclawd e2e complete: processed_groups={} input={} output={}",
                    processed_groups,
                    input.display(),
                    output.display()
                );
            }
            Err(err) => {
                eprintln!("nanoclawd e2e failed: {}", err);
                std::process::exit(1);
            }
        }

        return;
    }

    print_usage();
    println!("nanoclawd bootstrap complete (runtime loop port in progress)");
}
