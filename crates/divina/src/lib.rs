// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code
)]
#![deny(clippy::all, clippy::nursery, clippy::pedantic)]
#![recursion_limit = "128"]
#![doc(
  html_logo_url = "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/160/twitter/282/ribbon_1f380.png",
  html_favicon_url = "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/160/twitter/282/ribbon_1f380.png"
)]

// #[macro_use]
// extern crate log;

#[cfg(windows)]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(unix)]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod cli;

use divina_compile::Compiler;
use divina_config::Config;

#[derive(Default)]
pub struct Divina {
  config:   Config,
  compiler: Compiler,
}
impl Divina {
  #[must_use]
  pub fn new() -> Self { Self::default() }

  pub fn perform(&mut self) { crate::cli::execute(self); }

  /// Prepare `self.config` for configuration
  pub fn new_config(&mut self) -> &mut Self {
    self.config = Config::new();
    self
  }

  /// Configure `self.config`
  pub fn configure_config(&mut self) { self.config.configure("Divina.lua"); }

  /// Print `self.config`
  pub fn print_config(&self) {
    println!("{:?}", self.config);
  }

  #[must_use]
  pub const fn expose_config(&self) -> &Config { &self.config }

  pub fn configure_compiler(&mut self, compiler: Compiler) { self.compiler = compiler; }
}

/// Preliminary setup
pub fn setup() {
  dotenv::dotenv().ok();
  human_panic::setup_panic!(Metadata {
    version:  env!("CARGO_PKG_VERSION").into(),
    name:     env!("CARGO_PKG_NAME").into(),
    authors:  env!("CARGO_PKG_AUTHORS").into(),
    homepage: env!("CARGO_PKG_HOMEPAGE").into(),
  });
}
