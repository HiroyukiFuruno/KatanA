# KatanA Project Vision

## コンセプト

**KatanA はドキュメントをレビューするに徹するためのシェル。**

ユーザーは KatanA を通してドキュメントを開き、確認し、修正提案を受け取り、承認する。KatanA 自身はドキュメントの描画・編集・AI 判断の実装詳細を知らない。これらは専門化された外部ライブラリが担い、KatanA はそれらを薄い adapter で繋ぐだけ。

---

## コンポーネントマップ

```
KatanA（シェル）
  ├─ katana-document-preview   Preview 描画（Markdown / 画像 / PDF / Draw.io / Word / Excel / PPT / CSV...）
  ├─ katana-language-editor    テキスト編集 widget（言語非依存。KatanA は Markdown highlighter を注入）
  ├─ katana-chat-ui            Chat サイドパネル + autofix diff surface（ACP / Ollama）
  └─ katana-canvas-forge (kcf) 図表描画バックエンド（Mermaid / Draw.io 等。egui 非依存）
```

### 依存方向の原則

- KatanA → 外部ライブラリ（一方向のみ）
- 外部ライブラリ同士は原則無依存（kcf → katana-document-preview は可）
- 外部ライブラリは LLM / ACP に依存しない（katana-chat-ui のみ例外）

---

## neutral interface パターン

各外部ライブラリは 2-crate 構成を取る：

```
katana-{name}          neutral interface（egui 非依存）← KatanA はここだけに依存
katana-{name}-egui     egui 実装（MVP）
```

KatanA が依存する neutral interface が変わらない限り、egui 実装を差し替えても KatanA 側の変更はゼロ。

---

## egui 採用の位置付けと制約

egui は MVP 検証用として採用。検証速度とエコシステム統合を優先した選択であり、プロダクション品質の UI フレームワークとしての採用ではない。

### 既知の egui 制約（ドキュメントレビューツールとして致命的なもの）

| 制約 | 内容 |
|------|------|
| カラー絵文字非対応 | epaint の独自フォントアトラスが OS フォントフォールバックチェーンを無視する。SBIX/CBTF フォーマットのカラー絵文字（Apple Color Emoji 等）を正しく描画できない。エディタ・Preview 両面で致命的。 |
| immediate mode の再描画コスト | 毎フレーム全体を再描画するため、大きいドキュメントや複数 Preview 表示時に CPU 消費が増大する。 |
| IME との相性 | 日本語・中国語等の IME composition が egui TextEdit で不完全。確定前のインライン表示が正しく動作しないケースがある。 |
| ネイティブ挙動との乖離 | スクロール慣性なし、システムフォント非一致、OS ネイティブなドラッグ操作の制限など、macOS ユーザーの期待と乖離する。 |

---

## 独自 UI フレームワーク化の計画

Zed（GPUI）を参考に、将来 KatanA 向けの独自 UI フレームワークを開発する。上記 egui 制約はすべてこれで解決できる。

### 移行コストの見積もり

| 範囲 | 移行コスト | 理由 |
|------|-----------|------|
| `katana-{name}`（neutral interface crate）| ゼロ | egui 非依存。そのまま再利用 |
| `katana-core` / `katana-platform` | ゼロ | UI 非依存 |
| `features/*`（v1.0.1 で整理予定）| ゼロ | pure Rust、egui ゼロ |
| `katana-{name}-egui`（impl crate）| 中 | 独自フレームワーク版 impl に置き換えるだけ |
| `views/` / `widgets/`（katana-ui）| 大 | egui API 直接使用。書き直しが必要 |
| `shell/` / `frame/`（katana-ui）| 大 | eframe ライフサイクル依存 |

**neutral interface と `features/*` を先に固めることが、egui 完全置換の前提条件。**

### 移行ロードマップ

```
v0.26.0  katana-document-preview 分離
v0.27.0  katana-language-editor 分離
v0.28.0  katana-chat-ui migration
v1.0.1   features/* の egui ゼロ化・AppAction/AppState 境界整理
v1.x     独自 UI フレームワーク開発（views/ + widgets/ + shell/ 書き換え）
         → katana-{name}-egui を独自フレームワーク版 impl に順次置き換え
```

---

## KatanA が「知るべきこと」と「知るべきでないこと」

### 知るべきこと（KatanA の責務）

- どのファイルを開いているか
- どのパネルを表示しているか（Preview / Editor / Chat / Diagnostics）
- ユーザーのナビゲーション操作
- 外部ライブラリへの設定の受け渡し

### 知るべきでないこと（外部ライブラリの責務）

- Markdown / PDF / Draw.io の描画方法
- シンタックスハイライトの実装
- LLM との通信・プロトコル
- 図表の render 実装
- 絵文字フォント管理
