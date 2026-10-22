//! Plan hash, aprobare/revocare și policy gate (S02, S03 lexical, S08 parțial, S20, S22).
#![allow(clippy::unwrap_used)]

mod common;

use common::{
    APPROVER, NOW, exec_config, exec_config_value, exec_plan, exec_plan_value, plan_from,
};
use rab_core::approval::{Approval, ApprovalError};
use rab_core::config::{AgentsConfig, IssueCode, Mode};
use rab_core::ids::{ExternalAction, ToolId};
use rab_core::paths::PathError;
use rab_core::plan::{Plan, PlanStatus, is_sha256_hex};
use rab_core::policy::{Denial, PolicyContext, Target, ToolRequest, authorize_tool};
use serde_json::json;

const ROOT: &str = "D:/Roblox/ObbyVulcanic";

fn req<'a>(task_id: &'a str, tool: ToolId, target: Target<'a>) -> ToolRequest<'a> {
    ToolRequest {
        task_id,
        tool,
        target,
    }
}

#[test]
fn hash_is_stable_across_key_order_and_whitespace() {
    let plan = exec_plan(ROOT);
    let pretty = serde_json::to_string_pretty(&exec_plan_value(ROOT)).unwrap();
    // Chei scrise în ordine inversă, manual (serde_json::Map ar le sorta).
    let value = exec_plan_value(ROOT);
    let mut reordered = String::from("{");
    for (i, (k, v)) in value.as_object().unwrap().iter().rev().enumerate() {
        if i > 0 {
            reordered.push_str(" ,\n ");
        }
        reordered.push_str(&format!("{} : {v}", serde_json::Value::String(k.clone())));
    }
    reordered.push('}');
    let a = Plan::parse(&pretty).unwrap().sha256();
    let b = Plan::parse(&reordered).unwrap().sha256();
    assert_eq!(a, plan.sha256());
    assert_eq!(a, b);
    assert!(is_sha256_hex(&a));
}

#[test]
fn hash_ignores_status_but_not_content() {
    let base = exec_plan(ROOT);
    let mut approved_status = base.clone();
    approved_status.status = PlanStatus::Approved;
    assert_eq!(base.sha256(), approved_status.sha256());

    let mut changed = base.clone();
    changed.objective.push_str(" și publică jocul");
    assert_ne!(base.sha256(), changed.sha256());

    let mut more_paths = base.clone();
    more_paths.scope.write_paths.push("assets".into());
    assert_ne!(base.sha256(), more_paths.sha256());
}

#[test]
fn approval_written_in_document_is_rejected() {
    let mut v = exec_plan_value(ROOT);
    v["approval"] = json!({ "approved": true });
    let issues = Plan::parse(&v.to_string())
        .unwrap()
        .validate(Mode::Contract);
    assert!(issues.iter().any(|i| i.path == "$.approval"));

    let mut v = exec_plan_value(ROOT);
    v["status"] = json!("approved");
    assert!(
        !Plan::parse(&v.to_string())
            .unwrap()
            .validate(Mode::Contract)
            .is_empty()
    );
}

#[test]
fn plan_structure_is_validated() {
    let mut v = exec_plan_value(ROOT);
    v["tasks"][0]["depends_on"] = json!(["verify"]);
    let issues = Plan::parse(&v.to_string())
        .unwrap()
        .validate(Mode::Contract);
    assert!(
        issues
            .iter()
            .any(|i| i.code == IssueCode::Inconsistent && i.message.contains("ciclu"))
    );

    let mut v = exec_plan_value(ROOT);
    v["tasks"][1]["deliverables"] = json!(["../outside.md"]);
    assert!(
        !Plan::parse(&v.to_string())
            .unwrap()
            .validate(Mode::Contract)
            .is_empty()
    );

    let mut v = exec_plan_value(ROOT);
    v["tasks"][1]["deliverables"] = json!(["assets/x.png"]);
    let issues = Plan::parse(&v.to_string())
        .unwrap()
        .validate(Mode::Contract);
    assert!(issues.iter().any(|i| i.code == IssueCode::Inconsistent));
}

#[test]
fn example_plan_cannot_be_approved() {
    let example = Plan::parse(rab_core::examples::PLAN_EXAMPLE_JSON).unwrap();
    assert!(matches!(
        Approval::grant(&example, APPROVER, NOW),
        Err(ApprovalError::NotAwaitingApproval)
    ));
    let mut awaiting = example.clone();
    awaiting.status = PlanStatus::AwaitingApproval;
    assert!(matches!(
        Approval::grant(&awaiting, APPROVER, NOW),
        Err(ApprovalError::InvalidPlan(_))
    ));
}

