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

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../.git/logs/HEAD");
    if let Some(output) = Command::new("git")
        .args(["describe", "--dirty", "--always"])
        .output()
        .ok()
        .filter(|output| output.status.success())
    {
        let sha = String::from_utf8_lossy(&output.stdout);
        println!("cargo:rustc-env=COMMIT_SHA={}", sha.trim());
    }
}
