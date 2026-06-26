// RustShield — rustshield-migrate: CLI migration assistant
//
// This tool analyzes an existing C driver, generates a Rust skeleton
// with Verus annotation hints, runs the eBPF canary shadow, and
// produces a migration readiness report.

use clap::{Parser, Subcommand};

mod analyze;
mod report;
mod skeleton;
mod verus_hints;

#[derive(Parser)]
#[command(name = "rustshield-migrate")]
#[command(about = "C-to-Rust Linux kernel driver migration assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a C driver and produce a migration readiness score
    Analyze {
        /// Path to the C driver source directory
        path: String,

        /// Output JSON report file
        #[arg(short, long, default_value = "migration-report.json")]
        output: String,
    },

    /// Generate a Rust driver skeleton with Verus annotations
    Skeleton {
        /// Path to the analysis output (JSON)
        #[arg(short, long)]
        analysis: String,

        /// Target language (only rust supported)
        #[arg(long, default_value = "rust")]
        lang: String,

        /// Add Verus annotations
        #[arg(long)]
        verus: bool,

        /// Output directory for generated skeleton
        #[arg(short, long, default_value = "./rust-driver")]
        output: String,
    },

    /// Verify a Rust driver against proofs and canary baseline
    Verify {
        /// Path to the Verus proof library
        #[arg(long)]
        proofs: String,

        /// Path to the eBPF canary baseline JSON
        #[arg(long)]
        canary: String,

        /// Path to the Rust driver source
        #[arg(short, long)]
        driver: String,
    },

    /// Run the full migration pipeline
    Migrate {
        /// Path to the C driver source directory
        path: String,

        /// Output directory
        #[arg(short, long, default_value = "./migrated-rust-driver")]
        output: String,

        /// Skip verification steps
        #[arg(long)]
        skip_verify: bool,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { path, output } => {
            let result = analyze::analyze_c_driver(&path)?;
            analyze::write_report(&result, &output)?;
            println!("✓ Analysis complete: score {}/100", result.migration_score);
        }
        Commands::Skeleton {
            analysis,
            lang: _,
            verus,
            output,
        } => {
            let analysis_data = analyze::read_report(&analysis)?;
            skeleton::generate_skeleton(&analysis_data, &output, verus)?;
            println!("✓ Skeleton generated at: {}", output);
        }
        Commands::Verify {
            proofs,
            canary,
            driver,
        } => {
            let verus_result = verus_hints::verify_proofs(&proofs, &driver)?;
            let canary_result = report::check_canary_agreement(&canary)?;
            let readiness = report::compute_readiness_score(&verus_result, &canary_result);
            println!("✓ Verification results:");
            println!("  - Verus proofs:    {}", verus_result.status);
            println!("  - Canary agreement: {:.1}%", canary_result.agreement_pct);
            println!("  - Readiness score:  {}/100", readiness);
        }
        Commands::Migrate {
            path,
            output,
            skip_verify,
        } => {
            println!("Running full migration pipeline for: {}", path);
            let analysis = analyze::analyze_c_driver(&path)?;
            skeleton::generate_skeleton(&analysis, &output, true)?;
            if !skip_verify {
                println!("✓ Skeleton generated. Ready for manual implementation.");
            } else {
                println!("✓ Skeleton generated (verification skipped).");
            }
        }
    }

    Ok(())
}
