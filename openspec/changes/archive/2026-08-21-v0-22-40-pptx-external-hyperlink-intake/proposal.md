## Why

KatanA v0.22.39 consumes published `katana-document-viewer 0.5.3`, but a valid
PPTX containing an OOXML `TargetMode="External"` hyperlink relationship fails before
its first slide is displayed. The failure is in KDV's Office preflight policy, not in
KatanA's document host.

Published KDV v0.5.4 accepts only the standard transitional and strict OOXML hyperlink
relationship types while retaining the worker's network isolation. KatanA must consume
that published registry release and prove the actual host can display the affected PPTX
without taking ownership of Office parsing, layout, or resource access.

## What Changes

- Consume exact registry `katana-document-viewer = "=0.5.4"` in KatanA v0.22.40.
- Keep the official transitive `office2pdf 0.6.7` chain and reject retired,
  path, and git sources in the release contract.
- Add a test-only PPTX relationship fixture generator and headless host acceptance that
  opens an external-hyperlink PPTX through the existing KatanA document surface and
  requires a rendered PPTX `Page` frame.
- Synchronize the adjacent patch SemVer guard and release evidence for
  v0.22.39 -> v0.22.40.

## Non-Goals

- Adding an Office/PDF engine, archive parser, layout engine, hyperlink handler, font
  resolver, KUC dependency, or network policy to KatanA.
- Fetching, opening, or executing the hyperlink target from a PPTX.
- Changing KDV/KRR/KUC public APIs or recreating their document implementation in KatanA.
- Introducing Chromium, WebView, PDFium, LibreOffice, or git/path dependencies.
