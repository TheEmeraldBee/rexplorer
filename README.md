# Rust Explorer
A command line tool for looking through your project files!

# Installation

Simply install the binary with cargo
```bash
cargo install rexplorer
rexplorer
```

# Usage
Press `Space` To Get the Controls.

`t` will attempt to create a file, and if it exists, it will do nothing
`f` will attempt to create a folder, and if it exists, it will do nothing
`Left, Right, Up, and Down` will navigate within the folders.
`CTRL + d` will delete a file, first asking for a `y` input.

# Issues
In case you can't figure out an issue, please ensure to look at the log in your home directory at `~/.rexplorer/logs/`

If that doesn't help, feel free to create a GitHub Issue.

# Roadmap
- [x] Simple File Navigation
- [x] Create and Delete Files & Folders
- [ ] Maintain location on quit
- [ ] Run Command In Folder with `|`