# FASTQ tools

Application to show information about and scramble FASTQ files to provide non-sensitive data for development purposes

## Usage

This application provides the following subcommands

### Info

To show information about compressed FASTQ files use:

```shell
cat file_fastq.gz | gzip -d | fastq-tools info
```

This will result in output like

![Info subcommand](docs/info_subcommand.jpg)

### Scramble

To scramble compressed FASTQ files use:

```shell
cat file_fastq.gz | gzip -d | fastq-tools scramble | gzip > scrambled_fastq.gz
```

This will scramble headers and sequences and write the output into `scrambled_fastq.gz`.