#[test]
fn approved_plan_allows_scoped_write() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let permit = authorize_tool(
        &ctx,
        &req(
            "generate",
            ToolId::ProjectApplyValidatedPatch,
            Target::ProjectPath("src\\server\\Checkpoint.server.luau"),
        ),
    )
    .unwrap();
    assert_eq!(
        permit.path.unwrap().as_string(),
        "src/server/Checkpoint.server.luau"
    );
    assert_eq!(permit.plan_sha256, plan.sha256());
}

#[test]
fn s02_plan_change_after_approval_blocks_effects() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();

    // Un tool output sau o notă modifică planul după aprobare.
    let mut tampered = plan.clone();
    tampered.scope.write_paths.push("C_drive_copy".into());
    tampered.tasks[0].allowed_tools.push(ToolId::ReportsWrite);
    let ctx = PolicyContext {
        config: &config,
        plan: &tampered,
        approval: Some(&approval),
        cancelled: false,
    };
    let denial = authorize_tool(
        &ctx,
        &req(
            "generate",
            ToolId::ProjectApplyValidatedPatch,
            Target::ProjectPath("src/a.luau"),
        ),
    );
    assert_eq!(denial, Err(Denial::PlanHashMismatch));

    // „Restart”: aprobarea serializată și reîncărcată rămâne legată de hash.
    let restored: Plan = Plan::parse(&serde_json::to_string(&tampered).unwrap()).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &restored,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::PlanHashMismatch)
    );

    // Alt plan / altă versiune.
    let mut v2 = plan.clone();
    v2.version = 2;
    let ctx = PolicyContext {
        config: &config,
        plan: &v2,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::ApprovalPlanMismatch)
    );
}

#[test]
fn revoked_or_missing_approval_denies() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: None,
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::NoApproval)
    );

    let mut approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    approval.revoke("2026-10-22T12:05:00Z");
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::ApprovalRevoked)
    );
}

#[test]
fn example_config_never_executes() {
    let example = AgentsConfig::parse(rab_core::examples::AGENTS_EXAMPLE_JSON).unwrap();
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &example,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::ExampleConfig)
    );
}

#[test]
fn tools_are_limited_to_the_task() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::ReportsWrite,
                Target::ProjectPath("reports/x.md")
            )
        ),
        Err(Denial::ToolNotAllowedForTask(ToolId::ReportsWrite))
    );
    assert_eq!(
        authorize_tool(&ctx, &req("deploy", ToolId::ProjectRead, Target::None)),
        Err(Denial::UnknownTask("deploy".into()))
    );
    assert_eq!(
        authorize_tool(
            &ctx,
            &req("generate", ToolId::ProjectApplyValidatedPatch, Target::None)
        ),
        Err(Denial::TargetMismatch(ToolId::ProjectApplyValidatedPatch))
    );
}

#[test]
fn s03_paths_outside_project_or_scope_are_denied() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let write = |p: &'static str| {
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::ProjectApplyValidatedPatch,
                Target::ProjectPath(p),
            ),
        )
    };
    assert_eq!(
        write("../escape.luau"),
        Err(Denial::InvalidPath(PathError::ParentTraversal))
    );
    assert_eq!(
        write("C:\\Windows\\win.ini"),
        Err(Denial::InvalidPath(PathError::Absolute))
    );
    assert_eq!(
        write("\\\\server\\share\\x"),
        Err(Denial::InvalidPath(PathError::Unc))
    );
    assert_eq!(
        write("\\\\?\\C:\\x"),
        Err(Denial::InvalidPath(PathError::DevicePath))
    );
    assert_eq!(
        write("src/a.luau:ads"),
        Err(Denial::InvalidPath(PathError::AlternateDataStream))
    );
    assert_eq!(
        write("vault/notes.md"),
        Err(Denial::PathOutsideWriteScope("vault/notes.md".into()))
    );
    assert_eq!(
        write("src2/a.luau"),
        Err(Denial::PathOutsideWriteScope("src2/a.luau".into()))
    );
    assert!(
        write("SRC/Upper.luau").is_ok(),
        "comparare fără majuscule, ca pe NTFS"
    );
}

