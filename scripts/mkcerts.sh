#!/bin/bash
#
# Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
#

# Certificate generation and testing script for osovd-gateway HTTPS with client certificates
# Usage: ./mkcerts.sh [output_dir] [validity_days] [--no-verify]

set -e

# Default parameters
OUTPUT_DIR="${1:-./gen/certs}"
VALIDITY_DAYS="${2:-3650}"  # Default 10 years
NO_VERIFY=false
URL="https://127.0.0.1:8443/opensovd"

# Parse arguments
for arg in "$@"; do
    case $arg in
        --no-verify)
            NO_VERIFY=true
            shift
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"
cd "$OUTPUT_DIR"

echo "Generating certificates ($VALIDITY_DAYS days) in: $OUTPUT_DIR"

# CA
openssl genrsa -out ca-key.pem 4096
openssl req -new -x509 -days "$VALIDITY_DAYS" -key ca-key.pem -sha256 -out ca-cert.pem -subj "/C=DE/O=OpenSOVD/CN=OpenSOVD CA" -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,digitalSignature,keyCertSign,cRLSign"

# Server
openssl genrsa -out server-key.pem 4096
openssl req -subj "/C=DE/O=OpenSOVD/CN=127.0.0.1" -sha256 -new -key server-key.pem -out server.csr
openssl x509 -req -days "$VALIDITY_DAYS" -in server.csr -CA ca-cert.pem -CAkey ca-key.pem -out server-cert.pem -CAcreateserial -extfile <(echo "basicConstraints=CA:FALSE"; echo "keyUsage=critical,digitalSignature,keyEncipherment"; echo "extendedKeyUsage=serverAuth"; echo "subjectAltName=IP:127.0.0.1,DNS:localhost")

# Client
openssl genrsa -out client-key.pem 4096
openssl req -subj "/C=DE/O=OpenSOVD/CN=test-client" -new -key client-key.pem -out client.csr
openssl x509 -req -days "$VALIDITY_DAYS" -in client.csr -CA ca-cert.pem -CAkey ca-key.pem -out client-cert.pem -CAcreateserial -extfile <(echo "basicConstraints=CA:FALSE"; echo "keyUsage=critical,digitalSignature,keyEncipherment"; echo "extendedKeyUsage=clientAuth")

rm ./*.csr

chmod 600 ./*-key.pem
echo "Certificates generated"

# Get absolute path for commands
CERT_DIR="$(pwd)"

# Exit early if --no-verify is specified
if [ "$NO_VERIFY" = true ]; then
    exit 0
fi

# Test
echo "Testing..."

# Capture cargo output to show only on failure
cargo_log="/tmp/cargo_output_$$"
cargo run -q --bin osovd-gateway --features "openssl" -- \
    --url "$URL" \
    --cert "$CERT_DIR/server-cert.pem" \
    --key "$CERT_DIR/server-key.pem" \
    --cacert "$CERT_DIR/ca-cert.pem" >"$cargo_log" 2>&1 &

SERVER_PID=$!
sleep 5

# Check if server is running
if kill -0 $SERVER_PID 2>/dev/null; then
    if curl -s --max-time 5 --cacert "$CERT_DIR/ca-cert.pem" --cert "$CERT_DIR/client-cert.pem" --key "$CERT_DIR/client-key.pem" "$URL" >/dev/null 2>&1; then
        echo "HTTPS with client cert working"
    else
        echo "HTTPS with client cert failed"
    fi
else
    echo "Server failed to start:"
    cat "$cargo_log"
fi

# Clean up log file
rm -f "$cargo_log"

kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "Done! Start server:"
echo "cargo run --bin osovd-gateway --features \"openssl,jsonschema-schemars\" -- --url \"$URL\" --cert '$CERT_DIR/server-cert.pem' --key '$CERT_DIR/server-key.pem' --cacert '$CERT_DIR/ca-cert.pem'"
echo ""
echo "Test: curl --cacert '$CERT_DIR/ca-cert.pem' --cert '$CERT_DIR/client-cert.pem' --key '$CERT_DIR/client-key.pem' \"$URL\""
