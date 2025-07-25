#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EsmExternalsConfig {
  None,
  Loose,
  Strict,
}

#[derive(Debug, Clone)]
pub struct ExperimentalConfig {
  pub esm_externals: EsmExternalsConfig,
}

#[derive(Debug, Clone)]
pub struct NextConfigComplete {
  pub server_external_packages: Option<Vec<String>>,
  pub transpile_packages: Option<Vec<String>>,
  pub experimental: Option<ExperimentalConfig>,
}
