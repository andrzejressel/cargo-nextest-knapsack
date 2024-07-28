use crate::ci_providers::ci_provider_base::CiProvider;

pub(crate) struct GithubActionsCiProvider;

impl CiProvider for GithubActionsCiProvider {
    fn get_ci_node_total(&self) -> Option<usize> {
        None
    }

    fn get_ci_node_index(&self) -> Option<usize> {
        None
    }

    fn get_ci_node_build_id(&self) -> Option<String> {
        std::env::var("GITHUB_RUN_ID").ok()
    }

    fn get_commit_hash(&self) -> Option<String> {
        std::env::var("GITHUB_SHA").ok()
    }

    fn is_fixed_queue_split(&self) -> bool {
        true
    }

    fn get_branch(&self) -> Option<String> {
        std::env::var("GITHUB_REF")
            .ok()
            .or(std::env::var("GITHUB_SHA").ok())
    }
}
