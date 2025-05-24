use std::str::FromStr;

use areq::range::Range;
use areq::version::pure::PureVersion;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!("Press Ctrl-D to exit");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                match Range::<PureVersion>::from_str(&line) {
                    Ok(range) => println!("Range: {}", range),
                    Err(err) => print_error(err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                print_error(err);
                break;
            }
        }
    }
    Ok(())
}

fn print_error(err: impl std::error::Error) {
    println!("Error: {}", err);
    if let Some(mut source) = err.source() {
        println!("Caused by:");

        println!("  - {}", source);
        while let Some(cause) = source.source() {
            println!("  - {}", cause);
            source = cause;
        }
    }
}
