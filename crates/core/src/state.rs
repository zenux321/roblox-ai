//! Mașina de stări a planului și a rulării.
//!
//! Plan: `draft → awaiting_approval → approved`; revocarea sau editarea
//! readuce planul în `awaiting_approval`. Run: `queued → running →
//! succeeded | failed | cancelled`, cu stări reluabile `paused`,
//! `waiting_dependency`, `waiting_approval`, `recovery_required`.
//! Un task reușit nu se mai execută, nici după reconectarea UI.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::plan::{Plan, PlanStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("tranziție nepermisă: {from:?} → {to:?}")]
pub struct InvalidTransition<S: std::fmt::Debug> {
    pub from: S,
    pub to: S,
}

pub fn plan_transition_allowed(from: PlanStatus, to: PlanStatus) -> bool {
    use PlanStatus::*;
    matches!(
        (from, to),
        (Draft, AwaitingApproval)
            | (AwaitingApproval, Draft)
            | (AwaitingApproval, Approved)
            | (Approved, AwaitingApproval)
            | (Draft | AwaitingApproval | Approved, Superseded)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Paused,
    WaitingDependency,
    WaitingApproval,
    RecoveryRequired,
    Succeeded,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            RunStatus::Succeeded | RunStatus::Failed | RunStatus::Cancelled
        )
    }
}

