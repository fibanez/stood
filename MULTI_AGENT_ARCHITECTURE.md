# Multi-Agent Enterprise Prompt Builder Architecture

## Overview

This document describes the original three-layer multi-agent architecture that was implemented in the Enterprise Prompt Builder before it was simplified for the egui migration. This serves as a blueprint for restoring the full multi-agent functionality.

## Three-Layer Architecture

### Layer 1: Coordinating Agent (Persistent)
- **Role**: Orchestrates the entire prompt building process
- **Lifetime**: Persistent throughout the entire session
- **Responsibilities**:
  - Maintains conversation state with the user
  - Decides when to spawn specialized task agents
  - Accumulates context between different prompt sections
  - Handles user feedback and task recovery
  - Tracks overall progress across all 7 required sections

### Layer 2: Task Agents (Ephemeral)
- **Role**: Specialized agents for each prompt section
- **Lifetime**: Created for specific tasks, destroyed after completion
- **Agent Types**:
  1. **Task Context Agent** - Handles "Task Context & Goals" section
  2. **Data Section Agent** - Manages "Data Section & XML Tags" 
  3. **Instructions Agent** - Creates "Detailed Task Instructions"
  4. **Examples Agent** - Generates "Examples" section
  5. **Critical Instructions Agent** - Handles "Critical Instructions Repetition"
  6. **Tools Agent** - Manages "Tool Descriptions"
  7. **Evaluation Agent** - Creates "Evaluation Prompt"

### Layer 3: Utility Agents (On-Demand)
- **Role**: Support services for the task agents
- **Examples**:
  - Research agents for gathering domain-specific information
  - Validation agents for checking section completeness
  - Format conversion agents for different output types

## Context Accumulation System

### Context Flow
```
User Input → Coordinating Agent → Task Agent → Context Accumulation → Next Task Agent
```

### Context Structure
```rust
pub struct AccumulatedContext {
    pub completed_sections: Vec<CompletedSection>,
    pub user_preferences: UserPreferences,
    pub domain_knowledge: DomainContext,
    pub style_guidelines: StylePreferences,
    pub section_dependencies: HashMap<String, Vec<String>>,
}

pub struct CompletedSection {
    pub section_name: String,
    pub content: String,
    pub metadata: SectionMetadata,
    pub validation_status: ValidationStatus,
}
```

## Agent Coordination Protocol

### Coordinating Agent → Task Agent Communication
```rust
pub struct TaskSpawnRequest {
    pub section_name: String,
    pub accumulated_context: AccumulatedContext,
    pub user_input: String,
    pub success_criteria: Vec<String>,
    pub previous_attempts: Vec<AttemptHistory>,
}

pub struct TaskCompletionResponse {
    pub section_content: String,
    pub confidence_score: f32,
    pub validation_results: ValidationResults,
    pub context_updates: ContextUpdates,
    pub next_section_recommendations: Vec<String>,
}
```

### Task Agent Tools
Each task agent had access to specialized tools:

```rust
// Common tools for all task agents
#[tool]
async fn request_user_clarification(question: String) -> Result<String, String>;

#[tool] 
async fn validate_section_completeness(section: String, criteria: Vec<String>) -> Result<ValidationReport, String>;

#[tool]
async fn research_domain_context(domain: String, focus_areas: Vec<String>) -> Result<DomainInfo, String>;

// Specialized tools per agent type
#[tool] // For Examples Agent
async fn generate_diverse_examples(task_description: String, count: usize) -> Result<Vec<Example>, String>;

#[tool] // For Tools Agent  
async fn discover_available_tools(context: String) -> Result<Vec<ToolDescription>, String>;

#[tool] // For Evaluation Agent
async fn create_validation_criteria(task_context: String) -> Result<Vec<Criteria>, String>;
```

## Progress Tracking System

