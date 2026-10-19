use std::process::Command;

pub(crate) struct Runtime {}

impl Runtime {
    pub fn container_runtime_installed() -> Result<String, anyhow::Error> {
        let runtime = if is_installed("podman") {
            "podman"
        } else if is_installed("docker") {
            "docker"
        } else {
            anyhow::bail!("Install Podman or Docker to use dev containers. Check https://containers.dev/ for requirements.");
        };
        Ok(runtime.to_string())
    }
}

fn is_installed(command: &str) -> bool {
    Command::new(command)
        .arg("info")
        .output()
        .is_ok_and(|output| output.status.success())
}