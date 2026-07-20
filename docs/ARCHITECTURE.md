# Sentinel Architecture

## Architectural objective

Sentinel is designed around one reusable deterministic analysis engine.

The engine is exposed through two layers:

1. a single-command research and debugging interface;
2. a future long-running runtime interface for AI coding agents.

```text
                    sentinel-core
                         │
          ┌──────────────┴──────────────┐
          │                             │
sentinel analyze                  sentinel runtime
research/debug API                agent session protection
          │                             │
          └──── shared analysis engine ─┘
```

The first implementation builds and evaluates `sentinel-core` through `sentinel analyze`.

The runtime layer is added only after the analysis engine is sufficiently reliable.

## High-level research pipeline

```text
Command string
    ↓
Command parser
    ↓
Normalized action model
    ↓
Mode-specific context collection
    ↓
Deterministic rule evaluation
    ↓
Score aggregation
    ↓
Label, severity, and decision
    ↓
Text or JSON output
```

## Future runtime pipeline

```text
sentinel codex
    ↓
Session Manager
    ↓
Agent Process
    ↓
Agent Hook / Tool Adapter / Controlled Executor
    ↓
Structured Action Request
    ↓
Shared Analysis Engine
    ↓
Allow / Require Approval / Deny
    ↓
Executor
    ↓
Action Result Returned to Agent
    ↓
Wait for Next Action
```

Simply launching an agent as a child process is not sufficient to intercept its shell actions.

A real runtime integration must use an explicit interception mechanism such as:

* an agent-native hook;
* an MCP or tool adapter;
* a Sentinel-controlled shell executor;
* another documented action-routing interface.

## Repository structure

Initial structure:

```text
sentinel/
├── Cargo.toml
├── AGENTS.md
├── README.md
├── LICENSE
├── SECURITY.md
├── docs/
│   ├── PROJECT_SPEC.md
│   ├── RESEARCH_PLAN.md
│   ├── ARCHITECTURE.md
│   ├── THREAT_MODEL.md
│   └── CURRENT_MILESTONE.md
├── crates/
│   ├── sentinel-core/
│   └── sentinel-cli/
├── benchmark/
│   ├── cases/
│   ├── fixtures/
│   ├── scripts/
│   └── LABELING_GUIDE.md
├── scripts/
│   └── verify.sh
└── .github/
    └── workflows/
        └── ci.yml
```

Do not create more crates until a concrete separation is required.

## `sentinel-core`

`sentinel-core` contains all security and research logic.

Responsibilities:

* domain types;
* command parsing;
* command normalization;
* action representation;
* analysis modes;
* filesystem context collection;
* Git context collection;
* deterministic rule evaluation;
* score aggregation;
* decision mapping;
* stable analysis results.

It must not depend on:

* terminal colors;
* interactive prompts;
* agent-specific integrations;
* process execution;
* GitHub Actions;
* benchmark presentation code.

## `sentinel-cli`

During the research phase, `sentinel-cli` provides:

```bash
sentinel analyze "<command>"
```

Responsibilities:

* parse CLI arguments;
* choose analysis mode;
* choose text or JSON output;
* resolve the supplied working directory;
* call `sentinel-core`;
* print the result;
* return documented exit codes.

It must not contain security rules.

Later, `sentinel-cli` may also expose:

```bash
sentinel run -- <agent-command>
sentinel codex
sentinel claude
sentinel cursor
```

The runtime commands should orchestrate separate runtime modules rather than placing interception logic directly into CLI parsing code.

## Suggested future runtime components

When runtime work begins, add components only as needed.

Potential structure:

```text
crates/
├── sentinel-core/
├── sentinel-cli/
└── sentinel-runtime/
```

A separate `sentinel-runtime` crate is justified only when session management and execution become substantial.

### Session Manager

Responsibilities:

* create a session ID;
* start the configured agent;
* configure the supported action adapter;
* track the child process;
* continue until the agent exits;
* associate actions with the current session.

### Action Adapter

Responsibilities:

* receive shell-action requests;
* validate request structure;
* attach actor, session, action, and working-directory metadata;
* forward actions to the shared analysis engine;
* return decisions and execution results.

The adapter must not independently decide risk.

### Approval Controller

Responsibilities:

* display high-risk or critical actions;
* request explicit human approval;
* support allow-once or deny;
* avoid broad permanent approvals in the initial runtime release.

### Executor

