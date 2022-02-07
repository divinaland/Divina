--- @class Package
--- @field public name string
--- @field public version string
--- @field public license string
--- @field public minimum_divina_version string
--- @field public sources string[]
--- @field public compiler string
Package = {
  name = "just_return",
  version = "0.1.0",
  license = "GPL-3.0-only",
  minimum_divina_version = Divina.version,
  sources = {
    "Main.asm",
  },
  type = Divina.Type.Bin,
  arch = Divina.Arch.x64,
  compiler = "nasm",
}

return Package
