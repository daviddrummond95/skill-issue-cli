mod config;
mod engine;
mod finding;
mod output;
mod remote;
mod rules;
mod scanner;

use clap::Parser;
use config::{CliArgs, Config, ConfigFile};
use engine::Engine;
use rules::RuleRegistry;
use std::path::PathBuf;

fn main() {
    let args = CliArgs::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    let quiet = args.quiet;
    let verbose = args.verbose;
    let is_remote = args.remote.is_some();

    // Skip config file loading for remote scans
    let config_file = if is_remote {
        None
    } else {
        let config_path = args
            .config
            .clone()
            .unwrap_or_else(|| args.path.join(".skill-issue.toml"));
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                    Ok(cf) => Some(cf),
                    Err(e) => {
                        eprintln!("warning: failed to parse config file: {e}");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("warning: failed to read config file: {e}");
                    None
                }
            }
        } else {
            None
        }
    };

    let config = Config::from_args_and_file(args, config_file);

    // Scan files — either remote or local
    let (files, display_path) = if let Some(ref spec) = config.remote {
        if verbose {
            eprintln!("Scanning remote: {spec}");
        }

        let files = match remote::fetch_remote_skill(
            spec,
            config.github_token.as_deref(),
            verbose,
        ) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(2);
            }
        };

        let display_path = PathBuf::from(spec);
        (files, display_path)
    } else {
        if verbose {
            eprintln!("Scanning: {}", config.path.display());
        }

        let files = match scanner::scan_directory(&config.path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(2);
            }
        };

        let display_path = config.path.clone();
        (files, display_path)
    };

    if verbose {
        eprintln!("Found {} files to analyze", files.len());
    }

    // Load rules
    let mut registry = RuleRegistry::new();
    registry.load_defaults();

    if verbose {
        eprintln!("Loaded {} rules", registry.all_rules().len());
    }

    // Run engine
    let engine = Engine::new(&config, &registry);
    let findings = engine.run(&files);

    // Output
    let output = output::format_findings(&config.format, &findings, &display_path);
    if !quiet || !findings.is_empty() {
        println!("{output}");
    }

    // Summary on stderr if not quiet
    if !quiet && verbose {
        eprintln!(
            "Scan complete: {} files, {} findings",
            files.len(),
            findings.len()
        );
    }

    let exit_code = Engine::exit_code(&findings, config.error_on);
    std::process::exit(exit_code);
}
