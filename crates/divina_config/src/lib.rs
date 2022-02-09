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

use std::{fmt, fmt::Formatter, io::Read};

use rlua::{Lua, Table};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(unused)]
#[derive(PartialEq)]
enum GetRequired {
  Yes,
  No,
}

/// Get and store a variable from Lua context, if it doesn't exist; `None`.
///
/// With help from:
///
/// - <https://users.rust-lang.org/t/how-to-use-self-or-any-variable-in-macro-generated-function-body/6264>
/// - <https://github.com/steveklabnik/trpl/blob/master/advanced-macros.md>
/// - <https://www.reddit.com/r/rust/comments/iim7b7/how_to_identify_rust_tyttident_expr_patassociated/>
/// - <https://medium.com/@phoomparin/a-beginners-guide-to-rust-macros-5c75594498f1>
#[macro_export]
macro_rules! get_or_none {
  ($table:ident, $key:expr, $key_type:ty, $assign_to:tt, $required:expr) => {
    $assign_to = if $required == GetRequired::No {
      match $table.get::<_, $key_type>($key) {
        Ok(some_value) => Some(some_value),
        Err(_) => None,
      }
    } else {
      Some($table.get::<_, $key_type>($key).expect(&format!(
        "could not access required global `{}`, perhaps you've forgotten to define it ?",
        $key,
      )))
    }
  };
  ($table:ident, $from_table:expr, $key:expr, $key_type:ty, $assign_to:tt, $required:expr) => {
    $assign_to = if $required == GetRequired::No {
      match $table.get::<_, $key_type>($key) {
        Ok(some_value) => Some(some_value),
        Err(_) => None,
      }
    } else {
      Some($table.get::<_, $key_type>($key).expect(&format!(
        "could not access required global `{}.{}`, perhaps you've forgotten to define it ?",
        $from_table, $key,
      )))
    }
  };
}
#[macro_export]
macro_rules! get_table {
  ($assign_to:ident, $key:expr, $globals:ident) => {
    let $assign_to = $globals.get::<_, Table<'_>>($key).expect(&format!(
      "could not get `{}`, perhaps you've made an error ?",
      $key
    ));
  };
}
#[macro_export]
macro_rules! get_enum_or_none {
  ($table:ident, $key:expr, $assign_to:tt, $from_u8:tt, $error_message:expr, $required:expr) => {
    $assign_to = if $required == GetRequired::No {
      match $table.get::<_, u8>($key) {
        Ok(some_value) => Some($from_u8(some_value).expect($error_message)),
        Err(_) => None,
      }
    } else {
      Some(
        $from_u8($table.get::<_, u8>($key).expect(&format!(
          "could not access required global `{}`, perhaps you've forgotten to define it ?",
          $key,
        )))
        .expect($error_message),
      )
    }
  };
  (
    $table:ident,
    $from_table:expr,
    $key:expr,
    $assign_to:tt,
    $from_u8:tt,
    $error_message:expr,
    $required:expr
  ) => {
    $assign_to = if $required == GetRequired::No {
      match $table.get::<_, u8>($key) {
        Ok(some_value) => Some($from_u8(some_value).expect($error_message)),
        Err(_) => None,
      }
    } else {
      Some(
        $from_u8($table.get::<_, u8>($key).expect(&format!(
          "could not access required global `{}.{}`, perhaps you've forgotten to define it ?",
          $from_table, $key,
        )))
        .expect($error_message),
      )
    }
  };
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfigType {
  Package,
  Workspace,
}
impl fmt::Display for ConfigType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    // write!(f, "{:?}", self)
    fmt::Debug::fmt(self, f)
  }
}

