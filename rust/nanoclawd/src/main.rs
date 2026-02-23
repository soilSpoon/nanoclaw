use std::path::PathBuf;

use nanoclaw_core::config::RuntimeConfig;
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

    if parse_flag(&args, "--e2e") {
        let input = match parse_opt(&args, "--input") {
            Some(v) => PathBuf::from(v),
            None => {
                eprintln!("missing --input for --e2e");
                print_usage();
                std::process::exit(2);
            }
        };

        let output = match parse_opt(&args, "--output") {
            Some(v) => PathBuf::from(v),
            None => {
                eprintln!("missing --output for --e2e");
                print_usage();
                std::process::exit(2);
            }
        };

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