### State Management
```rust
pub struct TaskOrchestrationState {
    pub current_section: usize,
    pub completed_sections: Vec<String>,
    pub accumulated_context: AccumulatedContext,
    pub pending_sections: Vec<String>,
    pub active_task_agent: Option<String>,
    pub section_dependencies: HashMap<String, Vec<String>>,
    pub user_feedback_history: Vec<FeedbackEntry>,
}

impl TaskOrchestrationState {
    pub fn progress_percentage(&self) -> f32 {
        (self.completed_sections.len() as f32 / 7.0) * 100.0
    }
    
    pub fn can_proceed_to_next(&self) -> bool {
        // Check if current section meets minimum requirements
        // Validate dependencies are satisfied
        // Ensure user feedback has been addressed
    }
    
    pub fn get_next_section(&self) -> Option<&str> {
        // Intelligent section ordering based on dependencies
        // Skip sections that user indicated are not needed
        // Prioritize sections that unlock multiple dependencies
    }
}
```

## Error Handling & Recovery

### Task Agent Failure Recovery
- If a task agent fails, the coordinating agent:
  1. Analyzes the failure reason
  2. Adjusts the context/approach
  3. Respawns the agent with modified parameters
  4. Falls back to user collaboration if multiple failures occur

### User Feedback Integration
- Real-time feedback incorporation during section building
- Ability to restart specific sections without losing other work  
- Version control for section iterations

## Implementation Roadmap

### Phase 1: Restore Coordinating Agent
1. Implement persistent `CoordinatingAgent` struct
2. Add context accumulation system
3. Integrate with existing `ConversationAgent`
4. Add progress tracking UI components

### Phase 2: Add Task Agents
1. Create base `TaskAgent` trait
2. Implement the 7 specialized task agents
3. Add agent spawning/cleanup logic
4. Implement section validation system

### Phase 3: Advanced Features  
1. Add utility agents for research/validation
2. Implement user feedback integration
3. Add section dependency management
4. Create export/import functionality for completed prompts

### Phase 4: UI Enhancements
1. Add agent activity visualization
2. Implement section-by-section progress display
3. Add real-time agent communication logs
4. Create prompt preview/editing interface

## Agent Source Tracking Enhancement

With the current agent source tracking system in place, the multi-agent restoration should:

1. **Track Active Agents**: Show which task agent is currently working
2. **Display Agent Hierarchy**: Indicate when coordinating agent delegates to task agents
3. **Tool Call Attribution**: Show which specific agent made each tool call
4. **Context Handoffs**: Visualize when context is passed between agents

## Example Multi-Agent Flow

```
User: "I need a prompt for data analysis tasks"

CoordinatingAgent: "I'll help you build a comprehensive data analysis prompt. Let me start with understanding your specific context and goals."

→ Spawns TaskContextAgent

TaskContextAgent: "Based on your input, I understand you need data analysis capabilities. Let me clarify a few points..."
[Uses ask_user tool]

TaskContextAgent → CoordinatingAgent: "Section complete. Context accumulated: data analysis, Python/R focus, business intelligence use case"

CoordinatingAgent: "Great! Now let me work on the data structure and XML formatting."

→ Spawns DataSectionAgent with accumulated context

DataSectionAgent: "Given the data analysis context, I'll create XML tags for datasets, analysis parameters, and output formats..."

[Process continues through all 7 sections]

CoordinatingAgent: "All sections complete! Here's your comprehensive enterprise data analysis prompt..."
```

## Benefits of Multi-Agent Architecture

1. **Specialization**: Each agent is expert in their specific domain
2. **Parallel Processing**: Multiple sections can be developed simultaneously
3. **Context Preservation**: Rich context flows between specialized agents
4. **Error Isolation**: Failures in one section don't affect others
5. **User Experience**: More natural conversation flow with appropriate expertise
6. **Extensibility**: Easy to add new section types or specialized agents

This architecture transforms the prompt builder from a single-agent system into a sophisticated multi-agent orchestration platform while maintaining the user-friendly interface.