use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupConfig {
    /// These are the variables that'll be used for templating.
    pub variables: HashMap<String, String>,
}
