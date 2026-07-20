use std::path::PathBuf;

use sentinel_core::{
    AnalysisMode, AnalysisRequest, AnalysisResult, ContextSummary, Decision, Finding, RiskLabel,
    Severity,
};
use serde_json::{Value, json};

#[test]
fn analysis_modes_have_stable_json_values() {
    let cases = [
        (AnalysisMode::CommandOnly, "command-only"),
        (AnalysisMode::FilesystemAware, "filesystem"),
        (AnalysisMode::GitAware, "git"),
    ];

    for (mode, expected) in cases {
        assert_eq!(serde_json::to_value(mode).unwrap(), json!(expected));
    }
}

#[test]
fn labels_severities_and_decisions_have_stable_json_values() {
    let labels = [
        (RiskLabel::Legitimate, "legitimate"),
        (RiskLabel::Dangerous, "dangerous"),
    ];
    let severities = [
        (Severity::Safe, "safe"),
        (Severity::Low, "low"),
        (Severity::Medium, "medium"),
        (Severity::High, "high"),
        (Severity::Critical, "critical"),
    ];
    let decisions = [
        (Decision::Allow, "allow"),
        (Decision::RequireApproval, "require_approval"),
        (Decision::Deny, "deny"),
        (Decision::AnalysisIncomplete, "analysis_incomplete"),
    ];

    for (label, expected) in labels {
        assert_eq!(serde_json::to_value(label).unwrap(), json!(expected));
    }
    for (severity, expected) in severities {
        assert_eq!(serde_json::to_value(severity).unwrap(), json!(expected));
    }
    for (decision, expected) in decisions {
        assert_eq!(serde_json::to_value(decision).unwrap(), json!(expected));
    }
}

#[test]
fn analysis_request_round_trips_through_json() {
    let request = AnalysisRequest {
        command: "cargo test".to_owned(),
        working_directory: PathBuf::from("workspace/project"),
        mode: AnalysisMode::CommandOnly,
    };

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(
        value,
        json!({
            "command": "cargo test",
            "working_directory": "workspace/project",
            "mode": "command-only"
        })
    );
    assert_eq!(
        serde_json::from_value::<AnalysisRequest>(value).unwrap(),
        request
    );
}

#[test]
fn dangerous_analysis_result_has_the_documented_shape() {
    let result = AnalysisResult {
        command: "rm -rf src".to_owned(),
        mode: AnalysisMode::CommandOnly,
        label: RiskLabel::Dangerous,
        severity: Severity::High,
        score: 30,
        decision: Decision::Deny,
        findings: vec![Finding {
            rule_id: "filesystem.recursive-delete".to_owned(),
            severity: Severity::High,
            score_delta: 30,
            message: "The command recursively deletes a directory.".to_owned(),
            matched_fragment: Some("rm -rf src".to_owned()),
        }],
        context_summary: ContextSummary {
            filesystem_used: false,
            git_used: false,
            context_complete: true,
            warnings: vec![],
        },
    };

    let value = serde_json::to_value(&result).unwrap();

    assert_eq!(
        value,
        json!({
            "command": "rm -rf src",
            "mode": "command-only",
            "label": "dangerous",
            "severity": "high",
            "score": 30,
            "decision": "deny",
            "findings": [{
                "rule_id": "filesystem.recursive-delete",
                "severity": "high",
                "score_delta": 30,
                "message": "The command recursively deletes a directory.",
                "matched_fragment": "rm -rf src"
            }],
            "context_summary": {
                "filesystem_used": false,
                "git_used": false,
                "context_complete": true,
                "warnings": []
            }
        })
    );
    assert_eq!(
        serde_json::from_value::<AnalysisResult>(value).unwrap(),
        result
    );
}

#[test]
fn legitimate_result_keeps_empty_collections_in_json() {
    let result = AnalysisResult {
        command: "cargo test".to_owned(),
        mode: AnalysisMode::CommandOnly,
        label: RiskLabel::Legitimate,
        severity: Severity::Safe,
        score: 0,
        decision: Decision::Allow,
        findings: vec![],
        context_summary: ContextSummary {
            filesystem_used: false,
            git_used: false,
            context_complete: true,
            warnings: vec![],
        },
    };

    let value = serde_json::to_value(result).unwrap();

    assert_eq!(value["findings"], json!([]));
    assert_eq!(value["context_summary"]["warnings"], json!([]));
}

#[test]
fn missing_matched_fragment_serializes_as_null() {
    let finding = Finding {
        rule_id: "analysis.no-risk".to_owned(),
        severity: Severity::Safe,
        score_delta: 0,
        message: "No dangerous command-only pattern was detected.".to_owned(),
        matched_fragment: None,
    };

    let value = serde_json::to_value(finding).unwrap();

    assert_eq!(value["matched_fragment"], Value::Null);
}
