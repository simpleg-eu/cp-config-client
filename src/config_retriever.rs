/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::io::Cursor;
use std::path::PathBuf;

use cp_core::error::Error;
use cp_core::ok_or_return_error;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use zip::ZipArchive;

use crate::error_kind::{
    DECOMPRESS_FAILURE, INVALID_ACCESS_TOKEN, REQUEST_FAILURE, UNEXPECTED_ERROR,
};

pub struct ConfigRetrieverArgs {
    pub access_token: String,
    pub output_path: PathBuf,
    pub host: String,
    pub stage: String,
    pub environment: String,
    pub component: String,
}

pub async fn config_retrieve(args: ConfigRetrieverArgs) -> Result<(), Error> {
    let request_url = format!(
        "{}/config?stage={}&environment={}&component={}",
        args.host, args.stage, args.environment, args.component
    );

    let headers = get_headers(&args)?;

    let client = reqwest::Client::new();
    let response = ok_or_return_error!(
        client.get(request_url).headers(headers).send().await,
        REQUEST_FAILURE,
        "failed to get configuration from server: "
    );

    let status_code = response.status();
    let content = ok_or_return_error!(
        response.bytes().await,
        UNEXPECTED_ERROR,
        "failed to read response's bytes: "
    );

    match status_code {
        StatusCode::UNAUTHORIZED => {
            return Err(Error::new(INVALID_ACCESS_TOKEN, "invalid access token"));
        }
        StatusCode::BAD_REQUEST => return Err(Error::new(UNEXPECTED_ERROR, "unexpected error: ")),
        StatusCode::OK => (),
        _ => {
            return Err(Error::new(
                UNEXPECTED_ERROR,
                format!("unexpected status code received: {}", status_code),
            ));
        }
    }

    let cursor = Cursor::new(content);

    let mut zip_archive = ok_or_return_error!(
        ZipArchive::new(cursor),
        DECOMPRESS_FAILURE,
        "failed to read 'ZipArchive' from bytes: "
    );

    ok_or_return_error!(
        zip_archive.extract(args.output_path),
        DECOMPRESS_FAILURE,
        "failed to decompress bytes: "
    );

    Ok(())
}

pub fn get_headers(args: &ConfigRetrieverArgs) -> Result<HeaderMap, Error> {
    let authorization_header: HeaderValue = ok_or_return_error!(
        HeaderValue::from_str(format!("Bearer {}", args.access_token).as_str()),
        UNEXPECTED_ERROR,
        "failed to create authorization header: "
    );

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("Authorization", authorization_header);

    Ok(headers)
}

#[cfg(test)]
pub mod tests {
    use cp_core::config_reader::ConfigReader;
    use cp_core::secrets::get_secrets_manager;
    use cp_core::test_base::get_unit_test_data_path;

    use crate::config_retriever::{config_retrieve, ConfigRetrieverArgs};

    const TEST_STAGE: &str = "dummy";

    #[tokio::test]
    pub async fn retrieve_downloads_expected_file() {
        let environment = "development";
        let component = "dummy";
        let expected_file = "application.yaml";
        let output_path = "output";
        let working_path = "working_path";
        let mut test_data_path = get_unit_test_data_path(file!());
        test_data_path.push("config.yaml");
        let config_reader = ConfigReader::default();
        let config = config_reader
            .read(test_data_path)
            .expect("failed to read configuration from test data");
        let access_token_secret = config
            .get("AccessTokenSecret")
            .expect("failed to read 'AccessTokenSecret'")
            .as_str()
            .expect("failed to read 'AccessTokenSecret' as string");
        let access_token = get_secrets_manager()
            .expect("expected 'SecretsManager'")
            .get_secret(access_token_secret)
            .expect("expected 'access_token'");
        let host = config
            .get("Host")
            .expect("failed to read 'Host'")
            .as_str()
            .expect("failed to read 'Host' as string");
        let args = ConfigRetrieverArgs {
            access_token,
            output_path: output_path.into(),
            host: host.into(),
            stage: TEST_STAGE.into(),
            environment: environment.into(),
            component: component.into(),
        };

        let retrieve_result = config_retrieve(args).await;
        let expected_file_metadata =
            std::fs::metadata(format!("{}/{}", output_path, expected_file));
        std::fs::remove_dir_all(output_path);

        retrieve_result.expect("failed to get configuration");
        assert!(expected_file_metadata.is_ok());
    }
}
