extern crate clap;
extern crate encoding_rs;

use clap::{App, Arg, SubCommand};
use std::io::{Read, Write};
use std::str::FromStr;
use std::io::BufRead;
use encoding_rs::{Decoder};

#[derive(Debug)]
struct BinaryOptions {
    pub max_size: u64,
    pub delimiter: Option<u8>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub prefix: Option<String>,
}

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

impl BinaryOptions {
    pub fn new(max_size: u64, input: Option<String>, output: Option<String>, prefix: Option<String>) -> BinaryOptions {
        BinaryOptions {
            max_size: max_size,
            delimiter: None,
            input: input,
            output: output,
            prefix: prefix,
        }
    }
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<BinaryOptions, Errors> {
        let max_size = match matches.value_of("max-size") {
            Some(v) => match v.parse::<u64>() {
                Ok(v) => v,
                Err(e) => return Err(Errors::Arg(ArgumentError::new("max-size", &format!("parse error: {:?}", e))))
            },
            None => return Err(Errors::Arg(ArgumentError::new("max-size", "max-size is empty")))
        };
        let prefix = matches.value_of("prefix").and_then(|v| Some(String::from(v)));
        let input = matches.value_of("input").and_then(|v| Some(String::from(v)));
        let output = matches.value_of("output").and_then(|v| Some(String::from(v)));
        Ok(Self::new(max_size, input, output, prefix))
    }
    
}

#[derive(Debug)]
struct LineOptions {
    pub max_lines: u64,
    pub max_chars: Option<u64>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub prefix: Option<String>,
    pub encoding: Option<String>,
}

impl LineOptions {
    pub fn new(max_lines: u64, max_chars: Option<u64>, input: Option<String>, output: Option<String>, prefix: Option<String>, encoding: Option<String>) -> LineOptions {
        LineOptions {
            max_lines: max_lines,
            max_chars: max_chars,
            input: input,
            output: output,
            prefix: prefix,
            encoding: encoding,
        }
    }
    pub fn from_arg_matches(matches: &clap::ArgMatches) -> Result<LineOptions, Errors> {
        let max_size = match matches.value_of("max-lines") {
            Some(v) => match v.parse::<u64>() {
                Ok(v) => v,
                Err(e) => return Err(Errors::Arg(ArgumentError::new("max-size", &format!("parse error: {:?}", e))))
            },
            None => return Err(Errors::Arg(ArgumentError::new("max-size", "max-size is empty")))
        };
        let max_chars = match matches.value_of("max-chars") {
            Some(v) => match v.parse::<u64>() {
                Ok(v) => Some(v),
                Err(e) => return Err(Errors::Arg(ArgumentError::new("max-chars", &format!("parse error: {:?}", e))))
            },
            None => None
        };

        let prefix = matches.value_of("prefix").and_then(|v| Some(String::from(v)));
        let input = matches.value_of("input").and_then(|v| Some(String::from(v)));
        let output = matches.value_of("output").and_then(|v| Some(String::from(v)));
        let encoding = matches.value_of("encoding").and_then(|v| Some(String::from(v)));
        Ok(Self::new(max_size, max_chars, input, output, prefix, encoding))
    }
}

#[derive(Debug)]
struct ArgumentError {
    name: String,
    description: String
}

impl ArgumentError {
    pub fn new(name: &str, description: &str) -> ArgumentError {
        ArgumentError {
            name: String::from(name),
            description: String::from(description)
        }
    }
}

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.description)
    }
}

impl std::error::Error for ArgumentError {
}

#[derive(Debug)]
enum Errors {
    Io(std::io::Error),
    Arg(ArgumentError)
}

impl Errors {
    pub fn from_io(e: &std::io::Error, prefix: &str) -> Errors {
        Errors::Io(std::io::Error::new(e.kind(), format!("{}: {:?}", prefix, e)))
    }
}

fn get_file_or_stdin(filepath: &Option<String>) -> Result<Box<dyn Read>, Errors> {
    if let Some(filepath) = filepath {
        match std::fs::File::open(filepath) {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(Errors::from_io(&e, "opening input file"))
        }
    } else {
        Ok(Box::new(std::io::stdin()))
    }
}

