use async_ssh2_tokio::client::{Client, AuthMethod, ServerCheckMethod, CommandExecutedResult};
use lazy_static::lazy_static;
use tokio::task::JoinSet;
use std::{sync::Arc, process};
use tokio::sync::Semaphore;
use std::path::Path;
use rpassword::prompt_password;

mod getopt; use crate::getopt::*;

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const COMMIT_ID: &str = env!("GIT_COMMITID");

lazy_static! { static ref GETOPT: getopt::Getopt = getopt::getopt(); }

pub fn usage() {
    eprintln!("{} {}: {}\n", &PKG_NAME, &PKG_VERSION, &COMMIT_ID);
    eprintln!("Usage: bulkssh [-v] -c command [router_addrs] ...");
    eprintln!("       -v            verbose mode");
    eprintln!("       -c command    (use quotes to escape white space in command)");
    eprintln!("                     (can be repeated, but typically uses new shell, so CWD/env not preserved");
    eprintln!("       -I filename   identity/private key file");
    eprintln!("       -u username   use specified username");
    eprintln!("       -P            prompt for a password, and use SSH password auth");
    eprintln!("       -g pattern    grep output for lines matching regular expression");
    eprintln!("       -n N          maximum number of concurrent sessions ({})", DEFAULT_MAX_SESSIONS);

    process::exit(1);
}

pub fn default_key_file() -> Option<String> {
    let checklist = [
        ".ssh/id_ed25519",
        ".ssh/id_rsa",
    ];
    for file in &checklist {
        #[allow(deprecated)]
        let path = format!("{}/{}",
            std::env::home_dir().unwrap().display(),
            file);
        if Path::new(&path).exists() {
            return Some(path);
        }
    }
    eprintln!("No suitable SSH private key file found. (You can specify filename with -I)");
    None
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

    if GETOPT.verbose {
        dbg!(&*GETOPT);
    }

    let mut tasks: JoinSet<Result<Vec<(String, String, CommandExecutedResult)>, async_ssh2_tokio::Error>> = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(GETOPT.max_sessions));

    let auth_method = {
        if GETOPT.private_key_file.is_none() || GETOPT.request_password {
            AuthMethod::with_password(&prompt_password("Password: ")?)
        } else {
            AuthMethod::with_key_file(&GETOPT.private_key_file.as_ref().unwrap(), None)
        }
    };

    for remote_host in &GETOPT.args {

        tasks.spawn({

            let remote_host = remote_host.clone();
            let semaphore = semaphore.clone();
            let auth_method = auth_method.clone();

            async move {

                let _semaphore = semaphore.acquire_owned().await.unwrap();

                if GETOPT.verbose {
                    eprintln!("Connecting to {}...", &remote_host);
                }

                let client = Client::connect((remote_host.clone(), 22),
                    &GETOPT.username, auth_method, ServerCheckMethod::NoCheck).await?;

                let mut results: Vec<(String, String, CommandExecutedResult)> = Vec::new();

                for command in &GETOPT.commands {
                    results.push(
                        (remote_host.clone(), command.clone(), client.execute(command).await?)
                    );
                }

                Ok(results)
            }
        });

    }

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(result) => {
                match result {
                    Ok(results) => {
                        for result in &results {
                            if result.2.exit_status==0 {
                                for line in result.2.output.split("\n") {
                                    if GETOPT.pattern.is_none() || GETOPT.pattern.as_ref().unwrap().is_match(&line) {
                                        println!("{} \"{}\": {}", &result.0, &result.1, &line);
                                    }
                                }
                            } else {
                                for line in result.2.output.split("\n") {
                                    if GETOPT.pattern.is_none() || GETOPT.pattern.as_ref().unwrap().is_match(&line) {
                                        println!("{} \"{}\" (exit status {}): {}",
                                            &result.0, &result.1, &result.2.exit_status, &line);
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            },
            Err(e) => {
                if e.is_panic() {
                    e.into_panic();
                } else {
                    eprintln!("Unexpected error: {}", e);
                }
            }
        }
    }

    Ok(())
}
