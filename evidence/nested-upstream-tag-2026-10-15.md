# Nested upstream release-tag identity

Scope: Issue335 and PR346 review comment4282450025
(`PRRT_kwDORm09y86sv96s`, reviewed commit82fca38b).

The evidence schema already permits nested Git tags. The old publication
caller extracted only the final URL path component, losing literal slash
prefixes before requesting the GitHub API. The verifier also accepted only
bare version tags, contradicting that schema.

The caller now extracts the entire suffix after the exact repository release
prefix and URL-decodes it once. API lookup and release identity retain that
complete tag. Only the final literal-slash component is compared with the
requested version. URL encoding does not re-decode a literal percent escape.
Public repository, published/non-draft/non-prerelease and URL checks remain.

## Verification

- Related pure unit tests: saved final run 41 passed in 0.197 seconds, exit0. Raw log:
  `tmp/nested-tag-final-tests-2026-10-15.log`.
- Raw and escaped slash tags, multiple prefixes, valid repository
  `releases/tag`, literal percent escapes, wrong version/URL and unsafe schema
  inputs are covered.
- Main review rejected a candidate that treated literal `%2F` as a version
  delimiter. A new assertion failed with `ValueError not raised` before
  removing that normalization; the corrected suite passes.
- An initial root-directory unittest invocation failed module imports; this
  was an invocation error, not a product regression. The successful run used
  the existing release-test directory/import convention.

These are pure contract tests, not live proof of a nested GitHub release,
current full coverage, current-HEAD approval, publication or cleanup success.
Signed integration a9638417 and normal push88715 subsequently completed at
published2703655e. Reply4282819292 was posted directly to the original finding,
then its thread was resolved and confirmed by fresh complete retrieval:
89threads, no nested pagination missing, REST161reviews/179comments/144conversation
comments across two pages each. Bot6191882028 explicitly reviewed2703655e73 and
reported no major issues. The three pre-existing manual-target policy threads
remain unresolved; no human policy decision or public release is inferred.