fn ensure_dir(dir: &std::path::Path) -> Result<(), Errors> {
    match std::fs::metadata(dir) {
        Ok(v) => {
            if !v.is_dir() {
                return Err(Errors::Io(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("{} is already exist and it is not directory", dir.to_owned().to_str().unwrap()))))
            }
            Ok(())
        },
        Err(_) => {
            std::fs::create_dir_all(dir.to_owned()).or_else(|e| Err(Errors::from_io(&e, "creating output directory")))
        }
    }?;
    Ok(())
}

fn get_lines_from_buf(decoder: &mut Decoder, bytes: &[u8], is_cr: bool) -> Result<(usize, Vec<(String, bool)>, bool), Errors> {
    let mut decoded = String::new();
    let mut strbuf = String::new();
    let mut lines: Vec<(String, bool)> = Vec::new();
    decoded.reserve(decoder.max_utf8_buffer_length(bytes.len()).unwrap());
    strbuf.reserve(1024);
    let mut is_cr_found = is_cr;
    let (_, readchars, _) = decoder.decode_to_string(bytes, &mut decoded, false);
    for c in decoded.chars() {
        if is_cr_found {
            if c == '\r' {
                // found CR CR
                lines.push((strbuf.clone(), true));
                strbuf.clear();
                strbuf.push('\r');
                is_cr_found = true;
            } else if c == '\n' {
                // found CR LF
                strbuf.push(c);
                lines.push((strbuf.clone(), true));
                strbuf.clear();
                is_cr_found = false;
            } else {
                // found CR ?(other than CR and LF)
                lines.push((strbuf.clone(), true));
                strbuf.clear();
                strbuf.push(c);
                is_cr_found = false;
            }
        } else if c == '\n' {
            strbuf.push(c);
            lines.push((strbuf.clone(), true));
            strbuf.clear();
        } else if c != '\r' {
            strbuf.push(c);
        } else {
            is_cr_found = true;
        }
    }
    if strbuf.len() != 0 {
        lines.push((strbuf.clone(), false));
    }
    return Ok((readchars, lines, is_cr_found));
}

fn open_file(index: i32, prefix: &str, output_file_path: &mut std::path::PathBuf) -> Result<std::fs::File, Errors> {
    output_file_path.set_file_name(format!("{}.{}", prefix, index));
    let output_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&output_file_path).or_else(|e| Err(Errors::from_io(&e, "in opening file")))?;
    output_file.set_len(0).or_else(|e| Err(Errors::from_io(&e, "truncating file")))?;
    Ok(output_file)
}

fn rolling_file(index: &mut i32, prefix: &str, output_file_path: &mut std::path::PathBuf, availablelines: &mut u64, max_lines: u64) -> Result<std::fs::File, Errors> {
    *index += 1;
    output_file_path.set_file_name(format!("{}.{}", prefix, index));
    // output_file = std::fs::File::create(output_file_path.to_owned()).or_else(|e| Err(Errors::Io(e)))?;
    let output_file = open_file(*index, &prefix, output_file_path)?;
    *availablelines = max_lines;
    Ok(output_file)
}

fn is_line_ending(s: &str) -> bool {
    s == "\r" || s == "\n" || s == "\r\n"
}

