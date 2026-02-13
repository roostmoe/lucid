use std::{collections::{BTreeMap, HashSet}, fs::{self, File}, io::Write};

use lucid_beacon_config::BeaconConfig;
use schemars::{schema::{InstanceType, RootSchema, Schema, SchemaObject}, schema_for};

fn generate_config_docs(schema: &RootSchema) -> String {
    let mut output = String::new();
    let mut processed = HashSet::new();

    output.push_str("= Configuration Reference\n:showtoc:\n:toc: left\n\n");

    output.push_str("[source,toml]\n----\n");
    if let Some(obj) = &schema.schema.object {
        for (name, _) in &obj.properties {
            output.push_str(&format!("[{}]\n...\n\n", name));
        }
    }
    output.push_str("----\n\n");

    if let Some(obj) = &schema.schema.object {
        for (name, prop_schema) in &obj.properties {
            generarte_section(
                name,
                prop_schema,
                &schema.definitions,
                &mut output,
                &mut processed,
                0,
            );
        }
    }

    output
}

fn generarte_section(
    name: &str,
    schema: &Schema,
    definitions: &BTreeMap<String, Schema>,
    output: &mut String,
    processed: &mut HashSet<String>,
    depth: usize,
) {
    let obj = match schema {
        Schema::Object(obj) => obj,
        _ => return,
    };

    // Resolve $ref if needed
    let resolved = if let Some(reference) = &obj.reference {
        let def_name = reference.strip_prefix("#/definitions/").unwrap();
        if processed.contains(def_name) {
            return;
        }
        processed.insert(def_name.to_string());
        definitions.get(def_name).and_then(|s| Some(s.clone().into_object()))
    } else {
        Some(obj.clone())
    };

    if resolved.is_none() { return };
    let resolved = resolved.unwrap();

    let header_level = "=".repeat(depth + 2);
    output.push_str(&format!("{} `[{}]`\n\n", header_level, name));

    if let Some(metadata) = &resolved.metadata {
        if let Some(desc) = &metadata.description {
            output.push_str(&format!("{}\n\n", desc));
        }
    }

    output.push_str("[source,toml]\n----\n");
    output.push_str(&format!("[{}]\n", name));

    if let Some(object) = &resolved.object {
        for (field_name, field_schema) in &object.properties {
            let field_obj = field_schema.clone().into_object();

            // Extract description comment
            if let Some(metadata) = &field_obj.metadata {
                if let Some(desc) = &metadata.description {
                    output.push_str("\n");
                    for line in desc.lines() {
                        output.push_str(&format!("# {}\n", line));
                    }
                }
            }

            // Generate example value
            let example = get_example_value(&field_obj);
            output.push_str(&format!("{} = {}\n", field_name, example));
        }
    }

    output.push_str("----\n\n");
}

fn get_example_value(obj: &SchemaObject) -> String {
    // check for example first
    if let Some(metadata) = &obj.metadata {
        if let Some(example) = &metadata.examples.first() {
            return format!("{}", example); // adjust formatting as needed
        }

        if let Some(default) = &metadata.default {
            return serde_json::to_string(default).unwrap_or_else(|_| format!("{}", default));
        }
    }

    // fallback based on type
    if let Some(instance_type) = &obj.instance_type {
        match instance_type {
            schemars::schema::SingleOrVec::Single(t) => match **t {
                InstanceType::String => "\"\"".to_string(),
                InstanceType::Number | InstanceType::Integer => "0".to_string(),
                InstanceType::Boolean => "false".to_string(),
                _ => "\"<value>\"".to_string(),
            },
            _ => "\"<value>\"".to_string(),
        }
    } else {
        "\"<value>\"".to_string()
    }
}


fn main() {
    let schema = schema_for!(BeaconConfig);
    let docs = generate_config_docs(&schema);

    let out_dir = format!("{}/gen", env!("CARGO_MANIFEST_DIR"));
    if fs::exists(format!("{}/gen", env!("CARGO_MANIFEST_DIR"))).expect("Failed to check if gen dir exists") {
        fs::remove_dir_all(out_dir.clone()).expect("Failed to delete output directory");
    }

    fs::create_dir(out_dir.clone()).expect("Failed to create gen dir");
    let mut buffer = File::create(format!("{}/config.adoc", out_dir)).expect("Failed to create openapi.json file");

    buffer.write_all(docs.as_bytes()).expect("Failed to write configuration docs");
}
