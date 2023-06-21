# Conseil

**Conseil** is a GUI for easily converting Git commit info to Markdown files.

## Installation

After installing the Rust compiler and Cargo, the package manager for Rust, simply clone this repository and run:
```bash
cargo run --release
```

## Usage

All Conseil needs is the file path of the repository you want to use and the commit ID you want to write information for. If no ID is given, the most recent commit on HEAD will be used.

Once the placeholder text is edited as desired, the Export button converts all information to a file in Conseil's directory titled `entry.md`

## To-Do List

* Add filesystem sidebar to pick repos more easily
* Add pick list for commits
* Add ability to change branches
* Add ability to name Markdown file before it is created
* Add option to use git commit description to autofill parts of the entry