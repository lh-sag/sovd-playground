// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0
//

#![cfg(feature = "metrics")]

use once_cell::sync::Lazy;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, 
    HistogramTimer, HistogramVec, IntCounterVec, IntGaugeVec,
};

/// Counter for total lock acquisitions
static LOCK_ACQUISITIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "opensovd_lock_acquisitions_total",
        "Total number of lock acquisitions",
        &["resource_type", "lock_type"]
    )
    .expect("Failed to create lock acquisitions counter")
});

/// Histogram for lock acquisition duration
static LOCK_ACQUISITION_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "opensovd_lock_acquisition_duration_seconds",
        "Duration of lock acquisition in seconds",
        &["resource_type", "lock_type"]
    )
    .expect("Failed to create lock acquisition duration histogram")
});


/// Gauge for currently held locks
static ACTIVE_LOCKS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "opensovd_active_locks",
        "Number of currently held locks",
        &["resource_type", "lock_type"]
    )
    .expect("Failed to create active locks gauge")
});

/// Timer guard that records duration and decrements active locks on drop
pub struct LockTimerGuard {
    _timer: HistogramTimer,  // Automatically records duration on drop
    resource_type: &'static str,
    lock_type: &'static str,
}

impl Drop for LockTimerGuard {
    fn drop(&mut self) {
        // The timer records duration automatically when dropped
        // Decrement active locks
        ACTIVE_LOCKS
            .with_label_values(&[self.resource_type, self.lock_type])
            .dec();
    }
}

/// Record the start of a lock acquisition attempt
pub fn record_lock_acquisition(resource_type: &'static str, lock_type: &'static str) -> LockTimerGuard {
    // Increment acquisition counter
    LOCK_ACQUISITIONS
        .with_label_values(&[resource_type, lock_type])
        .inc();
    
    // Increment active locks
    ACTIVE_LOCKS
        .with_label_values(&[resource_type, lock_type])
        .inc();
    
    // Start timer
    let timer = LOCK_ACQUISITION_DURATION
        .with_label_values(&[resource_type, lock_type])
        .start_timer();
    
    LockTimerGuard {
        _timer: timer,
        resource_type,
        lock_type,
    }
}


/// Get a Prometheus text format representation of all metrics
pub fn metrics_to_string() -> String {
    use prometheus::{Encoder, TextEncoder};
    
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");
    
    String::from_utf8(buffer).expect("Failed to convert metrics to string")
}