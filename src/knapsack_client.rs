use crate::ci_providers::ci_provider_base::CiProvider;
use crate::ci_providers::ci_provider_wrapper::CiProviderWrapper;
use crate::models::{Test, TestResult};
use crate::test_finder::TestFinder;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

pub(crate) struct KnapsackClient {
    initialized: bool,
    endpoint: String,
    api_key: String,
    test_finder: Box<dyn TestFinder>,
    ci_provider_wrapper: CiProviderWrapper,
}

impl KnapsackClient {}

#[derive(Deserialize)]
struct KnapsackResponseWithFiles {
    test_files: Vec<KnapsackTestFile>,
}

#[derive(Deserialize)]
struct KnapsackTestFile {
    path: String,
}

impl KnapsackClient {
    pub(crate) fn new(
        endpoint: String,
        api_key: String,
        test_finder: Box<dyn TestFinder>,
        ci_provider_wrapper: CiProviderWrapper,
    ) -> Self {
        KnapsackClient {
            initialized: false,
            api_key,
            endpoint,
            test_finder,
            ci_provider_wrapper,
        }
    }

    pub(crate) fn get_tests(&mut self) -> Result<Vec<Test>> {
        if !self.initialized {
            self.initialized = true;

            match self.initialize_queue_1()? {
                Some(tests) => Ok(tests),
                None => self.initialize_queue_2(),
            }
        } else {
            self.initialize_queue_3()
        }
    }

    fn initialize_queue_1(&self) -> Result<Option<Vec<Test>>> {
        let node_total = self
            .ci_provider_wrapper
            .get_ci_node_total()
            .context("Failed to get node total")?;

        let node_index = self
            .ci_provider_wrapper
            .get_ci_node_index()
            .context("Failed to get node index")?;

        let branch = self
            .ci_provider_wrapper
            .get_branch()
            .context("Failed to get branch")?;

        let commit_hash = self
            .ci_provider_wrapper
            .get_commit_hash()
            .context("Failed to get commit hash")?;

        let json = json!({
              "can_initialize_queue": true,
              "attempt_connect_to_queue": true,
              "fixed_queue_split": self.ci_provider_wrapper.is_fixed_queue_split(),
              "commit_hash": commit_hash,
              "branch": branch,
              "node_total": node_total,
              "node_index": node_index,
              "node_build_id": self.ci_provider_wrapper.get_ci_node_build_id(),
        });

        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Failed to build client")?;

        let res = client
            .post(format!("{}/v1/queues/queue", self.endpoint))
            .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", self.api_key.clone())
            .header("KNAPSACK-PRO-CLIENT-NAME", "cargo-nextest-knapsack")
            .header("KNAPSACK-PRO-CLIENT-VERSION", env!("CARGO_PKG_VERSION"))
            .json(&json)
            .build()
            .context("Failed to build request")?;

        let result = client.execute(res).context("Failed to execute request")?;

        let status = result.status();

        if !status.is_success() {
            let output = result
                .text()
                .unwrap_or("Failed to get response".to_string());
            anyhow::bail!("Failed to initialize queue: [{status}] [{output}]")
        }

        let output = result.json::<Value>().context("Failed to parse response")?;

        if output
            .get("code")
            .unwrap_or(&Value::Null)
            .as_str()
            .is_some_and(|s| s == "ATTEMPT_CONNECT_TO_QUEUE_FAILED")
        {
            return Ok(None);
        } else {
            let response: KnapsackResponseWithFiles = serde_json::from_value(output)
                .context("Failed to parse response to KnapsackResponseWithFiles")?;

            let mut files = vec![];

            for file in response.test_files {
                files.push(
                    Test::from_knapsack_file(&file.path)
                        .with_context(|| format!("Failed to parse test file: {}", &file.path))?,
                );
            }

            Ok(Some(files))
        }
    }

