mod cli;
mod fastq;

use crate::cli::{Args, Command};
use crate::fastq::Header;
use clap::Parser;
use regex::Regex;

fn scramble_sequence(value: &str, seed: u32) -> String {
    let ahead_1 = Regex::new(r"T([ACG])").unwrap();
    let ahead_2 = Regex::new(r"A([CGT])").unwrap();
    let ahead_3 = Regex::new(r"G([ACT])").unwrap();
    let ahead_4 = Regex::new(r"C([AGT])").unwrap();

    let mut chars = value.chars().collect::<Vec<_>>();

    (1..value.len()).for_each(|idx| {
        if idx % (1 + (seed % 3) as usize) == 0 && chars[idx] > chars[idx - 1] {
            chars.swap(idx, idx - 1);
        }
    });

    let mut result = chars.iter().collect::<String>();

    ahead_1.find_iter(value).for_each(|m| {
        if !m.is_empty() {
            result.replace_range(m.start()..m.end(), "CT")
        }
    });

    ahead_2.find_iter(value).for_each(|m| {
        if !m.is_empty() && seed % 2 != 0 {
            result.replace_range(m.start()..m.end(), "GA")
        }
    });

    ahead_3.find_iter(value).for_each(|m| {
        if !m.is_empty() && seed % 3 != 0 {
            result.replace_range(m.start()..m.end(), "CG")
        }
    });

    ahead_4.find_iter(value).for_each(|m| {
        if !m.is_empty() && seed % 5 != 0 {
            result.replace_range(m.start()..m.end(), "GC")
        }
    });

    result.to_string()
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Command::Info => {
            println!("Not implemented yet");
        }
        Command::Scramble => scramble(),
    }

    println!()
}

fn scramble() {
    let stdin = std::io::stdin();
    let mut buf = String::new();

    let mut line = 1;

    while let Ok(n) = stdin.read_line(&mut buf) {
        if n == 0 {
            break;
        }

        if buf.starts_with("@") {
            print!("{}", buf.parse::<Header>().unwrap().scramble())
        } else if buf.starts_with("+") {
            println!("+")
        } else if line % 4 == 0 {
            print!("{buf}")
        } else if line % 4 == 2 {
            print!("{}", scramble_sequence(&buf, line % 97))
        }

        line += 1;
        buf.clear();
    }
}
