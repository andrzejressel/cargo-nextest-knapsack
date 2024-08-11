use crate::ci_providers::ci_provider_wrapper::CiProviderWrapper;
use crate::ci_providers::github_actions::GithubActionsCiProvider;
use crate::knapsack_client::KnapsackClient;
use crate::test_context::{DefaultTestContext, TestContext};
use anyhow::Context;
use std::path::Path;

mod ci_providers;
mod knapsack_client;
mod models;
mod test_context;

fn main() -> anyhow::Result<()> {
    let knapsack_api_key = std::env::var("KNAPSACK_PRO_TEST_SUITE_TOKEN")
        .context("Could not find KNAPSACK_PRO_TEST_SUITE_TOKEN environment variable")?;

    println!("Caching workspace info");
    let context = DefaultTestContext::new(Path::new("."))?;
    println!("Workspace info cached");
    let ci_provider_wrapper = CiProviderWrapper::new(Box::new(GithubActionsCiProvider {}));

    let mut client = KnapsackClient::new(
        "https://api.knapsackpro.com".into(),
        knapsack_api_key,
        &context,
        ci_provider_wrapper,
    );

    let mut results = vec![];

    loop {
        let tests = client.get_tests()?;
        println!("Tests: {:?}", tests);
        if tests.is_empty() {
            break;
        }

        let mut local_results =
            context.run_tests(&tests).context("Failed to run tests")?;
        results.append(&mut local_results);
    }

    client.upload_test_results(&results)?;

    Ok(())
}
