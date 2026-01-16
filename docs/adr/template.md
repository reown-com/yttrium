# ADR-NNNN: Title

## Status

[Proposed | Accepted | Deprecated | Superseded by ADR-XXXX]

## Date

YYYY-MM-DD

## Context

What is the issue we're facing? What forces are at play?

Describe the technical and business context. Include:
- Current state of the codebase
- Problems or limitations being addressed
- External factors (dependencies, standards, constraints)

## Decision Drivers

- [Driver 1: e.g., "Need to support cross-platform compilation"]
- [Driver 2: e.g., "Must maintain backwards compatibility"]
- [Driver 3: e.g., "Performance is critical for mobile devices"]

## Considered Options

### Option 1: [Name]

Description of the first option.

**Pros:**
- Pro 1
- Pro 2

**Cons:**
- Con 1
- Con 2

### Option 2: [Name]

Description of the second option.

**Pros:**
- Pro 1
- Pro 2

**Cons:**
- Con 1
- Con 2

### Option 3: [Name]

Description of the third option.

**Pros:**
- Pro 1
- Pro 2

**Cons:**
- Con 1
- Con 2

## Decision

We decided on **Option X** because [reasoning].

Explain:
- Why this option was chosen over alternatives
- Key tradeoffs accepted
- How this addresses the decision drivers

## Consequences

### Positive

- [Benefit 1]
- [Benefit 2]
- [Benefit 3]

### Negative

- [Tradeoff 1]
- [Tradeoff 2]

### Neutral

- [Side effect that's neither good nor bad]

## Validation Rules

<!--
AI-enforceable rules extracted from this decision.
These rules are checked automatically on PRs and can be run locally with /validate-adr.
Remove this section if the decision cannot be enforced via code patterns.
-->

```yaml
rules:
  - id: "NNNN-rule-1"
    description: "Brief description of what this rule enforces"
    pattern: "regex pattern to detect violations"
    action: "warn"  # or "error" for blocking violations
    applies_to: ["*.rs"]  # glob patterns for files to check

  - id: "NNNN-rule-2"
    description: "Another rule from this decision"
    pattern: "another regex pattern"
    action: "error"
    applies_to: ["crates/yttrium/src/**/*.rs"]
```

## Implementation Notes

<!-- Optional: Add notes about how to implement this decision -->

- Migration steps if applicable
- Code examples showing the preferred pattern
- Links to relevant documentation

## Related

- Supersedes: [ADR-XXXX] (if this replaces an older decision)
- Related to: [ADR-YYYY, ADR-ZZZZ] (if connected to other decisions)
- References: [Links to relevant external documentation]
