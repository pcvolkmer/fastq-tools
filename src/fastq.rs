use crate::scramble_sequence;
use std::fmt::Display;
use std::str::FromStr;

pub enum Header {
    Casava18(Casava18Header),
    Illumina(IlluminaHeader),
}

pub struct Casava18Header {
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

pub struct IlluminaHeader {
    instrument_name: String,
    flowcell_lane: u32,
    tile_number: u32,
    x: u32,
    y: u32,
    index_number: String,
    pair_member: Pair,
}

impl Header {
    pub fn scramble(self) -> Self {
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

        match self {
            Header::Casava18(header) => Header::Casava18(Casava18Header {
                instrument_name: format!(
                    "TEST{:0<2}",
                    (string_sum(&header.instrument_name) * 17) % 97
                ),
                run_id: number(header.run_id),
                flowcell_id: string(&header.flowcell_id),
                flowcell_lane: number(header.flowcell_lane),
                tile_number: number(header.tile_number),
                x: header.x + string_sum(&header.instrument_name) as u32,
                y: header.y + string_sum(&header.instrument_name) as u32,
                pair_member: header.pair_member,
                filtered: header.filtered,
                control_bits: header.control_bits,
                index_sequence: scramble_sequence(&header.index_sequence, 1),
            }),
            Header::Illumina(header) => Header::Illumina(IlluminaHeader {
                instrument_name: format!(
                    "TEST{:0<2}",
                    (string_sum(&header.instrument_name) * 17) % 97
                ),
                flowcell_lane: number(header.flowcell_lane),
                tile_number: number(header.tile_number),
                x: header.x + string_sum(&header.instrument_name) as u32,
                y: header.y + string_sum(&header.instrument_name) as u32,
                index_number: header.index_number,
                pair_member: header.pair_member,
            }),
        }
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Header::Casava18(header) => {
                write!(
                    f,
                    "@{}:{}:{}:{}:{}:{}:{} {}:{}:{}:{}",
                    header.instrument_name,
                    header.run_id,
                    header.flowcell_id,
                    header.flowcell_lane,
                    header.tile_number,
                    header.x,
                    header.y,
                    match header.pair_member {
                        Pair::PairedEnd => "1",
                        Pair::MatePair => "2",
                    },
                    match header.filtered {
                        Filtered::Y => "Y",
                        Filtered::N => "N",
                    },
                    header.control_bits,
                    header.index_sequence
                )
            }
            Header::Illumina(header) => {
                write!(
                    f,
                    "@{}:{}:{}:{}:{}#{}/{}",
                    header.instrument_name,
                    header.flowcell_lane,
                    header.tile_number,
                    header.x,
                    header.y,
                    header.index_number,
                    match header.pair_member {
                        Pair::PairedEnd => "1",
                        Pair::MatePair => "2",
                    },
                )
            }
        }
    }
}

impl FromStr for Header {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("@") {
            return Err("Cannot parse FASTQ header".to_string());
        }

        let parts = s
            .split(" ")
            .flat_map(|s| s.split("#").collect::<Vec<_>>())
            .flat_map(|s| s.split("/").collect::<Vec<_>>())
            .flat_map(|main_part| main_part.split(":").collect::<Vec<_>>())
            .collect::<Vec<_>>();

        if parts.len() == 11 {
            return Ok(Header::Casava18(Casava18Header {
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
            }));
        } else if parts.len() == 7 {
            return Ok(Header::Illumina(IlluminaHeader {
                instrument_name: parts[0][1..].to_string(),
                flowcell_lane: parts[1]
                    .parse()
                    .expect("Valid Illumina header: Number value required"),
                tile_number: parts[2]
                    .parse()
                    .expect("Valid Illumina header: Number value required"),
                x: parts[3]
                    .parse()
                    .expect("Valid Illumina header: Number value required"),
                y: parts[4]
                    .parse()
                    .expect("Valid Illumina header: Number value required"),
                index_number: parts[5]
                    .parse()
                    .expect("Valid Illumina header: Number value required"),
                pair_member: match parts[6] {
                    "1" => Pair::PairedEnd,
                    "2" => Pair::MatePair,
                    _ => return Err("Invalid Illumina header".to_string()),
                },
            }));
        }

        Err("Cannot parse FASTQ header".to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum Pair {
    PairedEnd = 1,
    MatePair = 2,
}

#[derive(Debug, PartialEq)]
pub enum Filtered {
    Y,
    N,
}

#[cfg(test)]
mod tests {
    use crate::fastq::{Filtered, Pair};
    use crate::{scramble_sequence, Header};

    #[test]
    fn should_return_parsed_casava18_header() {
        let given = "@EAS139:136:FC706VJ:2:2104:15343:197393 1:Y:18:ATCACG";

        if let Ok(Header::Casava18(actual)) = given.parse::<Header>() {
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
        } else {
            panic!("Failed to parse FASTQ header");
        }
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

    #[test]
    fn should_return_parsed_illumna_header() {
        let given = "@HWUSI-EAS100R:6:73:941:1973#0/1";
        let actual = given.parse::<Header>();

        if let Ok(Header::Illumina(actual)) = actual {
            assert_eq!(actual.instrument_name, "HWUSI-EAS100R");
            assert_eq!(actual.flowcell_lane, 6);
            assert_eq!(actual.tile_number, 73);
            assert_eq!(actual.x, 941);
            assert_eq!(actual.y, 1973);
            assert_eq!(actual.index_number, "0");
            assert_eq!(actual.pair_member, Pair::PairedEnd);
        } else {
            panic!("Failed to parse FASTQ header");
        }
    }

    #[test]
    fn should_return_illumina_header_string() {
        let given = "@HWUSI-EAS100R:6:73:941:1973#0/1";
        let actual = given.parse::<Header>();

        assert!(actual.is_ok());

        let actual = actual.unwrap();
        assert_eq!(given, actual.to_string());
    }
}
