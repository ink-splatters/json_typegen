use clap::{Arg, Command};
use json_typegen_shared::internal_util::display_error_with_causes;
use json_typegen_shared::{Options, OutputMode, codegen, codegen_from_macro, parse};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

fn main_with_result() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("json_typegen CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Generate Rust types from JSON samples")
        .arg(
            Arg::new("input")
                .help(concat!(
                    "The input to generate types from. A sample, file, URL, or macro. To read ",
                    "from standard input, a dash, '-', can be used as the input argument."
                ))
                .num_args(1)
                .required(true),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("Name for the root generated type. Default: Root.")
                .num_args(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("What file to write the output to. Default: standard output.")
                .num_args(1),
        )
        .arg(
            Arg::new("options")
                .long("options")
                .help(concat!(
                    "Options for code generation, in the form of an options block. If input is a ",
                    "macro, this option is ignored."
                ))
                .num_args(1),
        )
        .arg(
            Arg::new("output-mode")
                .long("output-mode")
                .short('O')
                .value_parser([
                    "rust",
                    "typescript",
                    "typescript/typealias",
                    "kotlin",
                    "kotlin/jackson",
                    "kotlin/kotlinx",
                    "scala",
                    "python",
                    "json_schema",
                    "shape",
                ])
                .help("What to output.")
                .num_args(1),
        )
        .get_matches();

    let source = matches
        .get_one::<String>("input")
        .ok_or("Input argument is required")?;

    let input = if source == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        source.clone()
    };

    let code = if input.trim().starts_with("json_typegen") {
        codegen_from_macro(&input)
    } else {
        let name = matches
            .get_one::<String>("name")
            .map(String::as_str)
            .unwrap_or("Root");
        let mut options = match matches.get_one::<String>("options") {
            Some(block) => parse::options(block)?,
            None => Options::default(),
        };
        if let Some(output_mode) = matches.get_one::<String>("output-mode") {
            options.output_mode = OutputMode::parse(output_mode).ok_or("Invalid output mode")?;
        }
        codegen(name, &input, options)
    };

    if let Some(filename) = matches.get_one::<String>("output") {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;

        file.write_all(code?.as_bytes())?;
    } else {
        print!("{}", code?);
    }

    Ok(())
}

fn main() {
    let result = main_with_result();

    if let Err(e) = result {
        eprintln!("Error: {}", display_error_with_causes(&*e));
        std::process::exit(1);
    }
}
