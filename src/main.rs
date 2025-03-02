use std::{fmt::Display, str::FromStr};

use regex::Regex;

struct Header {
    instrument_name: String,
    run_id: u32,
    flowcell_id: String,
    flowcell_lane: u32,
    tile_number: u32,
    x: u32,
    y: u32,
    pair_member: Pair,
    filtered: Filtered,
    control_bits: u32,
    index_sequence: String,
}

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

impl Header {
    fn scramble(self) -> Self {
        fn number(value: u32) -> u32 {
            value % 3 + value % 17 + value % 271 + value % 911
        }

        fn string(value: &str) -> String {
            value
                .chars()
                .map(|c| (((c as u8 % 3 * c as u8 % 17) % 26) + 0x41) as char)
                .collect::<String>()
        }

        fn string_sum(value: &str) -> u8 {
            ((value.len() as u8) + value.chars().map(|c| c as u8 & 2).sum::<u8>()) % 97
        }

        Header {
            instrument_name: format!("TEST{:0<2}", (string_sum(&self.instrument_name) * 17) % 97),
            run_id: number(self.run_id),
            flowcell_id: string(&self.flowcell_id),
            flowcell_lane: number(self.flowcell_lane),
            tile_number: number(self.tile_number),
            x: self.x + string_sum(&self.instrument_name) as u32,
            y: self.y + string_sum(&self.instrument_name) as u32,
            pair_member: self.pair_member,
            filtered: self.filtered,
            control_bits: self.control_bits,
            index_sequence: scramble_sequence(&self.index_sequence, 1),
        }
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "@{}:{}:{}:{}:{}:{}:{} {}:{}:{}:{}",
            self.instrument_name,
            self.run_id,
            self.flowcell_id,
            self.flowcell_lane,
            self.tile_number,
            self.x,
            self.y,
            match self.pair_member {
                Pair::PairedEnd => "1",
                Pair::MatePair => "2",
            },
            match self.filtered {
                Filtered::Y => "Y",
                Filtered::N => "N",
            },
            self.control_bits,
            self.index_sequence
        )
    }
}

impl FromStr for Header {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("@") {
            return Err("Invalid Casava 1.8+ header".to_string());
        }

        let parts = s
            .split(" ")
            .flat_map(|main_part| main_part.split(":").collect::<Vec<_>>())
            .collect::<Vec<_>>();
        if parts.len() != 11 {
            return Err("Invalid Casava 1.8+ header".to_string());
        }

        Ok(Header {
            instrument_name: parts[0][1..].to_string(),
            run_id: parts[1]
                .parse()
                .expect("Valid Casava 1.8+ header: Number value required"),
            flowcell_id: parts[2].into(),
            flowcell_lane: parts[3]
                .parse()
                .expect("Valid Casava 1.8+ header: Number value required"),
            tile_number: parts[4]
                .parse()
                .expect("Valid Casava 1.8+ header: Number value required"),
            x: parts[5]
                .parse()
                .expect("Valid Casava 1.8+ header: Number value required"),
            y: parts[6]
                .parse()
                .expect("Valid Casava 1.8+ header: Number value required"),
            pair_member: match parts[7] {
                "1" => Pair::PairedEnd,
                "2" => Pair::MatePair,
                _ => return Err("Invalid Casava 1.8+ header".to_string()),
            },
            filtered: match parts[8] {
                "Y" => Filtered::Y,
                "N" => Filtered::N,
                _ => return Err("Invalid Casava 1.8+ header".to_string()),
            },
            control_bits: if parts[9]
                .parse::<u32>()
                .expect("Valid Casava 1.8+ header: Even value for control bits required")
                % 2
                == 0
            {
                parts[9].parse().expect("Number")
            } else {
                return Err("Invalid Casava 1.8+ header".to_string());
            },
            index_sequence: parts[10].into(),
        })
    }
}

#[derive(Debug, PartialEq)]
enum Pair {
    PairedEnd = 1,
    MatePair = 2,
}

#[derive(Debug, PartialEq)]
enum Filtered {
    Y,
    N,
}

fn main() {
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

    println!()
}

#[cfg(test)]
mod tests {
    use crate::{scramble_sequence, Filtered, Header, Pair};

    #[test]
    fn should_return_parsed_header() {
        let given = "@EAS139:136:FC706VJ:2:2104:15343:197393 1:Y:18:ATCACG";
        let actual = given.parse::<Header>();

        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(actual.instrument_name, "EAS139");
        assert_eq!(actual.run_id, 136);
        assert_eq!(actual.flowcell_id, "FC706VJ");
        assert_eq!(actual.flowcell_lane, 2);
        assert_eq!(actual.tile_number, 2104);
        assert_eq!(actual.x, 15343);
        assert_eq!(actual.y, 197393);
        assert_eq!(actual.pair_member, Pair::PairedEnd);
        assert_eq!(actual.filtered, Filtered::Y);
        assert_eq!(actual.control_bits, 18);
        assert_eq!(actual.index_sequence, "ATCACG");
    }

    #[test]
    fn should_return_header_string() {
        let given = "@EAS139:136:FC706VJ:2:2104:15343:197393 1:Y:18:ATCACG";
        let actual = given.parse::<Header>();

        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(given, actual.to_string());
    }

    #[test]
    fn should_return_scrambled_header_string() {
        let given = "@EAS139:136:FC706VJ:2:2104:15343:197393 1:Y:18:ATCACG";
        let actual = given.parse::<Header>();
        let expected = "@TEST73:273:CQEAACM:8:503:15353:197403 1:Y:18:GAGCGC";

        assert!(actual.is_ok());

        let actual = actual.unwrap().scramble();
        assert_eq!(expected, actual.to_string().as_str());
    }

    #[test]
    fn should_return_scrambled_sequence_string_seed1() {
        let given = "GATTTGGGGTTCAAAGCAGTATCGATCAAATAGTAAATCCATTTGTTCAACTCACAGTTT";
        let actual = scramble_sequence(given, 1);
        let expected = "CGATCTGGCGCGCAGCGCCGGAGCGAGCAGAGCGTAGATGCATCCGCGCGGCGCGCCGTT";

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_return_scrambled_sequence_string_seed42() {
        let given = "GATTTGGGGTTCAAAGCAGTATCGATCAAATAGTAAATCCATTTGTTCAACTCACAGTTT";
        let actual = scramble_sequence(given, 42);
        let expected = "GTTTCTGGTTCGCAGCGCTCTCGCTCGCATCTTCTATCTGCTTCTTCGCCGCGCGCTTTA";

        assert_eq!(expected, actual);
    }
}
