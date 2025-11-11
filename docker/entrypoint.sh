#!/bin/bash
# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

set -e

# If arguments provided, pass them directly to sovd-gateway
if [ $# -gt 0 ]; then
    exec /app/sovd-gateway "$@"
fi

CONFIG="/tmp/config.toml"

cat > "$CONFIG" << 'EOF'
# HTTP server
[[servers]]
name = "HTTP"
listen = "0.0.0.0:9000"
base = "/sovd"

EOF

# HTTPS with mTLS (if certs exist)
if [ -f "/gen/certs/server-cert.pem" ] && [ -f "/gen/certs/server-key.pem" ] && [ -f "/gen/certs/ca-cert.pem" ]; then
    cat >> "$CONFIG" << 'EOF'
# HTTPS with mTLS
[[servers]]
name = "HTTPS mTLS"
listen = "0.0.0.0:9443"
base = "/secure"
EOF

    # Add JWT auth if key exists
    if [ -f "/gen/jwt/public_key.pem" ]; then
        cat >> "$CONFIG" << 'EOF'
[servers.auth]
jwt_public_key_path = "/gen/jwt/public_key.pem"
EOF
    fi

    cat >> "$CONFIG" << 'EOF'
[servers.ssl]
cert = "/gen/certs/server-cert.pem"
key = "/gen/certs/server-key.pem"
cacert = "/gen/certs/ca-cert.pem"

EOF
fi

# Unix socket
cat >> "$CONFIG" << 'EOF'
# Unix socket
[[servers]]
name = "Unix Socket"
listen = "unix://tmp/sovd.sock"
base = "/private"
EOF

exec /app/sovd-gateway --config "$CONFIG"