Responsibilities:

* remain separate from analysis;
* execute only explicitly allowed actions;
* use the intended working directory;
* preserve exit code, stdout, and stderr;
* return results to the agent;
* never execute during benchmark runs.

## Domain types

Suggested initial types:

```rust
pub enum AnalysisMode {
    CommandOnly,
    FilesystemAware,
    GitAware,
}
```

```rust
pub enum RiskLabel {
    Legitimate,
    Dangerous,
}
```

```rust
pub enum Severity {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}
```

```rust
pub enum Decision {
    Allow,
    RequireApproval,
    Deny,
    AnalysisIncomplete,
}
```

```rust
pub struct AnalysisRequest {
    pub command: String,
    pub working_directory: PathBuf,
    pub mode: AnalysisMode,
}
```

```rust
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub score_delta: i32,
    pub message: String,
    pub matched_fragment: Option<String>,
}
```

```rust
pub struct ContextSummary {
    pub filesystem_used: bool,
    pub git_used: bool,
    pub context_complete: bool,
    pub warnings: Vec<String>,
}
```

```rust
pub struct AnalysisResult {
    pub command: String,
    pub mode: AnalysisMode,
    pub label: RiskLabel,
    pub severity: Severity,
    pub score: i32,
    pub decision: Decision,
    pub findings: Vec<Finding>,
    pub context_summary: ContextSummary,
}
```

These types should remain simple. They may be adjusted if implementation experience demonstrates a clearer design.

## Normalized action model

Rules should not independently parse raw strings.

Suggested representation:

```rust
pub struct ParsedCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub operators: Vec<ShellOperator>,
    pub children: Vec<ParsedCommand>,
}
```

Potential operators:

```rust
pub enum ShellOperator {
    Pipe,
    And,
    Or,
    Sequence,
}
```

The parser should support only the syntax required by the documented scope.

## Parser responsibilities

The parser should recognize:

* executable;
* arguments;
* repeated whitespace;
* quoted arguments;
* simple pipes;
* `&&`;
* `||`;
* `;`;
* common wrappers such as `sudo`;
* executable paths such as `/bin/rm`;
* split flags such as `-r -f`;
* common literal `sh -c` and `bash -c` forms where practical.

Normalize equivalent inputs:

```text
rm -rf src
rm -r -f src
/bin/rm -rf src
sudo rm -rf src
```

Do not perform arbitrary shell expansion.

Unsupported syntax must produce an explicit incomplete or unsupported result rather than silently defaulting to safe.

## Context collection

```rust
pub struct AnalysisContext {
    pub filesystem: Option<FilesystemContext>,
    pub git: Option<GitContext>,
}
```

### Command-only mode

```text
filesystem = None
git = None
```

This separation must be enforced so the research baseline cannot accidentally use context.

### Filesystem-aware mode

```text
filesystem = Some(...)
git = None
```

### Git-aware mode

```text
filesystem = Some(...)
git = Some(...)
```

## Filesystem context

Suggested types:

```rust
pub struct FilesystemContext {
    pub targets: Vec<TargetContext>,
    pub collection_complete: bool,
    pub warnings: Vec<String>,
}
```

```rust
pub struct TargetContext {
    pub raw_path: PathBuf,
    pub resolved_path: PathBuf,
    pub exists: bool,
    pub inside_project: bool,
    pub file_count: usize,
    pub directory_count: usize,
    pub total_size_bytes: u64,
    pub contains_source_files: bool,
    pub contains_credentials: bool,
    pub appears_generated: bool,
    pub is_symlink: bool,
}
```

Collection requirements:

* resolve paths relative to the requested working directory;
* avoid following symbolic links;
* impose traversal limits;
* use metadata where possible;
* handle inaccessible paths explicitly;
* never delete or modify files.

## Git context

Suggested type:

```rust
pub struct GitContext {
    pub is_repository: bool,
    pub repository_root: Option<PathBuf>,
    pub current_branch: Option<String>,
    pub is_protected_branch: bool,
    pub has_uncommitted_changes: bool,
    pub untracked_file_count: usize,
    pub tracked_targets: usize,
    pub modified_targets: usize,
    pub commits_at_risk: Option<usize>,
}
```

The first implementation may invoke read-only Git CLI commands.

Examples:

```bash
git rev-parse --show-toplevel
git branch --show-current
git status --porcelain
git ls-files
git rev-list --count
```

