# Sentinel Project Specification

## Overview

Sentinel is an open-source runtime safety tool for AI coding agents and developers.

Its long-term purpose is to protect an entire coding-agent session by evaluating shell actions before execution.

The intended primary user experience is:

```bash
sentinel codex
sentinel claude
sentinel cursor
```

or, generically:

```bash
sentinel run -- <agent-command>
```

During the session, Sentinel should analyse every supported shell action requested by the agent and return one of the following decisions:

```text
allow
require_approval
deny
analysis_incomplete
```

Before building the runtime integration, Sentinel will first be implemented as a deterministic analysis engine with a single-command research interface:

```bash
sentinel analyze "rm -rf src"
```

This interface is required for development, testing, benchmarking, debugging, and reproducible research.

## Research question

How accurately can Sentinel distinguish dangerous from legitimate shell commands, and to what extent does filesystem and Git context reduce false positives compared with command-only rules?

## Core problem

AI coding agents may receive permission to:

* execute shell commands;
* modify files;
* change Git history;
* install dependencies;
* run Docker commands;
* access credentials;
* interact with deployment environments.

The risk of a command cannot always be determined from its text alone.

For example:

```bash
rm -rf target
```

may be a legitimate cleanup operation in a Rust project.

However:

```bash
rm -rf src
```

may destroy source code.

Similarly:

```bash
git reset --hard
```

may be relatively harmless in a clean disposable branch, but dangerous when the repository contains uncommitted work.

Sentinel therefore evaluates commands using progressively richer context.

## Product interfaces

### 1. Research and analysis interface

```bash
sentinel analyze "<command>"
```

This interface:

* evaluates exactly one proposed command;
* never executes the command;
* supports deterministic text and JSON output;
* is used by the benchmark runner;
* is the first implementation milestone;
* uses the same core engine as the future runtime interface.

Example:

```bash
sentinel analyze "git reset --hard" \
  --mode git \
  --format json
```

### 2. Runtime session interface

```bash
sentinel codex
sentinel claude
sentinel cursor
```

Generic form:

```bash
sentinel run -- <agent-command> [arguments...]
```

This interface will:

1. start the selected coding agent;
2. establish a Sentinel-controlled command-execution path;
3. remain active for the entire agent session;
4. receive each supported shell-action request;
5. analyse the action using the shared engine;
6. allow, deny, or request approval;
7. execute only approved actions;
8. return the result to the agent;
9. continue until the agent exits.

The runtime interface is not part of the first research release.

## Development sequence

### Phase 1 — Research engine

Implement:

* command parser;
* normalized action model;
* deterministic rules;
* command-only analysis;
* filesystem-aware analysis;
* Git-aware analysis;
* stable JSON output;
* benchmark dataset;
* benchmark runner;
* evaluation results.

Primary interface:

```bash
sentinel analyze
```

### Phase 2 — Generic runtime layer

Implement:

* session manager;
* action IDs;
* session IDs;
* approval workflow;
* command executor;
* audit events;
* structured action adapter.

Primary interface:

```bash
sentinel run -- <agent-command>
```

### Phase 3 — First real agent integration

Implement one reliable agent integration, initially targeting Codex where practical.

Primary interface:

```bash
sentinel codex
```

### Phase 4 — Additional integrations

Potential later integrations:

```bash
sentinel claude
sentinel cursor
```

Support must only be claimed after the integration demonstrably routes supported shell actions through Sentinel.

## Analysis modes

All interfaces must use the same analysis modes.

### Command-only

Uses only the proposed command and its parsed structure.

It may inspect:

* executable;
* arguments;
* flags;
* pipes;
* command chains;
* wrappers;
* literal paths;
* known destructive operations.

It must not read filesystem or Git state.

### Filesystem-aware

Adds filesystem metadata.

It may inspect:

* whether target paths exist;
* resolved target locations;
* whether paths are inside the project;
* file and directory counts;
* approximate size;
* probable source directories;
* probable generated directories;
* credential paths;
* system paths;
* symbolic-link status.

It must not inspect Git state.

### Git-aware

Includes command and filesystem analysis plus Git repository state.

It may inspect:

* repository root;
* current branch;
* protected-branch status;
* tracked files affected;
* modified files affected;
* untracked files affected;
* dirty working-tree status;
* approximate commits at risk.

## Initial command scope

The first research version should focus on command families where context meaningfully affects risk.

### Filesystem deletion

Examples:

```bash
rm -rf target
rm -rf src
rm -rf .git
rm -rf ~/.ssh
```

### Git operations

Examples:

```bash
git reset --hard
git reset --hard HEAD~3
git clean -fd
git push --force origin main
git branch -D feature
```

### Permission changes

Examples:

```bash
chmod 777 file
chmod -R 777 .
```

### Download and execute

Examples:

```bash
curl URL | bash
curl URL | sh
wget -qO- URL | bash
```

### Command chaining and wrappers

Examples:

```bash
sudo rm -rf src
echo done && rm -rf src
bash -c "rm -rf src"
```

