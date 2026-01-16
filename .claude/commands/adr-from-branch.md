# Generate ADR from Branch

Analyze the commits and changes on a branch to generate an Architecture Decision Record.

## Usage

- `/adr-from-branch` - Analyze current branch
- `/adr-from-branch <branch-name>` - Analyze specific branch

## Arguments

$ARGUMENTS - Optional branch name (defaults to current branch)

## Instructions

1. **Determine branch**:
   - If branch name provided in $ARGUMENTS, use that
   - Otherwise, get current branch: `git branch --show-current`

2. **Get commits on branch**:
   - Find base branch (usually main): `git merge-base main <branch>`
   - List commits: `git log main..<branch> --oneline`
   - Get detailed commit messages: `git log main..<branch> --format="%H%n%s%n%b%n---"`

3. **Get cumulative diff**:
   - Run: `git diff main...<branch>`
   - Identify all files changed and patterns introduced

4. **Analyze for architectural decisions**:

   **Sources to consider:**
   - **Commit messages**: Often explain the reasoning behind changes
   - **Current conversation**: Use context from the ongoing Claude conversation if relevant - this may contain discussion about alternatives, tradeoffs, and the reasoning behind the approach taken
   - **Code changes**: The diff itself shows what was implemented
   - **Related PR** (if exists): Check `gh pr list --head <branch>` for associated PR description

   **Patterns that indicate architectural decisions:**
   - New dependencies added (Cargo.toml changes)
   - New module structure or reorganization
   - API design patterns introduced
   - Error handling approaches
   - New abstractions or traits defined
   - Performance optimizations
   - Security-related changes
   - Breaking changes to public APIs
   - Configuration changes
   - Build system modifications

5. **Check existing ADRs**:
   - Read `docs/adr/README.md` to find current ADR numbering
   - Read existing ADRs to understand context and avoid duplicates
   - Determine next ADR number (NNNN format, e.g., 0002)

6. **Generate ADR draft**:
   - Use the template from `docs/adr/template.md`
   - Fill in all sections based on branch analysis
   - Use commit messages as context for the decision
   - Extract validation rules if the decision can be enforced via patterns
   - Title should be descriptive of the decision

7. **Write ADR file**:
   - Write to `docs/adr/NNNN-title-with-dashes.md`
   - Use lowercase with dashes for the filename

8. **Update README index**:
   - Add the new ADR to the index table in `docs/adr/README.md`

9. **Present for review**:
   - Show the generated ADR content
   - Ask if any sections need refinement
   - Confirm before committing

## Output

- ADR file created at `docs/adr/NNNN-title.md`
- README.md index updated
- Summary of the decision captured

## Notes

- Not every branch needs an ADR - only those with significant architectural decisions
- If the branch is a simple bug fix or minor change, inform the user that no ADR is needed
- Focus on the "why" behind decisions, not just the "what"
- Commit messages often contain valuable context about decisions
- Validation rules should only be added if the decision can be meaningfully enforced
