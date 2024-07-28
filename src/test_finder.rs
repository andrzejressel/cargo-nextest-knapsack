use crate::models::Test;
use anyhow::Context;
use anyhow::Result;
use nextest_metadata::ListCommand;

pub(crate) trait TestFinder {
    fn find_tests_in_directory(&self) -> Result<Vec<Test>>;
}

pub(crate) struct TestFinderImpl;

impl TestFinder for TestFinderImpl {
    fn find_tests_in_directory(&self) -> Result<Vec<Test>> {
        find_tests_in_directory(".")
    }
}

fn find_tests_in_directory(directory: &str) -> Result<Vec<Test>> {
    println!("Searching for tests. This may take a while.");
    let mut command = ListCommand::new();
    command.add_arg("--workspace");
    command.current_dir(directory);
    let test_list = command
        .exec()
        .with_context(|| format!("Failed to list tests in directory [{}]", directory))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tests_in_directory() {
        let test_directory = "./tests/projects/project";

        let tests = find_tests_in_directory(test_directory).unwrap();

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
}
