# `@waynvanson/script-context`

## Why

Where are some libraries out in the ecosystem that can do parts of what this tool can do,
but they either do an underwhelming job or are too tied to one aspect.

## Use Cases

### `postinstall` Scripts

It will run either `postinstall:project` or `postinstall:package` based on the context,
using your package manager.

This will work across any lifecycle script.

> TODO - Rename to `project` and `package`.

```json
// package.json

{
  "name": "your-package-name",
  "version": "1.0.0",
  "scripts": {
    "postinstall": "script-context",
    "postinstall:project": "echo 'launched project postinstall'",
    "postinstall:package": "echo 'launched package postinstall'"
  }
}
```

### As library

Can be used in a node, neon/rust application directly.
