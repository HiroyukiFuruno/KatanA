## Context

v0.26.0にて実施されたPreviewコンポーネントの分離に続き、本設計ではエディタ機能を完全に分離された別クレートとして独立させます。現在の `katana-ui` はFat UIの傾向があり、エディタウィジェットのドメイン知識（絵文字フォントのハンドリング等のマルチプラットフォーム固有の問題を含む）を抱え込んでいます。本設計でこれらを `katana-editor` にカプセル化します。

v0.27.0 は後続の preview-driven local editing と組み合わせ、KatanA を code editor 主軸から preview-first な局所修正体験へ寄せるための土台です。egui `TextEdit` を置き換える独自入力 surface は、v0.22.5 入力強化で再浮上した別論点として `x-x-x-native-input-surface` に劣後・分離します。

## Goals / Non-Goals

**Goals:**
- エディタ機能を `katana-editor` クレートに分離する。
- macOSやWindowsにおける絵文字フォント問題など、マルチプラットフォーム特有の描画・フォント管理の複雑性を、将来の独自入力 surface と衝突しない設定注入 boundary として整理する。
- 各コンポーネントが独自にインテグレーションテストを実行できるようにする。

**Non-Goals:**
- アプリケーション全体のUIデザインを刷新すること。
- Previewコンポーネントの分離（v0.26.0にて実施済み）。
- egui `TextEdit` を完全に置き換える独自入力 surface を実装すること（`x-x-x-native-input-surface` の責務）。

## Decisions

- **完全な物理クレート分離**: `crates/` フォルダ配下に `katana-editor` を新規クレートとして作成し、完全に物理的な分離を行います。KatanAの肥大化によるCI/CD（UT/IT）のネック解消やマルチプラットフォーム対応の複雑性緩和のメリットを優先します。
- **インターフェースによる設定注入とデータ駆動アーキテクチャ**: グローバル設定（テーマやフォント、特にプラットフォーム依存の絵文字フォールバックフォント等）に直接依存しないよう、`EditorConfig` のような中間構造体を定義して設定を注入します。また、`katana-ui` は単にテキストデータや設定を渡し、相互のイベント通信はコールバックや mpsc チャネルに限定して循環参照を防ぎます。
- **独自入力 surface との境界**: 本変更は `katana-editor` の crate/component boundary を作ります。egui `TextEdit` の排除、IME composition、emoji-safe rendering、grapheme hit testing は `x-x-x-native-input-surface` で扱います。

## Risks / Trade-offs

- **[Trade-off]** KatanA固有の機能追加を行う際、分離したクレートと `katana-ui` の両方を同時に修正しなければならないケースが増える。
  - **Mitigation**: このオーバーヘッドは、肥大化防止とCI/CD高速化のための「許容すべきトレードオフ」としてチームで合意済みとする。
- **[Risk]** コンポーネント分離による入力から描画までのパフォーマンス（レイテンシ）悪化。
  - **Mitigation**: 状態管理レイヤー（State）と描画レイヤー（View）を厳密に分離し、バッファ全体の不要なディープコピーを避ける（必要に応じてArc等での参照や差分同期の採用を検討する）。
- **[Risk]** 絵文字フォントのレンダリング問題（macOS/Windowsのプラットフォーム差異）。
  - **Mitigation**: `EditorConfig` を通じて正しいフォールバックフォント設定を注入する仕組みを設ける。`TextEdit` 依存を排除する根本対応は `x-x-x-native-input-surface` へ分離する。
