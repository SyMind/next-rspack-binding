mod config_shared;
mod handle_externals;
mod next_externals_plugin;
mod plugin;

use napi::bindgen_prelude::*;
use rspack_binding_builder_macros::register_plugin;
use rspack_core::BoxPlugin;
use rspack_regex::RspackRegex;

use crate::{
  config_shared::{EsmExternalsConfig, ExperimentalConfig, NextConfigComplete},
  next_externals_plugin::{NextExternalsPlugin, NextExternalsPluginOptions},
};

#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

#[napi(object, object_to_js = false)]
pub struct NapiExperimentalConfig {
  pub esm_externals: Option<Either<String, bool>>,
}

impl From<NapiExperimentalConfig> for ExperimentalConfig {
  fn from(value: NapiExperimentalConfig) -> Self {
    ExperimentalConfig {
      esm_externals: match value.esm_externals {
        Some(esm_externals) => match esm_externals {
          Either::A(s) => {
            if s == "loose" {
              EsmExternalsConfig::Loose
            } else {
              EsmExternalsConfig::Strict
            }
          }
          Either::B(b) => {
            if b {
              EsmExternalsConfig::Strict
            } else {
              EsmExternalsConfig::None
            }
          }
        },
        None => EsmExternalsConfig::None,
      },
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct NapiNextConfigComplete {
  pub experimental: NapiExperimentalConfig,
  pub bundle_pages_router_dependencies: Option<bool>,
}

impl From<NapiNextConfigComplete> for NextConfigComplete {
  fn from(value: NapiNextConfigComplete) -> Self {
    let NapiNextConfigComplete {
      experimental,
      bundle_pages_router_dependencies,
    } = value;
    NextConfigComplete {
      experimental: experimental.into(),
      bundle_pages_router_dependencies,
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct NapiNextExternalsPluginOptions {
  pub compiler_type: String,
  pub config: NapiNextConfigComplete,
  pub builtin_modules: Vec<String>,
  pub opt_out_bundling_package_regex: RspackRegex,
  pub final_transpile_packages: Vec<String>,
  pub dir: String,
}

impl From<NapiNextExternalsPluginOptions> for NextExternalsPluginOptions {
  fn from(value: NapiNextExternalsPluginOptions) -> Self {
    let NapiNextExternalsPluginOptions {
      compiler_type,
      config,
      builtin_modules,
      opt_out_bundling_package_regex,
      final_transpile_packages,
      dir,
    } = value;
    NextExternalsPluginOptions {
      compiler_type,
      config: config.into(),
      builtin_modules,
      opt_out_bundling_package_regex,
      final_transpile_packages,
      dir,
    }
  }
}

// Export a plugin named `MyBannerPlugin`.
//
// The plugin needs to be wrapped with `require('@rspack/core').experiments.createNativePlugin`
// to be used in the host.
//
// Check out `lib/index.js` for more details.
//
// `register_plugin` is a macro that registers a plugin.
//
// The first argument to `register_plugin` is the name of the plugin.
// The second argument to `register_plugin` is a resolver function that is called with `napi::Env` and the options returned from the resolver function from JS side.
//
// The resolver function should return a `BoxPlugin` instance.
register_plugin!("MyBannerPlugin", |_env: Env, options: Unknown<'_>| {
  let banner = options
    .coerce_to_string()?
    .into_utf8()?
    .as_str()?
    .to_string();
  Ok(Box::new(plugin::MyBannerPlugin::new(banner)) as BoxPlugin)
});

register_plugin!("NextExternalsPlugin", |env: Env, object: Unknown<'_>| {
  let napi_options: NapiNextExternalsPluginOptions =
    unsafe { FromNapiValue::from_napi_value(env.raw(), object.raw())? };
  Ok(Box::new(NextExternalsPlugin::new(napi_options.into())) as BoxPlugin)
});
