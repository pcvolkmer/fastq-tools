mod cli;
mod fastq;
mod metadata_file;

use crate::cli::{Args, Command};
use crate::fastq::{Header, Pair};
use crate::metadata_file::MetadataFile;
use clap::Parser;
use console::Style;
use flate2::read::GzDecoder;
use itertools::Itertools;
use regex::Regex;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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

    let input_file = args.input_file;

    match &args.command {
        Command::Info => match input_reader(input_file, args.decompress) {
            Ok(input) => info(input),
            Err(err) => {
                eprintln!(
                    "{}\n",
                    Style::new().bold().red().apply_to(format!("🔥 {err}"))
                );
            }
        },
        Command::GrzMetadata => match input_file {
            Some(input_file) => {
                let file_metadata = match MetadataFile::read_file(input_file, args.decompress) {
                    Ok(file_metadata) => file_metadata,
                    Err(err) => {
                        eprintln!(
                            "{}\n",
                            Style::new().bold().red().apply_to(format!("🔥 {err}"))
                        );
                        return;
                    }
                };

                println!(
                    "{}\n",
                    serde_json::to_string_pretty(&file_metadata).unwrap()
                );
            }
            None => eprintln!(
                "{}\n",
                Style::new().bold().red().apply_to("🔥 No input file!")
            ),
        },
        Command::Scramble => match input_reader(input_file, args.decompress) {
            Ok(input) => scramble(input),
            Err(err) => {
                eprintln!(
                    "{}\n",
                    Style::new().bold().red().apply_to(format!("🔥 {err}"))
                );
            }
        },
    }
}

fn input_reader(input_file: Option<PathBuf>, decompress: bool) -> Result<Box<dyn BufRead>, String> {
    let input: Box<dyn BufRead> = match input_file {
        Some(input_file) => {
            let file = match File::open(input_file) {
                Ok(file) => file,
                _ => {
                    return Err("Cannot open input file".to_string());
                }
            };
            Box::new(BufReader::new(file))
        }
        _ => Box::new(BufReader::new(std::io::stdin())),
    };

    let input: Box<dyn BufRead> = if decompress {
        let gz_decoder = GzDecoder::new(input);
        Box::new(BufReader::new(gz_decoder))
    } else {
        Box::new(input)
    };

    Ok(input)
}

fn scramble(mut reader: impl BufRead) {
    let mut buf = String::new();

    let mut line = 1;
    while let Ok(n) = reader.read_line(&mut buf) {
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

fn info(mut reader: impl BufRead) {
    let mut buf = String::new();

    let mut headers = vec![];
    let mut read_lens = vec![];
    let mut quality_lens = vec![];

    let headline_style = Style::new().bold();
    let info_style = Style::new().bold().blue();
    let error_style = Style::new().bold().red();

    let mut line = 1;
    while let Ok(n) = reader.read_line(&mut buf) {
        if n == 0 {
            break;
        }

        if buf.starts_with("@") {
            if let Ok(header) = buf.parse::<Header>() {
                headers.push(header)
            } else {
                println!(
                    "{}",
                    error_style.apply_to(format!("🔥 Invalid header at line {}", line))
                );
            }
        } else if buf.starts_with("+") {
            // ignore optional description
        } else if line % 4 == 0 {
            // check if quality values differs from sequence values
            if Some(&buf.trim().len()) != read_lens.last() {
                println!(
                    "{}",
                    error_style
                        .apply_to(format!("🔥 Invalid quality string length at line {}", line))
                );
                return;
            }
            quality_lens.push(buf.trim().len());
        } else if line % 4 == 2 {
            read_lens.push(buf.trim().len());
        }

        line += 1;
        buf.clear();
    }

    if line == 1 {
        println!("{}", error_style.apply_to("🔥 No valid input"));
        return;
    }

    if line % 4 != 1 {
        println!(
            "{}",
            error_style.apply_to("🔥 File contains invalid or incomplete sequences")
        );
        return;
    }

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to(format!("Found {} complete sequence sets", headers.len()))
    );

    fn grouped_count<T>(it: impl Iterator<Item = T>) -> String
    where
        T: Display + Ord,
    {
        it.sorted()
            .chunk_by(|value| value.to_string())
            .into_iter()
            .map(|g| format!("   {} ({})", g.0, g.1.count()))
            .collect::<Vec<String>>()
            .join("\n")
    }

    // Instruments

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to("Unique instrument name(s):")
    );
    println!(
        "{}",
        grouped_count(headers.iter().map(|header| header.instrument_name()))
    );

    // Flowcell IDs

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to("Flowcell ID(s):")
    );
    println!(
        "{}",
        grouped_count(headers.iter().filter_map(|header| header.flowcell_id()))
    );

    // Flowcell Lanes

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to("Flowcell lane(s):")
    );
    println!(
        "{}",
        grouped_count(headers.iter().map(|header| header.flowcell_lane()))
    );

    // Read Orders

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to("Read order(s):")
    );
    println!(
        "{}",
        grouped_count(headers.iter().map(|header| match header.pair_member() {
            Pair::PairedEnd => "R1",
            Pair::MatePair => "R2",
        }))
    );

    // Read Lengths

    println!(
        "{} {}",
        info_style.apply_to("🛈 "),
        headline_style.apply_to("Read length(s):")
    );
    println!("{}", grouped_count(read_lens.iter()));
}
