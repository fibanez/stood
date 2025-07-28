# DOCS_TODO.md - Documentation Implementation Guide

## 📋 Documentation Philosophy & Approach

**CORE PRINCIPLES:**
- 📚 **Progressive Disclosure**: Start with "How to Use" → progress to "How it Works" → finally "How to Extend"
- 🎯 **Multi-Audience Support**: Serve both library users and developers with distinct pathways
- ✅ **Unified Documentation**: In-source rustdoc comments + GitHub markdown docs in single pass
- 🔄 **Continuous Validation**: Every commit triggers doc review, full reviews before releases
- 📏 **Cognitive Load Management**: Consistent structure reduces mental overhead
- ✍️ **User-Focused Writing**: Documentation that speaks directly to users' needs and goals

**Documentation Workflow Modes:**
1. **FULL_REVIEW**: Complete documentation audit (initial setup & pre-release)
2. **INCREMENTAL_UPDATE**: Post-commit targeted updates (daily development)

**IMPORTANT: Files to Ignore**
- Do NOT document files ending in `_TODO.md` - these are implementation guides for development workflow
- Focus only on source code, tests, examples, and creating GitHub markdown documentation

---

## ✍️ Documentation Writing Style Guidelines

### Voice and Tone
- **User-obsessed**: Frame benefits from the user's perspective ("You'll be able to..." rather than "The library provides...")
- **Confident but not boastful**: State features and benefits matter-of-factly
- **Conversational yet professional**: Use accessible language without being overly casual
- **Action-oriented**: Emphasize what users can do or achieve
- **Trustworthy**: Provide clear, straightforward communication without marketing fluff

### Writing Principles
- **Clarity above all**: Choose simple words over complex ones, use short sentences when possible
- **Scannable format**: Use bullet points for features, bold text for key information
- **Front-loaded benefits**: Place the most important information first
- **Specific over vague**: "Processes 10,000 records per second" not "high performance"
- **Active voice**: "Parse the configuration" not "The configuration can be parsed"

### GitHub Markdown Formatting Guidelines
- **Standard markdown emphasis**: Use `**text**` for bold, `*text*` for italic
- **Consistent formatting**: Maintain standard GitHub markdown format across all docs
- **Cross-reference format**: Use `[page title](page.md)` for internal doc links
- **Source code links**: Use `📚 [Link Description](../src/module/mod.rs)` to link docs to source files
- **GitHub relative links**: Use relative paths for linking to other files in the repository

### Common Phrases and Patterns
- "Developers who use this often combine it with..."
- "Commonly used together with..."
- Start sentences with action verbs: "Create," "Build," "Transform," "Parse"
- Use "you" and "your" frequently to personalize the experience
- "This enables you to..." rather than "This feature allows..."

### Structural Elements
- Concise headlines that include key attributes or benefits
- Feature bullets that start with capital letters but don't end with periods
- Clear hierarchy of information (most to least important)
- Consistent formatting across all documentation
- Code examples immediately after concepts

### What to Avoid
- Exclamation points (!)
- Superlatives and hyperbole ("amazing," "incredible," "best ever")
- Industry jargon without explanation
- Long paragraphs in API descriptions
- Passive voice constructions
- Marketing language or sales pitch tone
- Assumptions about user's knowledge level without context

---

## 🎯 Current Documentation Status

**Last Full Review**: [Never - Initial Setup Required]
**Last Incremental Update**: [Never]
**Documentation Coverage**: [Unknown - Needs Initial Audit]

---

## 📚 FULL_REVIEW Mode Tasks

### Phase 1: Project Structure Analysis
- [ ] **Task 1.1**: Identify workspace structure
  - [ ] Map all crates in workspace
  - [ ] Document crate dependencies and relationships
  - [ ] Create visual dependency diagram in docs/
  - [ ] Document cross-crate interactions and data flow
  - [ ] Identify public vs internal APIs per crate

- [ ] **Task 1.2**: Catalog core components
  - [ ] List all public modules with brief purpose
  - [ ] Identify key traits and their implementations
  - [ ] Map primary data structures and their relationships
  - [ ] Note critical functions that need detailed docs
  - [ ] Document unsafe code blocks (if any) with safety invariants

### Phase 2: In-Source Documentation (Rustdoc)
- [ ] **Task 2.1**: Crate and module documentation
  - [ ] Write crate-level docs (`//! ` in lib.rs) following structure:
    - One-line description
    - Expanded overview (2-3 paragraphs)
    - Quick start example
    - Feature flags and their effects
    - Links to main modules
  - [ ] Add module docs for each module:
    - Purpose and relationship to other modules
    - Common use cases
    - Performance characteristics
    - Links to key types/functions