    fn initialize_queue_2(&self) -> Result<Vec<Test>> {
        let node_total = self
            .ci_provider_wrapper
            .get_ci_node_total()
            .context("Failed to get node total")?;

        let node_index = self
            .ci_provider_wrapper
            .get_ci_node_index()
            .context("Failed to get node index")?;

        let branch = self
            .ci_provider_wrapper
            .get_branch()
            .context("Failed to get branch")?;

        let tests = self
            .test_finder
            .find_tests_in_directory()
            .context("Failed to find tests")?;

        let commit_hash = self
            .ci_provider_wrapper
            .get_commit_hash()
            .context("Failed to get commit hash")?;

        let tests_value = serde_json::Value::Array(
            tests
                .iter()
                .map(|test| {
                    json!({
                        "path": test.to_knapsack_file()
                    })
                })
                .collect(),
        );

        // let branch = ciProvider.get_branch().ok_or_else(|| anyhow!("No branch provided"))?;

        let json = json!({
              "can_initialize_queue": true,
              "attempt_connect_to_queue": false,
              "fixed_queue_split": self.ci_provider_wrapper.is_fixed_queue_split(),
              "commit_hash": commit_hash,
              "branch": branch,
              "node_total": node_total,
              "node_index": node_index,
              "node_build_id": self.ci_provider_wrapper.get_ci_node_build_id(),
              "test_files": tests_value
        });

        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Failed to build client")?;

        let res = client
            .post(format!("{}/v1/queues/queue", self.endpoint))
            .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", self.api_key.clone())
            .header("KNAPSACK-PRO-CLIENT-NAME", "cargo-nextest-knapsack")
            .header("KNAPSACK-PRO-CLIENT-VERSION", env!("CARGO_PKG_VERSION"))
            .json(&json)
            .build()
            .context("Failed to build request")?;

        let result = client.execute(res).context("Failed to execute request")?;

        let status = result.status();

        if !status.is_success() {
            let output = result
                .text()
                .unwrap_or("Failed to get response".to_string());
            anyhow::bail!("Failed to initialize queue: [{status}] [{output}]")
        }

        let response = result
            .json::<KnapsackResponseWithFiles>()
            .context("Failed to parse response")?;

        let mut files = vec![];

        for file in response.test_files {
            files.push(
                Test::from_knapsack_file(&file.path)
                    .with_context(|| format!("Failed to parse test file: {}", &file.path))?,
            );
        }

        Ok(files)
    }

    fn initialize_queue_3(&self) -> Result<Vec<Test>> {
        let node_total = self
            .ci_provider_wrapper
            .get_ci_node_total()
            .context("Failed to get node total")?;

        let node_index = self
            .ci_provider_wrapper
            .get_ci_node_index()
            .context("Failed to get node index")?;

        let branch = self
            .ci_provider_wrapper
            .get_branch()
            .context("Failed to get branch")?;

        let commit_hash = self
            .ci_provider_wrapper
            .get_commit_hash()
            .context("Failed to get commit hash")?;

        // let branch = ciProvider.get_branch().ok_or_else(|| anyhow!("No branch provided"))?;

        let json = json!({
              "can_initialize_queue": false,
              "attempt_connect_to_queue": false,
              "fixed_queue_split": self.ci_provider_wrapper.is_fixed_queue_split(),
              "commit_hash": commit_hash,
              "branch": branch,
              "node_total": node_total,
              "node_index": node_index,
              "node_build_id": self.ci_provider_wrapper.get_ci_node_build_id()
        });

        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Failed to build client")?;

        let res = client
            .post(format!("{}/v1/queues/queue", self.endpoint))
            .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", self.api_key.clone())
            .header("KNAPSACK-PRO-CLIENT-NAME", "cargo-nextest-knapsack")
            .header("KNAPSACK-PRO-CLIENT-VERSION", env!("CARGO_PKG_VERSION"))
            .json(&json)
            .build()
            .context("Failed to build request")?;

        let result = client.execute(res).context("Failed to execute request")?;

        let status = result.status();

        if !status.is_success() {
            let output = result
                .text()
                .unwrap_or("Failed to get response".to_string());
            anyhow::bail!("Failed to initialize queue: [{status}] [{output}]")
        }

        let response = result
            .json::<KnapsackResponseWithFiles>()
            .context("Failed to parse response")?;

        let mut files = vec![];

        for file in response.test_files {
            files.push(
                Test::from_knapsack_file(&file.path)
                    .with_context(|| format!("Failed to parse test file: {}", &file.path))?,
            );
        }

        Ok(files)
    }

