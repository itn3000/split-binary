extern crate getopts;
extern crate clap;

use clap::{App, Arg, SubCommand};
use std::io::{Read, Write};

struct BinaryOptions {
    pub max_size: Option<u64>,
    pub delimiter: Option<u8>,
    pub max_num: Option<u64>,
    pub input: Option<String>,
    pub output: Option<String>
}

struct LineOptions {
    pub max_lines: Option<u64>,
    pub delimiter: String,
    pub max_num: Option<u64>,
    pub input: Option<String>,
    pub output: Option<String>
}

enum CommandOptions {
    Binary(BinaryOptions),
    Line(LineOptions)
}

enum Errors {
    Io(std::io::Error)
}

enum StdInOrFile {
    Stdin(std::io::Stdin),
    File(std::fs::File)
}

enum StdOutOrFile {
    Stdout(std::io::Stdout),
    File(std::fs::File)
}

fn split_binary(opts: &BinaryOptions) -> Result<(), Errors> {
    let input = if let Some(inputfilepath) = &opts.input {
        StdInOrFile::File(match std::fs::File::open(std::path::Path::new(&inputfilepath)) {
            Ok(f) => f,
            Err(e) => return Err(Errors::Io(std::io::Error::from(e)))
        })
    }
    else {
        StdInOrFile::Stdin(std::io::stdin())
    };
    let output = if let Some(outputfilepath) = &opts.output {
        StdOutOrFile::File(match std::fs::File::create(std::path::Path::new(&outputfilepath)) {
            Ok(f) => f,
            Err(e) => return Err(Errors::Io(std::io::Error::from(e)))
        })
    } else {
        StdOutOrFile::Stdout(std::io::stdout())
    };
    let mut buf = [0u8;1024];
    loop {
        let input: dyn std::io::Read = match input {
            StdInOrFile::File(f) => f,
            StdInOrFile::Stdin(f) => f,
        };
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let app = App::new("dbin")
        .version("0.1.0")
        .author("itn3000")
        .about("binary/text splitter")
        .arg(Arg::with_name("input")
            .alias("i").help("input file(default: stdin)"))
        .arg(Arg::with_name("output")
            .alias("o").help("output folder(default: current directory)"))
        .arg(Arg::with_name("prefix").alias("p").help("output file prefix(default: using filename when filename can be used, or \"bin\" will be used)"))
        .subcommand(SubCommand::with_name("binary")
            .alias("b")
            .about("split by binary(default)")
            .arg(Arg::with_name("max-size").alias("m").help("max binary size of splitted binary"))
            .arg(Arg::with_name("delimiter").alias("d").help("split when any specified integer value is found(0 - 255)").use_delimiter(true).multiple(true))
            )
        .subcommand(SubCommand::with_name("text")
            .alias("t")
            .about("split by lines")
            .arg(Arg::with_name("max-lines").alias("m").help("max lines of splitted text"))
            .arg(Arg::with_name("delimiter")
                .alias("d")
                .help("split when specified string is found")
            ))
        ;
    let matches = app.get_matches();
    if let Some(matches) = matches.subcommand_matches("text") {
        // process as text
    }
    else {

    }
    // Options::new().optopt("m", "mode", "split mode", "valid values are bin or line(default: bin)")
    //     .optopt("d", "delimiter", "split delimiter");

    // println!("Hello, world!");
}
