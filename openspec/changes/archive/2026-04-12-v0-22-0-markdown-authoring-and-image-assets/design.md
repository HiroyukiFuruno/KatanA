## Context

現行の editor は `crates/katana-ui/src/views/panels/editor/ui.rs` の `egui::TextEdit::multiline` を中心に構成されており、source-first の plain text editor である。preview 側には `RenderedSection::LocalImage` があり local image preview の表示基盤はあるが、authoring 側には次の不足がある。

- Markdown 記法の挿入支援がない
- 画像ファイルを editor から添付する導線がない
- クリップボード画像を Markdown 資産として保存する仕組みがない
- 保存先や命名規則を settings で制御できない
- Markdown 内の local image path から workspace 内の対象ファイルへ戻る導線がない

ユーザー要望は、Markdown source を捨てることではなく、記法入力を減らしつつ画像ワークフローを一貫させることにある。そのため `v0.22.0` では、見た目そのまま編集する別 document model ではなく、Markdown source-first の authoring 強化として扱う。

## Goals / Non-Goals

**Goals:**

- Markdown source を主編集面のまま維持する
- selection / cursor を基準に Markdown 記法を挿入する authoring command を追加する
- local file attach と clipboard image paste を同じ image ingest pipeline で扱う
- 保存先 default を active Markdown file 起点の `./asset/img` に固定する
- 保存先、命名、ダイアログ表示の挙動を settings で変更できるようにする
- Markdown 内の local image reference から対象 asset の場所を辿れるようにする

**Non-Goals:**

- 見た目そのままで編集する別 editor model の導入
- remote URL image の自動ダウンロードと asset 化
- hash ベース dedupe や media library の構築
- 画像以外の任意 binary asset ingest

## Decisions

### 1. editor は Markdown source-first のまま維持する

`egui::TextEdit::multiline` を置き換えるのではなく、authoring command が現在の cursor / selection に対して Markdown 記法を挿入または整形する。

- 採用理由:
  - 現行の save / dirty buffer / preview sync 契約を壊さない
  - `v0.20.0` と `v0.21.0` の command / shortcut と自然に接続できる
- 代替案:
  - 見た目そのまま編集する別 editor を導入する: document model の二重化が必要で不採用

### 2. image ingest の default 保存先は active Markdown file 起点の `./asset/img` とする

画像保存先は workspace root ではなく active Markdown file の親ディレクトリから見た `./asset/img` を default にする。Markdown にはこの場所からの相対パスを挿入する。

- 採用理由:
  - ユーザー要望に一致する
  - 文書ごとの asset 管理が単純になる
- 代替案:
  - workspace 共通 assets ディレクトリに集約する: 文書単位の持ち運びが悪くなるため不採用

### 3. file attach と clipboard paste は同じ ingest pipeline に統一する

入力元がファイルかクリップボードかに関係なく、保存先決定、ファイル名決定、コピー、Markdown 挿入、refresh を同じ処理に通す。

- 採用理由:
  - naming / settings / test を 1 つの経路にまとめられる
  - 将来 drag-and-drop を追加しても流用しやすい
- 代替案:
  - file attach と clipboard paste を別実装にする: 挙動差分が増えるため不採用

### 4. file name default は timestamp、必要に応じて命名ダイアログを開く

default では timestamp ベースのファイル名を採用し、settings により命名ダイアログを常時表示または非表示に切り替える。

- 採用理由:
  - 最小操作で画像を挿入できる
  - 命名ポリシーを user preference に寄せられる
- 代替案:
  - 常にダイアログを出す: 画像貼り付けの速度が落ちるため不採用

### 5. local image reference の導線は「解決可能な local path のみ」を対象にする

Markdown 内の `![...](...)` のうち、active Markdown file から相対解決できる local path だけを reveal / navigate の対象にする。remote URL や存在しない path は対象外とし、missing state を明示する。

- 採用理由:
  - user expectation と実体が一致する
  - workspace 外 / remote path の誤案内を防げる
- 代替案:
  - すべての image reference に reveal を出す: 解決不能ケースが多く不採用

## Risks / Trade-offs

- **[Risk] `egui::TextEdit` の selection 制御が弱く複雑な変換を作りづらい**  
  -> Mitigation: `v0.22.0` では block-level / inline-level の基本挿入支援に絞る
- **[Risk] clipboard image 対応が platform 差分を抱える**  
  -> Mitigation: platform abstraction を切り、未対応環境では機能を disable する
- **[Risk] asset path が workspace 外を指す場合に導線が破綻する**  
  -> Mitigation: reveal 対象は local かつ解決可能な path に限定する
- **[Risk] timestamp naming が衝突や分かりづらさを生む**  
  -> Mitigation: 命名ダイアログと settings override を用意する

## Migration Plan

1. authoring command 群と selection transform を整理する
2. image ingest pipeline を追加し、file attach と clipboard paste を統合する
3. settings schema に asset 保存先 / naming policy / dialog policy を追加する
4. local image reference からの reveal / navigation を実装する
5. preview refresh と workspace refresh を image ingest 後に確実に走らせる

## Open Questions

- timestamp の具体的な format を秒単位にするかミリ秒単位にするか
- asset 保存先が存在しない場合に自動作成だけでよいか、確認ダイアログを出すか
- reveal 導線を preview 側、workspace 側、両方に出すか
