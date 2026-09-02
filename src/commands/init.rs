use std::{
    env,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

use crate::{
    config::bois::{RawConfiguration, RunMode},
    templating::render_template,
};

pub mod user {
    pub const BOIS: &str = include_str!("../../templates/dotfiles/bois.yml");
    pub const HOST: &str = include_str!("../../templates/dotfiles/host.yml");
    pub const TRAIT: &str = include_str!("../../templates/dotfiles/trait.yml");
}

pub mod system {
    pub const BOIS: &str = include_str!("../../templates/system/bois.yml");
    pub const HOST: &str = include_str!("../../templates/system/host.yml");
    pub const TRAIT: &str = include_str!("../../templates/system/trait.yml");
}

pub fn run_init(raw_config: RawConfiguration, directory: &Option<PathBuf>) -> Result<()> {
    let name = raw_config.resolve_name()?;
    let mode = raw_config.resolve_mode();

    let root_dir = if let Some(directory) = directory {
        if directory.is_absolute() {
            directory.clone()
        } else {
            let cwd =
                env::current_dir().context("Failed to determine current working directory.")?;
            cwd.join(directory)
        }
    } else {
        env::current_dir().context("Failed to determine current working directory.")?
    };

    if !root_dir.exists() {
        create_dir_all(&root_dir).context("Failed to create root bois directory")?;
    }

    // Read template files based on config mode.
    let (bois_content, host_content, trait_content) = match mode {
        RunMode::User => (user::BOIS, user::HOST, user::TRAIT),
        RunMode::System => (system::BOIS, system::HOST, system::TRAIT),
    };

    let mut variables = Mapping::new();
    variables.insert(
        serde_yaml::to_value("hostname").unwrap(),
        serde_yaml::to_value(&name).unwrap(),
    );
    let templated_bois_content = render_template(
        &PathBuf::from("internal/template"),
        bois_content,
        &Value::Mapping(variables),
        &None,
    )?;
    let config_path = root_dir.join("bois.yml");
    fs::write(config_path, templated_bois_content)?;

    let hosts_dir = root_dir.join("hosts").join(&name);
    create_dir_all(&hosts_dir).context("Failed to create hosts directory")?;
    let host_config_path = hosts_dir.join("host.yml");
    fs::write(host_config_path, host_content)?;
    let host_vars_path = hosts_dir.join("vars.yml");
    fs::write(
        host_vars_path,
        "# Variables that're available in all templates of this host.\n\
         #\n\
         #editor: nvim\n",
    )?;

    let traits_dir = root_dir.join("traits").join("base");
    create_dir_all(&traits_dir).context("Failed to create traits directory")?;
    let trait_config_path = traits_dir.join("trait.yml");
    fs::write(trait_config_path, trait_content)?;

    Ok(())
}
