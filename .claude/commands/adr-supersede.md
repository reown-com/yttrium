# Supersede an ADR

Mark an existing ADR as superseded by a newer one.

## Usage

- `/adr-supersede <old-number> <new-number>` - Mark old ADR as superseded by new ADR

## Arguments

$ARGUMENTS - Required: `<old-number> <new-number>` (e.g., "0001 0005")

## Instructions

1. **Parse arguments**:
   - Extract old ADR number and new ADR number from $ARGUMENTS
   - Validate both are 4-digit numbers

2. **Find ADR files**:
   - Locate old ADR: `docs/adr/<old-number>-*.md`
   - Locate new ADR: `docs/adr/<new-number>-*.md`
   - Error if either doesn't exist

3. **Validate operation**:
   - Old ADR should be "Accepted" status (warn if superseding non-accepted)
   - New ADR should exist and be "Accepted" or "Proposed"
   - Prevent circular references

4. **Update old ADR**:
   - Change status line from current status to: `Superseded by ADR-<new-number>`
   - Add note at top if desired

5. **Update new ADR**:
   - Add to Related section: `- Supersedes: ADR-<old-number>`
   - If Related section doesn't exist, create it

6. **Update README index**:
   - Update the status column for the old ADR in `docs/adr/README.md`

7. **Confirm changes**:
   - Show diff of changes made
   - Confirm with user before saving

## Example

```
$ /adr-supersede 0001 0005

Superseding ADR-0001 with ADR-0005...

ADR-0001: Error Handling Strategy
  Status: Accepted -> Superseded by ADR-0005

ADR-0005: Unified Error Framework
  Related: Added "Supersedes: ADR-0001"

README.md index updated.

Changes made. Commit these changes? [y/N]
```

## Output

- Old ADR status updated to "Superseded by ADR-XXXX"
- New ADR updated with supersedes reference
- README index updated
- Summary of changes shown

## Notes

- Superseding preserves history - old ADR remains for reference
- Old ADR's validation rules are automatically disabled (only Accepted ADRs are validated)
- Use this when a decision has evolved, not just been refined
- For minor updates to an existing decision, edit the ADR directly instead
