extern crate clap;
extern crate encoding_rs;

use clap::{App, Arg, SubCommand};
use std::io::{Read, Write};
use std::str::FromStr;
use encoding_rs::{Decoder};
use std::iter::Iterator;
use std::iter::FromIterator;

#[derive(Debug, Default)]
struct BinaryOptions {
    pub max_size: u64,
    pub delimiter: Option<u8>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub prefix: Option<String>,
    pub extra_suffix: Option<String>,
    pub is_numerical_suffix: bool,
}

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

impl BinaryOptions {
    pub fn new(max_size: u64) -> BinaryOptions {
        let mut ret = Self::default();
        ret.max_size = max_size;
        ret
    }
    fn default() -> Self {
        Default::default()
    }
    pub fn with_input(mut self, s: Option<&str>) -> Self {
        self.input = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_output(mut self, s: Option<&str>) -> Self {
        self.output = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_prefix(mut self, s: Option<&str>) -> Self {
        self.prefix = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_extra_suffix(mut self, s: Option<&str>) -> Self {
        self.extra_suffix = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_is_numerical_suffix(mut self, b: bool) -> Self {
        self.is_numerical_suffix = b;
        self
    }
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<BinaryOptions, Errors> {
        let max_size = match matches.value_of("max-size") {
            Some(v) => match v.parse::<u64>() {
                Ok(v) => v,
                Err(e) => return Err(Errors::Arg(ArgumentError::new("max-size", &format!("parse error: {:?}", e))))
            },
            None => return Err(Errors::Arg(ArgumentError::new("max-size", "max-size is empty")))
        };
        return Ok(Self::new(max_size)
            .with_input(matches.value_of("input"))
            .with_output(matches.value_of("output"))
            .with_prefix(matches.value_of("prefix"))
            .with_extra_suffix(matches.value_of("extra-suffix"))
            .with_is_numerical_suffix(matches.is_present("numerical-suffix")));
    }
    
}

#[derive(Debug, Default)]
struct LineOptions {
    pub max_lines: u64,
    pub max_chars: Option<u64>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub prefix: Option<String>,
    pub encoding: Option<String>,
    pub is_numerical_suffix: bool,
    pub extra_suffix: Option<String>
}

impl LineOptions {
    pub fn new(max_lines: u64) -> Self {
        let mut ret = Self::default();
        ret.max_lines = max_lines;
        ret
    }
    fn default() -> Self {
        Default::default()
    }
    pub fn with_max_chars(mut self, max_chars: Option<u64>) -> Self {
        self.max_chars = max_chars;
        self
    }
    pub fn with_prefix(mut self, prefix: Option<&str>) -> Self {
        self.prefix = prefix.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_input(mut self, s: Option<&str>) -> Self {
        self.input = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_output(mut self, s: Option<&str>) -> Self {
        self.output = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_encoding(mut self, s: Option<&str>) -> Self {
        self.encoding = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_extra_suffix(mut self, s: Option<&str>) -> Self {
        self.extra_suffix = s.and_then(|v| Some(String::from(v)));
        self
    }
    pub fn with_is_numerical_suffix(mut self, b: bool) -> Self {
        self.is_numerical_suffix = b;
        self
    }
    fn parse_u64(s: &str, name: &str) -> Result<u64, Errors> {
        match s.parse::<u64>() {
            Ok(v) => Ok(v),
            Err(e) => return Err(Errors::Arg(ArgumentError::new(name, &format!("parse error: {:?}", e))))
        }
    }
    pub fn from_arg_matches(matches: &clap::ArgMatches) -> Result<LineOptions, Errors> {
        let max_size = match matches.value_of("max-lines") {
            Some(v) => Self::parse_u64(v, "max-size")?,
            None => return Err(Errors::Arg(ArgumentError::new("max-size", "max-size is empty")))
        };
        let max_chars = match matches.value_of("max-chars") {
            Some(v) => Some(Self::parse_u64(v, "max-chars")?),
            None => None
        };
        Ok(Self::new(max_size).with_prefix(matches.value_of("prefix"))
            .with_max_chars(max_chars)
            .with_input(matches.value_of("input"))
            .with_output(matches.value_of("output"))
            .with_encoding(matches.value_of("encoding"))
            .with_extra_suffix(matches.value_of("extra-suffix"))
            .with_is_numerical_suffix(matches.is_present("numerical-suffix")))
        // Ok(ret)
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

fn open_file(suffixstr: &mut String, prefix: &str, output_file_path: &mut std::path::PathBuf, is_numerical_suffix: bool, extra_suffix: &str) -> Result<std::fs::File, Errors> {
    if suffixstr == "" {
        suffixstr.push_str(match is_numerical_suffix { true => "0", false => "aa" });
    }
    let next_suffixstr = get_next_suffix(suffixstr, is_numerical_suffix);
    output_file_path.set_file_name(format!("{}{}{}", prefix, suffixstr, extra_suffix));
    let output_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&output_file_path).or_else(|e| Err(Errors::from_io(&e, "in opening file")))?;
    output_file.set_len(0).or_else(|e| Err(Errors::from_io(&e, "truncating file")))?;
    suffixstr.clear();
    suffixstr.push_str(next_suffixstr.as_str());
    Ok(output_file)
}

fn get_next_suffix(current_suffix: &str, is_numerical_suffix: bool) -> String {
    let mut ret = String::new();
    if !is_numerical_suffix {
        let ztrimed = current_suffix.trim_start_matches("z");
        let (processed, _ ) = ztrimed.chars().rev().fold((Vec::new() as Vec<char>, true), |(st, should_increment), item| {
            let mut st = st;
            if should_increment {
                if item != 'z' {
                    st.push((item as u8 + 1) as char);
                    return (st, false);
                } else {
                    st.push('a');
                    return (st, true);
                }
            } else {
                st.push(item);
                return (st, false);
            }
        });
        for c in current_suffix.chars().take_while(|v| *v == 'z') {
            ret.push(c);
        }
        ret.push_str(String::from_iter(processed.iter().rev()).as_str());
        if processed[processed.len() - 1] == 'z' {
            println!("pushing aa");
            ret.push_str("aa");
        }
    } else {
        let value = current_suffix.parse::<u64>().unwrap();
        return format!("{}", value + 1);
    }
    ret
}

fn rolling_file(current_suffix: &mut String, prefix: &str, output_file_path: &mut std::path::PathBuf, availablelines: &mut u64, max_lines: u64, is_numerical: bool, extra_suffix: &str) -> Result<std::fs::File, Errors> {
    // output_file = std::fs::File::create(output_file_path.to_owned()).or_else(|e| Err(Errors::Io(e)))?;
    let output_file = open_file(current_suffix, &prefix, output_file_path, is_numerical, extra_suffix)?;
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
    let prefix = opts.prefix.clone().unwrap_or(String::from("x"));
    output_file_path.push(format!("{}.{}", prefix, ""));
    let (max_chars, is_max_chars_set) = match opts.max_chars {
        Some(v) => (v, true),
        None => (0, false)
    };
    let mut current_suffix = String::new();
    let extra_suffix = opts.extra_suffix.clone().unwrap_or(String::new());
    let mut output_file = open_file(&mut current_suffix, &prefix, &mut output_file_path, opts.is_numerical_suffix, &extra_suffix)?;
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
                            let next_output_file = rolling_file(&mut current_suffix, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines, opts.is_numerical_suffix, &extra_suffix)?;
                            output_file = next_output_file;
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
                        let next_output_file = rolling_file(&mut current_suffix, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines, opts.is_numerical_suffix, &extra_suffix)?;
                        output_file = next_output_file;
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
                    let next_output_file = rolling_file(&mut current_suffix, &prefix, &mut output_file_path, &mut availablelines, opts.max_lines, opts.is_numerical_suffix, &extra_suffix)?;
                    output_file = next_output_file;
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

fn split_binary(opts: &BinaryOptions) -> Result<(), Errors> {
    let mut input = get_file_or_stdin(&opts.input)?;
    let mut buf = [0u8;1024];
    let output_directory = match &opts.output {
        Some(v) => std::path::PathBuf::from(v),
        None => match std::env::current_dir().or_else(|e| Err(Errors::from_io(&e, "getting current directory"))) {
            Ok(v) => v,
            Err(e) => return Err(e)
        }
    };
    ensure_dir(&output_directory)?;
    let prefix = opts.prefix.clone().unwrap_or(String::from("x"));
    let mut output_file_path = output_directory.to_path_buf();
    output_file_path.push(format!("{}.{}", prefix, ""));
    let mut current_suffix = String::new();
    let extra_suffix = opts.extra_suffix.clone().unwrap_or_default();
    let mut output_file = open_file(&mut current_suffix, &prefix, &mut output_file_path, opts.is_numerical_suffix, &extra_suffix)?;
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
                let next_output_file = rolling_file(&mut current_suffix, &prefix, &mut output_file_path, &mut available, opts.max_size, opts.is_numerical_suffix, &extra_suffix)?;
                output_file = next_output_file;
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
        .help("output file prefix(default: using filename when filename can be used, or \"x\" will be used)")
}

fn create_numeric_suffix_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("numerical-suffix")
        .short("n")
        .long("numerical-suffix")
        .takes_value(false)
        .help("add numerical suffix('0', '1',...) to output file(default: 'aa', 'ab',...)")
}

fn create_extra_suffix_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("extra-suffix")
        .long("extra-suffix")
        .takes_value(true)
        .help("add extra suffix to output file")
}

fn create_binary_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("binary")
            .alias("b")
            .about("split by binary")
            .arg(Arg::with_name("max-size").alias("m").required(true).help("max binary size of splitted binary"))
            .arg(create_input_option())
            .arg(create_output_option())
            .arg(create_prefix_option())
            .arg(create_numeric_suffix_option())
            .arg(create_extra_suffix_option())
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
            .arg(create_numeric_suffix_option())
            .arg(create_extra_suffix_option())
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
