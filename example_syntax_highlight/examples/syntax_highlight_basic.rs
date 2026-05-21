use std::fmt::Display;
use bat::StripAnsiMode;
use clap::{Parser, ValueEnum};
use std::io::IsTerminal;

/// Pretty-prints JSON to standard output.
#[derive(Debug, Clone, Parser)]
struct Args {
    /// Color mode for output, by default it will auto-detect based on whether stdout is a TTY.
    #[clap(short = 'c', long = "color", default_value_t)]
    color: ColorMode,
}

#[derive(Debug, Default, Clone, ValueEnum)]
enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl Display for ColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMode::Auto => write!(f, "auto"),
            ColorMode::Always => write!(f, "always"),
            ColorMode::Never => write!(f, "never"),
        }
    }
}

fn main() {
    let args = Args::parse();

    let structure = serde_json::json!({
        "example_project": {
            "name": "example",
            "simple_list": [1, 2, 3],
            "complex_list": [
                {
                    "key": "workspace",
                    "value": "example_rich_rust"
                },
                {
                    "key": "example",
                    "value": true,
                }
            ],
            "tags": {
                "environment": "prod",
                "tier": "frontend",
                "depth": 1024,
            }
        }
    });

    pretty_print_lang("json", args.color, serde_json::to_string_pretty(&structure).unwrap());

    println!();
}

fn pretty_print_lang(lang: &str, color_mode: ColorMode, data: impl AsRef<[u8]>) {
    let (strip_ansi, colored_output) = match color_mode {
        ColorMode::Auto => {
            // apparently StripAnsiMode::Auto does _not_ disable ANSI when stdout is not a TTY so we have to
            // do it ourselves
            let strip_ansi = match std::io::stdout().is_terminal() {
                true => StripAnsiMode::Auto,
                false => StripAnsiMode::Always,
            };

            // and StripAnsiMode::Always is not enough
            let colored_output = matches!(strip_ansi, StripAnsiMode::Auto);
            (strip_ansi, colored_output)
        }
        ColorMode::Always => {
            (StripAnsiMode::Never, true)
        }
        ColorMode::Never => {
            (StripAnsiMode::Always, false)
        }
    };

    let mut printer = {
        let mut p = bat::PrettyPrinter::new();

        p.language(lang)
            .input_from_bytes(data.as_ref())
            .strip_ansi(strip_ansi)
            .colored_output(colored_output);

        p
    };

    let _bool_for_some_unknown_undocumented_reason = printer.print().unwrap();
}
