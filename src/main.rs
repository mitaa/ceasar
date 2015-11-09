#![feature(io)]
use std::io::{
    Read,
    Write,
};

extern crate getopts;
use getopts::Options;
use std::env;


#[macro_use]
extern crate log;
extern crate env_logger;


const A_MAX: u8 = 26;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn transform<C: Read, O: Write>(mut pipe: O, ctext: C, mut shift: i8) {
    while shift >= A_MAX as i8
        { shift -= A_MAX as i8; }
    while shift < 0
        { shift += A_MAX as i8; }
    debug_assert!(shift >= 0 && shift <= (A_MAX as i8));

    let offset: u8 = shift as u8;

    for cres in ctext.chars() {
        let mut c = cres.expect("this shouldn't happen...") as u8;

        if c >= ('a' as u8) && c <= ('z' as u8) {
            c += offset;
            while c > ('z' as u8)
                { c -= A_MAX; }

        } else if c >= ('A' as u8) && c <= ('Z' as u8) {
            c += offset;
            while c > ('Z' as u8)
                { c -= A_MAX }
        }

        if let Err(f) = pipe.write(&[c]) {
            error!("broken output pipe: {}", f.to_string());
            return;
        }
    }
}


fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("i", "input", "input plaintext", "INPUT");
    opts.optopt("s", "shift", "shift value", "SHIFT");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            error!("{}", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let shift: i8 = match matches.opt_str("shift") {
        Some(shift) => {
            match shift.parse() {
                Ok(v) => v,
                Err(f) => {
                    error!("cannot parse shift value: `{}` ({})", shift, f.to_string());
                    return;
                },
            }
        },
        None => {
            print_usage(&program, opts);
            return;
        }
    };

    match matches.opt_str("input") {
        Some(input) => {
            let mut pipe = std::io::stdout();
            transform(&mut pipe, input.as_bytes(), shift);
            if let Err(f) = pipe.write("\n".as_bytes()) {
                error!("broken output pipe: {}", f.to_string());
            }
        }
        None => {
            // let's for now assume a trailing newline in the input stream...
            transform(std::io::stdout(), std::io::stdin(), shift);
        }
    };
}