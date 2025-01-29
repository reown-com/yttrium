fn main() {
    let git_hash = match std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(output) => output,
            Err(e) => {
                println!("Failed to get GET_HASH (in getting string from output): {e}");
                "unknown-string".to_string()
            }
        },
        Err(e) => {
            println!("Failed to get GET_HASH (in calling git): {e}");
            "unknown-git".to_string()
        }
    };
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
