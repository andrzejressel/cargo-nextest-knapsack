use crate::ci_providers::ci_provider_base::CiProvider;
use anyhow::{anyhow, Result};

pub(crate) struct CiProviderWrapper {
    ci_provider: Box<dyn CiProvider>,
}

impl CiProviderWrapper {
    pub(crate) fn new(ci_provider: Box<dyn CiProvider>) -> Self {
        CiProviderWrapper { ci_provider }
    }

    pub(crate) fn get_ci_node_build_id(&self) -> String {
        self.ci_provider
            .get_ci_node_build_id()
            .or_else(|| std::env::var("KNAPSACK_PRO_CI_NODE_BUILD_ID").ok())
            .unwrap_or("missing-build-id".into())
    }

    pub(crate) fn get_ci_node_index(&self) -> Result<usize> {
        match self.ci_provider.get_ci_node_index() {
            None => Self::get_ci_node_index_from_env_var(),
            Some(i) => Ok(i),
        }
    }

    pub(crate) fn get_ci_node_total(&self) -> Result<usize> {
        match self.ci_provider.get_ci_node_total() {
            None => Self::get_ci_node_total_from_env_var(),
            Some(i) => Ok(i),
        }
    }

    pub(crate) fn is_fixed_queue_split(&self) -> bool {
        self.ci_provider.is_fixed_queue_split()
    }

    pub(crate) fn get_branch(&self) -> Result<String> {
        self.ci_provider
            .get_branch()
            .ok_or_else(|| anyhow!("No branch provided"))
    }

    pub(crate) fn get_commit_hash(&self) -> Result<String> {
        self.ci_provider
            .get_commit_hash()
            .ok_or_else(|| anyhow!("No commit hash provided"))
    }

    fn get_ci_node_total_from_env_var() -> Result<usize> {
        std::env::var("KNAPSACK_PRO_CI_NODE_TOTAL")
            .map_err(|e| anyhow!("Failed get KNAPSACK_PRO_CI_NODE_TOTAL from env vars: [{e}]",))
            .and_then(|s| {
                s.parse::<usize>().map_err(|e| {
                    anyhow!("Failed to parse KNAPSACK_PRO_CI_NODE_TOTAL to usize: [{e}]")
                })
            })
    }

    fn get_ci_node_index_from_env_var() -> Result<usize> {
        std::env::var("KNAPSACK_PRO_CI_NODE_INDEX")
            .map_err(|e| anyhow!("Failed get KNAPSACK_PRO_CI_NODE_INDEX from env vars: [{e}]",))
            .and_then(|s| {
                s.parse::<usize>().map_err(|e| {
                    anyhow!("Failed to parse KNAPSACK_PRO_CI_NODE_INDEX to usize: [{e}]")
                })
            })
    }
}