- [ ] **Task 2.2**: Document all public APIs
  - [ ] For each struct/enum:
    - [ ] One-line summary
    - [ ] Detailed description with use cases
    - [ ] Example showing typical usage
    - [ ] Document all public fields
    - [ ] Performance implications
    - [ ] Link related types
  - [ ] For traits:
    - [ ] Purpose and when to implement
    - [ ] Required vs provided methods
    - [ ] Implementation example
    - [ ] Common implementors
    - [ ] Performance expectations
  - [ ] For functions:
    - [ ] One-line summary
    - [ ] Parameters with purpose
    - [ ] Return value meaning
    - [ ] Examples section
    - [ ] Errors/Panics sections if applicable
    - [ ] Safety section for unsafe functions

- [ ] **Task 2.3**: Document error types and handling
  - [ ] For each error type:
    - [ ] When this error occurs
    - [ ] How to handle/recover
    - [ ] Prevention strategies
  - [ ] Document error propagation patterns
  - [ ] Common error scenarios and solutions

### Phase 3: Test Documentation
- [ ] **Task 3.1**: Unit test documentation
  - [ ] For each test module:
    - [ ] Add module-level doc explaining test purpose
    - [ ] Group related tests with section comments
  - [ ] For each test function:
    - [ ] Add doc comment explaining what is being tested
    - [ ] Document edge cases being covered
    - [ ] Explain why this test matters
    - [ ] Note any special setup/teardown

- [ ] **Task 3.2**: Integration test documentation
  - [ ] Document test scenarios and workflows
  - [ ] Explain integration points being tested
  - [ ] Document test data and fixtures
  - [ ] Note performance benchmarks if relevant

- [ ] **Task 3.3**: Example documentation
  - [ ] For each example in `examples/`:
    - [ ] Add comprehensive comments explaining the example
    - [ ] Document the use case it demonstrates
    - [ ] Explain key concepts being illustrated
    - [ ] Add links to relevant API docs
    - [ ] Include expected output/behavior

### Phase 4: GitHub Docs Knowledge Base
- [ ] **Task 4.1**: Create documentation structure
  - [ ] Create `docs/` directory with GitHub markdown files:
    - [ ] index.md - Navigation and overview
    - [ ] architecture.md - System design and diagrams
    - [ ] patterns.md - Common usage patterns
    - [ ] troubleshooting.md - Common issues and solutions
    - [ ] antipatterns.md - What NOT to do and why
    - [ ] performance.md - Performance characteristics
    - [ ] migration.md - Version migration guides

- [ ] **Task 4.2**: Write conceptual documentation
  - [ ] Architecture overview with diagrams
  - [ ] Performance characteristics and benchmarks
  - [ ] Memory usage patterns
  - [ ] Threading and concurrency model
  - [ ] Integration patterns with other libraries

- [ ] **Task 4.3**: Create learning pathways
  - [ ] Beginner's guide (tutorial style)
  - [ ] Cookbook with practical recipes
  - [ ] Advanced topics (internals, optimization)
  - [ ] Debugging guide
  - [ ] Contributing guide for developers

### Phase 5: Cross-Reference and Linking
- [ ] **Task 5.1**: Internal cross-references
  - [ ] Link rustdoc to docs/ pages
  - [ ] Cross-reference related APIs
  - [ ] Connect examples to concepts
  - [ ] Link tests to features they verify

- [ ] **Task 5.2**: docs/ to Source Code Cross-References
  - [ ] Add source code links to all Key Components sections
  - [ ] Create bidirectional links (docs/ → Source, Source → docs/)
  - [ ] Implement consistent emoji patterns (📚 API, 📖 Examples, 🧪 Tests)
  - [ ] Link examples directory files from docs/ pages
  - [ ] Add configuration file references (⚙️ Cargo.toml, 🔧 configs)
  - [ ] Update all module documentation with concept links back to docs/

- [ ] **Task 5.3**: External references
  - [ ] Document dependencies and why they're used
  - [ ] Link to relevant external documentation
  - [ ] Reference papers/articles for algorithms

### Phase 6: Feature and Configuration Documentation
- [ ] **Task 6.1**: Feature flag documentation
  - [ ] Document each feature flag:
    - [ ] What it enables/disables
    - [ ] Performance impact
    - [ ] API changes
    - [ ] Compatibility considerations
  - [ ] Create feature combination matrix
  - [ ] Document feature dependencies

