# DINAH — automation and bot policy

## Repository authority

- Bots may analyze, propose changes, create branches and open Pull Requests.
- No bot may merge Pull Requests or enable auto-merge. Merging code and configuration changes is the responsibility of a person accountable for the project.
- Operational exception: the benchmark archiving workflow keeps its existing behavior and may automatically commit the artifacts it generates to `main`, once CI and benchmarks have passed for the same commit. This exception does not authorize other bots to write to `main`.
- Bots are not granted administration permissions. Every integration must use the minimum permissions it needs.
- Bot-generated changes must explain the motive, scope, risks and the checks that were run.
- The artifact archiving workflow stays automatic: after the required workflows pass for the same SHA, it saves artifacts under `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/` and commits the changes to `main`.
- A roadmap task is not marked complete without reviewable evidence in code, documentation, tests or results.

## Validation

- Code changes must pass the project CI before merge.
- Dependency updates must be reviewed like normal changes; auto-merge is not enabled.
- If a check fails or was not run, the PR must say so and must not present itself as ready to integrate.
- Bots must not modify credentials, secrets, workflow permissions or protection rules as part of an ordinary PR.

## Current GitHub limitation

The repository is private and the GitHub API reports that branch protection and rulesets require a higher plan or a public repository. Until this is resolved, the documented policy is not equivalent to a technical barrier imposed by GitHub. Automation credentials must not have administration permission or direct write access to `main`; any additional configuration must be verified separately.

## Benchmark archiving

The existing result-archiving workflow remains unchanged: it waits for CI and benchmarks to have completed successfully for the same commit, downloads and organizes the bundles, and automatically commits the artifacts under `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/`. Do not turn this archiving into a Pull Request or require manual approval for each run.
