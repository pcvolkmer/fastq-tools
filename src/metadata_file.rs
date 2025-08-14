use crate::fastq::{Header, Pair};
use crate::input_reader;
use crate::metadata_file::MetadataError::{CannotReadFile, ReadError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataFile {
    /// Type of checksum algorithm used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_type: Option<ChecksumType>,

    /// checksum of the file
    pub file_checksum: String,

    /// Path relative to the submission files directory, e.g.:
    /// 'patient_001/patient_001_dna.fastq.gz' if the file is located in <submission
    /// root>/files/patient_001/patient_001_dna.fastq.gz
    pub file_path: String,

    /// Size of the file in bytes
    pub file_size_in_bytes: u64,

    /// Type of the file; if BED file is submitted, only 1 file is allowed.
    pub file_type: FileType,

    /// Indicates the flow cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flowcell_id: Option<String>,

    /// Indicates the lane
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane_id: Option<String>,

    /// The read length; in the case of long-read sequencing it is the rounded average read
    /// length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_length: Option<i64>,

    /// Indicates the read order for paired-end reads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_order: Option<ReadOrder>,
}

/// Type of checksum algorithm used
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumType {
    Sha256,
}

/// Type of the file; if BED file is submitted, only 1 file is allowed.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Bam,

    Bed,

    Fastq,

    Vcf,
}

/// Indicates the read order for paired-end reads.
#[derive(Debug, Serialize, Deserialize)]
pub enum ReadOrder {
    R1,

    R2,
}

pub enum MetadataError {
    CannotReadFile,
    UnsupportedFile,
    ReadError(String),
}

impl Debug for MetadataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for MetadataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MetadataError::CannotReadFile => "Cannot read file".into(),
                MetadataError::UnsupportedFile => "Unsupported file type".into(),
                MetadataError::ReadError(err) => format!("Error reading file: {}", err),
            }
        )
    }
}

impl Error for MetadataError {}

impl MetadataFile {
    pub fn read_file(path: PathBuf, decompress: bool) -> Result<MetadataFile, MetadataError> {
        let path = match path.to_str() {
            Some(path) => path,
            None => return Err(MetadataError::CannotReadFile),
        };

        let file = File::open(path).map_err(|_| CannotReadFile)?;

        let file_type = if path.to_lowercase().ends_with(".bam") {
            FileType::Bam
        } else if path.to_lowercase().ends_with(".vcf") {
            FileType::Vcf
        } else if path.to_lowercase().ends_with(".bed") {
            FileType::Bed
        } else if path.to_lowercase().ends_with(".fastq")
            || path.to_lowercase().ends_with(".fastq.gz")
        {
            FileType::Fastq
        } else {
            return Err(MetadataError::UnsupportedFile);
        };

        let file_checksum = match fs::read(path) {
            Ok(content) => {
                let mut hasher = Sha256::new();
                hasher.update(content.as_slice());
                let hash = hasher.finalize();
                base16ct::lower::encode_string(&hash)
            }
            Err(_) => {
                return Err(CannotReadFile);
            }
        };

        if let FileType::Fastq = file_type {
            match input_reader(Some(PathBuf::from(path)), decompress) {
                Ok(input_reader) => {
                    let input_metadata = MetadataFile::read(input_reader)?;

                    Ok(MetadataFile {
                        file_type,
                        file_checksum,
                        checksum_type: Some(ChecksumType::Sha256),
                        file_size_in_bytes: file.metadata().map_err(|_| CannotReadFile)?.len(),
                        flowcell_id: input_metadata.flowcell_id,
                        read_order: input_metadata.read_order,
                        file_path: path.to_string(),
                        read_length: input_metadata.read_length,
                        lane_id: input_metadata.lane_id,
                    })
                }
                Err(err) => Err(ReadError(err.to_string())),
            }
        } else {
            Ok(MetadataFile {
                file_type,
                file_checksum,
                checksum_type: Some(ChecksumType::Sha256),
                file_size_in_bytes: file.metadata().map_err(|_| CannotReadFile)?.len(),
                flowcell_id: None,
                read_order: None,
                file_path: path.to_string(),
                read_length: None,
                lane_id: None,
            })
        }
    }

