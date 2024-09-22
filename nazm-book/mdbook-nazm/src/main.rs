use mdbook::errors::Result;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_nazm::NazmPreprocessor;
use std::process::exit;

fn main() {
    // Collect the command-line arguments into a vector
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1].as_str() == "supports" {
        if args.len() < 3 {
            eprintln!("Error: 'supports' subcommand requires a 'renderer' argument.");
            exit(1)
        }

        let renderer = &args[2];

        if NazmPreprocessor.supports_renderer(renderer) {
            exit(0)
        } else {
            exit(1)
        }
    }

    if let Err(err) = handle_preprocessing() {
        eprintln!("Error: {err}");
        exit(1)
    }
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(std::io::stdin())?;

    let processed_book = NazmPreprocessor.run(&ctx, book)?;

    serde_json::to_writer(std::io::stdout(), &processed_book)?;

    Ok(())
}
