# Native inventory collection: limited verification

Source commit: `e4242fcb2725a62c28319464d2771e05b49566bd` (#343).

The parser, immutable condition-aware records, and bounded POSIX child collector
are separated. Each output stream is limited to 1 MiB and collection has a
30-second deadline. Failure kills and reaps the owned child. Windows collection
is explicitly rejected before launching a child; Windows support is unfinished.

Records retain raw bytes, SHA-256, resolved argv, run kind, cases and conditions.
Comparison reparses raw bytes and checks the stored inventory. Missing conditions,
footerless output, invalid identities and truthy non-boolean measurement flags
cannot establish equivalence. Declared matching conditions do not prove trusted
CI execution or authorize reuse of CI, coverage or acceptance results.

Final verification: `python3 -W error -m unittest discover -s scripts/release
-p 'test_*.py'`, actual exit 0, 79 tests in 5.898 seconds (session 7088).
The new real-file regression rejects a missing `./native-list` in the requested
directory even when a different executable exists in the calling directory.
`git diff --check` also succeeded. Normal signed commit exited 0 (session 30220).

A diagnostic native listing contained 1,249 cases and a matching footer, with
115,821 raw bytes and SHA-256
`862a69373a03135df802a77b007fdbd57b5e3ff8117760ff5bf391768552764c`.
That binary came from dirty source; measurement and condition completeness remain
false. Listing cases is not evidence that the cases executed successfully.

Full cross-OS collection, coverage differences, trusted production orchestration,
CI reuse qualification and public release acceptance remain unfinished.
