#![allow(unused_imports)]

use async_ssh2_tokio::client::{Client, AuthMethod, ServerCheckMethod, CommandExecutedResult};
use lazy_static::lazy_static;
use tokio::task::JoinSet;
use std::{sync::Arc, process};
use tokio::sync::Semaphore;
use std::error::Error;

mod getopt; use crate::getopt::*;

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const COMMIT_ID: &str = env!("GIT_COMMITID");

lazy_static! { static ref GETOPT: getopt::Getopt = getopt::getopt(); }

pub fn usage() {
    eprintln!("{} {}: {}\n", &PKG_NAME, &PKG_VERSION, &COMMIT_ID);
    eprintln!("Usage: bulkssh [-v] -c command [router_addrs] ...");
    eprintln!("       -v            verbose");
    eprintln!("       -c            command (use quotes to escape white space in command)");
    eprintln!("       -I            identity/private key file");
    eprintln!("       -u            username");
    eprintln!("       -n N          maximum number of concurrent sessions ({})", DEFAULT_MAX_SESSIONS);

    process::exit(1);
}


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Hello, world!");

    let mut tasks: JoinSet<Result<Vec<(String, String, CommandExecutedResult)>, async_ssh2_tokio::Error>> = JoinSet::new();

    let semaphore = Arc::new(Semaphore::new(GETOPT.max_sessions));

    for remote_host in &GETOPT.args {

        tasks.spawn({

            let remote_host = remote_host.clone();

            let semaphore = semaphore.clone();

            async move {

                let _semaphore = semaphore.acquire_owned().await.unwrap();

                if GETOPT.verbose {
                    eprintln!("Connecting to {}:", &remote_host);
                }
                let auth_method = AuthMethod::with_key_file(&GETOPT.private_key_file, None);
                let client = Client::connect((remote_host.clone(), 22),
                    &GETOPT.username, auth_method, ServerCheckMethod::NoCheck).await?;

                let mut results: Vec<(String, String, CommandExecutedResult)> = Vec::new();

                for command in &GETOPT.commands {
                    results.push(
                        (remote_host.clone(), command.clone(), client.execute(command).await?)
                    );
                }
//                let result = client.execute(&GETOPT.command).await?;

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
                                    println!("{} \"{}\": {}", &result.0, &result.1, &line);
                                }
                            } else {
                                for line in result.2.output.split("\n") {
                                    println!("{} \"{}\" (exit status {}): {}",
                                        &result.0, &result.1, &result.2.exit_status, &line);
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