pub fn run_transition_allowed(from: RunStatus, to: RunStatus) -> bool {
    use RunStatus::*;
    if from.is_terminal() {
        return false;
    }
    match from {
        Queued => matches!(to, Running | Cancelled),
        Running => !matches!(to, Queued | Running),
        Paused | WaitingDependency | WaitingApproval => matches!(to, Running | Cancelled),
        // Reluarea după recovery cere reconciliere explicită (vezi `resolve_recovery`).
        RecoveryRequired => matches!(to, Running | Failed | Cancelled),
        Succeeded | Failed | Cancelled => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum TaskState {
    Pending,
    Running {
        attempt: u32,
    },
    Succeeded {
        attempt: u32,
        receipt_sha256: String,
    },
    Failed {
        attempts: u32,
    },
    /// Efectul este incert (crash, timeout după trimitere); nu se repetă orb.
    RecoveryRequired {
        attempt: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RunError {
    #[error("sarcina `{0}` nu există")]
    UnknownTask(String),
    #[error("sarcina `{0}` a reușit deja și nu se reexecută")]
    AlreadySucceeded(String),
    #[error("sarcina `{0}` rulează deja")]
    AlreadyRunning(String),
    #[error("sarcina `{0}` așteaptă dependențe")]
    DependenciesPending(String),
    #[error("sarcina `{0}` cere reconciliere înainte de reluare")]
    RecoveryPending(String),
    #[error("sarcina `{0}` nu este în starea cerută")]
    WrongTaskState(String),
    #[error("rularea nu permite operația în starea {0:?}")]
    WrongRunStatus(RunStatus),
    #[error("limita de reîncercări a fost atinsă pentru `{0}`")]
    RetriesExhausted(String),
    #[error("tranziție nepermisă {from:?} → {to:?}")]
    Transition { from: RunStatus, to: RunStatus },
}

/// Rezultatul reconcilierii unui efect incert, decis de adaptor sau de utilizator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Efectul a fost găsit (de ex. commit, fișier cu hash-ul așteptat).
    EffectConfirmed { receipt_sha256: String },
    /// Efectul sigur nu s-a produs; sarcina poate fi reluată.
    EffectAbsent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub plan_id: String,
    pub plan_sha256: String,
    pub status: RunStatus,
    max_retries: u32,
    tasks: Vec<(String, Vec<String>, TaskState)>,
}

impl Run {
    pub fn new(plan: &Plan) -> Self {
        Self {
            plan_id: plan.id.clone(),
            plan_sha256: plan.sha256(),
            status: RunStatus::Queued,
            max_retries: plan.budget.max_retries_per_task,
            tasks: plan
                .tasks
                .iter()
                .map(|t| (t.id.clone(), t.depends_on.clone(), TaskState::Pending))
                .collect(),
        }
    }

    /// Sarcinile în ordinea planului, cu starea lor.
    pub fn tasks(&self) -> impl Iterator<Item = (&str, &TaskState)> {
        self.tasks.iter().map(|(id, _, state)| (id.as_str(), state))
    }

    pub fn task_state(&self, id: &str) -> Option<&TaskState> {
        self.tasks
            .iter()
            .find(|(t, _, _)| t == id)
            .map(|(_, _, s)| s)
    }

    pub fn transition(&mut self, to: RunStatus) -> Result<(), RunError> {
        if run_transition_allowed(self.status, to) {
            self.status = to;
            Ok(())
        } else {
            Err(RunError::Transition {
                from: self.status,
                to,
            })
        }
    }

    fn entry_mut(&mut self, id: &str) -> Result<&mut (String, Vec<String>, TaskState), RunError> {
        self.tasks
            .iter_mut()
            .find(|(t, _, _)| t == id)
            .ok_or_else(|| RunError::UnknownTask(id.to_string()))
    }

    fn dependencies_done(&self, deps: &[String]) -> bool {
        deps.iter()
            .all(|d| matches!(self.task_state(d), Some(TaskState::Succeeded { .. })))
    }

    /// Sarcinile care pot porni acum, în ordinea planului: cele noi și cele
    /// eșuate sigur care mai au reîncercări.
    pub fn ready_tasks(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter(|(_, deps, s)| {
                let startable = match s {
                    TaskState::Pending => true,
                    TaskState::Failed { attempts } => *attempts <= self.max_retries,
                    _ => false,
                };
                startable && self.dependencies_done(deps)
            })
            .map(|(id, _, _)| id.clone())
            .collect()
    }

    pub fn start_task(&mut self, id: &str) -> Result<u32, RunError> {
        if self.status == RunStatus::Queued {
            self.transition(RunStatus::Running)?;
        }
        if self.status != RunStatus::Running {
            return Err(RunError::WrongRunStatus(self.status));
        }
        let deps = self.entry_mut(id)?.1.clone();
        let deps_done = self.dependencies_done(&deps);
        let max_retries = self.max_retries;
        let entry = self.entry_mut(id)?;
        let attempt = match &entry.2 {
            TaskState::Succeeded { .. } => return Err(RunError::AlreadySucceeded(id.into())),
            TaskState::Running { .. } => return Err(RunError::AlreadyRunning(id.into())),
            TaskState::RecoveryRequired { .. } => return Err(RunError::RecoveryPending(id.into())),
            TaskState::Failed { attempts } if *attempts > max_retries => {
                return Err(RunError::RetriesExhausted(id.into()));
            }
            TaskState::Failed { attempts } => attempts + 1,
            TaskState::Pending => 1,
        };
        if !deps_done {
            return Err(RunError::DependenciesPending(id.into()));
        }
        entry.2 = TaskState::Running { attempt };
        Ok(attempt)
    }

    pub fn complete_task(&mut self, id: &str, receipt_sha256: &str) -> Result<(), RunError> {
        let entry = self.entry_mut(id)?;
        let TaskState::Running { attempt } = entry.2 else {
            return Err(RunError::WrongTaskState(id.into()));
        };
        entry.2 = TaskState::Succeeded {
            attempt,
            receipt_sha256: receipt_sha256.to_string(),
        };
        if self
            .tasks
            .iter()
            .all(|(_, _, s)| matches!(s, TaskState::Succeeded { .. }))
        {
            self.transition(RunStatus::Succeeded)?;
        }
        Ok(())
    }

    /// Eșec sigur (fără efect produs). Se poate reîncerca până la limită.
    pub fn fail_task(&mut self, id: &str) -> Result<(), RunError> {
        let max_retries = self.max_retries;
        let entry = self.entry_mut(id)?;
        let TaskState::Running { attempt } = entry.2 else {
            return Err(RunError::WrongTaskState(id.into()));
        };
        entry.2 = TaskState::Failed { attempts: attempt };
        if attempt > max_retries {
            self.transition(RunStatus::Failed)?;
        }
        Ok(())
    }

    /// După restart/crash: sarcinile rămase „running” au efect incert.
    pub fn reconcile_after_restart(&mut self) {
        let mut any = false;
        for (_, _, state) in &mut self.tasks {
            if let TaskState::Running { attempt } = *state {
                *state = TaskState::RecoveryRequired { attempt };
                any = true;
            }
        }
        if any && !self.status.is_terminal() {
            self.status = RunStatus::RecoveryRequired;
        }
    }

    /// Reconectarea UI nu schimbă nimic: starea vine din rulare, nu din UI.
    pub fn on_ui_reconnect(&self) -> &Self {
        self
    }

    pub fn resolve_recovery(&mut self, id: &str, resolution: Resolution) -> Result<(), RunError> {
        let entry = self.entry_mut(id)?;
        let TaskState::RecoveryRequired { attempt } = entry.2 else {
            return Err(RunError::WrongTaskState(id.into()));
        };
        entry.2 = match resolution {
            Resolution::EffectConfirmed { receipt_sha256 } => TaskState::Succeeded {
                attempt,
                receipt_sha256,
            },
            Resolution::EffectAbsent => TaskState::Failed { attempts: attempt },
        };
        let still_pending = self
            .tasks
            .iter()
            .any(|(_, _, s)| matches!(s, TaskState::RecoveryRequired { .. }));
        if !still_pending && self.status == RunStatus::RecoveryRequired {
            let all_done = self
                .tasks
                .iter()
                .all(|(_, _, s)| matches!(s, TaskState::Succeeded { .. }));
            self.transition(RunStatus::Running)?;
            if all_done {
                self.transition(RunStatus::Succeeded)?;
            }
        }
        Ok(())
    }
}
