## Context

KatanA にはすでに、markdownlint diagnostics の Problems Panel、`AppAction::IngestImageFile` / `IngestClipboardImage`、preview 側の local image rendering、tab reorder、TOC パネルといった基盤がある。一方で、これらは別々に増えてきたため、user から見ると次の断絶が残っている。

- diagnostics は rule coverage と Problems Panel 中心で、editor 上の「どこが悪いか」「その場で何ができるか」が弱い
- image ingest は command / context menu 側に偏っており、GUI 操作や drag-and-drop と統一されていない
- workspace は directory open 前提で、単一 file を一時的に扱う flow がない
- explorer / tab / TOC はそれぞれ独立に改善されてきたが、authoring workflow の一部としては連動していない

`tmp/対応したい改善/memo.md` の要求は、個別の小修正よりも「Markdown を書く、lint を直す、asset を貼る、file を開く、構造を辿る」という一連の流れを再設計する内容である。そのため、本 change では source-first editor と既存 workspace shell を維持したまま、surface contract と操作導線を整理し直す。

## Goals / Non-Goals

**Goals:**

- markdownlint diagnostics を Problems Panel と editor inline surface の両方で一貫して扱う
- toolbar / context menu / shortcut / drag-and-drop が同じ authoring / ingest contract を共有する
- image ingest の保存先、命名、挿入位置、refresh 動作を 1 つの source of truth に揃える
- 単一 file open と explorer drag-and-drop を、既存 workspace shell を壊さず追加する
- TOC を長文向けの navigation widget として再構成し、見た目と状態管理を整理する

**Non-Goals:**

- WYSIWYG editor や rich-text document model への置き換え
- markdownlint CLI / Node.js runtime の同梱
- 任意 binary asset 全般の asset manager 化
- Finder / Explorer 相当の完全な file manager を KatanA 内に実装すること
- TOC の document outline source 自体を別 parser に差し替えること

## Decisions

### 1. diagnostics payload を単一の source of truth にし、Problems Panel と inline editor surface を派生表示に統一する

rule coverage、editor underline、hover popup、quick-fix entry は別々に判定せず、保存済み diagnostics payload を共通で参照する。inline surface は payload 内の file path / location / metadata を view へ投影するだけに留める。

- 採用理由:
  - rule coverage と editor decoration の不整合を避けられる
  - `v0.23.0` の local LLM autofix とも payload contract を共有できる
- 代替案:
  - editor 用に別 linter / 別 location 計算を持つ: surface ごとに結果がズレるため不採用

### 2. quick-fix は「安全な deterministic fix provider がある rule」に限定し、hover popup から起動する

hover popup に fix button を出すのは、局所書き換えで安全性を担保できる rule のみに限定する。provider がない rule は explanation と docs link のみを出し、fix button を常時表示しない。

- 採用理由:
  - 「簡単な警告だけ自動修正したい」という要求に合う
  - LLM 依存なしで動く quick fix と、`v0.23.0` の local LLM autofix を責務分離できる
- 代替案:
  - すべての diagnostic に fix button を出す: 失敗率と期待値のズレが大きいため不採用

### 3. image ingest は file attach / clipboard paste / external image drop を同一 pipeline に統一し、`./asset/img` を正規 default とする

入力元に関係なく、保存先解決、命名、directory 作成、Markdown 挿入、preview / explorer refresh を同じ ingest pipeline に通す。保存先 default は既存実装と archive 設計に揃えて active Markdown file 親基準の `./asset/img` とし、挿入位置は cursor / selection 優先、未確定時は文書末尾とする。

- 採用理由:
  - 現在の `DEFAULT_IMAGE_SAVE_DIRECTORY` と整合し、既存ユーザー設定を壊しにくい
  - drag-and-drop を追加しても file attach / clipboard paste と挙動差分が出にくい
- 代替案:
  - `assets/img` へ名称変更する: 既存設定値・実装・archive とズレるため今回は不採用

### 4. explorer thumbnail は「Markdown から参照されている local image」に限定し、tree 描画後の lazy hydration で更新する

explorer の image row を初回から同期読込せず、workspace load 後は通常 row を先に描画し、その後に reference 判定済み image asset だけ thumbnail queue へ積む。thumbnail cache key は absolute path と file mtime / size を含む。

