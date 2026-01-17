Hongdown for VS code
====================

A VS Code extension that formats Markdown files using [Hongdown], enforcing
[Hong Minhee's Markdown style conventions].

[Hongdown]: https://github.com/dahlia/hongdown
[Hong Minhee's Markdown style conventions]: https://github.com/dahlia/hongdown/blob/main/STYLE.md


Prerequisites
-------------

This extension requires the `hongdown` CLI to be installed on your system.
Install it using one of the following methods:

~~~~ bash
# npm
npm install -g hongdown

# mise
mise use -g github:dahlia/hongdown

# Cargo
cargo install hongdown
~~~~


Installation
------------

Install from the [Visual Studio Marketplace] or search for “Hongdown” in the
VS Code Extensions panel.

[Visual Studio Marketplace]: https://marketplace.visualstudio.com/items?itemName=hongdown.hongdown-vscode


Usage
-----

Once installed, Hongdown automatically registers as a Markdown formatter.

### Format on save

To enable format on save for Markdown files, add this to your VS Code settings:

~~~~ json
{
  "[markdown]": {
    "editor.defaultFormatter": "hongdown.hongdown-vscode",
    "editor.formatOnSave": true
  }
}
~~~~

### Manual formatting

 -  Press `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (macOS) to format
    the current document
 -  Right-click and select “Format Document”
 -  Use the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and run
    “Format Document”


Configuration
-------------

### VS code settings

| Setting            | Default | Description                                      |
| ------------------ | ------- | ------------------------------------------------ |
| `hongdown.disable` | `false` | Disable the formatter.  When set to `true`,      |
|                    |         | Hongdown will not appear in formatter selection. |
| `hongdown.path`    | `""`    | Path to hongdown executable.  Leave empty to use |
|                    |         | PATH (recommended for mise users).               |

The formatter is enabled by default.  Set `hongdown.disable` to `true` to
disable it for specific workspaces or globally.

### Project configuration

Hongdown reads formatting options from a `.hongdown.toml` file in your project
directory.  See the [Hongdown documentation] for available options.

[Hongdown documentation]: https://github.com/dahlia/hongdown#configuration-file


Troubleshooting
---------------

### “Failed to start Hongdown” Error

This error occurs when the extension cannot find the `hongdown` executable.

**Solutions:**

1.  **Install hongdown**: Make sure hongdown is installed (see Prerequisites).

2.  **Check PATH**: Ensure the directory containing `hongdown` is in your
    shell's PATH.

3.  **Set explicit path**: If you're using a version manager like mise, the
    PATH may not be available in VS Code.  Set the `hongdown.path` setting to
    the full path of the hongdown executable:

    ~~~~ json
    {
      "hongdown.path": "/Users/yourname/.local/share/mise/installs/hongdown/latest/bin/hongdown"
    }
    ~~~~

4.  **Restart VS Code**: After installing hongdown or changing PATH, restart
    VS Code to pick up the changes.


Style rules
-----------

Hongdown enforces a consistent Markdown style:

 -  Level 1 and 2 headings use setext-style (underlined)
 -  Unordered lists use ` -  ` (space-hyphen-two spaces)
 -  Code blocks are fenced with four tildes
 -  Lines wrap at 80 characters (configurable)
 -  External URLs are converted to reference-style links

See the [full style guide] for details.

[full style guide]: https://github.com/dahlia/hongdown/blob/main/STYLE.md


License
-------

Distributed under the [GPL-3.0-or-later].

[GPL-3.0-or-later]: https://www.gnu.org/licenses/gpl-3.0.html
