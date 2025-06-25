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

//! OpenSOVD tracing crate providing conditional tracing support.
//!
//! When the `tracing` feature is enabled, these macros expand to actual tracing calls.
//! When the feature is disabled, they compile to no-ops, eliminating all tracing overhead at compile time.

/// Trace-level logging macro that conditionally compiles based on the `tracing` feature.
///
/// # Examples
/// ```
/// use opensovd_tracing::trace;
/// trace!("This is a trace message");
/// trace!(target: "my_target", "Message with target");
/// trace!("Message with data: {}", 42);
/// ```
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)*) => {
        ::tracing::trace!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        ::tracing::trace!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        ()
    };
}

/// Debug-level logging macro that conditionally compiles based on the `tracing` feature.
///
/// # Examples
/// ```
/// use opensovd_tracing::debug;
/// debug!("This is a debug message");
/// debug!(target: "my_target", "Message with target");
/// debug!("Message with data: {}", 42);
/// ```
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)*) => {
        ::tracing::debug!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        ::tracing::debug!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        ()
    };
}

/// Info-level logging macro that conditionally compiles based on the `tracing` feature.
///
/// # Examples
/// ```
/// use opensovd_tracing::info;
/// info!("This is an info message");
/// info!(target: "my_target", "Message with target");
/// info!("Message with data: {}", 42);
/// ```
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)*) => {
        ::tracing::info!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        ::tracing::info!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        ()
    };
}

/// Warn-level logging macro that conditionally compiles based on the `tracing` feature.
///
/// # Examples
/// ```
/// use opensovd_tracing::warn;
/// warn!("This is a warning message");
/// warn!(target: "my_target", "Message with target");
/// warn!("Message with data: {}", 42);
/// ```
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)*) => {
        ::tracing::warn!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        ::tracing::warn!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ()
    };
}

/// Error-level logging macro that conditionally compiles based on the `tracing` feature.
///
/// # Examples
/// ```
/// use opensovd_tracing::error;
/// error!("This is an error message");
/// error!(target: "my_target", "Message with target");
/// error!("Message with data: {}", 42);
/// ```
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)*) => {
        ::tracing::error!(target: $target, $($arg)*)
    };
    ($($arg:tt)*) => {
        ::tracing::error!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macros_compile() {
        // These should compile without errors regardless of feature flags
        trace!("test trace");
        debug!("test debug");
        info!("test info");
        warn!("test warn");
        error!("test error");
    }
}
