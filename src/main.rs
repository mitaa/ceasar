#![feature(io)]
use std::io::{
    Read,
    Write,
};

use std::env;

const A_MAX: u8 = 26;

fn print_usage(program: &str) {
    let brief = format!("Usage: {} SHIFT [PLAINTEXT] [options]", program);
    print!(
"Usage: {} SHIFT [PLAINTEXT] [options]

Options:
    -h, --help      print this help menu
", program);
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
            println!("broken output pipe: {}", f.to_string());
            return;
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args[1..].iter().map(|s|&s[..]).any(|s| s == "-h" || s == "--help") || args.len() <= 1 {
        print_usage(&program);
        return;
    }

    let shift: i8 = match args[1].parse() {
        Ok(v) => v,
        Err(f) => {
            println!("cannot parse shift value: `{}` ({})", args[1], f.to_string());
            return;
        },
    };

    match args.get(2) {
        Some(input) => {
            let mut pipe = std::io::stdout();
            transform(&mut pipe, input.as_bytes(), shift);
            if let Err(f) = pipe.write("\n".as_bytes()) {
                println!("broken output pipe: {}", f.to_string());
            }
        }
        None => {
            // let's for now assume a trailing newline in the input stream...
            transform(std::io::stdout(), std::io::stdin(), shift);
        }
    };
}



macro_rules! reftest {
    ($plain: expr, $cipher: expr, 13) => ({
        let mut cipher = Vec::new();
        let mut cipher1 = Vec::new();
        transform(&mut cipher, $plain.as_bytes(), 13);
        assert_eq!($cipher, std::str::from_utf8(&cipher).unwrap());

        transform(&mut cipher1, &cipher[..], 13);
        assert_eq!($plain, std::str::from_utf8(&cipher1).unwrap());
    });
    ($plain: expr, $cipher: expr, $shift: expr) => ({
        let mut cipher = Vec::new();
        transform(&mut cipher, $plain.as_bytes(), $shift);
        assert_eq!($cipher, std::str::from_utf8(&cipher).unwrap());
    })
}

#[test]
fn alphabet() {
    reftest!("ABCDEFGHIJKLMNOPQRSTUVWXYZ", "XYZABCDEFGHIJKLMNOPQRSTUVW", 23);
    reftest!(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        "NOPQRSTUVWXYZABCDEFGHIJKLMnopqrstuvwxyzabcdefghijklm", 13
    );
}

#[test]
fn storytime() {
    reftest!("THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG", "QEB NRFZH YOLTK CLU GRJMP LSBO QEB IXWV ALD", -3);
    reftest!("defend the east wall of the castle", "efgfoe uif fbtu xbmm pg uif dbtumf", 1);
}