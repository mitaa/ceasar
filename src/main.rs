#![cfg_attr(test, feature(test))]

use std::io::{
    self,
    Read,
    Write,
};
use std::env;

use std::slice;


extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;


fn print_usage(program: &str) {
    println!(
"Usage: {} SHIFT [PLAINTEXT] [options]

Options:
    -h, --help      print this help menu", program);
}

fn transform<O: Write>(mut pipe: O, plaintext: &str, mut shift: i8) -> io::Result<usize> {
    while shift >= 26
        { shift -= 26; }
    while shift < 0
        { shift += 26; }
    debug_assert!(shift >= 0 && shift <= 26);

    let offset: u8 = shift as u8;
    // this will have to wait on `feature(stmt_expr_attributes)` (rust-lang/rust/pull/29850)
    // #[allow(unused_assignments)]
    let mut c = b'\0';

    for grapheme in plaintext.graphemes(true) {
        let bytes = if grapheme.len() == 1 {
            unsafe {
                let cres = *grapheme.as_bytes().get_unchecked(0) as char;
                match cres {
                    'a' ... 'z' => {
                        c = cres as u8 + offset;
                        if c > b'z'
                            { c -= 26; }
                    }
                    'A' ... 'Z' => {
                        c = cres as u8 + offset;
                        if c > b'Z'
                            { c -= 26 }
                    }
                    _ => c = cres as u8
                }
                slice::from_raw_parts(&c, 1)
            }
        } else {
            grapheme.as_bytes()
        };
        try!(pipe.write(bytes).map(|_|()));
    }
    Ok(0)
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

    let status = match args.get(2) {
        Some(input) => {
            let mut pipe = std::io::stdout();
            transform(&mut pipe, input, shift)
            .and(pipe.write(b"\n"))
        }
        None => {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input)
            .and(transform(std::io::stdout(), &input, shift))
        }
    };
    if let Err(f) = status {
        println!("broken pipe: {}", f.to_string());
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use self::test::Bencher;
    use super::{std, transform};
    use std::fs::File;
    use std::io::Read;

    macro_rules! reftest {
        ($plain: expr, $cipher: expr, 13) => ({
            let mut cipher = Vec::new();
            let mut cipher1 = Vec::new();
            transform(&mut cipher, $plain, 13);
            assert_eq!($cipher.as_bytes(), &cipher[..]);

            transform(&mut cipher1, std::str::from_utf8(&cipher).unwrap(), 13);
            assert_eq!($plain.as_bytes(), &cipher1[..]);
        });
        ($plain: expr, $cipher: expr, $shift: expr) => ({
            let mut cipher = Vec::new();
            transform(&mut cipher, $plain, $shift);
            assert_eq!($cipher.as_bytes(), &cipher[..]);
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

    #[bench]
    fn lorem_ipsum_stdout(b: &mut Bencher) {
        let mut input = String::new();
        let mut f = File::open("auxiliary/lorem_ipsum").unwrap();
        f.read_to_string(&mut input);

        b.iter(|| transform(std::io::stdout(), &input, 13));
    }

    #[bench]
    fn lorem_ipsum_string(b: &mut Bencher) {
        let mut input = String::new();
        let mut f = File::open("auxiliary/lorem_ipsum").unwrap();
        f.read_to_string(&mut input);

        b.iter(|| transform(&mut Vec::new(), &input, 13));
    }
}
