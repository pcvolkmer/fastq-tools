# FASTQ scrambler

Application to scramble FASTQ files to provide non-sensitive data for development purposes

## Usage

To scramble compressed FASTQ files use:

```shell
cat file_fastq.gz | gz -d | fastq-scrambler | gz > scrambled_fastq.gz
```