- 採用理由:
  - memo の「後追い表示で性能劣化を避ける」という要求に合う
  - 全 image file を対象にしないため、巨大 workspace でも負荷を限定できる
- 代替案:
  - explorer 内の全 image file に即時 thumbnail を出す: 初回描画と I/O が重くなり不採用

### 5. 単一 file open は「temporary workspace」と「current workspace session open」を分ける

single file open には 2 つの明示モードを持たせる。temporary workspace は synthetic label と system icon を持つ一時的 shell context とし、global workspace history / persisted list には保存しない。current workspace session open は現在の shell context を維持したまま tab として開き、workspace root 自体は置き換えない。

- 採用理由:
  - 「一時ワークスペース」と「今の作業文脈で開く」を user に明確に分けられる
  - global workspace の永続構造が directory 前提のままでも導入できる
- 代替案:
  - 単一 file open のたびに workspace root を file 親 directory へ差し替える: 履歴と user expectation を壊しやすく不採用

### 6. explorer からの drag-and-drop は「意図ゾーン」で判定する

explorer から tab strip へ drop した場合、tab 間インジケーター近傍は「指定位置に temporary tab 挿入」、strip の余白や末尾側は「末尾に追加して active 化」として扱う。explorer 内 move は同一 workspace root 内の file / directory move に限定し、confirmation setting は default `true` とする。

- 採用理由:
  - memo の「雑に放り投げれば末尾」「精密に置けば位置指定」という要求を UI で表現しやすい
  - move と open を gesture の違いで分離しやすい
- 代替案:
  - すべて precise insert 扱いにする: casual 操作の失敗率が高くなるため不採用

### 7. TOC は heading list を保ったまま presentation だけ accordion 化し、active state は text emphasis に寄せる

現在の TOC データ源はそのまま使い、各見出し row の描画を filled button から accordion row へ置き換える。default 展開状態は all open、panel header に expand all / collapse all icon を置き、guide line は layout setting として永続化する。

- 採用理由:
  - parser / scroll sync を壊さず、見た目と操作だけ改善できる
  - filled background を外しても active section は typography と line guide で示せる
- 代替案:
  - TOC データ構造ごと tree widget へ全面移行する: 既存 scroll sync 回帰の範囲が大きく不採用

### 8. 新しい user preference は既存 settings domain に収める

diagnostic decoration color は theme settings、TOC guide line visibility は layout settings、explorer move confirmation は workspace / behavior settings、image ingest naming / save rule は ingest settings に収める。新規の独立設定ファイルや別 repository は作らない。

- 採用理由:
  - 既存の settings save / restore contract を再利用できる
  - user が設定箇所を推測しやすい
- 代替案:
  - feature ごとに断片 settings を追加する: 設定探索性が下がるため不採用

## Risks / Trade-offs

- **[Risk] diagnostics の line/column と editor の char-range がズレる**
  -> Mitigation: buffer hash or diagnostics revision が不一致なら stale decoration を描画しない
- **[Risk] explorer thumbnail queue が workspace load 後に I/O を偏らせる**
  -> Mitigation: viewport-first queue、small LRU cache、decode concurrency 制限を入れる
- **[Risk] temporary workspace が通常 workspace history と混同される**
  -> Mitigation: synthetic label / icon / non-persisted contract を明示する
- **[Risk] drag-and-drop の意図判定が曖昧で誤操作になる**
  -> Mitigation: drop indicator と hover affordance を入れ、move は confirmation default-on とする
- **[Risk] quick-fix provider が誤修正を出す**
  -> Mitigation: whitelist rule のみ対象にし、適用前 preview or immediate local diff を提示する

## Migration Plan

1. diagnostics / settings / ingest / workspace の追加 contract を spec に確定する
2. diagnostics payload から inline surface と quick-fix provider registry を接続する
3. toolbar と unified image ingest pipeline を実装し、clipboard / file / external image drop を統合する
4. temporary workspace と file DnD open / move を追加し、tab / explorer DnD indicator を拡張する
5. TOC accordion と guide line settings を実装し、関連 UI regression test を追加する

## Open Questions

- deterministic quick-fix を `v0.26.0` でどの rule まで含めるか
- explorer thumbnail の reference index を workspace 全体で先読みするか、active document 起点で段階拡張するか
- current workspace session open で workspace 外 file を開いた場合の reveal / breadcrumbs 表現をどこまで持たせるか