#[test]
fn s22_external_actions_require_global_policy_plan_scope_and_exact_target() {
    let config = exec_config(ROOT);
    let mut v = exec_plan_value(ROOT);
    v["tasks"][0]["allowed_tools"] = json!([
        "project.read",
        "git.push",
        "roblox.publish",
        "roblox.upload_asset"
    ]);
    let plan = plan_from(v);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let external = |tool, action, destination| {
        authorize_tool(
            &ctx,
            &req(
                "generate",
                tool,
                Target::External {
                    action,
                    destination,
                },
            ),
        )
    };
    // Politica implicită interzice tot, chiar dacă agentul cere.
    assert_eq!(
        external(ToolId::RobloxPublish, ExternalAction::Publish, "place:123"),
        Err(Denial::GlobalPolicyForbids(ExternalAction::Publish))
    );
    assert_eq!(
        external(ToolId::GitPush, ExternalAction::GitPush, "andrei/obby"),
        Err(Denial::GlobalPolicyForbids(ExternalAction::GitPush))
    );

    // Politica globală permite push, dar planul nu.
    let mut cv = exec_config_value(ROOT);
    cv["policy"]["allow_git_push"] = json!(true);
    cv["policy"]["allow_publish"] = json!(true);
    cv["integrations"]["github"]["enabled"] = json!(true);
    cv["integrations"]["github"]["repository"] = json!("andrei/obby");
    let permissive = AgentsConfig::load(&cv.to_string(), Mode::Execution).unwrap();
    let ctx = PolicyContext {
        config: &permissive,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::GitPush,
                Target::External {
                    action: ExternalAction::GitPush,
                    destination: "andrei/obby"
                }
            )
        ),
        Err(Denial::PlanScopeForbids(ExternalAction::GitPush))
    );

    // Planul cu destinație exactă și aprobare validă permite numai acea țintă.
    let mut v = exec_plan_value(ROOT);
    v["tasks"][0]["allowed_tools"] = json!(["git.push", "roblox.publish"]);
    v["scope"]["allow_push"] = json!(true);
    v["scope"]["allow_publish"] = json!(true);
    v["scope"]["github_repository"] = json!("andrei/obby");
    let push_plan = plan_from(v);
    let push_approval = Approval::grant(&push_plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &permissive,
        plan: &push_plan,
        approval: Some(&push_approval),
        cancelled: false,
    };
    let push = |destination| {
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::GitPush,
                Target::External {
                    action: ExternalAction::GitPush,
                    destination,
                },
            ),
        )
    };
    assert!(push("andrei/obby").is_ok());
    assert_eq!(push("evil/fork"), Err(Denial::DestinationMismatch));
    // Publicarea cere place/universe exacte, inexistente în schema 0.1.
    assert_eq!(
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::RobloxPublish,
                Target::External {
                    action: ExternalAction::Publish,
                    destination: "place:1"
                }
            )
        ),
        Err(Denial::ExternalDestinationMissing(ExternalAction::Publish))
    );
    // Acțiunea cerută trebuie să corespundă instrumentului.
    assert_eq!(
        authorize_tool(
            &ctx,
            &req(
                "generate",
                ToolId::GitPush,
                Target::External {
                    action: ExternalAction::Publish,
                    destination: "andrei/obby"
                }
            )
        ),
        Err(Denial::TargetMismatch(ToolId::GitPush))
    );
}

#[test]
fn research_requires_allowlist_and_plan_destination() {
    let config = exec_config(ROOT);
    let mut v = exec_plan_value(ROOT);
    v["tasks"][0]["allowed_tools"] = json!(["research.fetch"]);
    v["scope"]["network_destinations"] =
        json!(["http://127.0.0.1:11434", "https://create.roblox.com"]);
    let plan = plan_from(v);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    let fetch = |url| {
        authorize_tool(
            &ctx,
            &req("generate", ToolId::ResearchFetch, Target::Url(url)),
        )
    };
    assert!(fetch("https://create.roblox.com/docs/studio/mcp").is_ok());
    assert!(
        matches!(
            fetch("https://rojo.space/docs"),
            Err(Denial::NetworkDenied(_))
        ),
        "în allowlist, dar nu în plan"
    );
    assert!(matches!(
        fetch("http://create.roblox.com/docs"),
        Err(Denial::NetworkDenied(_))
    ));
    assert!(matches!(
        fetch("https://127.0.0.1:11434/api/tags"),
        Err(Denial::NetworkDenied(_))
    ));
    assert!(matches!(
        fetch("https://169.254.169.254/latest"),
        Err(Denial::NetworkDenied(_))
    ));
    assert!(matches!(
        fetch("https://evil.com"),
        Err(Denial::NetworkDenied(_))
    ));
    assert!(matches!(
        fetch("https://user@create.roblox.com/"),
        Err(Denial::NetworkDenied(_))
    ));
}

#[test]
fn s20_cancelled_run_allows_no_new_tool_call() {
    let config = exec_config(ROOT);
    let plan = exec_plan(ROOT);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: true,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::RunCancelled)
    );
}

#[test]
fn plan_for_another_project_is_denied() {
    let config = exec_config(ROOT);
    let mut v = exec_plan_value(ROOT);
    v["project_id"] = json!("alt-proiect");
    let plan = plan_from(v);
    let approval = Approval::grant(&plan, APPROVER, NOW).unwrap();
    let ctx = PolicyContext {
        config: &config,
        plan: &plan,
        approval: Some(&approval),
        cancelled: false,
    };
    assert_eq!(
        authorize_tool(&ctx, &req("generate", ToolId::ProjectRead, Target::None)),
        Err(Denial::ProjectMismatch)
    );
}
