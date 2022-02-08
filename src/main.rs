use std::io::{self, Write};
use std::time::SystemTime;
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

const HELP: &'static str = "
Generate uuids.

Supports v1 and v4 uuids with hex and 'normal' formats.

> uuid
2b4a3c2c-a8a4-4c87-bbae-cc6b15a962a4
> uuid --format hex
f21318cab06c43468e999cb88037e1f5

Options:
-n, --amount [AMOUNT]           The amount of uuids to generate, default: 1
-f, --format [FORMAT]           The format to output the uuids, default: normal, possible values: [normal, hex]
-v, --version [VERSION]         The with uuid version to use, default: v4, possible values: [v1, v4]

Examples:
> uuid -n 10
> uuid -v 4 -n 100 -f hex
> uuid --version 4 --amount 100 --format hex

";

#[derive(Debug)]
enum Error {
    String(String),
    Error(pico_args::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::String(s) => {
                write!(f, "{}", s)
            }
            Error::Error(e)=> {
                write!(f, "{}", e)
            }
        }
    }
}

impl From<String> for Error {
    fn from(input: String) -> Error {
        Error::String(input)
    }
}

impl From<pico_args::Error> for Error {
    fn from(input: pico_args::Error) -> Error {
        Error::Error(input)
    }
}

impl std::error::Error for Error {}


#[derive(Debug, PartialEq)]
pub enum Format {
    Hex,
    Normal,
}

impl Format {
    pub const fn char_length(&self) -> usize {
        match self {
            Format::Normal => 36,
            Format::Hex => 32,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Version {
    V1,
    V4,
}

#[derive(Debug, PartialEq)]
pub struct Args {
    format: Format,
    version: Version,
    amount: usize,
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            print!("{}", HELP);
            std::process::exit(1);
        }
    };

    let stdout = io::stdout();
    let lock = stdout.lock();

    to_writer(args, lock)
}

fn parse_args() -> Result<Args, Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Args {
        version: pargs.value_from_fn(["-v", "--version"], parse_version).unwrap_or(Version::V4),
        format: pargs.value_from_fn(["-f", "--format"], parse_format).unwrap_or(Format::Normal),
        amount: pargs.value_from_str(["-n", "--amount"]).unwrap_or(1),
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        return Err(format!("unused arguments left: {:?}", remaining).into());
    }

    Ok(args)
}

fn parse_version(s: &str) -> Result<Version, &'static str> {
    match s.to_lowercase().as_ref() {
        "v1" | "1" => Ok(Version::V1),
        "v4" | "4" => Ok(Version::V4),
        _ => Err("invalid version")
    }
}

fn parse_format(s: &str) -> Result<Format, &'static str> {
    match s.to_lowercase().as_ref() {
        "normal" => Ok(Format::Normal),
        "hex" => Ok(Format::Hex),
        _ => Err("invalid format")
    }
}

pub fn to_writer<W: Write>(args: Args, mut writer: W) {
    let mut handler = Handler::new(args.version);
    for _ in 0..args.amount {
        let uuid = handler.next();
        if args.format == Format::Hex {
            writer
                .write_fmt(format_args!("{}\n", uuid.to_simple_ref()))
                .expect("writing to output failed");
        } else {
            writer
                .write_fmt(format_args!("{}\n", uuid))
                .expect("writing to output failed");
        }
    }
}

pub fn to_string(args: Args) -> String {
    let mut handler = Handler::new(args.version);
    let mut buffer = String::with_capacity((args.format.char_length() + 1) * args.amount);
    for _ in 0..args.amount {
        let uuid = handler.next();
        buffer.push_str(&if args.format == Format::Hex {
            uuid.to_simple_ref().to_string()
        } else {
            uuid.to_string()
        });
        buffer.push('\n');
    }

    buffer
}

struct Handler {
    context: Option<Context>,
    version: Version,
}

impl Handler {
    fn new(version: Version) -> Handler {
        if version == Version::V4 {
            Handler {
                context: None,
                version: Version::V4,
            }
        } else {
            let mut buffer = [0, 0];
            getrandom::getrandom(&mut buffer).expect("unable to get random number");

            Handler {
                context: Some(Context::new(u16::from_ne_bytes(buffer))),
                version: Version::V1,
            }
        }
    }

    fn next(&mut self) -> Uuid {
        if self.version == Version::V4 {
            Uuid::new_v4()
        } else {
            let (secs, nano) = get_time();
            let timestamp = Timestamp::from_unix(self.context.as_ref().unwrap(), secs, nano);
            Uuid::new_v1(timestamp, &process_id()).expect("failed to generate UUID")
        }
    }
}

fn get_time() -> (u64, u32) {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => {
            let secs = n.as_secs();
            let nano = n.subsec_nanos();
            (secs, nano)
        }
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn process_id() -> [u8; 6] {
    let mut id = [0, 0, 0, 0, 0, 0];
    getrandom::getrandom(&mut id).expect("unable to get random number");

    for (i, byte) in std::process::id().to_ne_bytes().into_iter().enumerate() {
        id[i + 2] = byte;
    }

    id
}
