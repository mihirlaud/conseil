# Conseil

**Conseil** is a GUI for easily converting Git commit info to Markdown files.

## Installation

After installing the Rust compiler and Cargo, the package manager for Rust, simply clone this repository and run:
```bash
cargo run --release
```

## Usage

Use the search bar in the top left to find the directory of the repository you want to use, and click its path from the list of results. Use the pick list to select the specific commit to turn into Markdown, and edit the placeholder text as needed.

Once you're done, the Export button converts all information to a file in Conseil's directory titled `entry.md`

## To-Do List

* ~~Add filesystem sidebar to pick repos more easily~~
* ~~Add pick list for commits~~
* Add ability to change branches
* Add ability to name Markdown file before it is created
* Add option to use git commit description to autofill parts of the entry