/**
 * @description
 * This file contains the runtime exports of the rust binary,
 * the declarations for the runtime exports.
 */
module.exports = require("./index.node")

export declare function cli(): void

export type InstallContext = "project" | "package"

export declare function installContext(): InstallContext
