# Design

The version-neutral generated identity core owns canonical asset names, platform/architecture sets, record shape, and payload bytes. Fixed-v42 adapters retain their existing API and acceptance policy. The historical download producer remains byte-for-byte unchanged because stored receipts bind its source hash.

The build producer checks actual clean checkout bytes and the committed Cargo version. The prepublication verifier independently derives Git-object provenance and requires all three records to match the exact product commit, workflow run, attempt, version, and five asset checksums. It then opens portable archives and actual DMG/MSI payloads with the existing verifiers.

The existing release workflow cannot admit itself while still running. A separate read-only `workflow_run` identity job therefore runs after completion. Existing PR/Draft conditions remain intact. It reads raw records from the triggering same-repository run, admits the immutable tag/source/run/attempt through the existing selector, freshly downloads all five public assets, and rejects replacement during collection. Its receipt explicitly says `asset-identity-only` and `document_qa=not_evaluated`.

The automatic job follows the existing admitted release-PR and master-dispatch source routes. Whole-run failure from optional cleanup does not suppress verification: required product/publication jobs and the public tag decide admission. Historical tag-push source binding is not expanded. The verifier controller must itself be clean/committed, and its receipt records the controller commit plus hashes of the raw build manifests.

No missing artifact fallback, v44 reconstruction, old approval expansion, controller allow-list widening, or manual user QA condition is allowed. Local synthetic archive tests establish regression coverage only. Actual v45 CI manifests and public payload verification remain separate outstanding release evidence.
