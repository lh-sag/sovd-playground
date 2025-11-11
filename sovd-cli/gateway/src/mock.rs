// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use serde_json::json;
use sovd_diagnostic::{DataCategory, Entity, ValueMetaData};

use crate::components::StaticComponent;

/// Create mock components for testing/demo purposes
pub fn create_mock_components() -> Vec<Arc<dyn Entity>> {
    vec![
        Arc::new(create_engine_component()),
        Arc::new(create_transmission_component()),
        Arc::new(create_hydraulics_component()),
    ]
}

fn create_engine_component() -> StaticComponent {
    let mut component = StaticComponent::new(
        "engine".to_string(),
        "Engine Control Unit".to_string(),
        vec!["OBD".to_string(), "powertrain".to_string()],
        None,
    );

    component.add_data_item(
        ValueMetaData {
            id: "temperature".to_string(),
            name: "Engine Temperature".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["engine".to_string(), "temperature".to_string()],
            tags: vec!["celsius".to_string()],
        },
        json!({"value": 90.5}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "rpm".to_string(),
            name: "Engine RPM".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["engine".to_string(), "performance".to_string()],
            tags: vec!["rpm".to_string()],
        },
        json!({"value": 2500}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "oil_pressure".to_string(),
            name: "Oil Pressure".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["engine".to_string(), "fluids".to_string()],
            tags: vec!["pressure".to_string()],
        },
        json!({"value": 45}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "serial_number".to_string(),
            name: "Engine Serial Number".to_string(),
            translation_id: None,
            category: DataCategory::IdentData,
            groups: vec!["engine".to_string(), "identification".to_string()],
            tags: vec!["serial".to_string()],
        },
        json!("ENG-2024-001"),
        false,
    );

    component.add_data_item(
        ValueMetaData {
            id: "turbo_boost".to_string(),
            name: "Turbocharger Boost Pressure".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-liebherr-engine".to_string()),
            groups: vec!["engine".to_string(), "turbo".to_string()],
            tags: vec!["pressure".to_string(), "boost".to_string()],
        },
        json!({"value": 18.5, "max": 25.0}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "def_level".to_string(),
            name: "Diesel Exhaust Fluid Level".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-liebherr-aftertreatment".to_string()),
            groups: vec!["engine".to_string(), "aftertreatment".to_string()],
            tags: vec!["def".to_string(), "fluid".to_string()],
        },
        json!({"level": 75}),
        true,
    );

    component
}

fn create_transmission_component() -> StaticComponent {
    let mut component = StaticComponent::new(
        "transmission".to_string(),
        "Transmission".to_string(),
        vec!["drivetrain".to_string()],
        None,
    );

    component.add_data_item(
        ValueMetaData {
            id: "gear".to_string(),
            name: "Current Gear".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["transmission".to_string(), "gearing".to_string()],
            tags: vec!["gear".to_string()],
        },
        json!({"current": 3, "max": 6}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "fluid_temp".to_string(),
            name: "Transmission Fluid Temperature".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["transmission".to_string(), "fluids".to_string()],
            tags: vec!["temperature".to_string()],
        },
        json!({"value": 75}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "shift_mode".to_string(),
            name: "Shift Mode".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["transmission".to_string(), "control".to_string()],
            tags: vec!["mode".to_string()],
        },
        json!("automatic"),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "part_number".to_string(),
            name: "Transmission Part Number".to_string(),
            translation_id: None,
            category: DataCategory::IdentData,
            groups: vec!["transmission".to_string(), "identification".to_string()],
            tags: vec!["part".to_string()],
        },
        json!("TRANS-X900-2024"),
        false,
    );

    component.add_data_item(
        ValueMetaData {
            id: "torque_converter_lockup".to_string(),
            name: "Torque Converter Lockup Status".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-allison-transmission".to_string()),
            groups: vec!["transmission".to_string(), "torque-converter".to_string()],
            tags: vec!["lockup".to_string(), "status".to_string()],
        },
        json!({"locked": true, "slip_rpm": 25}),
        true,
    );

    component
}

fn create_hydraulics_component() -> StaticComponent {
    let mut component = StaticComponent::new(
        "hydraulics".to_string(),
        "Hydraulic Control System".to_string(),
        vec!["hydraulics".to_string()],
        None,
    );

    component.add_data_item(
        ValueMetaData {
            id: "main_pressure".to_string(),
            name: "Main Hydraulic System Pressure".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-liebherr-hydraulics".to_string()),
            groups: vec!["hydraulics".to_string(), "pressure".to_string()],
            tags: vec!["main".to_string(), "system".to_string()],
        },
        json!({"value": 350, "max": 420}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "pilot_pressure".to_string(),
            name: "Pilot Control Pressure".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-liebherr-hydraulics".to_string()),
            groups: vec!["hydraulics".to_string(), "pilot".to_string()],
            tags: vec!["pilot".to_string(), "control".to_string()],
        },
        json!({"value": 28, "target": 30}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "boom_position".to_string(),
            name: "Boom Cylinder Position".to_string(),
            translation_id: None,
            category: DataCategory::Vendor("x-liebherr-construction".to_string()),
            groups: vec!["hydraulics".to_string(), "boom".to_string()],
            tags: vec!["position".to_string(), "cylinder".to_string()],
        },
        json!({"extension": 65}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "fluid_temperature".to_string(),
            name: "Hydraulic Fluid Temperature".to_string(),
            translation_id: None,
            category: DataCategory::CurrentData,
            groups: vec!["hydraulics".to_string(), "fluids".to_string()],
            tags: vec!["temperature".to_string()],
        },
        json!({"value": 45, "warning_threshold": 80}),
        true,
    );

    component.add_data_item(
        ValueMetaData {
            id: "pump_model".to_string(),
            name: "Hydraulic Pump Model".to_string(),
            translation_id: None,
            category: DataCategory::IdentData,
            groups: vec!["hydraulics".to_string(), "pump".to_string()],
            tags: vec!["model".to_string(), "identification".to_string()],
        },
        json!("LH-PUMP-350-V2"),
        false,
    );

    component
}
