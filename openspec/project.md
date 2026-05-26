# KatanA Project Vision

## プロダクトの思想

KatanA とは何か、何でないか——これを明確にすることがこのプロジェクトの出発点である。

### KatanA が解く問題

現代の IDE（VS Code / Zed 等）はコードを「書く」ことに最適化されている。ドキュメントの「レビュー」と「生成」は開発作業と混在しており、閲覧・確認・承認に特化した体験が存在しない。

具体的な課題：

- Markdown / PDF / Word / Excel / Draw.io を「レビューする」ための閲覧機能が IDE では不足している
- AI による提案を「確認して承認する」ワークフローが first-class になっていない
- 開発作業とドキュメント作業が同じ画面・同じ操作系に混在し、集中を妨げる

### KatanA の答え

**KatanA は実装支援ツールである。ドキュメントのレビューと生成に特化する。**

- ドキュメントを開き、読み、確認し、AI 提案を承認する——この体験に徹する
- 統合 IDE を目指すことは「やろうと思えばできる」が、意図的にしない
- 刀の思想：余計なものを削ぎ落とし、一点に力を集中させる

### 競合との違い

| | VS Code / Zed | KatanA |
|--|--------------|--------|
| 主たるユーザー | コードを書く開発者 | ドキュメントをレビューする人 |
| 最適化の軸 | 編集速度・LSP・Git | 閲覧品質・マルチフォーマット・AI 承認フロー |
| ドキュメント体験 | 開発の付随機能 | first-class の主機能 |
| AI の位置付け | コード補完の補助 | レビュー・生成の主役 |

Zed と張り合っていない。Zed が狙っていない「レビュアー」という領域を独自に切り開く。

---

## コンセプト

**KatanA はドキュメントのレビューに徹するマルチプラットフォームデスクトップアプリ。**

ユーザーは KatanA を通してドキュメントを開き、確認し、修正提案を受け取り、承認する。KatanA 自身はドキュメントの描画・編集・AI 判断の実装詳細を知らない。これらは専門化された外部ライブラリが担い、KatanA はそれらを薄い adapter で繋ぐ application host として振る舞う。

---

## コンポーネントマップ

```text
KatanA（シェル）
  ├─ katana-document-viewer       Viewer / export（Markdown / HTML / PDF / PNG / JPG / 画像 / PDF / Office / CSV...）
  ├─ katana-language-editor       テキスト編集 widget（言語非依存。KatanA は Markdown highlighter を注入）
  ├─ katana-chat-ui               Chat サイドパネル + autofix diff surface（ACP / Ollama）
  └─ katana-render-runtime (krr) 外部描画 runtime（Mermaid / Draw.io / PlantUML / math 等。egui 非依存）
```

### 依存方向の原則

- KatanA → 外部ライブラリ（一方向のみ）
- KDV は viewer / export pipeline を持ち、必要に応じて KRR の外部描画結果を利用してよいが、editor-viewer 同期制御は持たない。
- 外部ライブラリは LLM / ACP に依存しない（katana-chat-ui のみ例外）
- editor-viewer同期制御はKatanAだけが持つ。KatanAがviewerまたはeditorへ命令する。

---

## neutral interface パターン

各外部ライブラリは 2-crate 構成を取る：

```text
katana-{name}          neutral interface（egui 非依存）← KatanA はここだけに依存
katana-{name}-egui     egui 実装（MVP）
```

KatanA が依存する neutral interface が変わらない限り、egui 実装を差し替えても KatanA 側の変更はゼロ。

---

## egui 採用の位置付けと制約

egui は MVP 検証用として採用。検証速度とエコシステム統合を優先した選択であり、プロダクション品質の UI フレームワークとしての採用ではない。

### 既知の egui 制約（ドキュメントレビューツールとして致命的なもの）

| 制約 | 内容 |
| --- | --- |
| カラー絵文字非対応 | epaint の独自フォントアトラスが OS フォントフォールバックチェーンを無視する。SBIX/CBTF フォーマットのカラー絵文字（Apple Color Emoji 等）を正しく描画できない。エディタ・viewer 両面で致命的。 |
| immediate mode の再描画コスト | 毎フレーム全体を再描画するため、大きいドキュメントや複数viewer表示時に CPU 消費が増大する。 |
| IME との相性 | 日本語・中国語等の IME composition が egui TextEdit で不完全。確定前のインライン表示が正しく動作しないケースがある。 |
| ネイティブ挙動との乖離 | スクロール慣性なし、システムフォント非一致、OS ネイティブなドラッグ操作の制限など、macOS ユーザーの期待と乖離する。 |