Reasons to prefer the Git CLI initially:

* simpler implementation;
* lower dependency burden;
* easier debugging;
* no native `libgit2` build requirements;
* behavior consistent with installed Git.

## Rule engine

Use a small static rule interface.

```rust
pub trait Rule {
    fn id(&self) -> &'static str;

    fn evaluate(
        &self,
        command: &ParsedCommand,
        context: &AnalysisContext,
    ) -> Vec<Finding>;
}
```

The initial version should use a fixed list of built-in rules.

Do not implement:

* dynamic plugins;
* runtime rule downloads;
* user-written code execution;
* a rule marketplace.

## Scoring and decisions

Rules produce deterministic score changes.

Example:

```text
recursive forced deletion                 +30
known generated directory                 -30
source files affected                     +35
tracked files affected                    +25
modified files affected                   +30
credential path affected                  +60
protected branch force push               +50
```

The result builder maps:

```text
findings
    ↓
score
    ↓
severity
    ↓
binary label
    ↓
runtime decision
```

The threshold must exist in one clearly documented location.

## Shared analysis API

Conceptual entry point:

```rust
pub fn analyze(
    request: &AnalysisRequest,
) -> Result<AnalysisResult, AnalysisError> {
    let parsed = parse_command(&request.command)?;
    let context = collect_context(request, &parsed)?;
    let findings = evaluate_rules(&parsed, &context);
    Ok(build_result(request, findings, context))
}
```

The runtime layer must call this same function or the same lower-level analysis pipeline.

It must not implement a separate risk engine.

## Research interface

```bash
sentinel analyze "rm -rf src" \
  --mode git \
  --format json \
  --working-directory /workspace/project
```

This interface:

* never executes;
* produces stable machine-readable output;
* is safe for automated benchmark use;
* is also useful for debugging individual findings.

## Benchmark architecture

The benchmark layer should remain separate from Sentinel’s Rust core.

Suggested scripts:

```text
benchmark/scripts/
├── generate_cases.py
├── validate_cases.py
├── setup_fixture.py
├── run_benchmark.py
└── analyze_results.py
```

The benchmark runner should invoke the release CLI:

```bash
python benchmark/scripts/run_benchmark.py \
  --sentinel ./target/release/sentinel \
  --cases benchmark/cases/test.jsonl \
  --output results/final.json
```

For each case:

1. create a temporary fixture;
2. initialise Git state where needed;
3. invoke `sentinel analyze`;
4. record result and latency;
5. destroy the fixture.

It must never call the future executor.

## CI architecture

GitHub Actions should run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --release
```

Local verification should use the same commands through:

```bash
./scripts/verify.sh
```

Later, CI may also validate:

* benchmark JSONL files;
* Python tests;
* fixture generation;
* schema compatibility.

## Errors

Errors should distinguish:

* empty input;
* invalid command input;
* unsupported syntax;
* filesystem context unavailable;
* Git context unavailable;
* partial context;
* analysis failure;
* future adapter failure;
* future execution failure.

Context failure must not automatically produce an `Allow` decision.

## Deferred architecture

Do not implement during the research release:

* agent process interception;
* command execution;
* approval prompts;
* session manager;
* web API;
* web dashboard;
* database;
* cloud service;
* async runtime without a demonstrated need;
* dynamic plugins;
* OS kernel hooks;
* eBPF;
* `ptrace`;
* seccomp;
* full process monitoring;
* LLM-based final risk classification.

## Architectural release sequence

### `v0.1.0-research`

Contains:

* shared analysis core;
* `sentinel analyze`;
* all analysis modes;
* benchmark and results.

### Later runtime release

Contains:

* session manager;
* structured action adapter;
* approval controller;
* executor;
* generic runtime command.

### Later Codex integration release

Contains:

```bash
sentinel codex
```

and demonstrates multiple protected actions within one real coding session.

## Design philosophy

The architecture intentionally prioritizes human maintainability over
maximum flexibility.

Sentinel is expected to evolve for years.

Therefore:

- simple modules are preferred over generic frameworks;
- explicit data flow is preferred over hidden behavior;
- composition is preferred over inheritance;
- concrete types are preferred over unnecessary generic abstractions;
- readability is preferred over reducing line count;
- straightforward implementations are preferred over clever optimizations.

Every major module should have one obvious responsibility.

If a contributor cannot understand a module within roughly twenty minutes,
the architecture should be simplified.