    pub(crate) fn upload_test_results(&self, test_results: &Vec<TestResult>) -> Result<()> {
        let node_total = self
            .ci_provider_wrapper
            .get_ci_node_total()
            .context("Failed to get node total")?;

        let node_index = self
            .ci_provider_wrapper
            .get_ci_node_index()
            .context("Failed to get node index")?;

        let branch = self
            .ci_provider_wrapper
            .get_branch()
            .context("Failed to get branch")?;

        let commit_hash = self
            .ci_provider_wrapper
            .get_commit_hash()
            .context("Failed to get commit hash")?;

        let tests_value = serde_json::Value::Array(
            test_results
                .iter()
                .map(|test| {
                    json!({
                        "path": test.test.to_knapsack_file(),
                        "time_execution": test.exec_time
                    })
                })
                .collect(),
        );

        // let branch = ciProvider.get_branch().ok_or_else(|| anyhow!("No branch provided"))?;

        let json = json!({
              "commit_hash": commit_hash,
              "branch": branch,
              "node_total": node_total,
              "node_index": node_index,
              "test_files": tests_value
        });

        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Failed to build client")?;

        let res = client
            .post(format!("{}/v1/build_subsets", self.endpoint))
            .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", self.api_key.clone())
            .header("KNAPSACK-PRO-CLIENT-NAME", "cargo-nextest-knapsack")
            .header("KNAPSACK-PRO-CLIENT-VERSION", env!("CARGO_PKG_VERSION"))
            .json(&json)
            .build()
            .context("Failed to build request")?;

        let result = client.execute(res).context("Failed to execute request")?;

        let status = result.status();

        if !status.is_success() {
            let output = result
                .text()
                .unwrap_or("Failed to get response".to_string());
            anyhow::bail!("Failed to initialize queue: [{status}] [{output}]")
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Test;
    use crate::test_finder::TestFinder;
    use httpmock::prelude::*;

    #[test]
    fn should_initialize_queue() -> Result<()> {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.path("/v1/queues/queue")
                .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", "test_api_key")
                .json_body(json!({
                    "can_initialize_queue": true,
                    "attempt_connect_to_queue": true,
                    "fixed_queue_split": true,
                    "commit_hash": "commit_hash",
                    "branch": "branch",
                    "node_total": 4,
                    "node_index": 0,
                    "node_build_id": "build_id"
                }));

            then.status(200).json_body(json!({
                "test_files": [
                    {
                        "path": "a|b|c"
                    }
                ]
            }));
        });

        let mut client = KnapsackClient::new(
            server.base_url(),
            "test_api_key".to_string(),
            Box::new(TestTestFinder::new()),
            CiProviderWrapper::new(Box::new(TestProvider::new())),
        );

        let tests = client.get_tests()?;

        mock.assert();

        assert_eq!(
            tests,
            vec![Test {
                package_name: "a".to_string(),
                binary_name: "b".to_string(),
                test_name: "c".to_string(),
            }]
        );
        assert!(client.initialized);

        Ok(())
    }

