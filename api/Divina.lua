--- @class Divina
--- @field public version string Installed Divina version
--- @field public Type table<string, number> Package type
--- @field public Arch table<string, number> Architecture to compile for
Divina = {
  version,
  Type = {
    Bin = 1,
    Lib = 2,
  },
  Arch = {
    x86 = 1,
    x64 = 2,
  },
}

--- @return void
function test() end

--- @class Package
--- @field public name string
--- @field public version string
--- @field public description string
--- @field public compile_options string[]
--- @field public minimum_divina_version string
--- @field public sources string[]
--- @field public compiler string
--- @field public visual_studio string
Package = {
  name,
  version,
  description,
  compile_options,
  minimum_divina_version,
  sources,
  type,
  arch,
  compiler,
  visual_studio,
}

--- @class Workspace
--- @field public members string[]
Workspace = {
  members,
}
