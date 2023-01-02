module.exports = require("./index.node")

export declare function cli(): void

export type InstallContext = "project" | "package"

export declare function installContext(): InstallContext
