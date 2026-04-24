# Linter Fix Feature — Verification Document

This file is used to verify the behavior of the Lint Fix feature in KatanA.
It contains intentional lint violations.

---

## How to Use the Fix Feature

1. Open this file in the editor (Code mode or Split view)
2. Hover over the **💡 icon** displayed to the left of the line number
3. A tooltip will show **`Fix`** (fix this line only) and **`Fix All`** (fix all for this rule) buttons
4. Click to automatically fix the corresponding section

> **Note**: The Fix button only appears on lines where the icon is displayed.
> If the icon does not appear, it means the violation is not fixable.

---

## Violation Samples (Fixable)

# MD022: Headings should be su

rrounded by blank lines
The heading just above this line has no blank lines around it (MD022 violation).

# MD022: Same here

## Further subheading

No blank line between

 the heading lines above.

---

## Violation Samples (Fixable) — MD023: Headings must start at the beginning of the line

# Indented heading (MD023 violation)

## Same here

---

## Violation Samples (Fixable) — MD027: Multiple spaces after blockquote symbol

> Multiple spaces present (MD027 violation)
> More spaces
> This is also a violation

---

## Violation Samples (Fixable) — MD012: Multiple consecutive blank lines

Two blank lines above (MD012 violation).

Two more lines.

---

## Violation Samples (Fixable) — MD029: Ordered list item prefix

1. First
1. Second (Stays as 1 — MD029 violation)
1. Third (Stays as 1 — MD029 violation)

---

## Violation Samples (Fixable) — MD032: Lists should be surrounded by blank lines

The list comes immediately after the sentence (MD032 violation):

- Item A
- Item B
The next sentence is immediately after the list (MD032 violation).

---

## Violation Samples (Non-fixable) — MD003: Heading style

The Fix button **will not be displayed** for these (non-fixable).

Setext style heading (Potential MD003 violation):

Text equivalent to a heading
====================

---

*There is a newline at the end of this file (MD047 compliant)*
