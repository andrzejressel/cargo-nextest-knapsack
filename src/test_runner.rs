use crate::models::{Test, TestResult};
use anyhow::Context;
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub(crate) fn run_tests(directory: &str, tests: &Vec<Test>) -> anyhow::Result<Vec<TestResult>> {
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
        .current_dir(directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")
        .args(&[
            "nextest",
            "run",
            "--workspace",
            "--message-format",
            "libtest-json",
        ])
        .args(&args);

    let output = command.output().context("Failed to get output")?;

    let status = command.status().context("Failed to get status")?;
    if !status.success() {
        anyhow::bail!("Failed to run tests: {}", status);
    }

    let mut test_results = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let v: Value =
            serde_json::from_str(line).with_context(|| format!("Cannot parse JSON: {}", line))?;

        if v.get("type").unwrap() == "test" && v.get("event").unwrap() == "ok" {
            let name = v.get("name").unwrap().as_str().unwrap().to_string();
            let exec_time = v.get("exec_time").unwrap().as_f64().unwrap();

            let test = nextest_names_map
                .get(&name)
                .with_context(|| format!("Unknown test: {}", name))?;

            test_results.push(TestResult {
                test: test.clone().clone(),
                exec_time,
            });
        }
    }

    Ok(test_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn should_run_tests() -> Result<()> {
        let test_directory = "./tests/projects/project";

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

        let result = run_tests(test_directory, &vec![test_1, test_2])?;

        assert_eq!(result.len(), 2);

        Ok(())
    }
}
