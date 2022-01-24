#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use alphanumeric_sort::compare_str;
use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

struct KakMessage(String, Option<String>);

#[derive(Parser)]
#[clap(about, version, author)]
struct Options {
    #[clap(short, long)]
    fifo_name: PathBuf,
    #[clap(short = 'S', long)]
    // TODO: Can we invert a boolean? This name is terrible
    no_skip_whitespace: bool,
    #[clap(short = 'R', long, required = true)]
    regex: String,
    #[clap(multiple_occurrences = true, required = true)]
    selections: Vec<String>,
    #[clap(short, long)]
    lexicographic_sort: bool,
    #[clap(short, long)]
    reverse: bool,
}

fn main() {
    match run() {
        Ok(()) => send_message(&KakMessage("Replaced successfully".to_string(), None)),
        Err(msg) => send_message(&msg),
    }
}

fn send_message(msg: &KakMessage) {
    // TODO: This isn't echoing anything
    let msg_str = msg.0.replace('\'', "''");
    print!("echo '{}';", msg_str);

    if let Some(debug_info) = &msg.1 {
        print!("echo -debug '{}';", msg_str);
        print!("echo -debug '{}';", debug_info.replace('\'', "''"));
    }
}

fn run() -> Result<(), KakMessage> {
    let options = Options::try_parse()?;

    let replacement_re = options.regex;

    let re = Regex::new(&replacement_re)
        .map_err(|_| format!("Invalid regular expression: {}", replacement_re))?;

    let mut zipped = options
        .selections
        .iter()
        .zip(
            options
                .selections
                .iter()
                .map(|a| {
                    if options.no_skip_whitespace {
                        a
                    } else {
                        a.trim()
                    }
                })
                .map(|a| {
                    let captures = re.captures(a)?;
                    captures
                        .get(1)
                        .or_else(|| captures.get(0))
                        .map(|m| m.as_str())
                }),
        )
        .collect::<Vec<(&String, Option<&str>)>>();

    zipped.sort_by(|(a, a_key), (b, b_key)| {
        let a = a_key.unwrap_or(a);
        let b = b_key.unwrap_or(b);

        if options.lexicographic_sort {
            a.cmp(b)
        } else {
            compare_str(a, b)
        }
    });

    print!("reg '\"'");

    let iter: Box<dyn Iterator<Item = _>> = if options.reverse {
        Box::new(zipped.iter().rev())
    } else {
        Box::new(zipped.iter())
    };

    for i in iter {
        let new_selection = i.0.replace('\'', "''");
        print!(" '{}'", new_selection);
    }
    print!(" ;");
    Ok(())
}

impl From<std::io::Error> for KakMessage {
    fn from(err: std::io::Error) -> Self {
        Self(
            "Error writing to fifo".to_string(),
            Some(format!("{:?}", err)),
        )
    }
}

impl From<clap::Error> for KakMessage {
    fn from(err: clap::Error) -> Self {
        Self(
            "Error parsing arguments".to_string(),
            Some(format!("{:?}", err)), // Some(err.message.pieces.map(|p| p.0).join()),
        )
    }
}

impl From<String> for KakMessage {
    fn from(err: String) -> Self {
        Self(err, None)
    }
}
