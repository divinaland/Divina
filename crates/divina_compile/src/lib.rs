// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

#![feature(stmt_expr_attributes)]
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

use std::{fs, process::Command};

use divina_config::Arch;

#[derive(Debug)]
struct Source {
  filename: String,
  path:     String,
}

#[derive(Debug)]
struct Package {
  name:     String,
  sources:  Vec<Source>,
  arch:     Arch,
  compiler: String,
}

#[derive(Default, Debug)]
pub struct Compiler {
  sources: Vec<Package>,
}
impl Compiler {
  #[must_use]
  pub fn new() -> Self { Self::default() }

  pub fn find_sources(&mut self, config: &divina_config::Config) -> &Self {
    if config.config_type == divina_config::ConfigType::Workspace {
      for member in config.members.as_ref().expect(
        "!! could not access 'Config.members' from `workspace`, this *shouldn't* be possible",
      ) {
        let mut package = Package {
          name:     member.name.clone().expect(
            "!! could not access `Config.?.name` from `workspace`, this *shouldn't* be possible",
          ),
          sources:  Vec::new(),
          arch:     member
            .arch
            .clone()
            .expect("!! could not access 'Config.members.?.arch', this *shouldn't* be possible"),
          compiler: member
            .compiler
            .clone()
            .unwrap_or_else(|| "yasm".to_string()),
        };

        member
          .sources
          .as_ref()
          .expect(
            "!! could not access 'Config.sources' from 'workspace.members', this *shouldn't* be \
             possible",
          )
          .iter()
          .for_each(|source| {
            if !source.is_empty() {
              package.sources.push(Source {
                path:     format!(
                  "{}/{}",
                  member.path.as_ref().expect(
                    "!! could not access 'Config.members.?.path', this *shouldn't* be possible"
                  ),
                  source
                ),
                filename: {
                  let mut sources = source.split('.');
                  // Remove the file extension
                  sources.next_back();

                  sources.collect()
                },
              });
            }
          });

        self.sources.push(package);
      }
    } else {
      let mut package = Package {
        name:     config
          .name
          .clone()
          .expect("!! could not access `Config.name` from `Package`, this *shouldn't* be possible"),
        sources:  Vec::new(),
        arch:     config
          .arch
          .clone()
          .expect("!! could not access 'Config.arch', this *shouldn't* be possible"),
        compiler: if config.compiler.is_some() {
          config
            .compiler
            .clone()
            .expect("!! could not access 'Config.compiler', this *shouldn't be possible")
        } else {
          "yasm".to_string()
        },
      };

      config
        .sources
        .as_ref()
        .expect("!! could not access 'Config.sources' from 'Package', this *shouldn't* be possible")
        .iter()
        .for_each(|source| {
          if !source.is_empty() {
            package.sources.push(Source {
              path:     format!(
                "{}/{}",
                config
                  .path
                  .as_ref()
                  .expect("!! could not access 'Config.path', this *shouldn't* be possible"),
                source
              ),
              filename: {
                let mut sources = source.split('.');
                // Remove the file extension
                sources.next_back();

                sources.collect()
              },
            });
          }
        });

      self.sources.push(package);
    }

    self
  }

  /// # Panics
  /// if caller has insufficient permissions to create a directory
  #[must_use]
  pub fn compile(&self) -> &Self {
    if !std::path::Path::new("out/").exists() {
      println!(":: creating directory 'out/'");
      fs::create_dir_all("out/").expect("!! could not create directory 'out/', check permissions");
    }

    for package in &self.sources {
      if !std::path::Path::new(&format!("out/{}/", package.name)).exists() {
        println!(
          ":: {} @@ creating directory 'out/{}/'",
          package.name, package.name
        );
        fs::create_dir_all(&format!("out/{}/", package.name)).unwrap_or_else(|_| {
          panic!(
            "!! could not create directory 'out/{}/', check permissions",
            package.name
          )
        });
      }

      for source in &package.sources {
        println!(
          ":: {} @@ {} ?? compiling source '{}'",
          package.name, package.compiler, source.path
        );

        #[cfg(unix)]
        Command::new(&package.compiler)
          .args([
            "-f",
            if package.arch == Arch::X86 {
              "elf32"
            } else {
              "elf64"
            },
            &source.path,
            "-o",
            &format!("out/{}/{}.o", package.name, source.filename),
          ])
          .output()
          .expect(&format!(
            "!! failed to call command `{}` in `Compiler.compile`",
            package.compiler
          ));

        #[cfg(windows)]
        Command::new(&package.compiler)
          .args([
            "-f",
            if package.arch == Arch::X86 {
              "win32"
            } else {
              "win64"
            },
            &source.path,
            "-o",
            &format!("out/{}/{}.obj", package.name, source.filename),
          ])
          .output()
          .unwrap_or_else(|_| {
            panic!(
              "!! failed to call command `{}` in `Compiler.compile`",
              package.compiler
            )
          });
      }
    }

    self
  }

  /// # Panics
  /// if Visual Studio 2019 is not installed
  pub fn link(&self) {
    for package in &self.sources {
      let mut filenames = Vec::new();
      #[allow(unused)]
      let mut arch = &Arch::X86;

      for source in &package.sources {
        filenames.push(format!("out/{}/{}.{}", package.name, source.filename, {
          if cfg!(windows) {
            "obj"
          } else {
            "o"
          }
        }));

        #[allow(unused)]
        arch = &package.arch;
      }

      #[cfg(unix)]
      {
        println!(
          ":: {} @@ linking sources: '{}'",
          package.name,
          filenames.join("', '")
        );

        Command::new("ld")
          .args([
            "-dynamic-linker",
            "/lib64/ld-linux-x86-64.so.2",
            "-lc",
            "-o",
            &format!("out/{}/{}", package.name, package.name),
          ])
          .args(filenames.iter())
          .output()
          .expect("!! failed to call command `ld` in `Compiler.link`");
      }

      #[cfg(windows)]
      {
        println!(
          ":: {} @@ entering visual studio 2019 developer command prompt environment",
          package.name
        );
        println!(
          ":: {} @@ linking sources: '{}'",
          package.name,
          filenames.join("', '")
        );
        if arch == &Arch::X64 {
          windows::link_64(&filenames.join(" "), &package.name);
        } else {
          windows::link_32(&filenames.join(" "), &package.name);
        }
      }
    }
  }

  #[must_use]
  pub fn print_config(&self) -> &Self {
    println!("{:?}", self);

    self
  }
}

#[cfg(windows)]
#[rustfmt::skip] // Preserve raw string literal positions
mod windows {
  /// Thank lord for the [shellfn](https://github.com/synek317/shellfn) crate...
  ///
  /// I unironically spent **SIX** hours -- give or take a few minutes... --
  /// trying to get the Windows linker working. After searching far and wide,
  /// the shellfn crate happened to come up in my search results after a
  /// random search, and it worked!
  ///
  /// Thanks, shellfn.
  #[shellfn::shell(cmd = "powershell")]
  pub fn link_32(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars32.bat"
  "# }
  #[shellfn::shell(cmd = "powershell")]
  pub fn link_64(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
  "# }
}
