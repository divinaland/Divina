--- @class Package
--- @field public name string
--- @field public version string
--- @field public description string
--- @field public compile_options string[]
--- @field public minimum_divina_version string
--- @field public sources string[]
Package = {
  name = "greet_from_library",
  version = "0.1.0",
  description = "Greet me... from a library!",
  compile_options = {},
  minimum_divina_version = Divina.version,
  sources = {
    "Main.asm",
    "Library.asm",
  },
  type = Divina.Type.Bin,
  arch = Divina.Arch.x64,
}

return Package
