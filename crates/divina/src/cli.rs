// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use structopt::clap::{App, AppSettings, Arg, SubCommand};

/// Create CLI
fn cli() -> App<'static, 'static> {
  App::new(env!("CARGO_PKG_NAME"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommands(vec![
      SubCommand::with_name("init").about("").args(&[
        Arg::with_name("type")
          .long("type")
          .takes_value(true)
          .possible_values(&["bin", "lib"]),
        Arg::with_name("git").long("git").takes_value(true),
        Arg::with_name("path").index(1).takes_value(true),
      ]),
      SubCommand::with_name("build").about("Build your project"),
      SubCommand::with_name("clean")
        .about("Cleanup Divina's non-essential temporary files and directories"),
      SubCommand::with_name("config")
        .about("")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(vec![
          SubCommand::with_name("show").about("Print your configuration"),
          SubCommand::with_name("validate").about("Check if your configuration will compile"),
          SubCommand::with_name("compiler")
            .about("Access the Divina compiler wrapper")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommands(vec![
              SubCommand::with_name("show").about("Print Divina's compiler configuration")
            ]),
        ]),
    ])
    .args(&[
      Arg::with_name("debug").short("d").long("debug"),
      Arg::with_name("trace").short("t").long("trace"),
    ])
}

/// Execute CLI
pub fn execute(divina: &mut crate::Divina) {
  let matches = cli().get_matches();

  match matches.subcommand() {
    ("init", Some(init_matches)) => {
      let repository = init_matches
        .value_of("git")
        .unwrap_or("https://github.com/divinaland/init.git");
      let path = init_matches.value_of("path").unwrap_or(".");

      if Path::new("Divina.lua").exists() {
        divina_util::exit_with!(
          1,
          "!! could not clone init repository to '{}', a 'Divina.lua' already exists",
          path
        );
      }

      divina_git::clone(repository, &format!("./{}", path))
        .expect("!! could not clone init repository, perhaps the repository is invalid ?");
    }
    ("build", Some(_build_matches)) => {
      divina
        .compiler
        .find_sources(&divina.expose_config().clone())
        .compile()
        .link();
    }
    ("clean", Some(_clean_matches)) =>
      if Path::new("out/").exists() {
        println!(":: removing directory 'out/'");
        std::fs::remove_dir_all("out/")
          .expect("!! could not remove directory 'out/', check permissions");
      } else {
        println!(":: directory 'out/' does not exist");
      },
    ("config", Some(config_matches)) =>
      match config_matches.subcommand() {
        ("show", _) => divina.print_config(),
        ("validate", _) => println!(":: no issues found"),
        ("compiler", Some(config_compiler_matches)) =>
          match config_compiler_matches.subcommand() {
            ("show", _) => {
              let _ = divina
                .compiler
                .find_sources(&divina.expose_config().clone())
                .print_config();
            }
            _ => unreachable!(),
          },
        _ => unreachable!(),
      },
    _ => unreachable!(),
  }
}
