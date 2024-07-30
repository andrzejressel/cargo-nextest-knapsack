use regex::Regex;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Test {
    pub(crate) package_name: String,
    pub(crate) binary_name: String,
    pub(crate) test_name: String,
}

impl Test {
    pub(crate) fn to_knapsack_file(&self) -> String {
        format!(
            "{}|{}|{}",
            self.package_name, self.binary_name, self.test_name
        )
    }

    pub(crate) fn to_nextest_name(&self) -> String {
        format!(
            "{}::{}${}",
            self.package_name, self.binary_name, self.test_name
        )
    }

    pub(crate) fn to_nextest_filter(&self) -> Vec<String> {
        vec![
            "-E".into(),
            format!("package({}) & test(={})", self.package_name, self.test_name),
        ]
    }

    pub(crate) fn from_knapsack_file(line: &str) -> anyhow::Result<Self> {
        let regex = Regex::new(r"(.*)\|(.*)\|(.*)")?;
        let Some(caps) = regex.captures(line) else {
            anyhow::bail!("Invalid test file format: {}", line)
        };

        Ok(Self {
            package_name: caps[1].to_string(),
            binary_name: caps[2].to_string(),
            test_name: caps[3].to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct TestResult {
    pub(crate) test: Test,
    pub(crate) exec_time: f64,
}
