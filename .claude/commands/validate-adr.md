# Validate Code Against ADRs

Check if code complies with established Architecture Decision Records.

## Usage

- `/validate-adr` - Check uncommitted changes (staged + unstaged)
- `/validate-adr --all` - Check entire codebase
- `/validate-adr --file <path>` - Check specific file
- `/validate-adr --staged` - Check only staged changes

## Arguments

$ARGUMENTS - Optional flags: `--all`, `--file <path>`, `--staged`

## Instructions

1. **Read all accepted ADRs**:
   - List ADR files: `docs/adr/*.md`
   - For each ADR, check if status is "Accepted"
   - Extract validation rules from the YAML block in each accepted ADR

2. **Determine what to validate**:
   - Default (no args): `git diff HEAD` (uncommitted changes)
   - `--staged`: `git diff --cached`
   - `--all`: All files matching rule patterns
   - `--file <path>`: Specific file only

3. **For each validation rule**:
   - Parse the rule's `applies_to` glob patterns
   - Check if any target files match the pattern
   - If matches found, search for the rule's `pattern` regex
   - Record violations with file, line number, and context

4. **Generate report**:

   ```
   ADR Validation Report
   =====================

   Checking against N accepted ADRs with M rules...

   [PASS] ADR-0001 Rule 0001-no-unwrap-in-pub-fn
          No violations found

   [WARN] ADR-0001 Rule 0001-no-panic-in-ffi
          crates/kotlin-ffi/src/lib.rs:42
          > panic!("unexpected state")
          Suggestion: Return Result type instead of panicking

   [ERROR] ADR-0002 Rule 0002-cache-ttl
           crates/yttrium/src/pay/cache.rs:15
           > static CACHE: OnceLock<PaymentCache> = OnceLock::new();
           Suggestion: Add TTL or explicit invalidation mechanism

   Summary:
   - Rules checked: M
   - Passed: X
   - Warnings: Y
   - Errors: Z

   Overall: [PASS | FAIL]
   ```

5. **Provide suggestions**:
   - For each violation, suggest how to fix based on the ADR's decision
   - Reference the relevant ADR for full context

## Output

- Validation report showing all rule checks
- List of violations with file locations and line numbers
- Suggestions for fixing each violation
- Overall pass/fail status (FAIL if any errors, PASS if only warnings or clean)

## Notes

- Warnings (`action: "warn"`) don't fail validation but should be reviewed
- Errors (`action: "error"`) indicate violations that should be fixed
- Rules only apply to files matching their `applies_to` patterns
- Regex patterns are applied line-by-line unless multiline mode specified
- Consider context when reporting - some patterns may have valid uses
