use std::process::{Command, Stdio};

fn main() {
    build_contracts();
}

fn build_contracts() {
    install_foundry();
    compile_contracts(
        "safe-smart-account/contracts/proxies/SafeProxyFactory.sol",
    );
    compile_contracts("safe-smart-account/contracts/Safe.sol");
    compile_contracts("safe-smart-account/contracts/libraries/MultiSend.sol");
    compile_contracts(
        "safe-modules/modules/4337/contracts/SafeModuleSetup.sol",
    );

    {
        println!("cargo::rerun-if-changed=safe7579/pnpm-lock.yaml");
        let output = Command::new("pnpm")
            .current_dir("safe7579")
            .args(["install"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        println!("`pnpm install` status: {:?}", output.status);
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("`pnpm install` stdout: {stdout:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        println!("`pnpm install` stderr: {stderr:?}");
        assert!(output.status.success());
    }
    compile_contracts_with_args(
        "safe7579/src/Safe7579Launchpad.sol",
        &["--config-path=safe7579/foundry.toml".to_owned()],
    );
    compile_contracts_with_args(
        "safe7579/src/Safe7579.sol",
        &["--config-path=safe7579/foundry.toml".to_owned()],
    );

    {
        println!("cargo::rerun-if-changed=src/contracts/yarn.lock");
        let output = Command::new("yarn")
            .current_dir("src/contracts")
            .args(["install"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        println!("`yarn install` status: {:?}", output.status);
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("`yarn install` stdout: {stdout:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        println!("`yarn install` stderr: {stderr:?}");
        assert!(output.status.success());
    }
    compile_contracts_with_args(
        "src/contracts/contracts/samples/SimpleAccountFactory.sol",
        &["--remappings=@openzeppelin=crates/yttrium/src/contracts/node_modules/@openzeppelin".to_owned()],
    );
    compile_contracts_with_args(
        "src/contracts/contracts/samples/SimpleAccount.sol",
        &["--remappings=@openzeppelin=crates/yttrium/src/contracts/node_modules/@openzeppelin".to_owned()],
    );
    compile_contracts_with_args(
        "src/contracts/contracts/core/EntryPoint.sol",
        &["--remappings=@openzeppelin=crates/yttrium/src/contracts/node_modules/@openzeppelin".to_owned()],
    );
    // extract_bytecodes();
}

fn format_foundry_dir(path: &str) -> String {
    format!(
        "{}/.foundry/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        path
    )
}

fn install_foundry() {
    let bin_finished_flag = format_foundry_dir("bin/.finished");
    if std::fs::metadata(&bin_finished_flag).is_ok() {
        return;
    }

    let bin_folder = format_foundry_dir("bin");
    std::fs::remove_dir_all(&bin_folder).ok();
    std::fs::create_dir_all(&bin_folder).unwrap();
    let output = Command::new("bash")
        .args(["-c", &format!("curl https://raw.githubusercontent.com/foundry-rs/foundry/e0ea59cae26d945445d9cf21fdf22f4a18ac5bb2/foundryup/foundryup | FOUNDRY_DIR={} bash", format_foundry_dir(""))])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    println!("foundryup status: {:?}", output.status);
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("foundryup stdout: {stdout:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("foundryup stderr: {stderr:?}");
    assert!(output.status.success());

    std::fs::write(bin_finished_flag, "").unwrap();
}

fn compile_contracts(contracts_dir: &str) {
    compile_contracts_with_args(contracts_dir, &[]);
}

fn compile_contracts_with_args(contracts_dir: &str, args: &[String]) {
    println!("cargo::rerun-if-changed={contracts_dir}");
    let mut built_args = vec![
        "build".to_owned(),
        contracts_dir.to_owned(),
        "--skip=test".to_owned(),
        "--cache-path".to_owned(),
        format_foundry_dir("forge/cache"),
        "--out".to_owned(),
        format_foundry_dir("forge/out"),
    ];
    built_args.extend_from_slice(args);
    let output = Command::new(format_foundry_dir("bin/forge"))
        .args(&built_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    println!("forge status: {:?}", output.status);
    let stdout = String::from_utf8(output.stdout).unwrap();
    println!("forge stdout: {stdout:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("forge stderr: {stderr:?}");
    assert!(output.status.success());
}

// const ERC6492_FILE: &str = "forge/out/Erc6492.sol/ValidateSigOffchain.json";
// const ERC6492_BYTECODE_FILE: &str =
// "forge/out/Erc6492.sol/ValidateSigOffchain.bytecode";
// const ERC1271_MOCK_FILE: &str = "forge/out/Erc1271Mock.sol/Erc1271Mock.json";
// const ERC1271_MOCK_BYTECODE_FILE: &str =
// "forge/out/Erc1271Mock.sol/Erc1271Mock.bytecode"; fn extract_bytecodes() {
//     extract_bytecode(
//         &format_foundry_dir(ERC6492_FILE),
//         &format_foundry_dir(ERC6492_BYTECODE_FILE),
//     );
//     extract_bytecode(
//         &format_foundry_dir(ERC1271_MOCK_FILE),
//         &format_foundry_dir(ERC1271_MOCK_BYTECODE_FILE),
//     );
// }

// fn extract_bytecode(input_file: &str, output_file: &str) {
//     let contents =
// serde_json::from_slice::<Value>(&std::fs::read(input_file).unwrap()).
// unwrap();     let bytecode = contents
//         .get("bytecode")
//         .unwrap()
//         .get("object")
//         .unwrap()
//         .as_str()
//         .unwrap()
//         .strip_prefix("0x")
//         .unwrap();
//     let bytecode = alloy_primitives::hex::decode(bytecode).unwrap();
//     std::fs::write(output_file, bytecode).unwrap();
// }
