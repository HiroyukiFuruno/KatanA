## Context

`crates/katana-ui/src/i18n/logic.rs` には、dictionary mismatch や未対応 language code に対する `expect` / `panic!` path が残っている。embedded locale の構造破損は開発時に失敗させてよいが、user settings や OS locale 由来の runtime value でアプリが起動不能になるのは避ける必要がある。

また、現行の `I18nOps::tf` は `{key}` の単純置換であり、plural、missing parameter、escaped brace、locale-specific formatting を表現しにくい。

## Goals / Non-Goals

**Goals:**

- user-provided language value で panic しない fallback を実装する。
- message formatting の責務を adapter に閉じる。
- 既存 JSON locale を急に全面移行しない。
- plural-sensitive message の移行候補を棚卸しする。
- locale quality check を CI または lint で検知できるようにする。

**Non-Goals:**

- 全 locale file を一括で Fluent / ICU format に変換すること。
- 翻訳文の全面見直し。
- LLM translation overlay の実装。

## Decisions

### 1. Runtime fallback と embedded corruption を分ける

未知の user language は fallback language へ倒す。一方、embedded JSON の parse error や schema mismatch は開発・ビルド品質の問題として fail fast を維持する。

### 2. Formatter adapter を先に作る

Fluent / ICU を直接 UI call site に漏らさず、KatanA-owned formatter input と typed named arguments を定義する。初期実装は現行 JSON string を adapter 内で扱ってよい。

### 3. Plural migration は小さく始める

最初は problem count、search result count、file count など、数値に依存する message を棚卸しして代表例だけ移す。全翻訳の一括変換は scope に含めない。

### 4. Locale checks は current data を見る

古い PR や古いメモの翻訳指摘ではなく、現在の locale JSON を検査する。既に自然翻訳済みの値を stale finding として再オープンしない。

## Risks / Trade-offs

- [Risk] string key 化で型安全性が落ちる → generated key または typed wrapper を用意する。
- [Risk] ICU / Fluent の依存が重い → adapter 内で評価し、binary size と startup cost を task の合格条件に含める。
- [Risk] locale diff が大きくなる → 最初は high-value message に限定する。

## Migration Plan

1. fallback-aware language resolver を追加する。
2. `I18nOps::tf` を formatter adapter 経由へ移す。
3. plural-sensitive message を棚卸しする。
4. Fluent / ICU 候補を adapter 内で spike する。
5. locale quality checks を追加する。

## Open Questions

- fallback language は常に English に固定するか、settings default として持たせるか。
- formatter message id を nested struct field と対応させるか、独立 key にするか。
