/// Packages that should always be installed for this trait.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Packages {
    #[serde(default)]
    pub packages: HashMap<PackageManager, HashSet<String>>,
}
