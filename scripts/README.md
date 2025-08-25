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

## `update-assets.sh`

Update HTML5 UI assets to latest versions.

```bash
./update-assets.sh [assets_dir]
```
