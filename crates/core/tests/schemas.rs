//! Schemele JSON din `schemas/` sunt generate din tipurile Rust.
//! Regenerare: `UPDATE_SCHEMAS=1 cargo test -p rab-core --test schemas`.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use rab_core::config::{AgentsConfig, CollaborationConfig};
use rab_core::plan::Plan;

fn check(name: &str, schema: schemars::Schema) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(name);
    let generated = format!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
    if std::env::var_os("UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert_eq!(
        committed, generated,
        "schema {name} este depășită; rulează cu UPDATE_SCHEMAS=1"
    );
}

#[test]
fn schemas_match_rust_contracts() {
    check(
        "agents-config.v0.1.schema.json",
        schemars::schema_for!(AgentsConfig),
    );
    check(
        "collaboration-config.v0.2.schema.json",
        schemars::schema_for!(CollaborationConfig),
    );
    check("plan.v0.1.schema.json", schemars::schema_for!(Plan));
}

#[test]
fn schemas_forbid_unknown_fields() {
    let schema = serde_json::to_value(schemars::schema_for!(AgentsConfig)).unwrap();
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    assert_eq!(
        schema["$defs"]["PolicyConfig"]["additionalProperties"],
        serde_json::json!(false)
    );
}
