# Tasks

## Definition of Ready

- [ ] この change は特定 version に割り当てない調査バックログとして扱う。
- [ ] v0.22.7 のガントチャート修正とは分離し、リリース前の追加修正範囲に含めない。
- [ ] 調査結果を後続の versioned change へ移送できる粒度で記録する。

## 0. task.mdの更新

- [ ] 0.1 .codex/skills/openspec-tasks-template/SKILL.mdを利用して本task.mdをtemplate通りにアップデートする。

## 1. mmdc baseline 調査

- [ ] 1.1 旧 KatanA 実装が `mmdc` に渡していた引数、背景色、テーマ、出力形式を確認する。
- [ ] 1.2 `mmdc` の既定 viewport / output size を実測し、固定値なのか入力依存なのかを整理する。
- [ ] 1.3 ガントチャートの赤い「今日」線が `mmdc` 出力でどのように扱われていたかを記録する。
- [ ] 1.4 Mermaid CLI / Puppeteer / Mermaid.js のバージョン差で結果が変わる可能性を記録する。

## 2. Mermaid.js renderer 調査

- [ ] 2.1 現行 Mermaid.js 経路の viewport、container 幅、SVG `viewBox`、PNG capture 対象を確認する。
- [ ] 2.2 キャッシュキーに含めるべき描画条件を確認する。
- [ ] 2.3 図形ごとに、親要素の幅を拾うか、内容幅で描画されるかを確認する。
- [ ] 2.4 Mermaid 初期化設定で解決できる差分と、SVG 後処理が必要な差分を分離する。

## 3. Fixture と証跡作成

- [ ] 3.1 flowchart / sequence / class / state / entity relationship / gantt / pie / journey / mindmap / timeline の fixture を用意する。
- [ ] 3.2 各 fixture について、`mmdc` baseline と Mermaid.js output の画像証跡を生成する。
- [ ] 3.3 証跡の出力先は `.gitignore` 対象に置き、fixture と生成手順だけを git 管理対象にする。
- [ ] 3.4 `scripts/screenshot` で確認できるものは、ユーザーが手動操作しなくても確認できるシナリオにする。

## 4. 差分分類と後続計画

- [ ] 4.1 差分を layout / size / theme / typography / marker / interaction / error handling / cache behavior に分類する。
- [ ] 4.2 即時修正、versioned change 化、許容差分として文書化、のいずれかに振り分ける。
- [ ] 4.3 SVG 後処理が必要な候補は、対象図形と対象要素を限定して設計する。
- [ ] 4.4 後続 versioned change を作成する場合は、本 change の調査結果を参照元として明記する。

## 5. Completion

- [ ] 5.1 調査結果を `tmp/` または `openspec/changes/mermaid-rendering-compatibility-investigation/` 配下の文書に整理する。
- [ ] 5.2 OpenSpec を検証し、schema と要求文の形式が崩れていないことを確認する。
- [ ] 5.3 この change を実装完了として扱わず、後続対応の入力として残す。
