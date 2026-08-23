//! Helpers to inspect the filesystem during comparisons.

use std::{
    io::ErrorKind,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

use anyhow::Result;
use nix::unistd::{Gid, Group as NixGroup, Uid, User as NixUser};

use crate::{changeset::FileType, error::Error};

/// Read a file's actual content for comparison.
pub fn read_actual_content(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|err| Error::IoPath(path.to_path_buf(), "reading file", err).into())
}

/// The entry found at a given path on the system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActualEntry {
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

impl ActualEntry {
    /// The entry's filetype, or `None` if there's nothing at that path.
    pub fn file_type(&self) -> Option<FileType> {
        match self {
            ActualEntry::Missing => None,
            ActualEntry::File { .. } => Some(FileType::File),
            ActualEntry::Directory { .. } => Some(FileType::Directory),
            ActualEntry::Symlink => Some(FileType::Symlink),
            ActualEntry::Special => Some(FileType::Special),
        }
    }
    /// Inspect the given path on the system.
    ///
    /// Symlinks are reported as such and not followed.
    pub fn read(path: &Path) -> Result<ActualEntry> {
        let metadata = match path.symlink_metadata() {
            Ok(metadata) => metadata,
            // NotADirectory means some parent of this path is a file on the system.
            // E.g. `/this/is/some/path` and `some` is a file. That kind of issue is handled at
            // the parent's own tree node, so its children count as missing.
            Err(err) if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
                return Ok(ActualEntry::Missing);
            }
            Err(err) => {
                return Err(Error::IoPath(path.to_path_buf(), "reading metadata", err).into());
            }
        };

        // Symlink handling.
        let metadata_type = metadata.file_type();
        if metadata_type.is_symlink() {
            return Ok(ActualEntry::Symlink);
        }

        // Directory and file handling
        if metadata_type.is_dir() || metadata_type.is_file() {
            let mode = mask_mode(metadata.permissions().mode());
            let owner = user_name(metadata.uid());
            let group = group_name(metadata.gid());

            if metadata_type.is_dir() {
                Ok(ActualEntry::Directory { mode, owner, group })
            } else {
                Ok(ActualEntry::File { mode, owner, group })
            }
        } else {
            // Other special filetypes which we don't manage
            Ok(ActualEntry::Special)
        }
    }
}

/// Whether the symlink at the given path ultimately points to a directory.
///
/// `false` is returned for:
///
/// - Dangling symlinks
/// - Link loops
/// - Links that route through a file
pub fn points_to_directory(path: &Path) -> Result<bool> {
    match path.metadata() {
        Ok(metadata) => Ok(metadata.is_dir()),
        // `ELOOP`: too many levels of symbolic links, i.e. a link loop.
        Err(err)
            if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory)
                || err.raw_os_error() == Some(nix::libc::ELOOP) =>
        {
            Ok(false)
        }
        Err(err) => Err(Error::IoPath(path.to_path_buf(), "resolving symlink", err).into()),
    }
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
