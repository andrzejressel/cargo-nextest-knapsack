use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::Context;
use nextest_metadata::ListCommand;
use serde_json::Value;
use crate::models::{Test, TestResult};

pub(crate) trait TestContext {
    fn find_tests(&self) -> anyhow::Result<Vec<Test>>;
    fn run_tests(&self, tests: &Vec<Test>) -> anyhow::Result<Vec<TestResult>>;
}


pub(crate) struct DefaultTestContext {
    directory: PathBuf,
    cargo_metadata_path: PathBuf,
    binaries_metadata_path: PathBuf,
}

impl TestContext for DefaultTestContext {
    fn find_tests(&self) -> anyhow::Result<Vec<Test>> {
        let mut command = ListCommand::new();
        command.add_arg("--binaries-metadata").add_arg(self.binaries_metadata_path.to_str().unwrap());
        command.add_arg("--cargo-metadata").add_arg(self.cargo_metadata_path.to_str().unwrap());
        command.current_dir(self.directory.to_str().unwrap().to_string());
        let test_list = command
            .exec()
            .with_context(|| format!("Failed to list tests in directory [{}]", self.directory.to_str().unwrap()))?;

        let mut tests = Vec::new();

        for (_, suite) in test_list.rust_suites {
            for (test_name, _) in suite.test_cases {
                tests.push(Test {
                    package_name: suite.package_name.clone(),
                    binary_name: suite.binary.binary_name.clone(),
                    test_name: test_name.clone(),
                });
            }
        }
        println!("Found {} tests.", tests.len());

        Ok(tests)

    }

    fn run_tests(&self, tests: &Vec<Test>) -> anyhow::Result<Vec<TestResult>> {
        let nextest_names_map = tests
            .iter()
            .map(|test| (test.to_nextest_name(), test))
            .collect::<HashMap<_, _>>();

        let args = tests
            .iter()
            .map(|t| t.to_nextest_filter())
            .flatten()
            .collect::<Vec<_>>();

        let mut command = Command::new("cargo");
        command
            .current_dir(&self.directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
            .args(&[
                "nextest",
                "run",
                "--message-format",
                "libtest-json",
                "--binaries-metadata",
                self.binaries_metadata_path.to_str().unwrap(),
                "--cargo-metadata",
                self.cargo_metadata_path.to_str().unwrap(),
            ])
            .args(&args);

        let mut spawn = command.spawn().context("Failed to spawn cargo nextest")?;

        let status = spawn.wait().context("Failed to get status")?;
        if !status.success() {
            anyhow::bail!("Failed to run tests: {}", status);
        }

        let mut test_results = Vec::new();

        let stdout = spawn.wait_with_output()?.stdout;

        for line in String::from_utf8_lossy(&stdout).lines() {
            let v: Value =
                serde_json::from_str(line).with_context(|| format!("Cannot parse JSON: {}", line))?;

            if v.get("type").unwrap() == "test" && v.get("event").unwrap() == "ok" {
                let name = v.get("name").unwrap().as_str().unwrap().to_string();
                let exec_time = v.get("exec_time").unwrap().as_f64().unwrap();

                let test = nextest_names_map
                    .get(&name)
                    .with_context(|| format!("Unknown test: {}", name))?;

                test_results.push(TestResult {
                    test: (*test).clone(),
                    exec_time,
                });
            }
        }

        Ok(test_results)
    }
}

impl DefaultTestContext {
    pub(crate) fn new(directory: &Path) -> anyhow::Result<Self> {
        let cargo_metadata_path = Self::prepare_cargo_metadata(directory)?;
        let binaries_metadata_path = Self::prepare_binaries_metadata(directory)?;
        Ok(Self {
            directory: directory.to_path_buf(),
            cargo_metadata_path,
            binaries_metadata_path
        })
    }

    fn prepare_binaries_metadata(directory: &Path) -> anyhow::Result<PathBuf> {
        let file_name = directory.join("target/nextest-knapsack/binaries-metadata.json");
        fs::create_dir_all(directory.join("target/nextest-knapsack")).context("failed to create directory for nextest-knapsack")?;
        let file = File::create(&file_name)
            .context("failed to open file")?;

        let mut cmd = Command::new("cargo")
            .args("nextest list --workspace --list-type binaries-only --message-format json".split(" "))
            .stdout(file)
            .current_dir(directory)
            .spawn()
            .context("failed to run cargo nextest")?;

        let exit_status = cmd.wait().context("failed to wait for cargo nextest")?;

        if !exit_status.success() {
            anyhow::bail!("Failed to prepare binaries metadata");
        }

        Ok("target/nextest-knapsack/binaries-metadata.json".into())
    }

    fn prepare_cargo_metadata(directory: &Path) -> anyhow::Result<PathBuf> {
        let file_name = directory.join("target/nextest-knapsack/cargo-metadata.json");
        fs::create_dir_all(directory.join("target/nextest-knapsack")).context("failed to create directory for nextest-knapsack")?;
        let file = File::create(&file_name)
            .context("failed to open file")?;

        let mut cmd = Command::new("cargo")
            .args(&["metadata", "--format-version", "1"])
            .stdout(file)
            .current_dir(directory)
            .spawn()
            .context("failed to run cargo metadata")?;

        let exit_status = cmd.wait().context("failed to wait for cargo metadata")?;

        if !exit_status.success() {
            anyhow::bail!("Failed to prepare cargo metadata");
        }

        Ok("target/nextest-knapsack/cargo-metadata.json".into())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use super::*;

    #[test]
    #[serial]
    fn test_find_tests_in_directory() {
        let test_directory = "./tests/projects/project";

        let context = DefaultTestContext::new(Path::new(test_directory)).unwrap();

        let tests = context.find_tests().unwrap();

        assert_eq!(
            tests,
            vec![
                Test {
                    package_name: "project".into(),
                    binary_name: "tests".into(),
                    test_name: "root_external_test".into()
                },
                Test {
                    package_name: "project".into(),
                    binary_name: "project".into(),
                    test_name: "dir::file::tests::test_in_subdirectory".into()
                },
                Test {
                    package_name: "project".into(),
                    binary_name: "project".into(),
                    test_name: "dir::file::tests::test_in_subdirectory_2".into()
                },
                Test {
                    package_name: "project".into(),
                    binary_name: "project".into(),
                    test_name: "tests::root_inline_test".into()
                },
                Test {
                    package_name: "some_crate".into(),
                    binary_name: "tests".into(),
                    test_name: "crate_external_test".into()
                },
                Test {
                    package_name: "some_crate".into(),
                    binary_name: "some_crate".into(),
                    test_name: "tests::crate_inline_test".into()
                }
            ]
        )
    }

    #[test]
    #[serial]
    fn should_run_tests() -> anyhow::Result<()> {
        let test_directory = "./tests/projects/project";

        let context = DefaultTestContext::new(Path::new(test_directory)).unwrap();

        let test_1 = Test {
            package_name: "project".into(),
            binary_name: "tests".into(),
            test_name: "root_external_test".into(),
        };
        let test_2 = Test {
            package_name: "project".into(),
            binary_name: "project".into(),
            test_name: "dir::file::tests::test_in_subdirectory".into(),
        };

        let result = context.run_tests(&vec![test_1, test_2])?;

        assert_eq!(result.len(), 2);

        Ok(())
    }

}