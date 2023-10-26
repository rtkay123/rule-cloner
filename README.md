# Rule Cloner

[![ci](https://github.com/rtkay123/rule-cloner/actions/workflows/ci.yml/badge.svg)](https://github.com/rtkay123/rule-cloner/actions/workflows/ci.yml)
![GitHub tag (with filter)](https://img.shields.io/github/v/tag/rtkay123/rule-cloner)

TLDR: Clones a rule-executor repository and dynamically installs compatible rule libraries

Usage:

```sh
./rule-cloner -o /clone/to/this/directory -c /path/to/config
```

> [!IMPORTANT]  
> Use an absolute path for the `--output` or `-o` argument

# Config
A default git ref is specified on the `executor` table. You can use a  An example [config](config.toml.example) has been provided, to get you started. You can override rule sources by their `name`.

### config.executor
| Key | Description |
| ------ | ------ |
| `source` | the executor repository |
| `ref` | anything that could be used as a git ref (commit, branch, tag) |


### config.rules
| Key | Description |
| ------ | ------ |
| `registry` | npm registry |
| `scope` | package scope |
| `prefix` | A common prefix for all your package names |
| `rules` | A list of rule packages excluding the prefix |
| `override` | (more below) |

### config.rules.override
A list of tables you can use to override the package sources. You need the `name` field to identify the rule you want to override. For each rule:
You can override the executer ref with `executer-ref` as well as specify either a `git` source or a registry (which allows for changing the scope, prefix and version (which is `latest` by default).
