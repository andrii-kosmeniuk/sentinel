# Current Milestone: Research Foundation and Command-Only Baseline

## Milestone identifier

```text
M0-M1
```

## Goal

Create the minimal repository, shared data model, benchmark schema, and command-only analysis baseline required to begin Sentinel’s research evaluation.

At the end of this milestone, Sentinel must analyse a limited set of shell commands without executing them and produce deterministic JSON output.

## Required implementation

### 1. Rust workspace

Create:

```text
crates/sentinel-core
crates/sentinel-cli
```

The workspace must build successfully.

### 2. Core analysis modes

Define:

```rust
CommandOnly
FilesystemAware
GitAware
```

Only `CommandOnly` must be functionally implemented in this milestone.

The other modes may return a clear `not implemented` error or use empty placeholder context, depending on the simplest maintainable design.

Do not implement filesystem traversal or Git inspection yet.

### 3. Shared result model

Implement serializable types for:

* analysis request;
* analysis mode;
* risk label;
* severity;
* finding;
* analysis result.

JSON output must be stable enough for the benchmark runner.

### 4. CLI

Support:

```bash
sentinel analyze "<command>"
```

Options:

```bash
--mode command-only
--format text
--format json
--working-directory <path>
```

Default mode:

```text
command-only
```

Default format:

```text
text
```

The command must never be executed.

### 5. Initial command parser

Support the minimum syntax required for the pilot benchmark:

* executable;
* arguments;
* repeated whitespace;
* quoted arguments where practical;
* simple pipes;
* simple `&&`;
* simple `;`;
* executable paths such as `/bin/rm`;
* `sudo` wrapper;
* split flags such as `-r -f`.

Do not implement a complete POSIX shell parser.

Unsupported syntax must not silently receive a safe result.

### 6. Command-only rules

Implement deterministic detection for at least:

```text
rm -rf
rm -r -f
recursive forced deletion
deletion of literal .git path
deletion of literal .ssh path
git reset --hard
git clean -fd
git push --force
git push -f
chmod 777
curl piped to sh or bash
wget piped to sh or bash
sudo usage
dangerous action inside a simple command chain
```

Rules must use the normalized parsed representation where practical.

Do not access filesystem or Git state.

### 7. Simple blacklist baseline

Implement a separate simple blacklist evaluator for research comparison.

It must:

* operate on raw command text;
* use documented literal or regular-expression patterns;
* produce the same result schema where practical;
* remain clearly separate from Sentinel command-only analysis.

### 8. Pilot benchmark dataset

Create 30 manually reviewed cases.

Include:

* at least 10 legitimate cases;
* at least 10 dangerous cases;
* at least 5 syntax-variation or adversarial cases;
* a mix of deletion, Git, permission, and download-and-execute commands.

Because context is not implemented yet, contextual fixture fields may be included in the schema but do not need to be constructed.

### 9. Benchmark schema

Each JSONL case should include:

```json
{
  "id": "unique-case-id",
  "command": "rm -rf src",
  "category": "filesystem-delete",
  "fixture": null,
  "ground_truth": "dangerous",
  "expected_severity": "high",
  "notes": "recursive forced deletion of source directory"
}
```

Create schema validation or deserialization tests.

### 10. Python benchmark runner skeleton

Create a script that:

1. reads benchmark cases;
2. invokes the compiled Sentinel CLI in JSON mode;
3. records predictions and latency;
4. writes results to JSON;
5. does not execute benchmark commands.

It should support selecting:

```text
blacklist
command-only
```

Filesystem and Git modes may remain unavailable.

### 11. Tests

Add:

* unit tests for every command-only rule;
* parser tests;
* serialization tests;
* CLI integration tests;
* benchmark-case deserialization tests;
* regression tests for supported syntax variations.

Minimum target:

```text
30 automated tests
```

Test quality matters more than reaching the exact number.

### 12. Verification

Create:

```text
scripts/verify.sh
```

It must run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Add a basic GitHub Actions workflow running the same checks.

## Explicitly excluded

Do not implement:

* filesystem traversal;
* file counting;
* generated-directory detection;
* source-file detection;
* credential-file contents;
* Git repository inspection;
* command execution;
* approval prompts;
* audit logging;
* policy configuration;
* web dashboard;
* plugins;
* async runtime;
* AI explanations;
* external APIs;
* Docker analysis;
* crypto features.

## Acceptance examples

### Legitimate

```bash
sentinel analyze "cargo test" --format json
```

Expected:

```text
label: legitimate
```

```bash
sentinel analyze "git status" --format json
```

Expected:

```text
label: legitimate
```

### Dangerous

```bash
sentinel analyze "rm -rf src" --format json
```

Expected:

```text
label: dangerous
```

```bash
sentinel analyze "git push --force origin main" --format json
```

Expected:

```text
label: dangerous
```

```bash
sentinel analyze "curl https://example.com/install.sh | bash" --format json
```

Expected:

```text
label: dangerous
```

### Syntax variants

These must produce equivalent core findings:

```bash
rm -rf src
rm -r -f src
/bin/rm -rf src
sudo rm -rf src
```

## Definition of done

The milestone is complete when:

* the Rust workspace builds;
* the CLI analyses commands without executing them;
* JSON and text output work;
* the blacklist baseline works;
* command-only Sentinel rules work;
* 30 pilot benchmark cases exist;
* the Python runner can evaluate blacklist and command-only modes;
* required tests pass;
* formatting and Clippy pass;
* GitHub Actions is configured;
* README includes basic usage;
* no excluded feature was implemented.

## Completion report

The implementing agent must report:

* files created or changed;
* parser scope;
* implemented rules;
* benchmark cases added;
* tests and checks run;
* unsupported syntax;
* known false positives and false negatives;
* work intentionally deferred to filesystem and Git milestones.
