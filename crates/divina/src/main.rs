// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

use divina::Divina;

#[tokio::main]
async fn main() {
  // Preliminary pokes
  divina::setup();

  let mut divina = Divina::new();
  // Store 'Divina.lua' configuration
  divina.new_config().configure_config();
  // Create a new compiler
  divina.configure_compiler(divina_compile::Compiler::new());
  // Handle CLI
  divina.perform();

  // Process doesn't exit on Unix properly, this solves it...
  #[cfg(unix)]
  std::process::exit(0);
}
