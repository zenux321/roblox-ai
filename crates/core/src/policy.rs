//! Policy gate: verificarea fiecărui tool call și a fiecărui apel de furnizor.
//!
//! Permisiunea efectivă este intersecția politicii globale (config), a
//! scope-ului planului aprobat și a instrumentelor sarcinii. Orice lipsă
//! înseamnă refuz. Funcțiile sunt pure; efectele le aplică executorul.

use std::sync::Mutex;

use thiserror::Error;

use crate::approval::{Approval, ApprovalCheck};
use crate::config::{AgentsConfig, is_public_hostname, parse_simple_url};
use crate::ids::{ExternalAction, ProviderId, ToolEffect, ToolId};
use crate::paths::{PathError, RelPath, parse_relative};
use crate::plan::{Plan, PlanBudget};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum Denial {
    #[error("rularea a fost anulată")]
    RunCancelled,
    #[error("configurația este exemplu și nu poate executa")]
    ExampleConfig,
    #[error("planul aparține altui proiect decât configurația activă")]
    ProjectMismatch,
    #[error("planul nu este aprobat")]
    NoApproval,
    #[error("aprobarea a fost revocată")]
    ApprovalRevoked,
    #[error("aprobarea aparține altui plan sau altei versiuni")]
    ApprovalPlanMismatch,
    #[error("planul s-a schimbat după aprobare")]
    PlanHashMismatch,
    #[error("sarcina `{0}` nu există în plan")]
    UnknownTask(String),
    #[error("instrumentul `{0}` nu este permis pentru sarcină")]
    ToolNotAllowedForTask(ToolId),
    #[error("ținta nu corespunde instrumentului `{0}`")]
    TargetMismatch(ToolId),
    #[error("cale invalidă: {0}")]
    InvalidPath(PathError),
    #[error("calea `{0}` este în afara căilor de scriere ale planului")]
    PathOutsideWriteScope(String),
    #[error("acces de rețea refuzat: {0}")]
    NetworkDenied(String),
    #[error("politica globală interzice {0:?}")]
    GlobalPolicyForbids(ExternalAction),
    #[error("planul nu include {0:?}")]
    PlanScopeForbids(ExternalAction),
    #[error("planul nu are destinația exactă pentru {0:?}")]
    ExternalDestinationMissing(ExternalAction),
    #[error("destinația diferă de cea din plan")]
    DestinationMismatch,
    #[error("furnizorul `{0}` este dezactivat")]
    ProviderDisabled(ProviderId),
    #[error("modelul `{0}` rulează remote și este blocat în modul local")]
    CloudModelBlocked(String),
    #[error("apelurile plătite sunt dezactivate")]
    PaidCallsDisabled,
    #[error("prețul este necunoscut; politica cere refuz")]
    UnknownPrice,
    #[error("bugetul este 0 USD")]
    ZeroBudget,
    #[error("bugetul ar fi depășit")]
    BudgetExceeded,
}

