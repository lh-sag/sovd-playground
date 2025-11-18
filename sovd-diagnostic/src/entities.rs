// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

/// Base trait for all SOVD entities
pub trait Entity: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn tags(&self) -> &[String];
    fn translation_id(&self) -> Option<&str> {
        None
    }
}

/// Special entity representing the SOVD server root
pub struct SovdServer {
    name: String,
}

impl SovdServer {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Entity for SovdServer {
    fn id(&self) -> &str {
        // Empty ID for root per ISO 17978-3
        ""
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn tags(&self) -> &[String] {
        &[]
    }

    fn translation_id(&self) -> Option<&str> {
        None
    }
}