    #[test]
    fn should_handle_initialized_queue() -> Result<()> {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.path("/v1/queues/queue")
                .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", "test_api_key")
                .json_body(json!({
                    "can_initialize_queue": true,
                    "attempt_connect_to_queue": true,
                    "fixed_queue_split": true,
                    "commit_hash": "commit_hash",
                    "branch": "branch",
                    "node_total": 4,
                    "node_index": 0,
                    "node_build_id": "build_id"
                }));

            then.status(200).json_body(json!({
                "code": "ATTEMPT_CONNECT_TO_QUEUE_FAILED"
            }));
        });

        let mock2 = server.mock(|when, then| {
            when.path("/v1/queues/queue")
                .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", "test_api_key")
                .json_body(json!({
                    "can_initialize_queue": true,
                    "attempt_connect_to_queue": false,
                    "fixed_queue_split": true,
                    "commit_hash": "commit_hash",
                    "branch": "branch",
                    "node_total": 4,
                    "node_index": 0,
                    "node_build_id": "build_id",
                    "test_files": [
                        {
                            "path": "pn|bn|tn"
                        }
                    ]
                }));

            then.status(200).json_body(json!({
                "test_files": [
                        {
                            "path": "pn|bn|tn"
                        }
                    ]
            }));
        });

        let mut client = KnapsackClient::new(
            server.base_url(),
            "test_api_key".to_string(),
            Box::new(TestTestFinder::new()),
            CiProviderWrapper::new(Box::new(TestProvider::new())),
        );

        let tests = client.get_tests()?;

        mock.assert();
        mock2.assert();

        assert_eq!(
            tests,
            vec![Test {
                package_name: "pn".to_string(),
                binary_name: "bn".to_string(),
                test_name: "tn".to_string(),
            }]
        );
        assert!(client.initialized);

        Ok(())
    }

    #[test]
    fn should_get_additional_tests() -> Result<()> {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.path("/v1/queues/queue")
                .header("KNAPSACK-PRO-TEST-SUITE-TOKEN", "test_api_key")
                .json_body(json!({
                    "can_initialize_queue": false,
                    "attempt_connect_to_queue": false,
                    "fixed_queue_split": true,
                    "commit_hash": "commit_hash",
                    "branch": "branch",
                    "node_total": 4,
                    "node_index": 0,
                    "node_build_id": "build_id"
                }));

            then.status(200).json_body(json!({
                "test_files": [
                        {
                            "path": "pn|bn|tn"
                        }
                    ]
            }));
        });

        let mut client = KnapsackClient::new(
            server.base_url(),
            "test_api_key".to_string(),
            Box::new(TestTestFinder::new()),
            CiProviderWrapper::new(Box::new(TestProvider::new())),
        );
        client.initialized = true;

        let tests = client.get_tests()?;

        mock.assert();

        assert_eq!(
            tests,
            vec![Test {
                package_name: "pn".to_string(),
                binary_name: "bn".to_string(),
                test_name: "tn".to_string(),
            }]
        );
        assert!(client.initialized);

        Ok(())
    }

    struct TestProvider;

    impl TestProvider {
        fn new() -> Self {
            Self {}
        }
    }

    // let provider = {};

    impl CiProvider for TestProvider {
        fn get_ci_node_total(&self) -> Option<usize> {
            Some(4)
        }

        fn get_ci_node_index(&self) -> Option<usize> {
            Some(0)
        }

        fn get_ci_node_build_id(&self) -> Option<String> {
            Some("build_id".into())
        }

        fn get_commit_hash(&self) -> Option<String> {
            Some("commit_hash".into())
        }

        fn is_fixed_queue_split(&self) -> bool {
            true
        }

        fn get_branch(&self) -> Option<String> {
            Some("branch".into())
        }
    }

    struct TestTestFinder;
    impl TestTestFinder {
        fn new() -> Self {
            Self {}
        }
    }

    impl TestFinder for TestTestFinder {
        fn find_tests_in_directory(&self) -> Result<Vec<Test>> {
            Ok(vec![Test {
                package_name: "pn".to_string(),
                binary_name: "bn".to_string(),
                test_name: "tn".to_string(),
            }])
        }
    }
}
