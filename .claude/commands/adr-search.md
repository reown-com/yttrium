# Search Architecture Decision Records

Search ADRs by topic, keyword, or content.

## Usage

- `/adr-search "<query>"` - Search all ADR content
- `/adr-search "<query>" --section <section>` - Search specific section only

## Arguments

$ARGUMENTS - Required: search query, Optional: `--section <section>`

Available sections: `context`, `decision`, `consequences`, `rules`

## Instructions

1. **Parse arguments**:
   - Extract search query from $ARGUMENTS
   - Extract optional section filter

2. **Read all ADRs**:
   - List all files in `docs/adr/` matching pattern `[0-9][0-9][0-9][0-9]-*.md`
   - Read content of each ADR

3. **Search each ADR**:
   - If section specified, search only that section
   - Otherwise, search entire ADR content
   - Use case-insensitive matching
   - Track which sections contain matches

4. **Rank results**:
   - Title matches rank highest
   - Decision section matches rank high
   - Context matches rank medium
   - Other matches rank lower

5. **Display results**:

   ```
   Search Results for "error handling"
   ===================================

   Found 3 matching ADRs:

   1. ADR-0001: Error Handling Strategy [Accepted]
      Match in: Title, Decision, Context
      > "Use Result types for all fallible operations..."
      > "Error messages need to be actionable for debugging..."

   2. ADR-0005: Unified Error Framework [Accepted]
      Match in: Context, Related
      > "Building on ADR-0001, we need a unified error..."

   3. ADR-0012: API Response Format [Proposed]
      Match in: Consequences
      > "...must include proper error handling..."

   Tip: Read full ADR with: cat docs/adr/0001-error-handling-strategy.md
   ```

6. **Show relevant excerpts**:
   - Include 1-2 line excerpts showing match context
   - Highlight matching terms if possible

## Output

- Ranked list of matching ADRs
- Excerpts showing match context
- Status of each ADR
- Tips for viewing full content

## Notes

- Searches are case-insensitive
- Quotes in query are optional but help with multi-word searches
- Use section filter to narrow down results
- Useful for finding relevant decisions before making new ones
- Also searches validation rule descriptions
