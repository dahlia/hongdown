Hong Minhee's Markdown style convention
=======================================

This document describes the Markdown style convention enforced by Hongdown.


Philosophy
----------

The core principle of this style is:

> **Markdown should be readable as plain text, not just after rendering.**

A well-formatted Markdown document should convey its structure and emphasis
clearly even when viewed in a plain text editor without any rendering.
You shouldn't need to render the document to HTML to understand its formatting
and hierarchy.

This philosophy leads to several practical implications:

 -  *Visual structure in source*: Headings, lists, and sections should be
    visually distinct in the raw text.
 -  *Consistent spacing*: Predictable whitespace patterns help readers scan
    the document structure.
 -  *Minimal escaping*: Choose delimiter styles that minimize the need for
    escape characters.
 -  *Reference-style links*: Keep prose readable by moving URLs out of the
    text flow.


Headings
--------

### Setext-style for top-level headings

Use Setext-style (underlined) headings for document titles (H1) and major
sections (H2):

~~~~ markdown
Document Title
==============

Section Name
------------
~~~~

*Rationale*: Setext headings create strong visual separation in plain text.
The underline makes the heading immediately recognizable without needing to
count `#` characters.

### ATX-style for subsections

Use ATX-style (`###`, `####`, etc.) for subsections within a section:

~~~~ markdown
### Subsection

#### Sub-subsection
~~~~

*Rationale*: ATX-style is more compact for deeper nesting levels where
Setext-style would be awkward.

### Sentence case

Use sentence case for headings (capitalize only the first word and proper
nouns):

~~~~ markdown
Development commands    ← Correct
Development Commands    ← Incorrect
~~~~

*Rationale*: Sentence case is easier to read and more natural in technical
documentation.

### Underline length

The underline of a Setext-style heading should match the display width of
the heading text, accounting for East Asian wide characters.


Emphasis
--------

### Asterisks for emphasis

Use asterisks (`*`) for emphasis by default:

~~~~ markdown
This is *emphasized* text.
This is **strongly emphasized** text.
~~~~

### Underscores when content contains asterisks

When the emphasized content contains asterisk characters, use underscores
to avoid escaping:

