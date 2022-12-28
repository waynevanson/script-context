# pioload (🥧-O-🔋)

script-env

## Approaches

What would really be nice is checking for the parent process path that spawned the process.

| Package Manager | Child CWD (rs)                                     | Parent CWD (js)                       |
| :-------------- | :------------------------------------------------- | :------------------------------------ |
| npm             | `PWD`, `npm_package_json` (`./package.json`)       | `INIT_CWD`, `npm_config_local_prefix` |
| yarn (one)      | `PWD`                                              | `INIT_CWD` ,                          |
| pnpm            | `PNPM_SCRIPT_SRC_DIRECTORY` (`+..+`),`PWD` `+..+`) | `INIT_CWD`                            |

Does PNPM actually have the ++ in the names?

yarn

```
npm*config_user_agent: 'yarn/1.22.19 npm/? node/v16.17.0 linux x64',
*: '/home/wayne/.volta/bin/yarn',
npm_config_registry: 'https://registry.yarnpkg.com',
PATH: '/tmp/yarn--1671966863651-0.39172338331863044:/home/wayne/code/pioload/node_modules/.bin:/home/wayne/.config/yarn/link/node_modules/.bin:/home/wayne/.volta/tools/image/node/16.17.0/libexec/lib/node_modules/npm/bin/node-gyp-bin:/home/wayne/.volta/tools/image/node/16.17.0/lib/node_modules/npm/bin/node-gyp-bin:/home/wayne/.volta/tools/image/node/16.17.0/bin/node_modules/npm/bin/node-gyp-bin:/home/wayne/.volta/tools/image/yarn/1.22.19/bin:/home/wayne/.volta/tools/image/node/16.17.0/bin:/home/wayne/.volta/bin:/home/wayne/.volta/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/wayne/.volta/bin:/home/wayne/.cargo/bin:/home/wayne/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin:/snap/bin:/home/wayne/.yarn/bin',
npm_execpath: '/home/wayne/.volta/tools/image/yarn/1.22.19/bin/yarn.js',
```

```
 npm_config_user_agent: 'pnpm/7.19.0 npm/? node/v16.17.0 linux x64',
  _: '/home/wayne/.volta/bin/pnpm',
  npm_config_node_gyp: '/home/wayne/.volta/tools/image/packages/pnpm/lib/node_modules/pnpm/dist/node_modules/node-gyp/bin/node-gyp.js',
  PATH: '/home/wayne/code/pioload/node_modules/.bin:/home/wayne/.volta/tools/image/packages/pnpm/lib/node_modules/pnpm/dist/node-gyp-bin:/home/wayne/.volta/tools/image/yarn/1.22.19/bin:/home/wayne/.volta/tools/image/node/16.17.0/bin:/home/wayne/.volta/bin:/home/wayne/.volta/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/wayne/.volta/bin:/home/wayne/.cargo/bin:/home/wayne/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin:/snap/bin:/home/wayne/.yarn/bin',
  npm_execpath: '/home/wayne/.volta/tools/image/packages/pnpm/lib/node_modules/pnpm/bin/pnpm.cjs',
```