    fn read(mut reader: impl BufRead) -> Result<MetadataFile, MetadataError> {
        let mut buf = String::new();

        let mut headers = vec![];
        let mut read_lens = vec![];
        let mut quality_lens = vec![];

        let mut line = 1;
        while let Ok(n) = reader.read_line(&mut buf) {
            if n == 0 {
                break;
            }

            if buf.starts_with("@") {
                if let Ok(header) = buf.parse::<Header>() {
                    headers.push(header)
                } else {
                    return Err(ReadError(format!("Invalid header at line {}", line)));
                }
            } else if buf.starts_with("+") {
                // ignore optional description
            } else if line % 4 == 0 {
                // check if quality values differs from sequence values
                if Some(&buf.trim().len()) != read_lens.last() {
                    return Err(ReadError(format!(
                        "Invalid quality string length at line {}",
                        line
                    )));
                }
                quality_lens.push(buf.trim().len());
            } else if line % 4 == 2 {
                read_lens.push(buf.trim().len());
            }

            line += 1;
            buf.clear();
        }

        if line == 1 {
            return Err(ReadError("No valid input".to_string()));
        }

        if line % 4 != 1 {
            return Err(ReadError(
                "File contains invalid or incomplete sequences".to_string(),
            ));
        }

        // Flowcell IDs

        let flowcell_ids = headers
            .iter()
            .filter_map(|header| header.flowcell_id())
            .sorted()
            .chunk_by(|value| value.clone())
            .into_iter()
            .map(|g| g.0)
            .collect::<Vec<String>>();

        // Flowcell Lanes

        let flowcell_lanes = headers
            .iter()
            .map(|header| header.flowcell_lane())
            .sorted()
            .chunk_by(|value| value.to_string())
            .into_iter()
            .map(|g| g.0)
            .collect::<Vec<String>>();

        // Read Orders

        let read_orders = headers
            .iter()
            .map(|header| match header.pair_member() {
                Pair::PairedEnd => "R1",
                Pair::MatePair => "R2",
            })
            .sorted()
            .chunk_by(|value| value.to_string())
            .into_iter()
            .map(|g| g.0)
            .collect::<Vec<String>>();

        // Read Lengths

        let read_leans = read_lens
            .iter()
            .sorted()
            .chunk_by(|value| value.to_string())
            .into_iter()
            .map(|g| g.0.parse::<i64>().unwrap())
            .collect::<Vec<i64>>();

        Ok(MetadataFile {
            checksum_type: Some(ChecksumType::Sha256),
            file_checksum: String::new(),
            file_path: String::new(),
            file_size_in_bytes: 0,
            file_type: FileType::Fastq,
            flowcell_id: if flowcell_ids.len() == 1 {
                Some(flowcell_ids.into_iter().nth(0).unwrap())
            } else {
                return Err(ReadError("Cannot find single flowcell id".to_string()));
            },
            lane_id: if flowcell_lanes.len() == 1 {
                Some(flowcell_lanes.into_iter().nth(0).unwrap())
            } else {
                return Err(ReadError("Cannot find single lane id".to_string()));
            },
            read_length: if read_leans.len() == 1 {
                Some(read_leans.into_iter().nth(0).unwrap())
            } else {
                return Err(ReadError("Cannot find single lane id".to_string()));
            },
            read_order: if read_orders.len() == 1 {
                match read_orders.into_iter().nth(0) {
                    None => None,
                    Some(value) => match value.as_str() {
                        "R1" => Some(ReadOrder::R1),
                        "R2" => Some(ReadOrder::R2),
                        _ => None,
                    },
                }
            } else {
                return Err(ReadError("Cannot find single lane id".to_string()));
            },
        })
    }
}
