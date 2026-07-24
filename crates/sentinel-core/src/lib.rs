mod error;
mod model;
mod parser;

pub use error::ParseError;
pub use model::{
    AnalysisMode, AnalysisRequest, AnalysisResult, ContextSummary, Decision, Finding, RiskLabel,
    Severity,
};
pub use parser::{ParsedCommand, ParsedCommandLine, ShellOperator, parse_command};