~~~~ markdown
The file _*.txt_ matches all text files.
The pattern __**/*.md__ matches recursively.
~~~~

*Rationale*: This produces cleaner source text by avoiding backslash escapes.

### Escape all underscores in regular text

Underscores in regular text are always escaped, even in the middle of words:

~~~~ markdown
Use the CONFIG\_FILE\_NAME constant.
~~~~

*Rationale*: While CommonMark doesn't treat intraword underscores as emphasis
delimiters, escaping ensures consistent rendering across all Markdown parsers.


Lists
-----

### Unordered list markers

Use ` -  ` (space, hyphen, two spaces) for unordered list items:

~~~~ markdown
 -  First item
 -  Second item
 -  Third item
~~~~

*Rationale*: The leading space creates visual indentation from the left margin.
The two trailing spaces align the text with a 4-space tab stop, making
continuation lines easy to align.

### Nested lists

Indent nested items by 4 spaces:

~~~~ markdown
 -  Parent item
     -  Child item
     -  Another child
 -  Another parent
~~~~

### Ordered list markers

Use `.` for odd nesting levels and `)` for even nesting levels:

~~~~ markdown
 1.  First item
 2.  Second item
     1)  Nested first
     2)  Nested second
 3.  Third item
~~~~

*Rationale*: Alternating markers make the nesting level visually apparent.

### Continuation lines

Align continuation lines with the start of the item text:

~~~~ markdown
 -  This is a list item with text that continues
    on the next line with proper alignment.
~~~~


Code
----

### Fenced code blocks with tildes

Use four tildes (`~~~~`) for fenced code blocks:

~~~~~ markdown
~~~~ rust
fn main() {
    println!("Hello, world!");
}
~~~~
~~~~~

*Rationale*: Tildes are visually distinct from the code content, which often
contains backticks for string literals or shell commands.

### Language identifiers

Always specify a language identifier for syntax highlighting.  If no specific
language applies, the identifier can be omitted:

~~~~~ markdown
~~~~ javascript
console.log("Hello");
~~~~
~~~~~

### Inline code spans

Use backticks for inline code.  When the content contains backticks, use
multiple backticks as delimiters:

~~~~ markdown
Use the `format()` function.
The syntax is `` `code` `` with backticks.
~~~~

Preserve original spacing in code spans.  If the original source has space
padding (`` ` code ` ``), it is preserved in the output.


Links
-----

### Reference-style for external URLs

Convert external URLs to reference-style links, with definitions placed at
the end of the current section:

~~~~ markdown
See the [documentation] for more details.

Read the [installation guide] before proceeding.

[documentation]: https://example.com/docs
[installation guide]: https://example.com/install
~~~~

*Rationale*: Reference-style links keep the prose readable by moving long URLs
out of the text flow.  Placing definitions at section end keeps related content
together.

### Inline style for relative URLs

Keep relative URLs and fragment links inline:

~~~~ markdown
See *[Chapter 2](./chapter2.md)* for more details.
Jump to the [installation section](#installation).
~~~~

*Rationale*: For inter-document links, the filename itself serves as a natural
identifier.  Using reference-style would create redundancy:

~~~~ markdown
See also *[Chapter 2]* for more details.

[Chapter 2]: ./chapter2.md
~~~~

The reference definition just repeats what the link text already conveys.

### Shortcut references when text matches label

When the link text matches the reference label, use shortcut reference syntax:

~~~~ markdown
See [GitHub] for the source code.

[GitHub]: https://github.com/example/repo
~~~~


Block quotes and alerts
-----------------------

### Block quote continuation

Continue block quotes with `>` on each line:

~~~~ markdown
> This is a block quote that spans
> multiple lines of text.
~~~~

### GitHub-style alerts

Use GitHub-flavored alert syntax for callouts:

~~~~ markdown
> [!NOTE]
> This is a note with additional information.

> [!WARNING]
> This action cannot be undone.
~~~~

Supported alert types: `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`.


Tables
------

### Pipe table formatting

Use pipe tables with proper column alignment:

~~~~ markdown
| Name    | Description                    |
| ------- | ------------------------------ |
| foo     | The foo component              |
| bar     | The bar component              |
~~~~

### Column width

Columns are padded to align pipes vertically.  East Asian wide characters
are counted as two columns for proper alignment.

### Escaped pipes in content

Pipe characters within cell content are escaped:

~~~~ markdown
| Pattern   | Meaning          |
| --------- | ---------------- |
| `a \| b`  | a or b           |
~~~~


Line wrapping
-------------

### Wrap at 80 characters

Wrap prose at approximately 80 display columns:

~~~~ markdown
This is a paragraph that demonstrates line wrapping.  When text exceeds the
80-character limit, it wraps to the next line while preserving word boundaries.
~~~~

*Rationale*: 80 characters is a widely supported terminal width that ensures
readability across different editors and viewers.

### East Asian character width

East Asian wide characters (CJK characters) are counted as two columns when
calculating line width.

### Preserve intentional short lines

Lines that are intentionally short in the source (well under the limit) are
preserved as-is, allowing for semantic line breaks.

### Long words

Words that exceed the line width limit are not broken and may extend beyond
80 characters.


Spacing
-------

### Blank lines between elements

Use one blank line between paragraphs, list items (in loose lists), and other
block elements.

### Two blank lines before sections

Use two blank lines before Setext-style section headings (H2):

~~~~ markdown
Previous section content.


New section
-----------
~~~~

*Rationale*: Extra spacing creates clear visual separation between major
sections in the plain text source.

### Trailing newline

Files end with exactly one trailing newline.


Special elements
----------------

### Footnotes

Footnote definitions are placed at the end of the section where they are
referenced:

~~~~ markdown
This claim needs a citation[^1].

[^1]: Source: Example Study, 2024.
~~~~

### Definition lists

Use the extended syntax for definition lists:

~~~~ markdown
Term
:   Definition of the term.

Another term
:   Its definition.
~~~~

### Abbreviations

Abbreviation definitions are preserved at the end of the document:

~~~~ markdown
The HTML specification defines this behavior.

*[HTML]: HyperText Markup Language
~~~~