/// Contextul unei verificări: configurația activă, planul și aprobarea lui.
#[derive(Debug, Clone, Copy)]
pub struct PolicyContext<'a> {
    pub config: &'a AgentsConfig,
    pub plan: &'a Plan,
    pub approval: Option<&'a Approval>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target<'a> {
    None,
    ProjectPath(&'a str),
    Url(&'a str),
    External {
        action: ExternalAction,
        destination: &'a str,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ToolRequest<'a> {
    pub task_id: &'a str,
    pub tool: ToolId,
    pub target: Target<'a>,
}

/// Dovada că un tool call a trecut de policy gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permit {
    pub task_id: String,
    pub tool: ToolId,
    pub plan_sha256: String,
    pub path: Option<RelPath>,
}

pub fn authorize_tool(ctx: &PolicyContext<'_>, req: &ToolRequest<'_>) -> Result<Permit, Denial> {
    let plan_sha256 = check_run_preconditions(ctx)?;
    let task = ctx
        .plan
        .task(req.task_id)
        .ok_or_else(|| Denial::UnknownTask(req.task_id.to_string()))?;
    if !task.allowed_tools.contains(&req.tool) {
        return Err(Denial::ToolNotAllowedForTask(req.tool));
    }

    let mut path = None;
    match (req.tool.effect(), req.target) {
        (ToolEffect::Read | ToolEffect::Propose, Target::None) => {}
        (ToolEffect::Read | ToolEffect::Propose, Target::ProjectPath(p)) => {
            path = Some(parse_relative(p).map_err(Denial::InvalidPath)?);
        }
        (ToolEffect::LocalWrite, Target::ProjectPath(p)) => {
            let rel = parse_relative(p).map_err(Denial::InvalidPath)?;
            if !ctx
                .plan
                .write_scopes()
                .iter()
                .any(|scope| rel.is_within(scope))
            {
                return Err(Denial::PathOutsideWriteScope(rel.as_string()));
            }
            path = Some(rel);
        }
        (ToolEffect::Network, Target::Url(url)) => check_research_url(ctx, url)?,
        (
            ToolEffect::External(action),
            Target::External {
                action: requested,
                destination,
            },
        ) if action == requested => {
            check_external(ctx, action, destination)?;
        }
        _ => return Err(Denial::TargetMismatch(req.tool)),
    }

    Ok(Permit {
        task_id: task.id.clone(),
        tool: req.tool,
        plan_sha256,
        path,
    })
}

/// Anulare, config exemplu și aprobare validă pentru exact acest plan.
fn check_run_preconditions(ctx: &PolicyContext<'_>) -> Result<String, Denial> {
    if ctx.cancelled {
        return Err(Denial::RunCancelled);
    }
    if ctx.config.example_only || ctx.plan.example_only {
        return Err(Denial::ExampleConfig);
    }
    if ctx.plan.project_id != ctx.config.project.id {
        return Err(Denial::ProjectMismatch);
    }
    let approval = ctx.approval.ok_or(Denial::NoApproval)?;
    match approval.check(ctx.plan) {
        ApprovalCheck::Valid => Ok(approval.plan_sha256.clone()),
        ApprovalCheck::Revoked => Err(Denial::ApprovalRevoked),
        ApprovalCheck::PlanMismatch => Err(Denial::ApprovalPlanMismatch),
        ApprovalCheck::HashMismatch => Err(Denial::PlanHashMismatch),
    }
}

/// Cercetare: https, host public din allowlist-ul global ȘI destinație din plan.
/// Rezolvarea DNS și blocarea IP-urilor private la conectare (S08 complet)
/// aparțin adaptorului de fetch din M5.
fn check_research_url(ctx: &PolicyContext<'_>, url: &str) -> Result<(), Denial> {
    let parsed =
        parse_simple_url(url).ok_or_else(|| Denial::NetworkDenied("URL invalid".into()))?;
    if parsed.scheme != "https" {
        return Err(Denial::NetworkDenied("doar https".into()));
    }
    if !is_public_hostname(&parsed.host) {
        return Err(Denial::NetworkDenied(
            "ținta nu este un hostname public".into(),
        ));
    }
    let policy = &ctx.config.policy;
    if !policy
        .research_allowed_hosts
        .iter()
        .any(|h| h == &parsed.host)
    {
        return Err(Denial::NetworkDenied(format!(
            "{} nu este în allowlist",
            parsed.host
        )));
    }
    let in_plan = ctx.plan.scope.network_destinations.iter().any(|dest| {
        parse_simple_url(dest)
            .is_some_and(|d| d.scheme == "https" && d.host == parsed.host && d.port == parsed.port)
    });
    if !in_plan {
        return Err(Denial::NetworkDenied(format!(
            "{} nu este în planul aprobat",
            parsed.host
        )));
    }
    Ok(())
}

fn check_external(
    ctx: &PolicyContext<'_>,
    action: ExternalAction,
    destination: &str,
) -> Result<(), Denial> {
    let policy = &ctx.config.policy;
    let globally_allowed = match action {
        ExternalAction::GitPush => policy.allow_git_push,
        ExternalAction::PullRequest => policy.allow_github_pr,
        ExternalAction::Upload => policy.allow_upload,
        ExternalAction::Publish => policy.allow_publish,
    };
    if !globally_allowed {
        return Err(Denial::GlobalPolicyForbids(action));
    }
    let scope = &ctx.plan.scope;
    match action {
        ExternalAction::GitPush | ExternalAction::PullRequest => {
            if !scope.allow_push {
                return Err(Denial::PlanScopeForbids(action));
            }
            let repo = scope
                .github_repository
                .as_deref()
                .ok_or(Denial::ExternalDestinationMissing(action))?;
            let github = &ctx.config.integrations.github;
            if !github.enabled || github.repository.as_deref() != Some(repo) || destination != repo
            {
                return Err(Denial::DestinationMismatch);
            }
            Ok(())
        }
        ExternalAction::Upload | ExternalAction::Publish => {
            let in_scope = match action {
                ExternalAction::Upload => scope.allow_upload,
                _ => scope.allow_publish,
            };
            if !in_scope {
                return Err(Denial::PlanScopeForbids(action));
            }
            // Schema 0.1 a planului nu are place/universe/asset exacte, deci
            // efectul rămâne interzis până la extinderea ei (ARCHITECTURE §4).
            Err(Denial::ExternalDestinationMissing(action))
        }
    }
}

/// Cerere către un furnizor de model.
#[derive(Debug, Clone, Copy)]
pub struct ProviderRequest<'a> {
    pub provider: ProviderId,
    pub model: &'a str,
    /// Cost estimat în USD; `None` înseamnă preț necunoscut.
    pub estimated_usd: Option<f64>,
}

/// Verifică un apel de furnizor. `plan_budget` este `None` pentru discuțiile
/// din chat, care nu au plan: atunci se aplică numai bugetul global.
pub fn authorize_provider_call(
    config: &AgentsConfig,
    plan_budget: Option<&PlanBudget>,
    req: &ProviderRequest<'_>,
) -> Result<f64, Denial> {
    if config.example_only {
        return Err(Denial::ExampleConfig);
    }
    if !config.provider_enabled(req.provider) {
        return Err(Denial::ProviderDisabled(req.provider));
    }
    if !req.provider.is_paid() {
        if !config.providers.ollama.allow_cloud_models && is_ollama_cloud_model(req.model) {
            return Err(Denial::CloudModelBlocked(req.model.to_string()));
        }
        return Ok(0.0);
    }
    if !config.policy.allow_paid_calls {
        return Err(Denial::PaidCallsDisabled);
    }
    let estimate = match req.estimated_usd {
        Some(v) if v.is_finite() && v >= 0.0 => v,
        _ => return Err(Denial::UnknownPrice),
    };
    let limit = plan_budget.map_or(config.budget.max_run_usd, |b| {
        b.max_usd.min(config.budget.max_run_usd)
    });
    if limit <= 0.0 || config.budget.max_month_usd <= 0.0 {
        return Err(Denial::ZeroBudget);
    }
    if estimate > limit {
        return Err(Denial::BudgetExceeded);
    }
    Ok(estimate)
}

/// Euristică parțială: modelele cloud din Ollama au tag-ul terminat în `cloud`
/// (de ex. `model:120b-cloud`). Verificarea completă, prin metadatele reale ale
/// modelului, se face în adaptorul Ollama din M3, după documentația oficială.
pub fn is_ollama_cloud_model(model: &str) -> bool {
    let lower = model.to_ascii_lowercase();
    let tag = lower
        .rsplit_once(':')
        .map_or(lower.as_str(), |(_, tag)| tag);
    tag == "cloud" || tag.ends_with("-cloud")
}

/// Registru de buget cu rezervare atomică (în memorie; persistența SQLite vine în M1).
#[derive(Debug)]
pub struct BudgetLedger {
    limit_usd: f64,
    state: Mutex<LedgerState>,
}

#[derive(Debug, Default)]
struct LedgerState {
    next_id: u64,
    committed_usd: f64,
    open: Vec<(u64, f64, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservationId(u64);

impl BudgetLedger {
    pub fn new(limit_usd: f64) -> Self {
        Self {
            limit_usd,
            state: Mutex::new(LedgerState::default()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, LedgerState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Rezervă suma sau refuză dacă rezervările + cheltuielile ar depăși limita.
    pub fn reserve(&self, amount_usd: f64) -> Result<ReservationId, Denial> {
        if !amount_usd.is_finite() || amount_usd < 0.0 {
            return Err(Denial::UnknownPrice);
        }
        if self.limit_usd <= 0.0 && amount_usd > 0.0 {
            return Err(Denial::ZeroBudget);
        }
        let mut s = self.lock();
        let reserved: f64 = s.open.iter().map(|(_, a, _)| a).sum();
        if s.committed_usd + reserved + amount_usd > self.limit_usd + 1e-9 {
            return Err(Denial::BudgetExceeded);
        }
        s.next_id += 1;
        let id = s.next_id;
        s.open.push((id, amount_usd, false));
        Ok(ReservationId(id))
    }

    /// Înregistrează costul real și eliberează rezerva.
    pub fn commit(&self, id: ReservationId, actual_usd: f64) {
        let mut s = self.lock();
        if let Some(pos) = s.open.iter().position(|(i, _, _)| *i == id.0) {
            s.open.remove(pos);
            s.committed_usd += actual_usd.max(0.0);
        }
    }

    /// Eliberează rezerva numai dacă cererea sigur nu a ajuns la furnizor.
    /// O rezervă marcată incertă nu se eliberează automat.
    pub fn release(&self, id: ReservationId) -> bool {
        let mut s = self.lock();
        match s
            .open
            .iter()
            .position(|(i, _, uncertain)| *i == id.0 && !uncertain)
        {
            Some(pos) => {
                s.open.remove(pos);
                true
            }
            None => false,
        }
    }

    /// Timeout după trimitere: costul este „în curs de verificare”.
    pub fn mark_uncertain(&self, id: ReservationId) {
        let mut s = self.lock();
        if let Some(entry) = s.open.iter_mut().find(|(i, _, _)| *i == id.0) {
            entry.2 = true;
        }
    }

    pub fn committed_usd(&self) -> f64 {
        self.lock().committed_usd
    }

    pub fn reserved_usd(&self) -> f64 {
        self.lock().open.iter().map(|(_, a, _)| a).sum()
    }
}
