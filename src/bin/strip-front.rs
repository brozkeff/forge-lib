use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let mut keep_keys: Option<String> = None;
    let mut file_path: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("strip-front {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--keep" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --keep requires a value");
                    return ExitCode::from(1);
                }
                keep_keys = Some(args[i].clone());
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return ExitCode::from(1);
            }
            _ => {
                file_path = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let Some(path) = file_path else {
        eprintln!("Usage: strip-front [--keep key1,key2] <file>");
        return ExitCode::from(1);
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };

    let output = if let Some(ref keys) = keep_keys {
        forge_lib::strip::strip_front_keep(&content, keys)
    } else {
        forge_lib::strip::strip_front(&content)
    };

    print!("{output}");
    ExitCode::SUCCESS
}
