## 1. Runtime Fallback

- [x] 1.1 `I18nOps` に fallback-aware language resolver を追加する
- [x] 1.2 user settings 由来の未知 language code で panic しないように dictionary lookup を修正する
- [x] 1.3 embedded locale corruption は fail fast のまま維持するテストを追加する

## 2. Formatter Adapter

- [ ] 2.1 message id と typed named arguments を受け取る formatter adapter を定義する
- [ ] 2.2 現行 `I18nOps::tf` を adapter 経由へ移し、互換 API として維持する
- [ ] 2.3 missing parameter、extra parameter、escaped brace の unit test を追加する

## 3. Plural Candidate Selection

- [ ] 3.1 count、problem total、file count、search result count を含む message を棚卸しする
- [ ] 3.2 Fluent / ICU 候補を adapter 内で spike し、binary size と startup cost を記録する
- [ ] 3.3 採用 engine または延期理由を design.md に追記する
- [ ] 3.4 英語と非英語1言語以上で plural formatting test を追加する

## 4. Locale Quality Gate

- [ ] 4.1 current locale files の missing key check を追加する
- [ ] 4.2 pseudo-translation / fallback marker check を追加する
- [ ] 4.3 formatter message key が全 supported locale に存在することを検査する
- [ ] 4.4 `make check` と `openspec validate i18n-runtime-safety` を通す
