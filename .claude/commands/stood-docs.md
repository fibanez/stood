---
description: Review code changes and update documentation for the Stood agent library
---

You are tasked with reviewing recent code changes and creating or updating documentation for the Stood agent library.

## Documentation Philosophy

### Core Principles
- **Progressive Disclosure**: Start with "How to Use" → progress to "How it Works" → finally "How to Extend"
- **Multi-Audience Support**: Serve both library users and contributors with distinct pathways
- **Unified Documentation**: In-source rustdoc comments + Markdown extended docs in `docs/`
- **Continuous Validation**: Every commit triggers doc review
- **Cognitive Load Management**: Consistent structure reduces mental overhead
- **User-Focused Writing**: Documentation that speaks directly to users' needs and goals

### Writing Style Guidelines

**Voice and Tone:**
- User-obsessed: Frame benefits from the user's perspective ("You'll be able to..." rather than "The library provides...")
- Confident but not boastful: State features and benefits matter-of-factly
- Conversational yet professional: Use accessible language without being overly casual
- Action-oriented: Emphasize what users can do or achieve
- Trustworthy: Provide clear, straightforward communication without marketing fluff

**Writing Principles:**
- Clarity above all: Choose simple words over complex ones, use short sentences when possible
- Scannable format: Use bullet points for features, bold text for key information
- Front-loaded benefits: Place the most important information first
- Specific over vague: "Processes 10,000 requests per second" not "high performance"
- Active voice: "Parse the configuration" not "The configuration can be parsed"

**Markdown Formatting:**
- Standard emphasis: Use `**text**` for bold, `*text*` for italic
- Consistent formatting: Follow GitHub Flavored Markdown standards
- Cross-reference format: Use `[Display Text](file-name.md)` for internal links
- Source code links: Use `[Link Description](../src/module/mod.rs)` to link Markdown to source files
- Code blocks: Use triple backticks with language specification: ```rust

**What to Avoid:**
- Exclamation points (!)
- Superlatives and hyperbole ("amazing," "incredible," "best ever")
- Industry jargon without explanation
- Long paragraphs in API descriptions
- Passive voice constructions
- Marketing language or sales pitch tone
- Assumptions about user's knowledge level without context

## Instructions

1. **Review the changes:**
   - Run `git status` to see untracked files
   - Run `git diff` to see staged and unstaged changes
   - Identify which systems, components, or features were modified

2. **Identify documentation needs:**
   - Check if new systems/patterns need documentation in `docs/`
   - Determine if existing docs need updates
   - Verify if rustdoc comments need to be added/updated
   - Check if `docs/README.md` (documentation index) needs updating

3. **Create or update documentation:**
   - For new systems: Create new `.md` files in `docs/`
   - For existing systems: Update relevant documentation files
   - Add rustdoc comments (`///`) to new/modified public functions and structs
   - Use the writing style guidelines above
   - Include code examples with proper syntax highlighting
   - Add cross-references using `[Display Text](file-name.md)` format

4. **Update the documentation index:**
   - Add new documentation files to `docs/README.md`
   - Ensure proper categorization (Getting Started, Advanced Features, etc.)
   - Maintain logical ordering within sections

5. **Quality checklist:**
   - [ ] Every public API has rustdoc documentation
   - [ ] Every example has explanatory comments
   - [ ] Every unsafe block has safety documentation
   - [ ] Error handling patterns are documented
   - [ ] Module interactions are documented
   - [ ] Cross-references are valid
   - [ ] No outdated information remains
   - [ ] Code examples compile and are idiomatic
   - [ ] Documentation follows the writing style guidelines

## Stood Library Documentation Structure

The Stood library documentation is organized as follows:

```
docs/
├── README.md              # Documentation index and navigation
├── api.md                 # Core API reference
├── architecture.md        # System design overview
├── examples.md            # Code examples and tutorials
├── tools.md               # Tool development guide
├── mcp.md                 # MCP integration guide
├── telemetry.md           # Observability and tracing
├── streaming.md           # Real-time response handling
├── callbacks.md           # Event handling system
├── conversation_manager.md # Message history management
└── context_manager.md     # Context window optimization
```

## Example Documentation Structure

**For a new system (e.g., `docs/my-new-feature.md`):**

```markdown
# My New Feature

Brief one-sentence description of what the feature does.

## Overview

What the feature is and why it exists. Focus on user benefits.

## How to Use

Step-by-step guide for using the feature. Include code examples.

```rust
use stood::agent::Agent;

// Example code here
```

## How it Works

Technical details about the implementation.

## Integration Points

How this feature connects with other parts of Stood:
- Agent integration
- Tool system integration
- Telemetry integration

## Testing

How to test code that uses this feature.

## Related Documentation

- [Related Feature](related-feature.md)
- [Source Code](../src/module/mod.rs)
```

## Rustdoc Standards for Stood

When documenting Rust code, follow these conventions:

```rust
/// Brief one-line description.
///
/// More detailed explanation of what this does and why you'd use it.
///
/// # Arguments
///
/// * `param` - Description of the parameter
///
/// # Returns
///
/// Description of what is returned.
///
/// # Errors
///
/// Describe error conditions and when they occur.
///
/// # Examples
///
/// ```rust
/// use stood::module::MyType;
///
/// let instance = MyType::new();
/// let result = instance.method()?;
/// ```
pub fn my_function(param: &str) -> Result<Output, Error> {
    // implementation
}
```

## Output Format

After reviewing and updating documentation, provide:
1. A summary of what was changed in the code
2. A list of documentation files created or updated
3. Key documentation additions or changes made
4. Any additional documentation work needed (if applicable)
