use std::{env, fs::create_dir_all, path::PathBuf};

use anyhow::{Context, Result};

use crate::config::{Configuration, Mode};

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
    let (_bois, _host, _group) = match config.mode {
        Mode::User => (user::BOIS, user::HOST, user::GROUP),
        Mode::System => (system::BOIS, system::HOST, system::GROUP),
    };

    let _config_path = root_dir.join("bois.yml");

    let hosts_dir = root_dir.join("hosts").join(&config.name);
    create_dir_all(&hosts_dir).context("Failed to create hosts directory")?;
    let _host_file_path = hosts_dir.join("hosts.yml");

    let groups_dir = root_dir.join("groups").join("base");
    create_dir_all(&groups_dir).context("Failed to create groups directory")?;
    let _group_file_path = groups_dir.join("hosts.yml");

    Ok(())
}
