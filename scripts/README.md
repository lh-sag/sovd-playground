# Scripts

Utility scripts.

## `mkcerts.sh`

Generate SSL/TLS certificates for HTTPS testing.

```bash
./mkcerts.sh [output_dir] [validity_days] [--no-verify]
```

Creates CA, server, and client certificates with proper X.509 extensions.

## `check-labeler.sh`

Validate GitHub labeler configuration is up-to-date.

```bash
./check-labeler.sh
```

## `mkjwt.sh`

Generate JWT RSA keys and test tokens for authentication.

```bash
./mkjwt.sh [output_dir]
```

Creates RSA key pair and sample JWT tokens for testing sovd-gateway authentication.
