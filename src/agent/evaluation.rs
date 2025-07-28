use crate::agent::Agent;
use serde::{Deserialize, Serialize};

/// Configuration for a single perspective in multi-perspective evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveConfig {
    /// Name of the perspective (e.g., "quality_check", "user_satisfaction")
    pub name: String,
    /// Prompt to use for this perspective's evaluation
    pub prompt: String,
    /// Weight given to this perspective in the final decision (0.0 to 1.0)
    pub weight: f32,
}

/// Evaluation strategies for determining event loop continuation
#[derive(Debug, Clone)]
pub enum EvaluationStrategy {
    /// No explicit evaluation - model-driven continuation (DEFAULT)
    /// The model itself decides when to continue or stop based on natural reasoning
    /// Agents will continue if the model requests tool calls or indicates more work needed
    None,
    
    /// Task evaluation using same agent with custom prompt
    /// The agent evaluates whether the user's request has been fully satisfied
    TaskEvaluation {
        /// Prompt to use for task completion evaluation
        evaluation_prompt: String,
        /// Maximum number of evaluation cycles before forcing termination
        max_iterations: u32,
    },
    
    /// Multi-perspective evaluation
    /// Multiple evaluation perspectives are combined to make the continuation decision
    MultiPerspective {
        /// List of perspectives to evaluate
        perspectives: Vec<PerspectiveConfig>,
    },
    
    /// Agent-based evaluation using separate evaluator agent
    /// A separate agent is used to evaluate the main agent's work
    AgentBased {
        /// The evaluator agent instance
        evaluator_agent: Box<Agent>,
        /// Prompt to use when calling the evaluator agent
        evaluation_prompt: String,
    },
}

impl Default for EvaluationStrategy {
    fn default() -> Self {
        EvaluationStrategy::None
    }
}

impl EvaluationStrategy {
    /// Create a new task evaluation strategy with custom prompt
    pub fn task_evaluation(prompt: impl Into<String>) -> Self {
        Self::TaskEvaluation {
            evaluation_prompt: prompt.into(),
            max_iterations: 5,
        }
    }
    
    /// Create a new multi-perspective evaluation strategy
    pub fn multi_perspective(perspectives: Vec<PerspectiveConfig>) -> Self {
        Self::MultiPerspective { perspectives }
    }
    
    /// Create a new agent-based evaluation strategy
    pub fn agent_based(evaluator_agent: Agent, evaluation_prompt: impl Into<String>) -> Self {
        Self::AgentBased {
            evaluator_agent: Box::new(evaluator_agent),
            evaluation_prompt: evaluation_prompt.into(),
        }
    }
    
    /// Get the name of the evaluation strategy for logging
    pub fn name(&self) -> &'static str {
        match self {
            EvaluationStrategy::None => "model_driven",
            EvaluationStrategy::TaskEvaluation { .. } => "task_evaluation",
            EvaluationStrategy::MultiPerspective { .. } => "multi_perspective",
            EvaluationStrategy::AgentBased { .. } => "agent_based",
        }
    }
    
    /// Check if this strategy requires LLM evaluation calls
    pub fn requires_evaluation(&self) -> bool {
        match self {
            EvaluationStrategy::None => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evaluation_strategy_default() {
        let strategy = EvaluationStrategy::default();
        assert_eq!(strategy.name(), "model_driven");
        assert!(!strategy.requires_evaluation());
    }
    
    #[test]
    fn test_task_evaluation_strategy() {
        let strategy = EvaluationStrategy::task_evaluation("Test prompt");
        assert_eq!(strategy.name(), "task_evaluation");
        assert!(strategy.requires_evaluation());
        
        if let EvaluationStrategy::TaskEvaluation { evaluation_prompt, max_iterations } = strategy {
            assert_eq!(evaluation_prompt, "Test prompt");
            assert_eq!(max_iterations, 5);
        } else {
            panic!("Expected TaskEvaluation strategy");
        }
    }
    
    #[test]
    fn test_perspective_config() {
        let perspective = PerspectiveConfig {
            name: "test_perspective".to_string(),
            prompt: "Test prompt".to_string(),
            weight: 0.5,
        };
        
        assert_eq!(perspective.name, "test_perspective");
        assert_eq!(perspective.prompt, "Test prompt");
        assert_eq!(perspective.weight, 0.5);
    }
}