# SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
# SPDX-License-Identifier: Apache-2.0

FROM debian:bookworm-slim

ARG TARGETARCH

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /app/assets
COPY docker-build/sovd-gateway-${TARGETARCH} /app/sovd-gateway
COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/sovd-gateway /app/entrypoint.sh
COPY assets/ /app/assets/
RUN useradd -m -u 1000 sovd && \
    chown -R sovd:sovd /app
USER sovd
WORKDIR /app
EXPOSE 9000 9443
ENTRYPOINT ["/app/entrypoint.sh"]
