#[derive(Clone, Debug)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Debug)]
pub struct File {
    /// The actual configuration file's content, without the bois configuration block.
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct Directory {
    pub entries: Vec<Entry>,
}

/// This struct represents the final rendered (templated) state.
/// The tree structure inside each Group corresponds to the actual structure that'll be deployed to
/// the filesystem lateron.
pub struct Rendered {
    groups: Vec<Directory>,
}
