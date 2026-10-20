use std::path::PathBuf;

use anyhow::Error;
use console::style;

#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum Language {
    #[value(name = "rust")]
    Rust,
    #[value(name = "dotnet")]
    Dotnet,
    #[value(name = "nodejs")]
    NodeJS
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Rust => "Rust",
            Self::Dotnet => ".NET",
            Self::NodeJS => "Node.js",
        })
    }
}

pub(crate) struct TemplateEngine {}

impl TemplateEngine {
    pub(crate) fn create_dev_container(
        try_run: bool,
        target_path: &PathBuf,
        name: &str,
        language: Language,
        allow_override: bool,
        ignore_directory_creation_error: bool
    ) -> anyhow::Result<bool> {
        if target_path.exists() && !allow_override {
            if !try_run {
                return Ok(false);
            }
            println!("{} Target already exists: {}", style("Warn:").yellow(), target_path.display());
        }

        let user_name = match language {
            Language::Rust => "dev",
            Language::Dotnet => "dev",
            Language::NodeJS => "node",
        };

        let home_var = if cfg!(target_os = "windows") {
            // cSpell:disable-next-line
            "USERPROFILE"
        } else {
            "HOME"
        };

        // to have path join work correct we require to not have a `/` at the start
        let shared_directory_path = ".local/share/devcontainers/vscode/projects";

        let home = std::env::var_os(home_var)
            .ok_or_else(|| anyhow::anyhow!("HOME environment variable is not set"))?;

        let project_directory = PathBuf::from(home)
            .join(shared_directory_path)
            .join(name);

        if !try_run {
            if let Err(err) = std::fs::create_dir_all(&project_directory) {
                let msg = format!("Could not create shared directory: {} Source Error: {}", project_directory.display(), err);
                if !ignore_directory_creation_error {
                    if !try_run {
                        return Err(Error::msg(msg));
                    }
                }
                println!("{} {}", style("Warn:").yellow(), msg);
            }
        } else {
            if let Ok(exists) = std::fs::exists(&project_directory) {
                println!("{} Project directory at {} exists: {}", style("Info:").bold(), project_directory.display(), exists);
            }
        }

        // cSpell:disable
        let container_env = match language {
            Language::Rust => format!(r#""containerEnv": {{
    "CARGO_HOME": "/home/{0}/.cargo",
    "RUSTUP_HOME": "/usr/local/rustup"
        }},"#, user_name),
            Language::Dotnet => format!(r#""containerEnv": {{
    "DOTNET_CLI_HOME": "/home/{0}",
    "DOTNET_SKIP_FIRST_TIME_EXPERIENCE": "1",
    "DOTNET_NOLOGO": "1"
        }},"#, user_name),
            Language::NodeJS => "".to_string(),
        };

        let vscode_properties = match language {
            Language::Rust => format!(r#""extensions": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "vadimcn.vscode-lldb"
      ]"#),
            Language::Dotnet => format!(r#""extensions": [
        "ms-dotnettools.csharp"
      ]"#),
            Language::NodeJS => format!(r#""extensions": [
        "dbaeumer.vscode-eslint",
        "esbenp.prettier-vscode",
        "svelte.svelte-vscode",
        "vitest.explorer"
      ]"#),
        };
        // cSpell:enable

        // include_str! requires explicit 'placeholder = variable' map
        let file_content = format!(
            // cSpell:disable-next-line
            include_str!("../templates/devcontainer.json.template"),
            user_name = user_name,
            name = name,
            shared_directory = format!("${{localEnv:{home_var}}}/{shared_directory_path}"),
            container_env = container_env,
            vscode_properties = vscode_properties
        );

        if try_run {
            println!("{}\r\n{}", style("File:").bold().blue(), file_content);
        } else {
            std::fs::write(target_path, file_content).or_else(|e| Err(anyhow::anyhow!("Could not write target file: {0} Source Error: {e}", target_path.display())))?;
        }
        
        Ok(true)
    }

    pub(crate) fn create_container_file(
        try_run: bool,
        target_path: &PathBuf,
        name: &str,
        language: Language,
        allow_override: bool
    ) -> anyhow::Result<bool> {
        if target_path.exists() && !allow_override {
            if !try_run {
                return Ok(false);
            }
            println!("{} Target already exists: {}", style("Warn:").yellow(), target_path.display());
        }

        let docker_image = match language {
            Language::Rust => "docker.io/library/rust:1.99.1-bookworm",
            Language::Dotnet => "mcr.microsoft.com/dotnet/sdk:10.0-noble",
            Language::NodeJS => "docker.io/library/node:24-bookworm",
        };

        let build_tools = match language {
            // cSpell:disable
            Language::Rust => "apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git pkg-config libssl-dev clang lldb \
    && rm -rf /var/lib/apt/lists/*",
            // cSpell:enable
            Language::Dotnet => "apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git curl \
    && rm -rf /var/lib/apt/lists/*",
            Language::NodeJS => "apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git python3 make g++ \
    && rm -rf /var/lib/apt/lists/* && npm install --global pnpm@latest-11",
        };

        let user_name = match language {
            Language::Rust => "dev",
            Language::Dotnet => "dev",
            Language::NodeJS => "node",
        };

        let user_permission = match language {
            Language::Rust => format!("RUN groupadd --gid 1000 {0} && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash {0}", user_name),
            Language::Dotnet => format!("RUN groupadd --gid 1000 {0} && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash {0} \
    && mkdir -p /{1} /home/{0}/.nuget/packages \
    && chown -R {0}:{0} /home/{0} /{1}", user_name, name),
            Language::NodeJS => format!("# Official Node image already provides unprivileged node user (UID 1000)."),
        };

        let env_variables = match language {
            Language::Rust => format!("ENV CARGO_HOME=/home/{0}/.cargo RUSTUP_HOME=/usr/local/rustup", user_name),
            // cSpell:disable-next-line
            Language::Dotnet => format!("ENV DOTNET_CLI_HOME=/home/{0} DOTNET_NOLOGO=1", user_name),
            Language::NodeJS => format!(""),
        };

        let directories = match language {
            Language::Rust => format!("RUN mkdir -p /home/{0}/.cargo /{1} && chown -R {0}:{0} /home/{0} /{1}
RUN mkdir -p /vscode/vscode-server \
    && chown -R {0}:{0} /vscode", user_name, name),
            Language::Dotnet => format!("RUN mkdir -p /vscode/vscode-server \
    && chown -R {0}:{0} /vscode", user_name),
            Language::NodeJS => format!("RUN mkdir -p /{1} && chown {0}:{0} /{1}
RUN mkdir -p /vscode/vscode-server \
    && chown -R {0}:{0} /vscode
RUN mkdir -p /home/{0}/.vscode-server \
    && chown -R {0}:{0} /home/{0}/.vscode-server
RUN chown {0}:{0} /home/{0}", user_name, name),
        };

        // include_str! requires explicit 'placeholder = variable' map
        let file_content = format!(
            // cSpell:disable-next-line
            include_str!("../templates/Containerfile.template"),
            docker_image = docker_image,
            build_tools = build_tools,
            user_name = user_name,
            user_permission = user_permission,
            env_variables = env_variables,
            directories = directories,
            name = name
        );

        if try_run {
            println!("{}\r\n{}", style("File:").bold().blue(), file_content);
        } else {
            std::fs::write(target_path, file_content).or_else(|e| Err(anyhow::anyhow!("Could not write target file: {0} Source Error: {e}", target_path.display())))?;
        }

        Ok(true)
    }
}
