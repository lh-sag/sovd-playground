// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use sovd_diagnostic::{DataCategory, DiagnosticBuilder, ValueMetaData};

use crate::components::{DataEntry, MockComponent, MockDataService};

pub fn create_mock_components(builder: DiagnosticBuilder) -> DiagnosticBuilder {
    let engine = Arc::new(create_engine_component());
    let transmission = Arc::new(create_transmission_component());
    let hydraulics = Arc::new(create_hydraulics_component());

    let engine_data = Arc::new(create_engine_data_service());
    let transmission_data = Arc::new(create_transmission_data_service());
    let hydraulics_data = Arc::new(create_hydraulics_data_service());

    builder
        .with_entity(engine, |ctx| {
            ctx.with_service(engine_data as Arc<dyn sovd_diagnostic::DataService>)
        })
        .with_entity(transmission, |ctx| {
            ctx.with_service(transmission_data as Arc<dyn sovd_diagnostic::DataService>)
        })
        .with_entity(hydraulics, |ctx| {
            ctx.with_service(hydraulics_data as Arc<dyn sovd_diagnostic::DataService>)
        })
}

fn create_engine_component() -> MockComponent {
    MockComponent::new(
        "engine".to_string(),
        "Engine Control Unit".to_string(),
        vec!["OBD".to_string(), "powertrain".to_string()],
    )
}

fn create_engine_data_service() -> MockDataService {
    let mut data = HashMap::new();

    data.insert(
        "temperature".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "temperature".to_string(),
                name: "Engine Temperature".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["engine".to_string(), "temperature".to_string()],
                tags: vec!["celsius".to_string()],
            },
            value: json!({"value": 90.5}),
            writable: true,
        },
    );

    data.insert(
        "rpm".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "rpm".to_string(),
                name: "Engine RPM".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["engine".to_string(), "performance".to_string()],
                tags: vec!["rpm".to_string()],
            },
            value: json!({"value": 2500}),
            writable: true,
        },
    );

    data.insert(
        "oil_pressure".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "oil_pressure".to_string(),
                name: "Oil Pressure".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["engine".to_string(), "fluids".to_string()],
                tags: vec!["pressure".to_string()],
            },
            value: json!({"value": 45}),
            writable: true,
        },
    );

    data.insert(
        "serial_number".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "serial_number".to_string(),
                name: "Engine Serial Number".to_string(),
                translation_id: None,
                category: DataCategory::IdentData,
                groups: vec!["engine".to_string(), "identification".to_string()],
                tags: vec!["serial".to_string()],
            },
            value: json!("ENG-2024-001"),
            writable: false,
        },
    );

    data.insert(
        "turbo_boost".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "turbo_boost".to_string(),
                name: "Turbocharger Boost Pressure".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-liebherr-engine".to_string()),
                groups: vec!["engine".to_string(), "turbo".to_string()],
                tags: vec!["pressure".to_string(), "boost".to_string()],
            },
            value: json!({"value": 18.5, "max": 25.0}),
            writable: true,
        },
    );

    data.insert(
        "def_level".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "def_level".to_string(),
                name: "Diesel Exhaust Fluid Level".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-liebherr-aftertreatment".to_string()),
                groups: vec!["engine".to_string(), "aftertreatment".to_string()],
                tags: vec!["def".to_string(), "fluid".to_string()],
            },
            value: json!({"level": 75}),
            writable: true,
        },
    );

    MockDataService::new(data)
}

fn create_transmission_component() -> MockComponent {
    MockComponent::new(
        "transmission".to_string(),
        "Transmission".to_string(),
        vec!["drivetrain".to_string()],
    )
}