#[derive(Debug, Clone)]
pub enum PackageType {
  Bin = 1,
  Lib = 2,
}
impl PackageType {
  #[must_use]
  pub const fn from_u8(n: u8) -> Option<Self> {
    match n {
      1 => Some(Self::Bin),
      2 => Some(Self::Lib),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arch {
  X86 = 1,
  X64 = 2,
}
impl Arch {
  #[must_use]
  pub const fn from_u8(n: u8) -> Option<Self> {
    match n {
      1 => Some(Self::X86),
      2 => Some(Self::X64),
      _ => None,
    }
  }

  #[must_use]
  pub fn from_string(arch: &Self) -> String {
    match arch {
      Arch::X86 => "32".to_string(),
      Arch::X64 => "64".to_string(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Config {
  pub name:                   Option<String>,
  pub version:                Option<String>,
  pub description:            Option<String>,
  pub license:                Option<String>,
  pub compile_options:        Option<Vec<String>>,
  pub minimum_divina_version: Option<String>,
  pub sources:                Option<Vec<String>>,
  pub config_type:            ConfigType,
  pub members:                Option<Vec<Self>>,
  pub package_type:           Option<PackageType>,
  pub path:                   Option<String>,
  pub arch:                   Option<Arch>,
  pub compiler:               Option<String>,
}
impl Config {
  /// Create a new `Config`
  #[must_use]
  pub fn new() -> Self { Self::default() }

  /// Grab configuration values from `Divina.lua` and set
  ///
  /// # Panics
  /// if there are any errors
  #[allow(clippy::too_many_lines)]
  pub fn configure(&mut self, file: &str) {
    let mut script = std::fs::File::open(file)
      .unwrap_or_else(|_| panic!("!! could not locate `{}`, perhaps it doesn't exist ?", file));
    let mut contents = String::new();
    script.read_to_string(&mut contents).unwrap_or_else(|_| {
      panic!(
        "!! could not read `{}`, perhaps there's an encoding error ?",
        file
      )
    });

    let lua = Lua::new();

    #[allow(clippy::cognitive_complexity)]
    lua.context(|ctx| {
      let globals = ctx.globals();
      let test_function = ctx
        .create_function(|_, ()| {
          println!("test");

          Ok(())
        })
        .expect("!! could not create function `test`, this *shouldn't* be possible");

      let divina_table = ctx
        .create_table()
        .expect("!! could not create table `Divina`, this *shouldn't* be possible");
      let type_table = ctx
        .create_table()
        .expect("!! could not create table `Divina.Type`, this *shouldn't* be possible");
      let arch_table = ctx
        .create_table()
        .expect("!! could not create table `Divina.Arch`, this *shouldn't* be possible");

      type_table
        .set("Bin", 1)
        .expect("!! could not set field `Divina.Type.Bin`, this *shouldn't* be possible");
      type_table
        .set("Lib", 2)
        .expect("!! could not set field `Divina.Type.Lib`, this *shouldn't* be possible");

      arch_table
        .set("x86", 1)
        .expect("!! could not set field `Divina.Type.x86`, this *shouldn't* be possible");
      arch_table
        .set("x64", 2)
        .expect("!! could not set field `Divina.Type.x64`, this *shouldn't* be possible");

      divina_table
        .set("version", VERSION)
        .expect("!! could not set field `Divina.version`, this *shouldn't* be possible");
      divina_table
        .set("Type", type_table)
        .expect("!! could not set field `Divina.Type`, this *shouldn't* be possible");
      divina_table
        .set("Arch", arch_table)
        .expect("!! could not set field `Divina.Arch`, this *shouldn't* be possible");

      globals
        .set("Divina", divina_table)
        .expect("!! could not set table `Divina`, this *shouldn't* be possible");
      globals
        .set("test", test_function)
        .expect("!! could not set function `test`, this *shouldn't* be possible");

      ctx.load(contents.as_bytes()).exec().unwrap_or_else(|_| {
        panic!(
          "!! could not execute `{}`, perhaps you've made a syntax error ?",
          file
        )
      });

      self.config_type = if globals.get::<_, Table<'_>>("Workspace").is_ok() {
        ConfigType::Workspace
      } else if globals.get::<_, Table<'_>>("Package").is_ok() {
        ConfigType::Package
      } else {
        divina_util::exit_with!(
          1,
          "!! '{}' is neither `Workspace` nor `Package`, perhaps you've forgotten to assign to it \
           ?",
          file
        );
      };

      if self.config_type == ConfigType::Package {
        get_table!(config_table, "Package", globals);

        get_or_none!(
          config_table,
          "Package",
          "name",
          String,
          (self.name),
          GetRequired::Yes
        );
        get_or_none!(
          config_table,
          "Package",
          "version",
          String,
          (self.version),
          GetRequired::Yes
        );
        get_or_none!(
          config_table,
          "Package",
          "description",
          String,
          (self.description),
          GetRequired::No
        );
        get_or_none!(
          config_table,
          "Package",
          "license",
          String,
          (self.license),
          GetRequired::No
        );
        get_or_none!(
          config_table,
          "Package",
          "compile_options",
          Vec<String>,
          (self.compile_options),
          GetRequired::No
        );
        get_or_none!(
          config_table,
          "Package",
          "minimum_divina_version",
          String,
          (self.minimum_divina_version),
          GetRequired::Yes
        );
        get_or_none!(
          config_table,
          "Package",
          "sources",
          Vec<String>,
          (self.sources),
          GetRequired::No
        );
        get_enum_or_none!(
          config_table,
          "Package",
          "type",
          (self.package_type),
          (PackageType::from_u8),
          "!! could not access `Package.type`, perhaps you've forgotten to assign it ?",
          GetRequired::Yes
        );
        get_enum_or_none!(
          config_table,
          "Package",
          "arch",
          (self.arch),
          (Arch::from_u8),
          "!! could not access `Package.arch`, perhaps you've forgotten to assign it ?",
          GetRequired::Yes
        );
        get_or_none!(
          config_table,
          "Package",
          "compiler",
          String,
          (self.compiler),
          GetRequired::No
        );
      } else {
        get_table!(workspace_table, "Workspace", globals);

        // This sequence of BS actually took about two hours to complete.
        //
        // With help from:
        //
        // - <https://github.com/amethyst/rlua/issues/57>
        let members;
        get_or_none!(
          workspace_table,
          "Workspace",
          "members",
          Vec<String>,
          members,
          GetRequired::No
        );
        if let Some(tables) = members {
          self.members = Some(Vec::new());

          for path in tables {
            let mut config = Self::new();
            config.configure(&format!("{}/Divina.lua", path));
            config.path = Some(path);

            self
              .members
              .as_mut()
              .expect("!! could not access 'Config.members', this *shouldn't* be possible")
              .push(config);
          }
        }
      }
    });
  }
}
impl Default for Config {
  fn default() -> Self {
    Self {
      name:                   None,
      version:                None,
      description:            None,
      license:                None,
      compile_options:        None,
      minimum_divina_version: None,
      sources:                None,
      config_type:            ConfigType::Package,
      members:                None,
      package_type:           None,
      path:                   None,
      arch:                   None,
      compiler:               None,
    }
  }
}
