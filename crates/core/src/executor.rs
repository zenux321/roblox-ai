//! Executor demonstrativ: aplică un patch text într-o rădăcină de proiect,
//! numai după policy gate. Fără shell și fără execuția textului unui model.
//!
//! Protecții prezente: validare lexicală, refuzul symlink-urilor pe drum,
//! verificarea hash-ului de bază, scriere temporară + rename, audit.
//! Lipsesc (Windows/worker): deschidere pe handle contra curselor junction
//! (S04) și backup înaintea scrierii.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::ids::ToolId;
use crate::paths::RelPath;
use crate::plan::sha256_hex;
use crate::policy::{Denial, PolicyContext, Target, ToolRequest, authorize_tool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPatch {
    pub path: String,
    /// Hash-ul conținutului pe care s-a bazat propunerea; `None` = fișier nou.
    pub expected_base_sha256: Option<String>,
    pub new_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Receipt {
    pub task_id: String,
    pub path: String,
    pub sha256: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AuditEvent {
    Refused {
        task_id: String,
        tool: String,
        reason: String,
    },
    Intent {
        task_id: String,
        tool: String,
        path: String,
    },
    Effect {
        task_id: String,
        path: String,
        sha256: String,
    },
    Conflict {
        task_id: String,
        path: String,
    },
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("refuzat: {0}")]
    Denied(#[from] Denial),
    #[error("conținutul depășește limita de {0} bytes")]
    TooLarge(u64),
    #[error("fișierul s-a schimbat față de baza propunerii")]
    BaseConflict,
    #[error("calea trece printr-o legătură simbolică")]
    SymlinkInPath,
    #[error("calea iese din rădăcina proiectului")]
    OutsideRoot,
    #[error("rădăcina executorului diferă de rădăcina din plan")]
    RootMismatch,
    #[error("eroare de fișier: {0}")]
    Io(#[from] std::io::Error),
}

pub struct FakeExecutor<'a> {
    ctx: PolicyContext<'a>,
    root: PathBuf,
    audit: Vec<AuditEvent>,
}

impl<'a> FakeExecutor<'a> {
    pub fn new(ctx: PolicyContext<'a>, root: &Path) -> Result<Self, ExecError> {
        let root = root.canonicalize()?;
        if Path::new(&ctx.plan.project_root).canonicalize()? != root {
            return Err(ExecError::RootMismatch);
        }
        Ok(Self {
            ctx,
            root,
            audit: Vec::new(),
        })
    }

    pub fn audit(&self) -> &[AuditEvent] {
        &self.audit
    }

    pub fn cancel(&mut self) {
        self.ctx.cancelled = true;
    }

    pub fn apply_validated_patch(
        &mut self,
        task_id: &str,
        patch: &TextPatch,
    ) -> Result<Receipt, ExecError> {
        let tool = ToolId::ProjectApplyValidatedPatch;
        let request = ToolRequest {
            task_id,
            tool,
            target: Target::ProjectPath(&patch.path),
        };
        let permit = match authorize_tool(&self.ctx, &request) {
            Ok(p) => p,
            Err(denial) => {
                self.audit.push(AuditEvent::Refused {
                    task_id: task_id.into(),
                    tool: tool.as_str().into(),
                    reason: denial.to_string(),
                });
                return Err(denial.into());
            }
        };
        let rel = permit.path.ok_or(Denial::TargetMismatch(tool))?;
        let limit = self.ctx.config.runtime.max_tool_output_bytes;
        if patch.new_content.len() as u64 > limit {
            return Err(ExecError::TooLarge(limit));
        }

        let target = self.resolve_inside_root(&rel)?;
        self.audit.push(AuditEvent::Intent {
            task_id: task_id.into(),
            tool: tool.as_str().into(),
            path: rel.as_string(),
        });

        let current = match fs::read(&target) {
            Ok(bytes) => Some(sha256_hex(&bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if current != patch.expected_base_sha256 {
            self.audit.push(AuditEvent::Conflict {
                task_id: task_id.into(),
                path: rel.as_string(),
            });
            return Err(ExecError::BaseConflict);
        }

        write_atomically(&target, patch.new_content.as_bytes())?;
        let sha256 = sha256_hex(patch.new_content.as_bytes());
        self.audit.push(AuditEvent::Effect {
            task_id: task_id.into(),
            path: rel.as_string(),
            sha256: sha256.clone(),
        });
        Ok(Receipt {
            task_id: task_id.into(),
            path: rel.as_string(),
            sha256,
            bytes: patch.new_content.len(),
        })
    }

    /// Creează directoarele lipsă și refuză orice symlink pe drum.
    fn resolve_inside_root(&self, rel: &RelPath) -> Result<PathBuf, ExecError> {
        let mut current = self.root.clone();
        let (last, dirs) = rel
            .components()
            .split_last()
            .ok_or(ExecError::OutsideRoot)?;
        for component in dirs {
            current.push(component);
            match fs::symlink_metadata(&current) {
                Ok(meta) if meta.file_type().is_symlink() => return Err(ExecError::SymlinkInPath),
                Ok(meta) if meta.is_dir() => {}
                Ok(_) => return Err(ExecError::OutsideRoot),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&current)?,
                Err(e) => return Err(e.into()),
            }
        }
        current.push(last);
        if let Ok(meta) = fs::symlink_metadata(&current)
            && meta.file_type().is_symlink()
        {
            return Err(ExecError::SymlinkInPath);
        }
        let parent = current
            .parent()
            .ok_or(ExecError::OutsideRoot)?
            .canonicalize()?;
        if !parent.starts_with(&self.root) {
            return Err(ExecError::OutsideRoot);
        }
        Ok(current)
    }
}

fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), ExecError> {
    let dir = target.parent().ok_or(ExecError::OutsideRoot)?;
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(ExecError::OutsideRoot)?;
    let tmp = dir.join(format!(".{file_name}.rab-tmp"));
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    if let Err(e) = fs::rename(&tmp, target) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}
