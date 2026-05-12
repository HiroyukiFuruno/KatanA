## Why

v0.22.14 のユーザーレビューフェーズで、ZenUML ブロックが白表示（テーマ背景に合成されず不透明白）になる問題が確認された。
根本原因は kcf 側の ZenUML 出力が `foreignObject` 依存の SVG だったため、kcf v0.1.4 で PNG wrapper に変換したが、その PNG の背景が不透明な白で返っていた（issue [HiroyukiFuruno/katana-canvas-forge#8](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/8)）。
v0.1.5 で kcf 側の ZenUML 出力契約が修正されたため、v0.22.15 として取り込む。

## What Changes

- `katana-canvas-forge` を `0.1.4` → `0.1.5` に更新する。
- KatanA 側のコード変更なし。依存更新のみ。

## Capabilities

### Modified Capabilities

- `diagram-block-preview`: ZenUML ブロックが白背景ではなくテーマ背景に合成された状態で表示される。

## Impact

- `Cargo.toml` / `Cargo.lock`: `katana-canvas-forge` を 0.1.5 へ更新（差分は更新済み）。
