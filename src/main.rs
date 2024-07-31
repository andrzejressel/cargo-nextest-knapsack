use std::io::Write;

use anyhow::Context;

use crate::ci_providers::ci_provider_wrapper::CiProviderWrapper;
use crate::ci_providers::github_actions::GithubActionsCiProvider;
use crate::knapsack_client::KnapsackClient;

mod ci_providers;
mod knapsack_client;
mod models;
mod test_finder;
mod test_runner;

fn main() -> anyhow::Result<()> {
    let knapsack_api_key = std::env::var("KNAPSACK_PRO_TEST_SUITE_TOKEN")
        .context("Could not find KNAPSACK_PRO_TEST_SUITE_TOKEN environment variable")?;

    let test_finder = Box::new(test_finder::TestFinderImpl {});
    let ci_provider_wrapper = CiProviderWrapper::new(Box::new(GithubActionsCiProvider {}));

    let mut client = KnapsackClient::new(
        "https://api.knapsackpro.com".into(),
        knapsack_api_key,
        test_finder,
        ci_provider_wrapper,
    );

    let mut results = vec![];

    loop {
        let tests = client.get_tests()?;
        println!("{:?}", tests);
        if tests.is_empty() {
            break;
        }

        let mut local_results =
            test_runner::run_tests(".".into(), &tests).context("Failed to run tests")?;
        results.append(&mut local_results);
    }

    client.upload_test_results(&results)?;

    Ok(())
}
