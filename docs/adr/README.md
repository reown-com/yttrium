# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for the yttrium project.

## What is an ADR?

An Architecture Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences.

## Why ADRs?

- **Documentation**: Record the reasoning behind decisions for future reference
- **Communication**: Help team members understand architectural choices
- **Onboarding**: Help new contributors understand why things are the way they are
- **Validation**: Machine-readable rules allow automated enforcement of decisions

## ADR Format

We use the MADR (Markdown ADR) format with an additional **Validation Rules** section that enables automated compliance checking.

Each ADR includes:
- **Status**: Proposed, Accepted, Deprecated, or Superseded
- **Context**: The problem or situation that led to the decision
- **Decision Drivers**: Key factors that influenced the decision
- **Considered Options**: Alternatives that were evaluated
- **Decision**: What was decided and why
- **Consequences**: Positive and negative outcomes
- **Validation Rules**: Machine-readable rules for automated enforcement

## ADR Lifecycle

```
Proposed → Accepted → [Deprecated | Superseded by ADR-XXXX]
```

1. **Proposed**: Draft ADR open for discussion
2. **Accepted**: Decision has been agreed upon and is in effect
3. **Deprecated**: Decision is no longer relevant (technology removed, etc.)
4. **Superseded**: Replaced by a newer ADR

## Commands

| Command | Description |
|---------|-------------|
| `/adr-from-pr` | Generate ADR from a pull request |
| `/adr-from-branch` | Generate ADR from current branch changes |
| `/validate-adr` | Validate code against ADR rules |
| `/adr-list` | List all ADRs with status |
| `/adr-supersede` | Mark an ADR as superseded |
| `/adr-search` | Search ADRs by topic |

## Creating an ADR

### Automatic Generation

Use Claude Code commands to generate ADRs from code changes:

```bash
# From a PR
/adr-from-pr 123

# From current branch
/adr-from-branch
```

### Manual Creation

1. Copy `template.md` to `NNNN-title-with-dashes.md`
2. Fill in all sections
3. Add validation rules if the decision can be enforced
4. Submit PR for review

## Validation Rules

Each ADR can include machine-readable validation rules in YAML format:

```yaml
rules:
  - id: "0001-rule-1"
    description: "Brief rule description"
    pattern: "regex pattern to match violations"
    action: "warn | error"
    applies_to: ["*.rs", "crates/pay/**"]
```

These rules are automatically checked:
- **On PRs**: GitHub Action validates all changes
- **Locally**: Use `/validate-adr` command

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| | *No ADRs yet* | | |
