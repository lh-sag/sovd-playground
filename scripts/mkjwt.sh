#!/bin/bash
# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

# JWT RSA key generation and testing script for sovd-gateway authentication
# Usage: ./mkjwt.sh [output_dir]

set -e

# Default parameters
OUTPUT_DIR="${1:-./gen/jwt}"


mkdir -p "$OUTPUT_DIR"
cd "$OUTPUT_DIR"

echo "Generating JWT keys in: $OUTPUT_DIR"

openssl genrsa -out private_key.pem 2048
openssl rsa -in private_key.pem -pubout -out public_key.pem

chmod 600 private_key.pem
chmod 644 public_key.pem

header='{"alg":"RS256","typ":"JWT"}'
header_b64=$(echo -n "$header" | base64 -w 0 | tr '+/' '-_' | tr -d '=')

now=$(date +%s)

# Valid token (expires in 1 hour)
exp=$((now + 3600))
payload="{\"sub\":\"test-user\",\"aud\":\"sovd\",\"iss\":\"sovd-test\",\"exp\":$exp,\"iat\":$now,\"scope\":[\"read\",\"write\"],\"roles\":[\"admin\",\"user\"]}"
payload_b64=$(echo -n "$payload" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
header_payload="$header_b64.$payload_b64"
signature=$(echo -n "$header_payload" | openssl dgst -sha256 -sign private_key.pem | base64 -w 0 | tr '+/' '-_' | tr -d '=')
echo "$header_payload.$signature" > token.jwt

KEY_DIR="$(pwd)"
URL="http://127.0.0.1:9000/sovd"
echo ""
echo "Done! Start server:"
echo "cargo run --bin sovd-gateway -- --url \"$URL\" --auth-jwt '$KEY_DIR/public_key.pem'"
echo ""
echo "Test unprotected: curl \"$URL/version-info\""
echo "Test protected:   curl -H 'Authorization: Bearer \$(cat $KEY_DIR/token.jwt)' -H 'Content-Type: application/json' \"$URL/v1/components\""
