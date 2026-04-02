## 1. Diagnostics 契約

- [ ] 1.1 他の AI エージェントがルール対象範囲、重大度モデル、位置情報表現を誤解なく導けるように Markdown diagnostics 契約を定義する
- [ ] 1.2 app 上の diagnostics と repository lint が乖離しないよう、既存 linter 基盤の再利用または拡張方針を定義する
- [ ] 1.3 heading structure、日英 Markdown の見出し同期、broken relative links、missing local assets を含む初期の決定的ルールセットを定義する

## 2. Problems Panel とナビゲーション

- [ ] 2.1 modal に依存せず Markdown diagnostics を表示できる常設の Problems Panel を追加する
- [ ] 2.2 各 diagnostic に重大度、メッセージ、file、location を理解できるだけの情報を表示する
- [ ] 2.3 Problems Panel から diagnostic 対象を開き、位置情報へ jump できるようにする
- [ ] 2.4 diagnostic からの遷移時に editor と preview の振る舞いを揃える

## 3. 更新契機と使い勝手

- [ ] 3.1 手動実行と save 契機更新に対する diagnostics refresh 挙動を明確に定義する
- [ ] 3.2 古い file や未解決 location があっても app が crash しない回復可能な failure handling を維持する
- [ ] 3.3 Problems Panel が結果ありの状態と empty state の両方を明確に表現できるようにする

## 4. 検証

- [ ] 4.1 初期 Markdown ルールセットと期待される findings を検証するテストを追加する
- [ ] 4.2 Problems Panel の描画、empty state、選択挙動を検証するテストを追加する
- [ ] 4.3 diagnostics からの editor / preview jump 挙動を検証するテストを追加する
- [ ] 4.4 diagnostics flow が決定的で non-LLM-dependent なままであることを確認する
