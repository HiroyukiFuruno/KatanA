"""Closed public record locations for v0.22.47 only."""

ROOT = "evidence/releases/v0.22.47/"
HOST = ROOT + "document-host/"
HOST_ACCEPTANCE = HOST + "acceptance.json"
HOST_RECEIPT = HOST + "download/receipt.json"
HOST_MANIFESTS = {name: HOST + f"download/receipt-build-{name}.json"
                  for name in ("macos", "linux", "windows")}
HOST_BUNDLE = HOST + "observations/bundle.json"
HOST_RUNS = tuple(HOST + f"observations/run-{index:02}.json" for index in range(1, 8))
HOST_FILES = frozenset({HOST_ACCEPTANCE, HOST_RECEIPT, HOST_BUNDLE, *HOST_MANIFESTS.values(), *HOST_RUNS})
CANONICAL_PNG = ROOT + "canonical-reference/sample-diagrams-top.png"
CANONICAL_RECORD = ROOT + "canonical-reference/capture.json"
CANONICAL_FILES = frozenset({CANONICAL_PNG, CANONICAL_RECORD})
