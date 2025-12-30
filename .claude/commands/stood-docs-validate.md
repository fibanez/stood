---
description: Validate and update documentation when referenced code no longer exists
---

You are tasked with validating documentation references against the actual Stood codebase and updating documentation when code has been removed or changed. This works in the opposite direction of `/stood-docs`:

- `/stood-docs`: Code changes → Update docs (git-driven)
- `/stood-docs-validate`: Docs → Validate code exists → Update docs (docs-driven)

## Validation Goals

Find and fix documentation that references code that no longer exists:
- File paths that don't exist
- Functions/structs/modules that were removed or renamed
- Outdated code examples
- Broken cross-references between documentation files

## Instructions

1. **Gather documentation files:**
   - List all `.md` files in `docs/`
   - Include `CLAUDE.md` and any other root-level documentation
   - Check example READMEs in `examples/` directories

2. **Extract code references from each doc:**
   For each documentation file, identify:
   - **File path references**: Patterns like `src/agent/mod.rs`, `src/telemetry/logging.rs`
   - **Source code links**: Markdown links like `[text](../src/path/file.rs)`
   - **Code block references**: Function/struct names in code examples
   - **Key Files sections**: Lists of important source files
   - **Module references**: References to Rust modules

3. **Validate each reference:**
   - Check if referenced files exist using Glob or file reads
   - For struct/function references in code examples, verify they exist in the codebase
   - Check cross-reference links between `.md` files
   - Verify example paths exist

4. **Update documentation for broken references:**

   **For deleted files:**
   - Remove references to files that no longer exist
   - Update or remove code examples that reference deleted modules
   - Add notes about deprecated/removed features if relevant

   **For renamed/moved files:**
   - Update file paths to new locations
   - Update import statements in code examples

   **For changed APIs:**
   - Update code examples to match current function signatures
   - Update struct field references

   **For orphaned documentation:**
   - Link orphaned docs to `docs/README.md` if still relevant
   - Mark for deletion if the documented feature no longer exists

5. **Update the documentation index:**
   - Remove entries from `docs/README.md` for deleted docs
   - Add entries for any new documentation created

## Stood-Specific Validation

**Key modules to validate references for:**
- `src/agent/` - Agent, AgentBuilder, EventLoop
- `src/tools/` - Tool trait, ToolRegistry, ToolExecutor
- `src/telemetry/` - TelemetryConfig, StoodTracer, StoodSpan, logging
- `src/mcp/` - MCP client and server
- `src/bedrock/` - BedrockClient, streaming
- `src/llm/` - Provider abstractions

**Known deleted files (update any references):**
- `src/telemetry/otel.rs` - DELETED
- `src/telemetry/metrics/` - DELETED (entire directory)
- `src/telemetry/otlp_debug.rs` - DELETED
- `src/telemetry/test_harness.rs` - DELETED

**Known deleted types (update any references):**
- `MetricsCollector` - removed
- `NoOpMetricsCollector` - removed
- `TokenMetrics`, `RequestMetrics`, `ToolMetrics`, `SystemMetrics` - removed

## Validation Patterns

**File path patterns to check:**
```
src/agent/*.rs
src/tools/*.rs
src/telemetry/*.rs
src/mcp/*.rs
src/bedrock/*.rs
examples/*/*.rs
```

**Markdown link patterns:**
```markdown
[Display Text](../src/path/file.rs)
[Doc Link](other-doc.md)
`src/module/file.rs`
```

## Output Format

After validating and updating, provide:

```
## Documentation Validation Report

### Fixed Issues
- `docs/telemetry.md`: Removed reference to deleted `src/telemetry/otel.rs`
- `docs/api.md`: Updated code example - removed `MetricsCollector` usage

### Updated Files
- `docs/telemetry.md` - Removed 3 broken references
- `docs/README.md` - Updated navigation links

### Remaining Issues (manual review needed)
- `examples/023_telemetry/README.md`: Entire example may need rewrite

### Summary
- Files checked: X
- References validated: Y
- Issues fixed: Z
- Files updated: W
```

## Writing Style Guidelines

When updating documentation, follow these principles:

**Voice and Tone:**
- User-focused: Frame from user's perspective
- Confident but not boastful
- Action-oriented

**What to Avoid:**
- Exclamation points
- Marketing language
- Passive voice

## After Validation

Inform the user of:
1. What documentation was updated
2. Any issues that need manual review
3. Whether a commit is recommended
