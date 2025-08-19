# Scripts

Utility scripts for the osovd-gateway project.

## `mkcerts.sh`

Generate SSL/TLS certificates for HTTPS testing.

```bash
./mkcerts.sh [output_dir] [validity_days] [--no-verify]
```

Creates CA, server, and client certificates with proper X.509 extensions.

## `check-labeler.sh`

Validate GitHub labeler configuration for CI.