Docker, Kubernetes, cloud, crypto, and remote execution support are deferred unless explicitly added to a later milestone.

## Risk output

The primary research label is binary:

```text
legitimate
dangerous
```

Sentinel may also return:

```text
safe
low
medium
high
critical
```

Suggested default mapping:

```text
safe, low, medium → legitimate
high, critical    → dangerous
```

The final threshold must be documented and frozen before held-out evaluation.

## Analysis result

Every analysis should return a stable structure similar to:

```json
{
  "command": "rm -rf src",
  "mode": "git",
  "label": "dangerous",
  "severity": "critical",
  "score": 90,
  "decision": "deny",
  "findings": [
    {
      "rule_id": "filesystem.recursive-delete",
      "severity": "high",
      "score_delta": 30,
      "message": "The command recursively deletes a directory.",
      "matched_fragment": "rm -rf src"
    },
    {
      "rule_id": "git.modified-files-at-risk",
      "severity": "critical",
      "score_delta": 30,
      "message": "Modified Git-tracked files may be lost.",
      "matched_fragment": null
    }
  ],
  "context_summary": {
    "filesystem_used": true,
    "git_used": true,
    "context_complete": true
  }
}
```

During the research phase, `decision` describes what the future runtime layer should do. No command is executed.

## Runtime action request

The future runtime layer should use a structured request:

```json
{
  "session_id": "session-123",
  "action_id": "action-008",
  "actor": {
    "type": "ai-agent",
    "name": "codex"
  },
  "action_type": "shell.execute",
  "command": "git reset --hard",
  "working_directory": "/workspace/project"
}
```

The analysis engine must not depend on the specific agent integration.

## Core principles

### Deterministic security decisions

LLMs must not assign the final risk level or execution decision.

AI may later explain deterministic findings in natural language.

### Analysis without side effects

`sentinel analyze` must never execute the command being analysed.

### Contextual evaluation

Context may increase or decrease risk.

For example:

```text
recursive forced deletion                 +30
known generated directory                 -30
source files affected                     +35
tracked files affected                    +25
modified files affected                   +30
credential path affected                  +60
```

### Explicit uncertainty

Unsupported syntax or incomplete context must not silently result in a safe classification.

### Shared implementation

The analysis interface, benchmark runner, and runtime interface must use the same:

* parser;
* normalized action model;
* context collectors;
* rules;
* scoring;
* result schema.

## Safety requirements

* Never execute benchmark commands.
* Use isolated temporary directories for filesystem tests.
* Use disposable temporary Git repositories.
* Do not follow symbolic links during recursive analysis.
* Do not read sensitive file contents when metadata is sufficient.
* Apply traversal and time limits.
* Use only read-only Git operations for context collection.
* Never automatically execute dangerous actions in the future runtime layer.
* Missing context must be reported explicitly.
* Security-critical failures must not default to allow.

## Research requirements

The research system must compare:

```text
B0: simple blacklist
B1: Sentinel command-only
B2: Sentinel + filesystem context
B3: Sentinel + filesystem + Git context
```

It must report:

* precision;
* recall;
* F1;
* false-positive rate;
* false-negative rate;
* relative false-positive reduction;
* per-category results;
* adversarial bypass rate;
* median, p95, and p99 latency.

## First research release

Suggested release:

```text
v0.1.0-research
```

It should include:

* `sentinel analyze`;
* all three analysis modes;
* deterministic findings and scoring;
* text and JSON output;
* blacklist baseline;
* benchmark dataset;
* benchmark runner;
* evaluation scripts;
* automated tests;
* threat model;
* documented limitations.

It does not include:

* `sentinel codex`;
* command execution;
* approval prompts;
* session management;
* agent interception;
* runtime monitoring;
* OS sandboxing;
* web dashboard;
* hosted service;
* AI-generated security decisions.

## First runtime release

A later release may be:

```text
v0.2.0-runtime
```

or:

```text
v0.3.0-runtime
```

depending on the release history.

It should include:

* session management;
* structured action interception;
* approval workflow;
* safe execution boundary;
* action and session audit records;
* at least one controlled runtime demonstration.

## First agent integration release

A later release may provide:

```bash
sentinel codex
```

This release must demonstrate that multiple supported shell actions during one Codex session are routed through Sentinel before execution.

## Non-goals

The initial project does not claim to:

* make arbitrary AI agents safe;
* understand every shell program;
* interpret arbitrary script semantics;
* stop malicious binaries;
* replace an operating-system sandbox;
* monitor every subprocess;
* protect remote infrastructure;
* infer user intent perfectly;
* support all agents through process launching alone.

### Human maintainability

Sentinel is intended to be maintained primarily by human engineers.

The project should not optimize for producing the smallest amount of code
or the most "AI-generated" architecture.

Instead it should optimize for:

- clear module boundaries;
- explicit data flow;
- minimal abstraction;
- predictable behavior;
- ease of debugging;
- ease of extension.

Every architectural decision should be understandable without requiring
an LLM to explain it.