---

## 独自 UI フレームワーク化の計画

Zed（GPUI）を参考に、将来 KatanA 向けの独自 UI フレームワークを開発する。上記 egui 制約はすべてこれで解決できる。

### 移行コストの見積もり

| 範囲 | 移行コスト | 理由 |
| --- | --- | --- |
| `katana-{name}`（neutral interface crate） | ゼロ | egui 非依存。そのまま再利用 |
| `katana-core` / `katana-platform` | ゼロ | UI 非依存 |
| `features/*`（v1.0.1 で整理予定） | ゼロ | pure Rust、egui ゼロ |
| `katana-{name}-egui`（impl crate） | 中 | 独自フレームワーク版 impl に置き換えるだけ |
| `views/` / `widgets/`（katana-ui） | 大 | egui API 直接使用。書き直しが必要 |
| `shell/` / `frame/`（katana-ui） | 大 | eframe ライフサイクル依存 |

**neutral interface と `features/*` を先に固めることが、egui 完全置換の前提条件。**

### 移行ロードマップ

```text
v0.23.0  katana-chat-ui v0.0.1 intake（ACP foundation・neutral interface）
v0.24.0  katana-chat-ui v0.1.0 intake（chat panel widget + autofix diff surface）
v0.25.0  katana-chat-ui future intake（document generation + translation overlay）

version なし  katana-render-runtime intake（外部描画 runtime 責務の分離）
version なし  katana-document-viewer rename / KMM DTO viewer + export ownership 整理
v0.22.26  katana-document-viewer v0.1.0 intake（kcf 依存を廃止し、preview / export を KDV 境界へ移行）
v0.22.27  katana-document-viewer v0.1.1 + katana-render-runtime v0.3.3 intake（KDR wrapper 直接依存を廃止し、preview / export を KDV + KRR 境界へ移行）

──── Floem 移行（egui 制約解消）────
v0.26.0  Floem Phase 1 intake = editor + chat input（IME / カラー絵文字解消・最優先）
v0.27.0  Floem Phase 2 intake = viewer（vello retained 描画）
v0.28.0  Floem Phase 3 intake = chrome（toolbar / sidebar / split pane / window loop）+ egui / eframe ゼロ

──── post-Floem ────
v0.29.0  preview-driven local editing（review-first 編集体験）
v1.0.0   desktop viewer polish（公式リリース）
v1.0.1   internal refactoring（katana-ui 整理 + i18n formatter 抽象化）
```

各 phase は独立した release。検証範囲を狭めるため editor / viewer / chrome を別 release として段階的に切り替える。

---

## KatanA が「知るべきこと」と「知るべきでないこと」

### 知るべきこと（KatanA の責務）

- どのファイルを開いているか
- どのパネルを表示しているか（Viewer / Editor / Chat / Diagnostics）
- ユーザーのナビゲーション操作
- 外部ライブラリへの設定の受け渡し
- macOS / Windows / Linux のプラットフォーム差異の吸収（ウィンドウ・ファイルシステム・OS API）

### 知るべきでないこと（外部ライブラリの責務）

- Markdown / PDF / Draw.io の描画方法
- シンタックスハイライトの実装
- LLM との通信・プロトコル
- 図表の render 実装
- 絵文字フォント管理

---

## UI フレームワーク移行方針（egui → Floem）

### なぜ egui を脱却するか

KatanA は egui を MVP 検証用として採用したが、以下の課題がプロダクション品質のドキュメントレビューツールとして致命的であることが実証された：

| 課題 | 内容 |
| --- | --- |
| カラー絵文字非対応 | epaint の独自フォントアトラスが OS フォントフォールバックチェーンを無視。Apple Color Emoji（SBIX/CBTF）が描画できない。エディタ入力・chat 入力・viewer の全面で欠落する |
| IME 不完全 | egui TextEdit の IME composition が不完全。日本語入力の確定前インライン表示が正しく動作しない。chat 入力・editor 入力の両方で発生する |
| レイアウト拡張不可 | egui_commonmark がレイアウト変更の拡張点を持たず、viewer の行間・マージン調整のために vendor/ にライブラリ本体を持ち込むことになった。upstream 更新を取り込めなくなる破壊的負債 |
| immediate mode の再描画コスト | 毎フレーム全体を再描画するため、大きいドキュメントや複数viewer表示時に CPU 消費が増大する |

