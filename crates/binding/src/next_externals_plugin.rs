use std::{
  path::Path,
  sync::{Arc, LazyLock},
};

use rspack_core::{
  ApplyContext, Compilation, CompilationProcessAssets, CompilerOptions, ExternalItem,
  ExternalItemFnCtx, ExternalItemFnResult, ExternalItemObject, ExternalItemValue, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_externals::ExternalsPlugin;
use rspack_sources::{ConcatSource, RawSource, SourceExt};
use rustc_hash::{FxHashMap, FxHashSet};

const SUPPORTED_NATIVE_MODULES: &[&str] = &["buffer", "events", "assert", "util", "async_hooks"];

const SUPPORTED_EDGE_POLYFILLS: LazyLock<FxHashSet<&'static str>> =
  LazyLock::new(|| SUPPORTED_NATIVE_MODULES.iter().copied().collect());

fn get_edge_polyfilled_modules() -> ExternalItem {
  let mut externals = ExternalItemObject::default();
  for &module in SUPPORTED_NATIVE_MODULES {
    externals.insert(
      module.to_string(),
      ExternalItemValue::String(format!("commonjs node:{}", module)),
    );
    externals.insert(
      format!("node:{}", module),
      ExternalItemValue::String(format!("commonjs node:{}", module)),
    );
  }
  ExternalItem::Object(externals)
}

fn is_node_js_module(module_name: &str, builtin_modules: &[String]) -> bool {
  builtin_modules
    .iter()
    .any(|builtin_module| builtin_module == module_name)
}

fn is_supported_edge_polyfill(module_name: &str) -> bool {
  SUPPORTED_EDGE_POLYFILLS.contains(module_name)
}

async fn handle_webpack_external_for_edge_runtime(
  ctx: ExternalItemFnCtx,
  builtin_modules: Arc<Vec<String>>,
) -> rspack_error::Result<ExternalItemFnResult> {
  let is_middleware_or_api_edge = match &ctx.context_info.issuer_layer {
    Some(layer) => layer == "middleware" || layer == "api-edge",
    None => false,
  };

  let result = if is_middleware_or_api_edge
    && is_node_js_module(&ctx.request, &builtin_modules)
    && !is_supported_edge_polyfill(&ctx.request)
  {
    let resolver = ctx
      .resolver_factory
      .get(ctx.resolve_options_with_dependency_type);
    // Allow user to provide and use their polyfills, as we do with buffer.
    match resolver
      .resolve(&Path::new(&ctx.context), &ctx.request)
      .await
    {
      Ok(_) => None,
      Err(_) => Some(ExternalItemValue::String(format!(
        "root globalThis.__import_unsupported('{}')",
        ctx.request
      ))),
    }
  } else {
    None
  };

  Ok(ExternalItemFnResult {
    external_type: None,
    result,
  })
}

#[derive(Debug)]
#[plugin]
pub struct NextExternalsPlugin {
  compiler_type: String,
  builtin_modules: Arc<Vec<String>>,
}

impl NextExternalsPlugin {
  pub fn new(compiler_type: String, builtin_modules: Vec<String>) -> Self {
    Self::new_inner(compiler_type, Arc::new(builtin_modules))
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

    let builtin_modules = self.builtin_modules.clone();
    let externals = if is_client || is_edge_server {
      if is_edge_server {
        vec![
          ExternalItem::String("next".to_string()),
          ExternalItem::Object(FxHashMap::from_iter([
            (
              "@builder.io/partytown".to_string(),
              ExternalItemValue::String("{}".to_string()),
            ),
            (
              "next/dist/compiled/etag".to_string(),
              ExternalItemValue::String("{}".to_string()),
            ),
          ])),
          get_edge_polyfilled_modules(),
          ExternalItem::Fn(Box::new(move |ctx| {
            let builtin_modules = builtin_modules.clone();
            Box::pin(
              async move { handle_webpack_external_for_edge_runtime(ctx, builtin_modules).await },
            )
          })),
        ]
      } else {
        vec![ExternalItem::String("next".to_string())]
      }
    } else {
      self
        .builtin_modules
        .iter()
        .map(|module| ExternalItem::String(module.to_string()))
        .chain([
          ExternalItem::Fn(Box::new(move |ctx| {
            // handle_externals()
            todo!()
          })),
        ])
        .collect::<Vec<_>>()
    };

    ExternalsPlugin::new(external_type, externals)
      .apply(PluginContext::with_context(ctx.context), options)?;

    Ok(())
  }
}
