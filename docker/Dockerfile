# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

FROM debian:bookworm-slim

ARG TARGETARCH

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /app/assets
COPY docker-build/sovd-gateway-${TARGETARCH} /app/sovd-gateway
RUN chmod +x /app/sovd-gateway
COPY assets/ /app/assets/
RUN useradd -m -u 1000 sovd && \
    chown -R sovd:sovd /app
USER sovd
WORKDIR /app
EXPOSE 9000
ENTRYPOINT ["/app/sovd-gateway"]
CMD ["--url", "http://0.0.0.0:9000/sovd"]
