set windows-shell := ["pwsh.exe", "-c"]

# renovate: datasource=crate depName=cargo-nextest packageName=cargo-nextest
NEXTEST_VERSION := "0.9.72"
# renovate: datasource=crate depName=sd packageName=sd
SD_VERSION := "1.0.0"

install-requirements:
    cargo binstall --no-confirm cargo-nextest@{{NEXTEST_VERSION}}
    cargo binstall --no-confirm sd@{{SD_VERSION}}

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

publish:
    cargo publish --all-features

update-version NEW_VERSION:
    sd "0.0.0-DEV" "{{NEW_VERSION}}" "Cargo.toml"
