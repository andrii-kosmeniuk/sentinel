# Sentinel Agent Instructions

## Project

Sentinel is an open-source, context-aware shell-command risk analyser for AI coding agents and developers.

It evaluates commands before execution using deterministic rules and optional filesystem and Git context.

## Research question

How accurately can Sentinel distinguish dangerous from legitimate shell commands, and to what extent does filesystem and Git context reduce false positives compared with command-only rules?

## Experimental variants

Preserve a clean separation between:

1. Command-only analysis
2. Command plus filesystem context
3. Command plus filesystem and Git context

All variants must share the same parser, result schema, benchmark cases, and evaluation pipeline.

## Core safety principles

* Analysis must never execute the command being analysed.
* Security decisions must be deterministic.
* AI may help explain findings but must not determine the final risk level.
* Never weaken a rule or change benchmark labels merely to improve reported metrics.
* Never run destructive benchmark commands against the developer's real filesystem.
* Use isolated temporary fixtures for filesystem and Git tests.
* Do not read sensitive file contents when path metadata is sufficient.
* Treat symbolic links, path traversal, shell wrappers, and command chaining as security-sensitive.

## Simplicity first

* Implement only the current milestone.
* Prefer the smallest maintainable design.
* Do not build speculative features.
* Do not introduce a plugin system, async runtime, web server, database, or AI integration unless explicitly required.
* Prefer concrete types over unnecessary generics.
* Avoid macros unless they clearly reduce repetition.
* Do not create abstractions with only one real implementation.
* Prefer the Rust standard library where practical.
* Do not add dependencies without explaining why they are necessary.

## Architecture

Keep these concerns separate:

* command parsing;
* normalized action representation;
* context collection;
* rule evaluation;
* policy decisions;
* CLI presentation;
* command execution;
* benchmark evaluation.

The core analysis library must not depend on terminal UI code.

The command-only variant must not access filesystem or Git context indirectly.

## Research integrity

* Keep development, validation, and held-out test cases separate.
* Do not tune rules or thresholds using the held-out test set.
* Record benchmark configuration, Sentinel version, and dataset version.
* Preserve reproducibility through fixed fixtures and deterministic outputs.
* Report false positives, false negatives, latency, and per-category results.
* Document unsupported shell syntax and known bypasses honestly.
* Never claim Sentinel makes arbitrary agent execution safe.

## Working style

Before implementing a non-trivial task:

1. Inspect the relevant code and documentation.
2. State assumptions and identify unclear requirements.
3. Propose the smallest implementation plan.
4. Define how each step will be verified.

When editing:

* Change only files required by the task.
* Do not refactor unrelated code.
* Match the existing style.
* Remove only unused code created by the current change.
* Preserve public interfaces unless the milestone explicitly requires a change.

If a verification failure reveals an ambiguous specification or would require changing a public contract, stop and explain:

* expected behavior;
* actual behavior;
* relevant implementation details;
* two or three possible resolutions;
* the recommended resolution.

Routine compiler errors, lint failures, and straightforward test failures should be fixed without interruption.

## Testing requirements

For every behavioral change:

* add or update tests;
* include safe and dangerous cases;
* include malformed or adversarial inputs where relevant;
* add a regression test for every confirmed bypass;
* do not modify a valid test merely to make implementation pass.

Use isolated temporary directories and repositories for contextual tests.

## Required checks

Before declaring a task complete, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If benchmark code changes, also run the relevant benchmark validation or dataset-schema checks defined in the repository.

## Completion report

Always summarize:

* files changed;
* behavior implemented;
* important design decisions;
* tests and checks run;
* known limitations;
* work intentionally excluded from the milestone.

## Current source of truth

Read these before starting implementation:

* `docs/PROJECT_SPEC.md`
* `docs/RESEARCH_PLAN.md`
* `docs/ARCHITECTURE.md`
* `docs/THREAT_MODEL.md`
* `docs/CURRENT_MILESTONE.md`

If these documents conflict, stop and identify the conflict before changing code.

## Human-first architecture

Sentinel is a long-lived open-source project intended to be maintained by human engineers.

Every implementation should optimize for:

- readability;
- simplicity;
- maintainability;
- explicit behavior;
- deterministic logic.

Avoid clever implementations that reduce readability.

Prefer straightforward code over compact or highly abstract code.

A new contributor should be able to understand an important module within
15–20 minutes.

Every public function should have a clear responsibility.

Avoid introducing abstractions until at least two concrete implementations
exist.

The code should remain maintainable without AI assistance.

## AI-generated code

Sentinel welcomes AI-assisted development, but AI-generated code must meet
the same engineering standards as human-written code.

Every contribution should prioritize long-term maintainability over short-term
development speed.

Generated code must:

- be understandable by a human reviewer without AI assistance;
- follow the project's existing architecture and coding style;
- avoid unnecessary abstraction or clever implementations;
- keep modules small and responsibilities clear;
- include appropriate tests when behavior changes;
- avoid speculative extensibility ("build it when needed");
- preserve deterministic and explicit behavior.

When choosing between a simpler and a more abstract implementation, prefer
the simpler solution unless the additional complexity is clearly justified.

The goal is not to maximize the amount of AI-generated code.

The goal is to build a codebase that human engineers can confidently
understand, review, debug, and maintain for years.