use std::{path::PathBuf};

use clap::{Parser};
use console::{Emoji, style};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

use crate::template_engine::{Language, TemplateEngine};

mod template_engine;
mod runtime;

#[derive(Parser)]
#[command(about = "Development container isolation tool to setup devcontainer environments.")]
struct Args {
    /// Path to the repository where the dev container should be created.
    #[arg(long)]
    target_path: Option<String>,
    /// Create and Write actions will not be performed.
    #[arg(default_value = "false", long)]
    try_run: Option<bool>,
    /// Override existing devcontainer files.
    #[arg(default_value = "false", long)]
    file_override: Option<bool>,
    /// Ignore errors on directory creation.
    #[arg(default_value = "false", long)]
    ignore_directory_creation_error: Option<bool>,
    /// Ignore runtime check error result.
    #[arg(default_value = "false", long)]
    ignore_runtime_check_error: Option<bool>
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let try_run = args.try_run.or(Some(false)).unwrap();
    let ignore_runtime_check_error = args.ignore_runtime_check_error.or(Some(false)).unwrap();

    // Runtime check
    let runtime_result = runtime::Runtime::container_runtime_installed();
    if let Err(run_err) = runtime_result {
        if !try_run && !ignore_runtime_check_error {
            return Err(run_err.into());
        } else {
            println!("{} No container runtime detected: {}", style("Error:").red(), run_err);
        }
    } else {
        println!("{} Container runtime: {}", style(Emoji("✔", "[ok]")).green(), style(runtime_result.ok().unwrap()).bold());
    }

    let repo_path_relative: PathBuf = if let Some(p) = args.target_path {
        PathBuf::from(p)
    } else {
        std::env::current_dir()?
    };
    let repo_path = std::fs::canonicalize(repo_path_relative)?;

    if !repo_path.is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid directory: {}", repo_path.display())).into());
    }
    println!("{} {}", style("Repo:").blue().bold(), repo_path.display());

    let use_folder_name = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use target folder as name")
        .default(true)
        .interact()?;

    let dci_name: String = if !use_folder_name {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Name")
            .allow_empty(false)
            .interact_text()?
    } else {
        repo_path.file_name().ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not read directory name"))?.to_string_lossy().to_string()
    };

    // Language selection
    let languages = [Language::Rust, Language::Dotnet, Language::NodeJS];
    let selection = Select::new()
        .with_prompt("Choose a language")
        .items(&languages)
        .default(0)
        .interact()?;
    let lang = languages[selection];
    
    // File and Directory creation
    let devcontainer_directory = repo_path.join(".devcontainer");
    // cSpell:disable-next-line
    let container_file = devcontainer_directory.join("Containerfile");
    let devcontainer_file = devcontainer_directory.join("devcontainer.json");

    let create_container = TemplateEngine::create_container_file(
        try_run, &container_file, &dci_name, 
        lang, args.file_override.or(Some(false)).unwrap()
    )?;
    file_creation_output(create_container, &container_file)?;
        
    let create_dev_container = TemplateEngine::create_dev_container(
        try_run, &devcontainer_file, &dci_name, 
        lang, args.file_override.or(Some(false)).unwrap(), args.ignore_directory_creation_error.or(Some(false)).unwrap()
    )?;
    file_creation_output(create_dev_container, &devcontainer_file)?;
    
    Ok(())
}

fn file_creation_output(created: bool, file: &PathBuf) -> anyhow::Result<()> {
    let file_str = file.to_str().ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("File path is invalid: {:?}", file)))?;
    if created {
        println!("{} Created {}", style(Emoji("✔", "[ok]")).green(), style(file_str).bold());
    } else {
        println!("Skipped {}", style(file_str).bold());
    }
    Ok(())
}
