# DOCX data-descriptor reproduction

## Fixture contract

`scripts/screenshot/generate_data_descriptor_docx.py` rewrites the existing
representative DOCX through a non-seekable stream. The resulting
`data-descriptor.docx` has SHA-256
`a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a`.
Python `zipfile` and `unzip -t` both accept the archive.

All 20 entries use ZIP general-purpose bit 3. For `word/document.xml`:

- local-header flag: `0x0008`
- local-header CRC32: `0`
- local-header compressed size: `0`
- local-header uncompressed size: `0`
- central-directory compressed size: `1383`
- central-directory uncompressed size: `4907`

This is the standards-compliant case where final sizes are supplied by the
data descriptor and central directory rather than the local header.

## KatanA host result with public KDV 0.5.5

The screenshot harness opened the generated fixture through the real KatanA
document worker. Source intake and worker handoff completed, then KDV failed
before its first frame:

```text
Layer: KDV worker
Operation: open
Format: docx
Cause: Office package is invalid: unsupported Zip archive:
The file length is not available in the local header
```

This records the exact previously missing DOCX reproduction and assigns the
failure to KDV's Office archive intake rather than KatanA host code. KDV Issue
[#46](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/46)
tracks the owner-layer fix without weakening entry-count, expanded-byte,
compression-ratio, encryption, timeout, or path-safety limits.

The post-self-review KatanA release rebuild reproduces the same typed boundary
unchanged: layer `KDV worker`, operation `open`, format `docx`, and cause
`Office package is invalid: unsupported Zip archive: The file length is not
available in the local header`. The KatanA host does not reinterpret or hide
the owner-layer error.
