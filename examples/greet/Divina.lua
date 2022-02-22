--  This variable is `local`, meaning Divina cannot see it.
--
--- @type string
local name = "greet-" .. tostring(os.time())

--  This variable is **not** `local`, Divina can see it.
--
--- @class Package
--- @field public name string
--- @field public version string
--- @field public description string
--- @field public compile_options string[]
--- @field public minimum_divina_version string
--- @field public sources string[]
--- @field public visual_studio string
Package = {
  name = name,
  version = "0.1.0",
  description = "Greet me!",
  compile_options = {},
  -- The `Divina` table is a special table that you can access anywhere from
  -- the `Divina.lua` script. It contains all sorts of important information.
  --
  -- Here we are setting our `minimum_divina_version` to `Divina.version`.
  minimum_divina_version = Divina.version,
  sources = {
    "Main.asm",
  },
  type = Divina.Type.Bin,
  arch = Divina.Arch.x64,
  visual_studio = "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\Build\\vcvars64.bat",
}

-- http://lua-users.org/wiki/ModulesTutorial
return Package
