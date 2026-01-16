# List Architecture Decision Records

Display all ADRs with their status, date, and summary.

## Usage

- `/adr-list` - List all ADRs
- `/adr-list --status <status>` - Filter by status (proposed, accepted, deprecated, superseded)

## Arguments

$ARGUMENTS - Optional: `--status <status>`

## Instructions

1. **Read ADR directory**:
   - List all files in `docs/adr/` matching pattern `[0-9][0-9][0-9][0-9]-*.md`
   - Exclude `README.md` and `template.md`

2. **Parse each ADR**:
   - Extract ADR number from filename
   - Extract title from first heading
   - Extract status from Status section
   - Extract date from Date section

3. **Apply filters**:
   - If `--status` provided, filter to matching ADRs only

4. **Display as table**:

   ```
   Architecture Decision Records
   =============================

   | #    | Title                      | Status    | Date       |
   |------|----------------------------|-----------|------------|
   | 0001 | Error Handling Strategy    | Accepted  | 2025-01-16 |
   | 0002 | Payment Options Cache      | Proposed  | 2025-01-16 |
   | 0003 | WebSocket Reconnection     | Accepted  | 2025-01-17 |
   | 0004 | Legacy Auth Flow           | Superseded| 2025-01-10 |

   Total: 4 ADRs (2 Accepted, 1 Proposed, 1 Superseded)

   Use /adr-search "<topic>" to search ADR contents
   ```

5. **Show summary stats**:
   - Count by status
   - Total count

## Output

- Formatted table of all ADRs
- Summary statistics
- Helpful hints for related commands

## Notes

- ADRs are sorted by number (chronological order)
- Status colors: Accepted (green), Proposed (yellow), Deprecated/Superseded (gray)
- Quick way to see what decisions are in effect
