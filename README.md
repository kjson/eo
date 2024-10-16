# eo
Command line tool to quickly edit files in object stores with `$EDITOR`.

## Usage

```bash
eo --help
A tool to edit files directly in cloud object storage

Usage: eo [OPTIONS] --storage <STORAGE> <--uri <URI>|--bucket <BUCKET>>

Options:
  -s, --storage <STORAGE>      Cloud storage provider (s3 for AWS S3, gcs for Google Cloud Storage) [default: s3]
  -b, --bucket <BUCKET>        Bucket name (mutually exclusive with --uri)
  -k, --key <KEY>              Object key (mutually exclusive with --uri)
  -u, --uri <URI>              Object URL (optional, mutually exclusive with --bucket and --key)
  -r, --region <REGION>        Cloud region (optional, defaults to environment config)
  -f, --file-path <FILE_PATH>  Local file path (optional, if you want to use your own temp file location)
  -h, --help                   Print help
  -V, --version                Print version
```

## Examples

```bash
# Edit a file in AWS S3
eo -s s3 -b mybucket -k mykey

# Edit a file in Google Cloud Storage
eo -s gcs -b mybucket -k mykey

# Edit a file in AWS S3 using a custom file path
eo -s s3 -b mybucket -k mykey -f /path/to/my/local/file

# Specify a URI and region
eo -s s3 -u s3://mybucket/mykey -r us-east-1
```

## Installation

```
cargo install --path .
```

## License

MIT
