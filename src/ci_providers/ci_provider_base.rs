// https://github.com/KnapsackPro/knapsack-pro-js/blob/main/packages/core/src/ci-providers/ci-provider.base.ts
pub(crate) trait CiProvider {
    fn get_ci_node_total(&self) -> Option<usize>;
    fn get_ci_node_index(&self) -> Option<usize>;
    fn get_ci_node_build_id(&self) -> Option<String>;
    fn get_commit_hash(&self) -> Option<String>;
    fn is_fixed_queue_split(&self) -> bool;
    fn get_branch(&self) -> Option<String>;
}