- [ ] **Task 6.2**: Configuration documentation
  - [ ] Document all configuration options
  - [ ] Provide configuration examples
  - [ ] Explain performance implications
  - [ ] Document defaults and rationales

---

## 🔄 INCREMENTAL_UPDATE Mode Tasks

### Quick Review Checklist (Per Commit)

1. **Change Analysis**
   - [ ] Identify changed files in commit
   - [ ] Categorize changes: API, implementation, tests, examples
   - [ ] Note breaking changes or new features

2. **Documentation Updates Required**
   
   **For API changes:**
   - [ ] Update rustdoc comments in source
   - [ ] Update examples if behavior changed
   - [ ] Update docs/patterns.md if usage patterns affected
   - [ ] Update docs/architecture.md if design changed
   - [ ] Add to docs/migration.md if breaking change

   **For new features:**
   - [ ] Add complete rustdoc documentation
   - [ ] Document feature in docs/
   - [ ] Update relevant examples with comments
   - [ ] Document related tests
   - [ ] Update feature flag docs if applicable

   **For bug fixes:**
   - [ ] Document the fix in code comments
   - [ ] Add to docs/troubleshooting.md if user-facing
   - [ ] Document test that prevents regression

   **For test changes:**
   - [ ] Update test documentation
   - [ ] Document new test scenarios
   - [ ] Explain why tests were added/modified

3. **Cross-Reference Updates**
   - [ ] Update links between related docs
   - [ ] Ensure consistency across documentation
   - [ ] Update changelog references

### Incremental Review Prompts for Claude Code

**For new features:**
```
Review the changes in [commit hash] and update documentation:
1. Add comprehensive rustdoc comments for all new public APIs
2. Document the feature in appropriate docs/ pages
3. Update examples with explanatory comments
4. Document all related tests explaining what they verify
5. Add docs/ cross-references: [Module API](../src/module/mod.rs)
6. Update rustdoc with concept links back to docs/ pages
7. Ensure bidirectional linking between concept and implementation
```

**For API modifications:**
```
Review the API changes in [files] and:
1. Update rustdoc comments to reflect new behavior and parameters
2. Document performance implications if any
3. Update docs/patterns.md with new usage patterns
4. Add migration notes to docs/migration.md if breaking
5. Update all affected example comments
6. Verify docs/ cross-reference links still work
7. Update source-to-concept links in rustdoc if module organization changed
```

**For test additions:**
```
For new tests in [commit hash]:
1. Add doc comments explaining what each test verifies
2. Document edge cases being tested
3. Explain the importance of these tests
4. Group related tests with section documentation
5. Add test file links to relevant docs/ pages: 🧪 [Test Examples](../tests/test.rs)
```

**For cross-reference system maintenance:**
```
For any structural changes to modules or docs/ pages:
1. Verify all [[file:../src/module/mod.rs|Description]] links work
2. Check that emoji patterns are consistent (📚 API, 📖 Examples, 🧪 Tests)
3. Ensure bidirectional links remain valid (docs/ ↔ Source)
4. Update Key Components sections if module organization changed
5. Test navigation links work in GitHub interface
```

---

## 📊 Documentation Quality Checklist

After each full review, verify:

- [ ] Every public API has rustdoc with example
- [ ] Every test has explanatory comment
- [ ] Every example is thoroughly commented
- [ ] Every unsafe block has safety documentation
- [ ] Every error type has handling documentation
- [ ] Every feature flag is documented
- [ ] Architecture diagrams are up to date
- [ ] Cross-references are valid
- [ ] No outdated information remains

### Cross-Reference System Verification
- [ ] All docs/ Key Components sections have source code links
- [ ] docs/ links use correct markdown syntax: `[Description](../src/module/mod.rs)`
- [ ] Emoji patterns are consistent: 📚 API, 📖 Examples, 🧪 Tests, ⚙️ Config
- [ ] Source code rustdoc includes concept links back to docs/
- [ ] Bidirectional navigation works (docs/ ↔ Source)
- [ ] All external file links work with GitHub navigation
- [ ] Examples directory files are linked from appropriate docs/ pages

---

## 📝 Documentation Templates

