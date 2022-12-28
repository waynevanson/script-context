module.exports = require("./index.node")

export declare interface CliParams {
  lifecycle: string | undefined
  dir: Record<"project" | "package", string | undefined>
  spawn: (command: string, args: ReadonlyArray<string>) => void
}

export declare function cli(params: CliParams): void