これらは egui のアーキテクチャに起因する根本的な制約であり、workaround では解決できない。

### egui MVP の価値

egui で動く実装を作ったことで以下が確定した：

- 必要な UX・レイアウトの形が実証済み
- vendor パッチが必要になる箇所が特定済み
- IME・絵文字の問題が具体的な痛みとして明確化
- neutral interface 設計の判断材料が揃った

**egui MVP は仕様確定のコストであり、設計の失敗ではない。** 問題は「neutral interface なしで直接実装を積み重ねた」ことであり、今回の外部ライブラリ分離でその負債を返している。

### 採用技術スタック

| 層 | 採用 | 理由 |
| --- | --- | --- |
| **UI フレームワーク** | Floem | Rust 純正・クロスプラットフォーム（macOS/Linux/Windows）・vello+cosmic-text+taffy+winit を内包。拡張点が open で vendor パッチ不要 |
| **文字描画・shaping** | cosmic-text | HarfBuzz ベース。システムフォント直アクセス・カラー絵文字（SBIX/CBTF）・IME composition 完全対応 |
| **2D レンダリング** | vello（wgpu） | compute-shader ベース 2D 描画。Metal/DX12/Vulkan で KatanA 固有の描画要件を vendor パッチなしで実現できる |
| **レイアウト** | taffy | flexbox + CSS Grid。Floem が内部で使用。split pane・sidebar が自然に書ける |
| **アーキテクチャ参考** | GPUI（Zed） | Entity モデル・Context パターン・非同期タスク設計・テキストエディタコアの設計を参考書として活用 |

**Rust 純正のみ採用。React / TypeScript / WebView は使用しない。**

### Floem をベース、GPUI/Zed を参考書として使う判断根拠

- **Floem**：wgpu ベースでクロスプラットフォームが保証されている。GPUI は Metal ファーストで Linux/Windows は後発
- **GPUI/Zed**：Zed のソースコードは公開されており、エディタ・ドキュメント系アプリの実装参考として最高の教材。Floem で解けない問題は Zed 実装を解析して同じ問題を Floem 上で解く。どうしても Floem で実現できない場合は GPUI の該当 crate を直接依存することも検討する

### impl crate の移行対応

各外部ライブラリは `-egui` impl を `-floem` impl に差し替える：

```text
katana-language-editor-egui   →   katana-language-editor-floem
katana-chat-ui-egui           →   katana-chat-ui-floem
katana-document-viewer-floem  ←   katana-document-preview から改名して採用
KatanA chrome（views/widgets）→   Floem views（taffy layout）
```

KatanA の `Cargo.toml` の impl crate 行を変えるだけ。neutral interface は全フェーズを通して変わらない。

### 移行フェーズ（KatanA リリースとの対応）

```text
Phase 1  入力サーフェス（最優先）= v0.26.0
  katana-language-editor-floem   ← editor input（IME・絵文字解決）
  katana-chat-ui-floem           ← chat input（IME・絵文字解決）
  eframe は paint_callback 経由で共存可能。段階移行できる

Phase 2  viewer 層 = v0.27.0
  katana-document-viewer-floem  ← vello で retained 描画
  PDF / 画像 / 図表も同じ surface で統一
  vendor/egui_commonmark/ パッチをここで除去

Phase 3  chrome 層（最後）= v0.28.0
  toolbar / sidebar / split pane / window loop を taffy + vello で実装
  eframe / egui / egui_extras / egui_* を完全除去。winit を直接使う
```

各 Phase は KatanA の独立 release として段階的に切り替える。検証範囲を狭めるため、editor + chat input → viewer → chrome の順で 3 release に分割している。

---

## KatanA リポジトリの責務（確定）

```text
katana-xxx repos    各ライブラリの実装 + 単体テスト（UT）
         ↓ git dependency として intake
KatanA              組み上げ（assembly）+ 統合テスト（IT）のみ
```

**KatanA に実装コードが増えたらアーキテクチャの失敗というシグナル。**

KatanA が担うのは：

- 各ライブラリを git dependency として intake する
- `EditorConfig` / `ViewerConfig` / `ChatConfig` 等を組み立てて各 widget に渡す
- ウィンドウ・ライフサイクル・プラットフォーム差異の吸収
- 統合テスト（各ライブラリが組み合わさって正しく動くことの検証）

KatanA が担わないのは：

- 文字描画の実装
- Markdown / PDF / 図表のレンダリング実装
- LLM / ACP の通信実装
- シンタックスハイライトの実装
