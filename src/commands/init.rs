use std::{
    env,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

use crate::{
    config::{Configuration, Mode},
    templating::render_template,
};

pub mod user {
    pub const BOIS: &str = include_str!("../../templates/dotfiles/bois.yml");
    pub const HOST: &str = include_str!("../../templates/dotfiles/host.yml");
    pub const GROUP: &str = include_str!("../../templates/dotfiles/group.yml");
}

pub mod system {
    pub const BOIS: &str = include_str!("../../templates/system/bois.yml");
    pub const HOST: &str = include_str!("../../templates/system/host.yml");
    pub const GROUP: &str = include_str!("../../templates/system/group.yml");
}

pub fn run_init(config: Configuration, directory: &Option<PathBuf>) -> Result<()> {
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
    let (bois_content, host_content, group_content) = match config.mode {
        Mode::User => (user::BOIS, user::HOST, user::GROUP),
        Mode::System => (system::BOIS, system::HOST, system::GROUP),
    };

    let mut variables = Mapping::new();
    variables.insert(
        serde_yaml::to_value("hostname").unwrap(),
        serde_yaml::to_value(&config.name).unwrap(),
    );
    let templated_bois_content = render_template(bois_content, &Value::Mapping(variables), &None)?;
    let config_path = root_dir.join("bois.yml");
    fs::write(config_path, templated_bois_content)?;

    let hosts_dir = root_dir.join("hosts").join(&config.name);
    create_dir_all(&hosts_dir).context("Failed to create hosts directory")?;
    let host_config_path = hosts_dir.join("hosts.yml");
    fs::write(host_config_path, host_content)?;

    let groups_dir = root_dir.join("groups").join("base");
    create_dir_all(&groups_dir).context("Failed to create groups directory")?;
    let group_config_path = groups_dir.join("hosts.yml");
    fs::write(group_config_path, group_content)?;

    Ok(())
}
