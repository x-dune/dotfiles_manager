
# dotfiles_manager (dfm)

A cli tool to manage dotfiles. Provides feature such as:

- Templating using [handlebars](https://handlebarsjs.com/)
- Symlinking

---

## How to use

1. Clone the repository & `cd` into it
2. Install `dfm` binary by running `cargo install --locked --path .`
3. Run `dfm` in a folder containing the following folder structure:

```
.
├── home
│   ├── .config
│   │   └── helix
│   │       └── config.toml.hbs
│   ├── .gitconfig.hbs
│   └── global.gitignore
└── values.toml
```
4.  `dfm` will
    - render out template files ending in `hbs` using values in `values.toml` and write output into the output folder
    - copy raw files into the output folder
    - symlink each file in the output folder against the home directory

```
out
├── .config
│   └── helix
│       └── config.toml
├── .gitconfig
└── global.gitignore
```

---
