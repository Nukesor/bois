use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

use crate::state::{directory::Directory, State};

/// This struct solely exists to handle state-to-state comparisons of deployed files.
///
/// During the cleanup detection phase, we must compare the previously deployed files to the
/// new desired state. If any files have been removed, they should be cleaned up.
///
/// To efficiently compare the before and after states, we transform the file portion of the
/// [State] object into a simple tree representation, which mimicks the actual deployed file tree.
/// We can then easily compare the two file trees and perform set operations (difference) on it.
///
/// We cannot just do this on the files in the state itself, as those are split into multiple
/// groups. Without [Tree], there's no representation of a "fully-deployed" view of the filesystem.
pub struct Tree {
    root: Node,
}

enum Node {
    Directory(HashMap<String, Node>),
    File(String),
}

impl Tree {
    pub fn from_state(state: &State) -> Tree {
        let mut tree = Tree {
            root: Node::Directory(HashMap::new()),
        };

        tree.add_directory(&state.host.files);

        for group in state.host.groups.iter() {
            tree.add_directory(&group.directory);
        }

        tree
    }

    /// Take a [Directory] from our [State] and completely add it into this tree.
    fn add_directory(&mut self, dir: &Directory) {}

    /// Get a node at the given path of this tree. None if the path doesn't exist.
    fn get_node_mut(&mut self, path: &Path) -> Option<&mut Node> {
        let parts: VecDeque<String> = path
            .to_path_buf()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        Self::get_node_mut_inner(&mut self.root, parts)
    }

    /// Recursively search a tree of nodes based on a given path.
    fn get_node_mut_inner(sub_tree: &mut Node, mut parts: VecDeque<String>) -> Option<&mut Node> {
        // Return the given node, if we reached the end of the given path.
        let Some(part) = parts.pop_front() else {
            return Some(sub_tree);
        };

        // If we need to go deeper, check if the node is a directory.
        // If it isn't a directory, we cannot go deeper.
        let Node::Directory(map) = sub_tree else {
            return None;
        };

        // Check if the next path exists, if not return early.
        let Some(node) = map.get_mut(&part) else {
            return None;
        };

        Self::get_node_mut_inner(node, parts)
    }

    /// Create all nodes that are needed to represent a path in a tree.
    fn create_nodes(&mut self, path: &Path) -> Option<&mut Node> {
        let parts: VecDeque<String> = path
            .to_path_buf()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        Self::create_nodes_inner(&mut self.root, parts)
    }

    /// Recursively create a branch in a tree of nodes based on a given path.
    ///
    /// Return None, if the path cannot be created.
    /// This is the case, if there's a file somewhere in the middle of the path that has the same
    /// name as a directory that's supposed to be there.
    fn create_nodes_inner(sub_tree: &mut Node, mut parts: VecDeque<String>) -> Option<&mut Node> {
        // Return the given node, if we reached the end of the given path and it already exists.
        let Some(part) = parts.pop_front() else {
            return Some(sub_tree);
        };

        // If we need to go deeper, check if the node is a directory.
        // If it isn't a directory, we cannot go deeper.
        // TODO: Should probably be an error.
        let Node::Directory(map) = sub_tree else {
            return None;
        };

        // Check if the next path exists, if not create it.
        let node = map.entry(part).or_insert(Node::Directory(HashMap::new()));

        Self::create_nodes_inner(node, parts)
    }
}
