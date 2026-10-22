//! Contractul furnizorilor de model și furnizorul fals pentru teste offline.
//!
//! `GuardedProvider` trece fiecare cerere prin policy gate și prin registrul de
//! buget înainte de a ajunge la furnizor; un refuz înseamnă zero apeluri.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use thiserror::Error;

use crate::config::AgentsConfig;
use crate::ids::ProviderId;
use crate::plan::PlanBudget;
use crate::policy::{BudgetLedger, Denial, ProviderRequest, authorize_provider_call};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerateResponse {
    pub text: String,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Ready,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ProviderError {
    #[error("refuzat de politică: {0}")]
    Denied(#[from] Denial),
    #[error("furnizorul nu este disponibil")]
    Unavailable,
    #[error("furnizorul nu are răspunsuri pregătite")]
    Exhausted,
}

pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn health(&self) -> Health;
    fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse, ProviderError>;
}

/// Furnizor determinist: răspunsuri prestabilite și numărarea apelurilor (spy).
#[derive(Debug)]
pub struct FakeProvider {
    id: ProviderId,
    cost_per_call_usd: f64,
    responses: Mutex<VecDeque<String>>,
    calls: AtomicUsize,
}

impl FakeProvider {
    pub fn new(id: ProviderId, responses: impl IntoIterator<Item = String>) -> Self {
        Self {
            id,
            cost_per_call_usd: 0.0,
            responses: Mutex::new(responses.into_iter().collect()),
            calls: AtomicUsize::new(0),
        }
    }

    pub fn with_cost(mut self, usd: f64) -> Self {
        self.cost_per_call_usd = usd;
        self
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Provider for FakeProvider {
    fn id(&self) -> ProviderId {
        self.id
    }

    fn health(&self) -> Health {
        Health::Ready
    }

    fn generate(&self, _req: &GenerateRequest) -> Result<GenerateResponse, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let text = self
            .responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
            .ok_or(ProviderError::Exhausted)?;
        Ok(GenerateResponse {
            text,
            cost_usd: self.cost_per_call_usd,
        })
    }
}

/// Furnizor protejat de policy gate și de bugetul rezervat atomic.
pub struct GuardedProvider<'a> {
    pub config: &'a AgentsConfig,
    pub ledger: &'a BudgetLedger,
    pub provider: &'a dyn Provider,
}

impl GuardedProvider<'_> {
    pub fn generate(
        &self,
        plan_budget: Option<&PlanBudget>,
        estimated_usd: Option<f64>,
        req: &GenerateRequest,
    ) -> Result<GenerateResponse, ProviderError> {
        let estimate = authorize_provider_call(
            self.config,
            plan_budget,
            &ProviderRequest {
                provider: self.provider.id(),
                model: &req.model,
                estimated_usd,
            },
        )?;
        let reservation = self.ledger.reserve(estimate)?;
        match self.provider.generate(req) {
            Ok(response) => {
                self.ledger.commit(reservation, response.cost_usd);
                Ok(response)
            }
            Err(err) => {
                // Eroarea furnizorului fals este sigură (nu s-a trimis nimic).
                // Adaptoarele reale marchează rezerva incertă la timeout.
                self.ledger.release(reservation);
                Err(err)
            }
        }
    }
}
