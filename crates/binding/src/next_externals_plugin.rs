use std::sync::{Arc, LazyLock};

use rspack_core::{
  ApplyContext, Compilation, CompilationProcessAssets, CompilerOptions, ExternalItem, ExternalItemObject, ExternalItemValue, Plugin, PluginContext
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_externals::ExternalsPlugin;
use rspack_sources::{ConcatSource, RawSource, SourceExt};
use rustc_hash::{FxHashMap, FxHashSet};

const SUPPORTED_NATIVE_MODULES: &[&str] = &[
    "buffer",
    "events", 
    "assert",
    "util",
    "async_hooks",
];

fn get_edge_polyfilled_modules() -> ExternalItem {
    let mut externals = ExternalItemObject::default();
    for &module in SUPPORTED_NATIVE_MODULES {
        externals.insert(module.to_string(), ExternalItemValue::String(format!("commonjs node:{}", module)));
        externals.insert(format!("node:{}", module), ExternalItemValue::String(format!("commonjs node:{}", module)));
    }
    ExternalItem::Object(externals)
}

#[derive(Debug)]
#[plugin]
pub struct NextExternalsPlugin {
  compiler_type: String,
}

impl NextExternalsPlugin {
  pub fn new(banner: String) -> Self {
    Self::new_inner(banner)
  }
}

impl Plugin for NextExternalsPlugin {
  fn name(&self) -> &'static str {
    "NextExternalsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    let is_client = self.compiler_type == "client";
    let is_edge_server = self.compiler_type == "edge-server";
    let is_node_server = self.compiler_type == "server";

    let external_type = if is_client || is_edge_server {
      "assign".to_string()
    } else {
      "commonjs2".to_string()
    };

    let externals = if is_client || is_edge_server {
        if is_edge_server {
            vec![
                ExternalItem::String("next".to_string()),
                ExternalItem::Object(FxHashMap::from_iter([
                    ("@builder.io/partytown".to_string(), ExternalItemValue::String("{}".to_string())),
                    ("next/dist/compiled/etag".to_string(), ExternalItemValue::String("{}".to_string())),
                ])),
                get_edge_polyfilled_modules(),
            ]
        } else {
            vec![ExternalItem::String("next".to_string())]
        }
    } else {
        vec![]
    };

    ExternalsPlugin::new(external_type, externals)
      .apply(PluginContext::with_context(ctx.context), options)?;

    Ok(())
  }
}