fn create_transmission_data_service() -> MockDataService {
    let mut data = HashMap::new();

    data.insert(
        "gear".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "gear".to_string(),
                name: "Current Gear".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["transmission".to_string(), "gearing".to_string()],
                tags: vec!["gear".to_string()],
            },
            value: json!({"current": 3, "max": 6}),
            writable: true,
        },
    );

    data.insert(
        "fluid_temp".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "fluid_temp".to_string(),
                name: "Transmission Fluid Temperature".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["transmission".to_string(), "fluids".to_string()],
                tags: vec!["temperature".to_string()],
            },
            value: json!({"value": 75}),
            writable: true,
        },
    );

    data.insert(
        "shift_mode".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "shift_mode".to_string(),
                name: "Shift Mode".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["transmission".to_string(), "control".to_string()],
                tags: vec!["mode".to_string()],
            },
            value: json!("automatic"),
            writable: true,
        },
    );

    data.insert(
        "part_number".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "part_number".to_string(),
                name: "Transmission Part Number".to_string(),
                translation_id: None,
                category: DataCategory::IdentData,
                groups: vec!["transmission".to_string(), "identification".to_string()],
                tags: vec!["part".to_string()],
            },
            value: json!("TRANS-X900-2024"),
            writable: false,
        },
    );

    data.insert(
        "torque_converter_lockup".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "torque_converter_lockup".to_string(),
                name: "Torque Converter Lockup Status".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-allison-transmission".to_string()),
                groups: vec!["transmission".to_string(), "torque-converter".to_string()],
                tags: vec!["lockup".to_string(), "status".to_string()],
            },
            value: json!({"locked": true, "slip_rpm": 25}),
            writable: true,
        },
    );

    MockDataService::new(data)
}

fn create_hydraulics_component() -> MockComponent {
    MockComponent::new(
        "hydraulics".to_string(),
        "Hydraulic Control System".to_string(),
        vec!["hydraulics".to_string()],
    )
}

fn create_hydraulics_data_service() -> MockDataService {
    let mut data = HashMap::new();

    data.insert(
        "main_pressure".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "main_pressure".to_string(),
                name: "Main Hydraulic System Pressure".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-liebherr-hydraulics".to_string()),
                groups: vec!["hydraulics".to_string(), "pressure".to_string()],
                tags: vec!["main".to_string(), "system".to_string()],
            },
            value: json!({"value": 350, "max": 420}),
            writable: true,
        },
    );

    data.insert(
        "pilot_pressure".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "pilot_pressure".to_string(),
                name: "Pilot Control Pressure".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-liebherr-hydraulics".to_string()),
                groups: vec!["hydraulics".to_string(), "pilot".to_string()],
                tags: vec!["pilot".to_string(), "control".to_string()],
            },
            value: json!({"value": 28, "target": 30}),
            writable: true,
        },
    );

    data.insert(
        "boom_position".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "boom_position".to_string(),
                name: "Boom Cylinder Position".to_string(),
                translation_id: None,
                category: DataCategory::Vendor("x-liebherr-construction".to_string()),
                groups: vec!["hydraulics".to_string(), "boom".to_string()],
                tags: vec!["position".to_string(), "cylinder".to_string()],
            },
            value: json!({"extension": 65}),
            writable: true,
        },
    );

    data.insert(
        "fluid_temperature".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "fluid_temperature".to_string(),
                name: "Hydraulic Fluid Temperature".to_string(),
                translation_id: None,
                category: DataCategory::CurrentData,
                groups: vec!["hydraulics".to_string(), "fluids".to_string()],
                tags: vec!["temperature".to_string()],
            },
            value: json!({"value": 45, "warning_threshold": 80}),
            writable: true,
        },
    );

    data.insert(
        "pump_model".to_string(),
        DataEntry {
            metadata: ValueMetaData {
                id: "pump_model".to_string(),
                name: "Hydraulic Pump Model".to_string(),
                translation_id: None,
                category: DataCategory::IdentData,
                groups: vec!["hydraulics".to_string(), "pump".to_string()],
                tags: vec!["model".to_string(), "identification".to_string()],
            },
            value: json!("LH-PUMP-350-V2"),
            writable: false,
        },
    );

    MockDataService::new(data)
}
