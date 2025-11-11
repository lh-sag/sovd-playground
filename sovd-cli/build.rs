// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
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
