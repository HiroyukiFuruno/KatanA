## 1. Settings Recovery Flow

- [ ] 1.1 AI settings に Ollama endpoint、model、接続確認、autofix 有効状態が一貫して見える UI を整理する
- [ ] 1.2 chat / autofix から AI settings へ戻る navigation action を追加する
- [ ] 1.3 model 未選択、provider 未設定、接続失敗、timeout の表示文言を i18n に追加する

## 2. Diagnostics Autofix Entry Point

- [ ] 2.1 Problems panel の file-level autofix entry point を provider 状態に応じて表示する
- [ ] 2.2 autofix が実行できない理由を diagnostics UI 上に表示する
- [ ] 2.3 diagnostics がない file では LLM request を送信しないことをテストする

## 3. Consistent Provider State

- [ ] 3.1 chat と autofix が同じ provider availability 判定を使うように state 取得を整理する
- [ ] 3.2 Ollama availability check の pending / success / failure を UI へ反映する
- [ ] 3.3 provider unavailable 時に通常編集機能が影響を受けないことを確認する

## 4. Review and Verification

- [ ] 4.1 semantic UI tests で disabled state、settings navigation、autofix action dispatch を確認する
- [ ] 4.2 UI screenshot または操作確認結果をユーザーに提示する
- [ ] 4.3 ユーザーフィードバックを tasks.md に追記し、対応済み状態を更新する
- [ ] 4.4 `make check` と `openspec validate local-llm-ui-integration` を通す