### Module Documentation Template:
```rust
//! Parse and validate CloudFormation templates with type safety.
//!
//! This module enables you to work with CloudFormation templates while maintaining
//! Rust's type safety guarantees. You'll get compile-time validation of resource
//! types and runtime validation of template structure.
//!
//! # Get Started
//!
//! Parse your first template:
//! ```
//! use crate::parser::Template;
//!
//! let template = Template::from_yaml(yaml_string)?;
//! let resources = template.resources();
//! ```
//!
//! # Architecture
//!
//! The parser uses a two-phase approach: structural validation followed by
//! semantic analysis. See [parser architecture](../docs/architecture.md#parser)
//! for implementation details.
//!
//! # Performance
//!
//! - Parses templates up to 1MB in under 100ms
//! - Memory usage scales linearly with template size
//! - Zero-copy parsing where possible
//!
//! # Feature Flags
//!
//! - `validation`: Enable full semantic validation (default: on)
//! - `async`: Add async parsing support for large templates
```

### Test Documentation Template:
```rust
/// Verifies that nested resource references resolve correctly across template sections.
///
/// This test ensures the reference resolver can handle complex scenarios where
/// resources reference other resources through multiple levels of indirection.
/// This matters because CloudFormation templates often have deep reference chains
/// that must resolve correctly for deployment to succeed.
///
/// Specific cases tested:
/// - Direct references using !Ref
/// - Attribute access via !GetAtt through 3 levels
/// - Circular reference detection
#[test]
fn resolves_nested_resource_references() {
    // Test implementation
}
```

### Function Documentation Template:
```rust
/// Parse a CloudFormation template from YAML format.
///
/// This function takes your YAML string and returns a fully validated template
/// structure. You'll get detailed error messages for any validation failures,
/// including line numbers and suggestions for fixes.
///
/// # Arguments
///
/// * `yaml` - Your CloudFormation template as a YAML string
///
/// # Returns
///
/// A validated `Template` ready for processing, or an error describing
/// what went wrong and how to fix it.
///
/// # Examples
///
/// Parse a simple template:
/// ```
/// use crate::parse_yaml;
///
/// let template = parse_yaml(r#"
///     Resources:
///       MyBucket:
///         Type: AWS::S3::Bucket
/// "#)?;
/// 
/// assert_eq!(template.resources().count(), 1);
/// ```
///
/// # Errors
///
/// Returns `ParseError` when:
/// - YAML syntax is invalid - check for indentation issues
/// - Required fields are missing - ensure Resources section exists
/// - Resource types are unknown - verify AWS resource type names
```

### Example Documentation Template:
```rust
//! # Build a Multi-Stack CloudFormation Template
//!
//! This example shows you how to create a template that deploys a web application
//! with proper separation of concerns across multiple stacks.
//!
//! ## What You'll Learn
//!
//! - Create reusable template components
//! - Share values between stacks using exports
//! - Validate cross-stack references
//!
//! ## Running This Example
//!
//! ```bash
//! cargo run --example multi_stack_template
//! ```
//!
//! You'll see output showing the generated template structure and validation results.
//!
//! ## Code Walkthrough
//!
//! First, you'll create the network stack that other stacks will reference...

// Define your network stack with exportable VPC ID
let network_stack = StackBuilder::new("NetworkStack")
    .add_vpc("MyVPC")  // This creates an exportable VPC
    .build();

// Next, create an application stack that references the network...
```

---

## 🎯 Next Actions

**For Initial Setup:**
1. ✅ Analyze project structure (Phase 1) - COMPLETED
2. ✅ Begin systematic documentation of all public APIs (Phase 2) - COMPLETED  
3. Document all existing tests (Phase 3) - PENDING
4. ✅ Create GitHub docs structure with core pages (Phase 4) - COMPLETED

**For Next Iteration (High Priority):**
1. **Create examples.md** - Missing page referenced in index.md navigation
   - Guided tutorials for getting started
   - Walkthrough of examples/ directory contents
   - Bridge between patterns.md and practical implementation
   - Expected output and explanations for each example

2. **Complete Cross-Reference System** - Bidirectional docs/ ↔ Source linking
   - Add source links to all Key Components sections in docs/ pages
   - Update rustdoc comments with concept links back to docs/
   - Implement consistent emoji patterns (📚 📖 🧪 ⚙️)
   - Link examples directory files from relevant docs/ pages
   - Verify all docs/ external file links work with GitHub navigation

**For Ongoing Development:**
1. Use incremental checklist after each commit
2. Maintain documentation-first approach for new features
3. Keep docs/ and rustdoc synchronized
4. Regular full reviews before releases
5. Maintain docs/ link integrity (check with each new page addition)
