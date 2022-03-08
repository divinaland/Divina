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

use std::fs;

use divina_config::Arch;

#[derive(Debug, Clone)]
struct Source {
  filename: String,
  path:     String,
}

#[derive(Debug, Clone)]
struct Package {
  name:          String,
  sources:       Vec<Source>,
  arch:          Arch,
  compiler:      String,
  #[allow(unused)]
  visual_studio: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Compiler {
  sources:    Vec<Package>,
  is_package: bool,
}
impl Compiler {
  #[must_use]
  pub fn new() -> Self { Self::default() }

  pub fn find_sources(&mut self, config: divina_config::Config) -> &Self {
    if config.config_type == divina_config::ConfigType::Workspace {
      for member in config.members.expect(
        "!! could not access 'Config.members' from `workspace`, this *shouldn't* be possible",
      ) {
        let mut package = Package {
          name:          member.name.expect(
            "!! could not access `Config.?.name` from `workspace`, this *shouldn't* be possible",
          ),
          sources:       Vec::new(),
          arch:          member
            .arch
            .expect("!! could not access 'Config.members.?.arch', this *shouldn't* be possible"),
          compiler:      member.compiler.unwrap_or_else(|| "yasm".to_string()),
          visual_studio: member.visual_studio,
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
        name:          config
          .name
          .expect("!! could not access `Config.name` from `Package`, this *shouldn't* be possible"),
        sources:       Vec::new(),
        arch:          config
          .arch
          .expect("!! could not access 'Config.arch', this *shouldn't* be possible"),
        compiler:      if config.compiler.is_some() {
          config
            .compiler
            .expect("!! could not access 'Config.compiler', this *shouldn't be possible")
        } else {
          "yasm".to_string()
        },
        visual_studio: config.visual_studio,
      };

      config
        .sources
        .as_ref()
        .expect("!! could not access 'Config.sources' from 'Package', this *shouldn't* be possible")
        .iter()
        .for_each(|source| {
          if !source.is_empty() {
            package.sources.push(Source {
              path:     source.to_string(),
              filename: {
                let mut sources = source.split('.');
                // Remove the file extension
                sources.next_back();

                let sources_no_extension = sources.collect::<String>();
                sources = sources_no_extension.split('/');

                sources
                  .next_back()
                  .expect("!! could not get filename from source, this is an anomaly")
                  .to_string()
              },
            });
          }
        });

      self.sources.push(package);
    }

    self.is_package = self.sources.len() == 1;

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
      let package_out_directory = if self.is_package {
        "out/".to_string()
      } else {
        format!("out/{}/", package.name)
      };

      if !std::path::Path::new(&package_out_directory).exists() {
        println!(
          ":: {} @@ creating directory '{}'",
          package.name, package_out_directory
        );
        fs::create_dir_all(&package_out_directory).unwrap_or_else(|_| {
          panic!(
            "!! could not create directory '{}', check permissions",
            package_out_directory
          )
        });
      }

      for source in &package.sources {
        println!(
          ":: {} @@ {} ?? compiling source '{}'",
          package.name, package.compiler, source.path
        );

        #[cfg(unix)]
        unix::compile(
          &package.compiler,
          if package.arch == Arch::X86 {
            "elf32"
          } else {
            "elf64"
          },
          &source.path,
          &if self.is_package {
            format!("out/{}.o", source.filename)
          } else {
            format!("out/{}/{}.o", package.name, source.filename)
          },
        );

        #[cfg(windows)]
        windows::compile(
          &package.compiler,
          if package.arch == Arch::X86 {
            "win32"
          } else {
            "win64"
          },
          &source.path,
          &if self.is_package {
            format!("out/{}.obj", source.filename)
          } else {
            format!("out/{}/{}.obj", package.name, source.filename)
          },
        );
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
        filenames.push(format!(
          "out/{}{}.{}",
          &if self.is_package {
            "".to_string()
          } else {
            format!("{}/", &package.name)
          },
          source.filename,
          {
            if cfg!(windows) {
              "obj"
            } else {
              "o"
            }
          }
        ));

        #[allow(unused)]
        arch = &package.arch;
      }

      #[cfg(windows)]
      println!(
        ":: {} @@ entering visual studio developer command prompt environment",
        package.name
      );

      println!(
        ":: {} @@ linking source{}: '{}'",
        package.name,
        if filenames.len() > 1 { "s" } else { "" },
        filenames.join("', '")
      );

      #[cfg(unix)]
      unix::link(
        "ld",
        "/lib64/ld-linux-x86-64.so.2",
        &if self.is_package {
          format!("out/{}", package.name)
        } else {
          format!("out/{}/{}", package.name, package.name)
        },
        &filenames.join(" "),
      );

      #[cfg(windows)]
      {
        if let Some(visual_studio_path) = &package.visual_studio {
          windows::link_package_custom(&filenames.join(" "), &package.name, visual_studio_path);
        } else if arch == &Arch::X64 {
          if self.is_package {
            windows::link_package_64(&filenames.join(" "), &package.name);
          } else {
            windows::link_workspace_64(&filenames.join(" "), &package.name);
          }
        } else if self.is_package {
          windows::link_package_32(&filenames.join(" "), &package.name);
        } else {
          windows::link_workspace_32(&filenames.join(" "), &package.name);
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

#[cfg(unix)]
#[rustfmt::skip] // Preserve raw string literal positions
mod unix {
  use shellfn::shell;

  #[shell]
  pub fn compile(compiler: &str, architecture: &str, source_path: &str, out_file: &str) -> String { r#"
    $COMPILER -f $ARCHITECTURE $SOURCE_PATH -o $OUT_FILE
  "# }

  #[shell]
  pub fn link(linker: &str, dynamic_linker: &str, out_file: &str, sources: &str) -> String { r#"
    $LINKER -dynamic-linker $DYNAMIC_LINKER -lc -o $OUT_FILE $SOURCES
  "# }
}

#[cfg(windows)]
#[rustfmt::skip]
mod windows {
  use shellfn::shell;

  #[shell(cmd = "powershell")]
  pub fn compile(compiler: &str, architecture: &str, sources: &str, out_file: &str) -> String { r#"
    $COMPILER -f $ARCHITECTURE $SOURCES -o $OUT_FILE
  "# }

  /// Thank lord for the [shellfn](https://github.com/synek317/shellfn) crate...
  ///
  /// I unironically spent **SIX** hours -- give or take a few minutes... --
  /// trying to get the Windows linker working. After searching far and wide,
  /// the shellfn crate happened to come up in my search results after a
  /// random search, and it worked!
  ///
  /// Thanks, shellfn.
  #[shell(cmd = "powershell")]
  pub fn link_workspace_32(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars32.bat"
  "# }
  #[shell(cmd = "powershell")]
  pub fn link_workspace_64(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
  "# }
  #[shell(cmd = "powershell")]
  pub fn link_package_32(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars32.bat"
  "# }
  #[shell(cmd = "powershell")]
  pub fn link_package_64(objects: &str, filename: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
  "# }
  #[shell(cmd = "powershell")]
  pub fn link_package_custom(objects: &str, filename: &str, visual_studio_path: &str) -> String { r#"
    "link /subsystem:console /out:out/$FILENAME.exe $OBJECTS kernel32.lib msvcrt.lib legacy_stdio_definitions.lib" | cmd /k "$VISUAL_STUDIO_PATH"
  "# }
}
