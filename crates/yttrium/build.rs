use std::{env, fs, path::Path};

fn main() {
    generate_pay_client();
}

fn generate_pay_client() {
    let src = Path::new("src/pay/openapi.json");
    println!("cargo::rerun-if-changed={}", src.display());

    let file = fs::File::open(src).expect("failed to open openapi.json");
    let spec = serde_json::from_reader(file).expect("failed to parse openapi.json");

    let mut settings = progenitor::GenerationSettings::default();
    settings.with_interface(progenitor::InterfaceStyle::Builder);
    settings.with_tag(progenitor::TagStyle::Separate);
    settings.with_derive("Debug".to_string());
    settings.with_derive("Clone".to_string());
    settings.with_derive("PartialEq".to_string());

    let mut generator = progenitor::Generator::new(&settings);

    let tokens = generator.generate_tokens(&spec).expect("failed to generate client");
    let ast = syn::parse2(tokens).expect("failed to parse generated tokens");
    let content = prettyplease::unparse(&ast);

    // Write next to mod.rs for easy inspection
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_path = Path::new(&manifest_dir).join("src/pay/generated.rs");
    fs::write(&out_path, content).expect("failed to write generated client");
}
