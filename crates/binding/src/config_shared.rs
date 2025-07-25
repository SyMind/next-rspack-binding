#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EsmExternalsConfig {
  #[default]
  None,
  Loose,
  Strict,
}

#[derive(Debug, Clone, Default)]
pub struct ExperimentalConfig {
  pub esm_externals: EsmExternalsConfig,
}

#[derive(Debug, Clone, Default)]
pub struct NextConfigComplete {
  pub server_external_packages: Option<Vec<String>>,
  pub transpile_packages: Option<Vec<String>>,
  pub experimental: ExperimentalConfig,
}
