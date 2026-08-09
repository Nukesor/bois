//! Helpers to inspect the live filesystem during comparisons.

use std::{
    io::ErrorKind,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

use anyhow::Result;
use nix::unistd::{Gid, Group as NixGroup, Uid, User as NixUser};

use crate::{changeset::FileType, error::Error};

/// A path found at a given path on the system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveEntry {
    Missing,
    File {
        mode: u32,
        owner: String,
        group: String,
    },
    Directory {
        mode: u32,
        owner: String,
        group: String,
    },
    /// We don't handle symlinks yet, so we don't track their metadata yet.
    /// TODO: Add relative(?)/absolute symlink support
    Symlink,
    /// A socket, device or other special file.
    /// This should always be handled as a conflict, as we don't manage these filetypes.
    Special,
}

impl LiveEntry {
    /// The entry's filetype, or `None` if there's nothing at that path.
    pub fn file_type(&self) -> Option<FileType> {
        match self {
            LiveEntry::Missing => None,
            LiveEntry::File { .. } => Some(FileType::File),
            LiveEntry::Directory { .. } => Some(FileType::Directory),
            LiveEntry::Symlink => Some(FileType::Symlink),
            LiveEntry::Special => Some(FileType::Special),
        }
    }
}

/// Inspect the given path on the live system.
///
/// We explicitly use [`Path::symlink_metadata`], so that symlinks are reported as symlinks
/// instead of being followed.
pub fn read_live_entry(path: &Path) -> Result<LiveEntry> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        // NotADirectory means some parent of this path is a file on the live system.
        // E.g. `/this/is/some/path` and `some` is a file. That kind of issue is then handled at the
        // parent's own tree node, so its children count as missing.
        Err(err) if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(LiveEntry::Missing);
        }
        Err(err) => {
            return Err(Error::IoPath(path.to_path_buf(), "reading metadata", err).into());
        }
    };

    let metadata_type = metadata.file_type();
    if metadata_type.is_symlink() {
        return Ok(LiveEntry::Symlink);
    }

    if metadata_type.is_dir() || metadata_type.is_file() {
        let mode = mask_mode(metadata.permissions().mode());
        let owner = user_name(metadata.uid());
        let group = group_name(metadata.gid());

        if metadata_type.is_dir() {
            Ok(LiveEntry::Directory { mode, owner, group })
        } else {
            Ok(LiveEntry::File { mode, owner, group })
        }
    } else {
        Ok(LiveEntry::Special)
    }
}

/// Read a live file's content for comparison.
pub fn read_live_content(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|err| Error::IoPath(path.to_path_buf(), "reading file", err).into())
}

/// Strip the filetype bits from an on-disk `st_mode`.
/// This then only leaves the permission bits.
pub fn mask_mode(mode: u32) -> u32 {
    mode & 0o7777
}

/// Resolve a uid to a user name.
/// Falls back to the numeric id if the user is unknown.
fn user_name(uid: u32) -> String {
    match NixUser::from_uid(Uid::from_raw(uid)) {
        Ok(Some(user)) => user.name,
        _ => uid.to_string(),
    }
}

/// Resolve a gid to a group name.
/// Falls back to the numeric id if the group is unknown.
fn group_name(gid: u32) -> String {
    match NixGroup::from_gid(Gid::from_raw(gid)) {
        Ok(Some(group)) => group.name,
        _ => gid.to_string(),
    }
}
