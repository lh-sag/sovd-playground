//
// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
//
use clap::Command;
use opensovd_tracing::info;

fn main() -> std::io::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt().compact().init();

    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

    info!(version = VERSION, "OpenSOVD CLI client");

    let _matches = Command::new(env!("CARGO_BIN_NAME"))
        .about("OpenSOVD cli client")
        .version(VERSION)
        .get_matches();

    info!("OpenSOVD CLI client finished successfully");
    Ok(())
}
