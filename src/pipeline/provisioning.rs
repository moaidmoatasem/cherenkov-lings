//! Simulated infrastructure provisioning that runs ahead of pipeline test jobs.
//!
//! The point of the simulation is pedagogical: a learner should see that CI
//! time is spent standing infrastructure up before a single test executes.
//! The phases are data (`provisioning_steps`) so the pipeline can render them
//! and the tests can assert on them; `simulate_provisioning` is only the
//! printing shell around that data.

use colored::Colorize;

/// One line of simulated provisioning output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisioningStep {
    /// The tool emitting the line: `terraform` or `docker`.
    pub tool: &'static str,
    /// The message body, without the tool prefix.
    pub message: &'static str,
}

/// Tools whose phases are simulated, in execution order.
pub const PROVISIONING_TOOLS: [&str; 2] = ["terraform", "docker"];

/// The provisioning phases, in the order a real pipeline would run them:
/// Terraform stands up cloud resources, then Docker Compose brings the
/// service containers online.
pub fn provisioning_steps() -> Vec<ProvisioningStep> {
    vec![
        ProvisioningStep {
            tool: "terraform",
            message: "Initializing provider plugins...",
        },
        ProvisioningStep {
            tool: "terraform",
            message: "Plan: 3 to add, 0 to change, 0 to destroy.",
        },
        ProvisioningStep {
            tool: "terraform",
            message: "Apply complete! Resources: 3 added, 0 changed, 0 destroyed.",
        },
        ProvisioningStep {
            tool: "docker",
            message: "Creating network crucible_default",
        },
        ProvisioningStep {
            tool: "docker",
            message: "Container crucible-db-1  Started",
        },
        ProvisioningStep {
            tool: "docker",
            message: "Container crucible-backend-1  Started",
        },
    ]
}

/// Render the provisioning phases to stdout ahead of the test jobs.
pub fn simulate_provisioning() {
    println!(
        "{}",
        "─── Infrastructure Provisioning ────────────────────────────────────────────────────"
            .dimmed()
    );

    let steps = provisioning_steps();
    let last_index_for = |tool: &str| {
        steps
            .iter()
            .rposition(|s| s.tool == tool)
            .expect("every listed tool has at least one step")
    };
    let last_terraform = last_index_for("terraform");
    let last_docker = last_index_for("docker");

    for (idx, step) in steps.iter().enumerate() {
        // The closing line of each tool's phase is bolded as its summary.
        let is_phase_summary = idx == last_terraform || idx == last_docker;
        let label = match (step.tool, is_phase_summary) {
            ("terraform", true) => step.tool.bright_green().bold(),
            ("terraform", false) => step.tool.bright_green(),
            (_, true) => step.tool.bright_blue().bold(),
            (_, false) => step.tool.bright_blue(),
        };
        println!("{} {}", label, step.message);

        if is_phase_summary {
            println!();
        }
    }
}
