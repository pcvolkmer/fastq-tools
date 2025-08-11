# FASTQ tools

Application to show information about and scramble FASTQ files to provide non-sensitive data for development purposes

## Usage

This application provides the following subcommands

### Scramble

To scramble compressed FASTQ files use:

```shell
cat file_fastq.gz | gzip -d | fastq-tools scramble | gzip > scrambled_fastq.gz
```