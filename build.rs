use std::error::Error;
use std::process::Command;

// Use these constants within src/main.rs
//
// pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
// pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
// pub const COMMIT_ID: &str = env!("GIT_COMMITID");

pub fn main() -> Result<(), Box<dyn Error>> {

    // git show -s --format="%ad %h %an <%ae> (%s)"
    let output = Command::new("git").args(&[
        "show",
        "-s",
        "--format=%ad %h %an <%ae> (%s)"
    ]).output().unwrap();

    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_COMMITID={git_hash}");

    Ok(())

}