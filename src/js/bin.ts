import { cli } from "../rs/index.cjs"
import { spawnSync } from "child_process"

// reassigning so we keep current context befor it switches
const project = process.env.INIT_CWD
const package_ = process.env.PWD

cli({
  lifecycle: process.env.npm_lifecycle_event,
  dir: { package: package_, project },
  spawn: (command, args) => {
    spawnSync(command, args, {
      cwd: package_,
      stdio: "inherit",
    })
  },
})
