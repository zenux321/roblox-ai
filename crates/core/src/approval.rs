//! Aprobarea unui plan, legată de hash-ul lui canonic.
//!
//! Aprobarea se creează numai printr-o acțiune autentificată din UI (în M1/M2
//! prin worker); textul „approved” dintr-o notă sau dintr-un fișier nu contează.

use serde::Serialize;
use thiserror::Error;

use crate::config::{Issue, Mode};
use crate::plan::{Plan, PlanStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Approval {
    pub plan_id: String,
    pub plan_version: u32,
    pub plan_sha256: String,
    /// Identitatea Windows a utilizatorului care a aprobat.
    pub approved_by: String,
    pub approved_at_utc: String,
    pub revoked_at_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApprovalError {
    #[error("planul nu este valid pentru execuție")]
    InvalidPlan(Vec<Issue>),
    #[error("planul nu așteaptă aprobare")]
    NotAwaitingApproval,
    #[error("lipsește identitatea celui care aprobă")]
    MissingApprover,
}

/// Starea verificării unei aprobări față de planul curent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalCheck {
    Valid,
    Revoked,
    PlanMismatch,
    HashMismatch,
}

impl Approval {
    pub fn grant(plan: &Plan, approver: &str, at_utc: &str) -> Result<Self, ApprovalError> {
        if approver.trim().is_empty() {
            return Err(ApprovalError::MissingApprover);
        }
        if plan.status != PlanStatus::AwaitingApproval {
            return Err(ApprovalError::NotAwaitingApproval);
        }
        let issues = plan.validate(Mode::Execution);
        if !issues.is_empty() {
            return Err(ApprovalError::InvalidPlan(issues));
        }
        Ok(Self {
            plan_id: plan.id.clone(),
            plan_version: plan.version,
            plan_sha256: plan.sha256(),
            approved_by: approver.to_string(),
            approved_at_utc: at_utc.to_string(),
            revoked_at_utc: None,
        })
    }

    pub fn revoke(&mut self, at_utc: &str) {
        if self.revoked_at_utc.is_none() {
            self.revoked_at_utc = Some(at_utc.to_string());
        }
    }

    pub fn check(&self, plan: &Plan) -> ApprovalCheck {
        if self.revoked_at_utc.is_some() {
            ApprovalCheck::Revoked
        } else if self.plan_id != plan.id || self.plan_version != plan.version {
            ApprovalCheck::PlanMismatch
        } else if self.plan_sha256 != plan.sha256() {
            ApprovalCheck::HashMismatch
        } else {
            ApprovalCheck::Valid
        }
    }
}
