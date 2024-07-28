set windows-shell := ["pwsh.exe", "-c"]

# renovate: datasource=crate depName=cargo-nextest packageName=cargo-nextest
NEXTEST_VERSION := "0.9.72"

install-requirements:
    cargo binstall --no-confirm cargo-nextest@{{NEXTEST_VERSION}}

[windows]
test KNAPSACK_API_TOKEN:
    #!pwsh
    cargo build
    $env:GITHUB_SHA = "{{uuid()}}"
    $env:GITHUB_RUN_ID = "{{uuid()}}"
    $env:GITHUB_REF = "{{uuid()}}"
    $env:KNAPSACK_PRO_CI_NODE_TOTAL = "2"
    $env:KNAPSACK_PRO_CI_NODE_INDEX = "0"
    $env:KNAPSACK_PRO_TEST_SUITE_TOKEN = "{{KNAPSACK_API_TOKEN}}"
    cp  target/debug/cargo-nextest-knapsack.exe target/debug/cargo-nextest-knapsack-2.exe
    $job1 = target\debug\cargo-nextest-knapsack-2.exe &
    $env:KNAPSACK_PRO_CI_NODE_INDEX = "1"
    $job2 = target\debug\cargo-nextest-knapsack-2.exe &
    Receive-Job $job1 -Wait
    Receive-Job $job2 -Wait