fn split_text_encoding(opts: &LineOptions) -> Result<(), Errors> {
    let mut input = get_file_or_stdin(&opts.input)?;
    let mut availablelines = opts.max_lines;
    let output_directory = match &opts.output {
        Some(v) => std::path::PathBuf::from_str(v.as_str()).unwrap(),
        None => std::env::current_dir().or_else(|e| Err(Errors::from_io(&e, "getting output_directory")))?
    };
    let (mut decoder, mut encoder) = match &opts.encoding {
        Some(v) => match encoding_rs::Encoding::for_label(v.as_bytes()) {
            Some(enc) => (enc.new_decoder(), enc.new_encoder()),
            None => return Err(Errors::Arg(ArgumentError::new("encoding", &format!("invalid encoding name:{}", v))))
        },
        None => (encoding_rs::UTF_8.new_decoder(), encoding_rs::UTF_8.new_encoder())
    };
    ensure_dir(&output_directory)?;
    let mut output_file_path = std::path::PathBuf::from(output_directory);
    let mut index = 0;
    let prefix = opts.prefix.clone().unwrap_or(String::from("bin"));
    output_file_path.push(format!("{}.{}", prefix, index));
    let (max_chars, is_max_chars_set) = match opts.max_chars {
        Some(v) => (v, true),
        None => (0, false)
    };
    let mut output_file = open_file(index, &prefix, &mut output_file_path)?;
    let mut buf = [0u8;1024];
    let mut readoffset = 0;
    let mut wbuf: Vec<u8> = Vec::new();
    wbuf.reserve(4096);
    let mut is_cr = false;
    loop {
        let bytesread = input.read(&mut buf[readoffset..]).or_else(|e| Err(Errors::from_io(&e, "reading file")))?;
        if bytesread == 0 {
            break;
        }
        let (readfrombuf, lines, is_cr_found) = get_lines_from_buf(&mut decoder, &buf[0..bytesread + readoffset], is_cr)?;
        is_cr = is_cr_found;
        if readfrombuf < bytesread + readoffset {
            let mut tmp: Vec<u8> = Vec::new();
            tmp.resize(bytesread + readoffset - readfrombuf, 0);
            tmp.clone_from_slice(&buf[readfrombuf..bytesread + readoffset]);
            buf.clone_from_slice(&tmp);
            readoffset = bytesread + readoffset - readfrombuf;
        }
        for (line, is_last_newline) in lines {
            if is_max_chars_set {
                let mut strbuf = String::new();
                let mut charcount = 0 as usize;
                for c in line.chars() {
                    strbuf.push(c);
                    charcount += 1;
                    if charcount >= max_chars as usize {
                        if availablelines == 0 {
                            output_file = rolling_file(&mut index, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines)?;
                        }
                        wbuf.reserve(strbuf.len());
                        let (_, _, _) = encoder.encode_from_utf8_to_vec(&strbuf, &mut wbuf, false);
                        output_file.write(&wbuf).or_else(|e| Err(Errors::from_io(&e, "writing to output file")))?;
                        if !strbuf.ends_with("\r") && !strbuf.ends_with("\n") {
                            output_file.write(LINE_ENDING.as_bytes()).or_else(|e| Err(Errors::from_io(&e, "writing newline")))?;
                        }
                        availablelines -= 1;
                        wbuf.clear();
                        strbuf.clear();
                        charcount = 0;
                    }
                }
                if strbuf.len() != 0 && !is_line_ending(&strbuf) {
                    if availablelines == 0 {
                        output_file = rolling_file(&mut index, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines)?;
                    }
                    wbuf.reserve(strbuf.len());
                    let (_, _, _) = encoder.encode_from_utf8_to_vec(&strbuf, &mut wbuf, false);
                    output_file.write(&wbuf).or_else(|e| Err(Errors::from_io(&e, "writing to output file")))?;
                    if is_last_newline {
                        availablelines -= 1;
                    }
                    wbuf.clear();
                    strbuf.clear();
                }
            } else {
                if availablelines == 0 {
                    output_file = rolling_file(&mut index, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines)?;
                }
                wbuf.reserve(line.len());
                let (_, _, _) = encoder.encode_from_utf8_to_vec(&line, &mut wbuf, false);
                output_file.write(&wbuf).or_else(|e| Err(Errors::from_io(&e, "writing to output file")))?;
                if is_last_newline {
                    availablelines -= 1;
                }
                wbuf.clear();
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn split_text(opts: &LineOptions) -> Result<(), Errors> {
    let input = get_file_or_stdin(&opts.input)?;
    let mut reader = std::io::BufReader::new(input);
    let mut line = String::new();
    let mut availablelines = opts.max_lines;
    let output_directory = match &opts.output {
        Some(v) => std::path::PathBuf::from_str(v.as_str()).unwrap(),
        None => std::env::current_dir().or_else(|e| Err(Errors::from_io(&e, "getting output_directory")))?
    };
    ensure_dir(&output_directory)?;
    let mut output_file_path = std::path::PathBuf::from(output_directory);
    let mut index = 0;
    let prefix = opts.prefix.clone().unwrap_or(String::from("bin"));
    output_file_path.push(format!("{}.{}", prefix, index));
    line.reserve(4096);
    let (max_chars, is_max_chars_set) = match opts.max_chars {
        Some(v) => (v, true),
        None => (0, false)
    };
    let mut output_file = open_file(index, &prefix, &mut output_file_path)?;
    loop {
        let read = reader.read_line(&mut line).or_else(|e| Err(Errors::from_io(&e, "reading line")))?;
        if read == 0 {
            break;
        }
        if availablelines == 0 {
            output_file = rolling_file(&mut index, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines)?;
        }
        if is_max_chars_set {
            let mut offset = 0;
            loop {
                let wlen = std::cmp::min(line.len() - offset, max_chars as usize);
                output_file.write(line[offset..offset+wlen].as_bytes()).or_else(|e| Err(Errors::from_io(&e, "writing output file")))?;
                offset += wlen;
                availablelines -= 1;
                if availablelines == 0 {
                    output_file = rolling_file(&mut index, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines)?;
                }
                if offset >= line.len() {
                    break;
                }
            }
            line.clear();
        } else {
            output_file.write(line.as_bytes()).or_else(|e| Err(Errors::from_io(&e, "writing output file")))?;
            availablelines -= 1;
            line.clear();
        }
    }
    Ok(())
}

fn split_binary(opts: &BinaryOptions) -> Result<(), Errors> {
    let mut input = get_file_or_stdin(&opts.input)?;
    let mut buf = [0u8;1024];
    let curdir = std::env::current_dir().or_else(|e| Err(Errors::from_io(&e, "getting current directory")))?;
    let output_directory = match &opts.output {
        Some(v) => std::path::PathBuf::from(v),
        None => curdir
    };
    ensure_dir(&output_directory)?;
    let prefix = opts.prefix.clone().unwrap_or(String::from("bin"));
    let mut output_file_path = output_directory.to_path_buf();
    let mut file_index = 0;
    output_file_path.push(format!("{}.{}", prefix, file_index));
    let mut output_file = std::fs::File::create(output_file_path.to_owned()).or_else(|e| Err(Errors::from_io(&e, "creating output file")))?;
    let mut available = opts.max_size;
    loop {
        let bytesread = input.read(&mut buf).or_else(|e| Err(Errors::from_io(&e, "reading from input file")))?;
        if bytesread == 0 {
            break;
        }
        let mut remaining = bytesread;
        while remaining > 0 {
            let bytesavailable = std::cmp::min(remaining as usize, available as usize);
            output_file.write(&buf[0..bytesavailable]).or_else(|e| Err(Errors::from_io(&e, "writing output file")))?;
            remaining -= bytesavailable;
            available -= bytesavailable as u64;
            if available == 0 && remaining != 0 {
                output_file = rolling_file(&mut file_index, &prefix, &mut output_file_path, &mut available, opts.max_size)?;
            }
        }
    }
    Ok(())
}

fn create_input_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("input")
        .short("i").long("input").takes_value(true).help("input file(default: stdin)")
}

fn create_output_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("output")
        .short("o").long("output").takes_value(true).help("output folder(default: current directory)")
}

fn create_prefix_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("prefix")
        .short("p")
        .long("prefix")
        .takes_value(true)
        .help("output file prefix(default: using filename when filename can be used, or \"bin\" will be used)")
}

fn create_binary_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("binary")
            .alias("b")
            .about("split by binary")
            .arg(Arg::with_name("max-size").alias("m").required(true).help("max binary size of splitted binary"))
            .arg(create_input_option())
            .arg(create_output_option())
            .arg(create_prefix_option())
}

