/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use clap::{App, Arg};
use std::process::exit;

use crate::config_retriever::{config_retrieve, ConfigRetrieverArgs};

mod config_retriever;
mod error_kind;

#[tokio::main]
async fn main() {
    let args = App::new("cp-config-client")
        .arg(
            Arg::with_name("access-token")
                .short("a")
                .long("access-token")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output-path")
                .short("o")
                .long("output-path")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("stage")
                .short("s")
                .long("stage")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("environment")
                .short("e")
                .long("environment")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("component")
                .short("c")
                .long("component")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let access_token = args.value_of("access-token").unwrap();
    let output_path = args.value_of("output-path").unwrap();
    let working_path = args.value_of("working-path").unwrap();
    let host = args.value_of("host").unwrap();
    let stage = args.value_of("stage").unwrap();
    let environment = args.value_of("environment").unwrap();
    let component = args.value_of("component").unwrap();

    let config_retriever_args = ConfigRetrieverArgs {
        access_token: access_token.into(),
        output_path: output_path.into(),
        host: host.into(),
        stage: stage.into(),
        environment: environment.into(),
        component: component.into(),
    };

    match config_retrieve(config_retriever_args).await {
        Ok(_) => println!("Successfully retrieved configuration."),
        Err(error) => {
            eprintln!("Failed to retrieve configuration: {}", error);
            exit(1);
        }
    }
}
