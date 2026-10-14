# Issue 335: immutable declaration origins

The pending production admission must not exempt a dependency merely because
its package name matches a local member. It also must classify unchanged
members when their inherited workspace declarations change. Cargo defines
workspace dependency inheritance and separate workspace roots in its
[workspace contract](https://doc.rust-lang.org/cargo/reference/workspaces.html).

`manifest_dependency_origins.py` retains each candidate's declaring manifest
path/blob and its actual supplied definition-provider path/blob. Inherited
entries retain the provider; direct entries retain their own manifest. No
internal-name exemption is applied here, and an external path with a local
package name remains a candidate. Existing snapshot/facade APIs are unchanged.

Actual SHA256 temporary Git repositories exercise six boundaries: equal raw
paths with distinct origins; same-name external paths; root-only changes to
unchanged members; dirty worktree replacement; wrong file kind/missing alias;
and target/build/dev/patch/replace origins. The initial missing-module import
error was a new API integration check, not a reproduced production assertion
failure. Final six tests exited 0 in 4.533 seconds. Release Python discovery
with warnings treated as errors exited 0: 142 tests, 25.235 seconds.

The source and tests remain below the file/function limits. Independent
read-only review found no scoped P0/P1. This is not proof of blob/content
authenticity, same-commit provider binding, registered workspace membership,
lock ownership, public registry state, trusted verification, hook activation
or release cleanup. The actual Git reader and per-commit ownership context
must establish those before any internal exemption or admission PASS.

Next: bind all manifests from each before/after tree to registered roots and
resolved package path/name/version, classify every affected manifest (not
only changed blobs), then connect genuine Issue evidence/publication and
existing-hook-first production admission. The full Issue 335 DoD remains open.

The limited source and six regressions were integrated by normal signed
commit `0bd8b1fdb3a95805169bc10aab078e38edd45c93` (session 23212 exit 0,
signature status G). Upload and current review remain separate.