fn create_text_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("text")
            .alias("t")
            .about("split by text")
            .arg(Arg::with_name("max-lines").alias("m").required(true).help("max line number per file"))
            .arg(Arg::with_name("max-chars").long("max-chars").takes_value(true).help("max characters per line"))
            .arg(Arg::with_name("encoding").short("e").long("encoding").takes_value(true).help("input text encoding(default: utf-8)"))
            .arg(create_input_option())
            .arg(create_output_option())
            .arg(create_prefix_option())
}

fn main() -> Result<(), Errors>{
    let app = App::new("dbin")
        .version("0.1.0")
        .author("itn3000")
        .about("binary/text splitter")
        .subcommand(create_binary_subcommand())
        .subcommand(create_text_subcommand())
        ;
    let matches = app.get_matches();
    if let Some(matches) = matches.subcommand_matches("text") {
        // process as text
        let opts = LineOptions::from_arg_matches(matches)?;
        println!("{:?}", opts);
        split_text_encoding(&opts)?;
    } else if let Some(matches) = matches.subcommand_matches("binary") {
        let opts = BinaryOptions::from_arg_matches(&matches)?;
        println!("{:?}", opts);
        split_binary(&opts)?;
    } else {
        println!("{}", matches.usage());
        println!("`--help` for more details");
    }
    Ok(())
    // Options::new().optopt("m", "mode", "split mode", "valid values are bin or line(default: bin)")
    //     .optopt("d", "delimiter", "split delimiter");

    // println!("Hello, world!");
}
