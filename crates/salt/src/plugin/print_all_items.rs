//! sample print-all-items plugin from rustc_plugin examples

use std::{borrow::Cow, env, process::Command};

use clap::Parser;
use rustc_middle::ty::TyCtxt;
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use serde::{Deserialize, Serialize};

// This struct is the plugin provided to the rustc_plugin framework,
// and it must be exported for use by the CLI/driver binaries.
pub struct PrintAllItemsPlugin;

// To parse CLI arguments, we use Clap for this example. But that
// detail is up to you.
#[derive(Parser, Serialize, Deserialize)]
pub struct PrintAllItemsPluginArgs {
  #[arg(short, long)]
  allcaps: bool,

  #[clap(last = true)]
  cargo_args: Vec<String>,
}

impl RustcPlugin for PrintAllItemsPlugin {
  type Args = PrintAllItemsPluginArgs;

  fn version(&self) -> Cow<'static, str> {
    env!("CARGO_PKG_VERSION").into()
  }

  fn driver_name(&self) -> Cow<'static, str> {
    "salt-driver".into()
  }

  // In the CLI, we ask Clap to parse arguments and also specify a CrateFilter.
  // If one of the CLI arguments was a specific file to analyze, then you
  // could provide a different filter.
  fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
    let args = PrintAllItemsPluginArgs::parse_from(env::args().skip(1));
    let filter = CrateFilter::AllCrates;
    RustcPluginArgs { args, filter }
  }

  // Pass Cargo arguments (like --feature) from the top-level CLI to Cargo.
  fn modify_cargo(&self, cargo: &mut Command, args: &Self::Args) {
    cargo.args(&args.cargo_args);
  }

  // In the driver, we use the Rustc API to start a compiler session
  // for the arguments given to us by rustc_plugin.
  fn run(
    self,
    compiler_args: Vec<String>,
    plugin_args: Self::Args,
  ) -> rustc_interface::interface::Result<()> {
    let mut callbacks = PrintAllItemsCallbacks { args: plugin_args };
    let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
    compiler.run()
  }
}

struct PrintAllItemsCallbacks {
  args: PrintAllItemsPluginArgs,
}

impl rustc_driver::Callbacks for PrintAllItemsCallbacks {
  // At the top-level, the Rustc API uses an event-based interface for
  // accessing the compiler at different stages of compilation. In this callback,
  // all the type-checking has completed.
  fn after_analysis<'tcx>(
    &mut self,
    _compiler: &rustc_interface::interface::Compiler,
    queries: &'tcx rustc_interface::Queries<'tcx>,
  ) -> rustc_driver::Compilation {
    // We extract a key data structure, the `TyCtxt`, which is all we need
    // for our simple task of printing out item names.
    queries
      .global_ctxt()
      .unwrap()
      .enter(|tcx| print_all_items(tcx, &self.args));

    // Note that you should generally allow compilation to continue. If
    // your plugin is being invoked on a dependency, then you need to ensure
    // the dependency is type-checked (its .rmeta file is emitted into target/)
    // so that its dependents can read the compiler outputs.
    rustc_driver::Compilation::Continue
  }
}

// The core of our analysis. It doesn't do much, just access some methods on the `TyCtxt`.
// I recommend reading the Rustc Development Guide to better understand which compiler APIs
// are relevant to whatever task you have.
fn print_all_items(tcx: TyCtxt, args: &PrintAllItemsPluginArgs) {
  let hir = tcx.hir();
  
  // for item_id in hir.items() {
  //   let item = hir.item(item_id);
  //   let mut msg = format!(
  //     "There is an item \"{}\" of type \"{}\"",
  //     item.ident,
  //     item.kind.descr()
  //   );
  //   if args.allcaps {
  //     msg = msg.to_uppercase();
  //   }
  //   println!("{msg}");
  // }

  let items: Vec<Item> = hir.items().map(|id| hir.item(id).into()).collect();
  let json_items = serde_json::to_string(&items).unwrap_or("asdf".to_string());
  println!("{json_items}");
}

#[derive(Serialize, Deserialize)]
struct Item {
  name: String, // rustc_span::symbol::Ident,
  ty: &'static str,
}

impl From<&rustc_hir::Item<'_>> for Item {
    fn from(item: &rustc_hir::Item<'_>) -> Self {
        Self { name: format!("{}", item.ident), ty: item.kind.descr() }
    }
}