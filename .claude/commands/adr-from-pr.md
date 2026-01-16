# Generate ADR from Pull Request

Analyze a PR and generate an Architecture Decision Record capturing the architectural decisions made.

## Usage

- `/adr-from-pr` - Analyze current PR (if on a PR branch)
- `/adr-from-pr <number>` - Analyze specific PR by number

## Arguments

$ARGUMENTS - Optional PR number (e.g., "123")

## Instructions

1. **Get PR information**:
   - If a PR number is provided in $ARGUMENTS, use that
   - Otherwise, detect current branch and find associated PR
   - Run: `gh pr view <number> --json title,body,number,headRefName,baseRefName`

2. **Get PR diff**:
   - Run: `gh pr diff <number>`
   - Identify files changed and the nature of changes

3. **Analyze for architectural decisions**:
   Look for patterns that indicate architectural decisions:
   - New dependencies added (Cargo.toml changes)
   - New module structure or reorganization
   - API design patterns introduced
   - Error handling approaches
   - New abstractions or traits defined
   - Performance optimizations
   - Security-related changes
   - Breaking changes to public APIs

4. **Check existing ADRs**:
   - Read `docs/adr/README.md` to find current ADR numbering
   - Read existing ADRs to understand context and avoid duplicates
   - Determine next ADR number (NNNN format, e.g., 0002)

5. **Generate ADR draft**:
   - Use the template from `docs/adr/template.md`
   - Fill in all sections based on PR analysis
   - Extract validation rules if the decision can be enforced via patterns
   - Title should be descriptive of the decision, not the PR

6. **Write ADR file**:
   - Write to `docs/adr/NNNN-title-with-dashes.md`
   - Use lowercase with dashes for the filename

7. **Update README index**:
   - Add the new ADR to the index table in `docs/adr/README.md`

8. **Present for review**:
   - Show the generated ADR content
   - Ask if any sections need refinement
   - Confirm before committing

## Output

- ADR file created at `docs/adr/NNNN-title.md`
- README.md index updated
- Summary of the decision captured

## Notes

- Not every PR needs an ADR - only those with significant architectural decisions
- If the PR is a simple bug fix or minor change, inform the user that no ADR is needed
- Focus on the "why" behind decisions, not just the "what"
- Validation rules should only be added if the decision can be meaningfully enforced
