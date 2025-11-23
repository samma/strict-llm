use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

pub struct SandboxPlugin {
    root: PathBuf,
}

#[allow(dead_code)]
impl SandboxPlugin {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for SandboxPlugin {
    fn default() -> Self {
        Self {
            root: PathBuf::from("examples/systems"),
        }
    }
}

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        let registry = SandboxRegistry::discover(self.root.clone());
        app.insert_resource(registry)
            .add_systems(Startup, log_sandboxes);
    }
}

#[derive(Resource, Debug)]
pub struct SandboxRegistry {
    pub root: PathBuf,
    pub available: Vec<String>,
    pub active: Option<String>,
}

impl SandboxRegistry {
    fn discover(root: PathBuf) -> Self {
        let mut available = Vec::new();
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                if entry.file_type().map(|f| f.is_dir()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        available.push(name.to_string());
                    }
                }
            }
        }

        available.sort();
        let env_active = std::env::var("SANDBOX_SCENE").ok();
        let active = env_active
            .as_ref()
            .and_then(|name| available.iter().find(|candidate| candidate == &name))
            .cloned();

        Self {
            root,
            available,
            active,
        }
    }
}

fn log_sandboxes(registry: Res<SandboxRegistry>) {
    if registry.available.is_empty() {
        info!(
            target: "sandbox",
            "No sandboxes under {}. Add a folder in examples/systems/<feature>/scene.ron",
            registry.root.display()
        );
        return;
    }

    info!(
        target: "sandbox",
        "Sandboxes: {:?} (active: {})",
        registry.available,
        registry
            .active
            .as_deref()
            .unwrap_or("none (set SANDBOX_SCENE)"),
    );
}
