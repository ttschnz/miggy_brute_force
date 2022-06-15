# Miggy is not save

## DISCLAIMER

This repo contains scripts that can be used for hacking. The distributor clearly states that the scripts are for education purposes only but provide a fully functional piece of hacking software.

## Instructions

### Building

Prerequisites: rust compiler and cargo installed

1. Clone this repo
2. Enter the command line with the repo as cwd
3. Execute `cargo build -r`

### Running

1. Download the application from the releases page
2. Execute it and follow its instructions:
   1. Create an account for https://win.migros.ch/promos/de/
   2. Open the developer options (<kbd>F12</kbd>, <kbd>ctrl</kbd>+<kbd>I</kbd>, or <kbd>⌘</kbd>+<kbd>I</kbd>)
   3. Navigate to the Network tab
   4. Enter a random bon of length 28 (`0000000000000000000000000000` for example)
   5. Look for a request with the name `play.php`
   6. Click on it
   7. Scroll down to `Request Headers`
   8. Right click on the `Cookie`-Value
   9. Copy it
   10. Paste it in the application
   11. Enter configuration as desired. Empty inputs will be replaced with the default values given.
