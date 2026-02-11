use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/pay/openapi.json");

    // Only generate pay API if the pay feature is enabled
    if env::var("CARGO_FEATURE_PAY").is_ok() {
        generate_pay_api();
    }
}

fn generate_pay_api() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let spec_path = manifest_dir.join("src/pay/openapi.json");

    // Read and preprocess the OpenAPI spec as JSON
    let spec_content =
        fs::read_to_string(&spec_path).expect("Failed to read openapi.json");
    let mut spec: serde_json::Value = serde_json::from_str(&spec_content)
        .expect("Failed to parse openapi.json");

    // Remove enum constraints from specific schemas to make them
    // forward-compatible (unknown values won't fail deserialization)
    remove_enum_constraint(&mut spec, "PaymentStatus");
    remove_enum_constraint(&mut spec, "CollectDataFieldType");

    // Write the preprocessed spec
    let preprocessed_path = out_dir.join("pay_openapi_preprocessed.json");
    fs::write(&preprocessed_path, serde_json::to_string_pretty(&spec).unwrap())
        .expect("Failed to write preprocessed spec");

    // Parse preprocessed JSON as OpenAPI spec
    let file = fs::File::open(&preprocessed_path).unwrap();
    let spec: openapiv3::OpenAPI =
        serde_json::from_reader(file).expect("Failed to parse as OpenAPI");

    // Generate progenitor code with Builder interface and Separate tags
    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);
    settings.with_tag(progenitor::TagStyle::Separate);
    settings.with_derive("PartialEq".to_string());
    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator
        .generate_tokens(&spec)
        .expect("Failed to generate progenitor tokens");
    let ast = syn::parse2(tokens).expect("Failed to parse generated tokens");
    let content = prettyplease::unparse(&ast);

    let codegen_path = out_dir.join("pay_codegen.rs");
    fs::write(&codegen_path, content).expect("Failed to write generated code");
}

fn remove_enum_constraint(spec: &mut serde_json::Value, schema_name: &str) {
    if let Some(schema) =
        spec.pointer_mut(&format!("/components/schemas/{}", schema_name))
    {
        if let Some(obj) = schema.as_object_mut() {
            obj.remove("enum");
        }
    }
}
