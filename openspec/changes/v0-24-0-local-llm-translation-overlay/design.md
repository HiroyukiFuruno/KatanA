## Context

KatanA には locale JSON ベースの既存 i18n があり、静的 UI 文言はその仕組みで翻訳される。一方で、`v0.18.0` の markdownlint English description、`v0.22.0` と `v0.23.0` の local LLM result など、実行時に生成または外部由来で入る英語 text は locale JSON の外にある。

ユーザー要望は「local LLM がオンなら自動表示が望ましい」であり、ここで既存 i18n を置き換える必要はない。`v0.24.0` では、static UI は既存 i18n のまま維持し、dynamic / external English text にだけ translation overlay をかける。

## Goals / Non-Goals

**Goals:**

- local LLM が有効で、app language が英語以外のときに eligible text を自動翻訳表示する
- original English text を失わずに translated view を提供する
- translation failure 時は原文へ安全に fallback する
- eligible target を inventory で管理し、適用範囲を明示する
- 翻訳済み text や overlay 自身を再翻訳しない

**Non-Goals:**

- static locale JSON を local LLM 翻訳で置き換えること
- user authored Markdown 全文の自動翻訳
- network translation service の追加
- 文脈のない短文を過剰に翻訳して UI すべてを変えること

## Decisions

### 1. translation overlay は dynamic / external English text に限定する

既存の static UI は locale JSON を source of truth とし、その外から来る英語 text だけを overlay 対象にする。

- 採用理由:
  - 既存 i18n contract を壊さない
  - 翻訳品質の責任範囲を限定できる
- 代替案:
  - UI 全文言を local LLM で再翻訳する: deterministic さを失うため不採用

### 2. auto translate は local LLM enabled + non-English UI language を条件にする

ユーザー要望どおり自動表示を基本とするが、local LLM が無効なときや UI language が英語のときは overlay を起動しない。

- 採用理由:
  - 不要な翻訳コストを防げる
  - 英語 user に余計な変換をかけない
- 代替案:
  - 常に翻訳ボタンだけ出す: 要望の自動表示に合わないため不採用

### 3. original English text は常に参照できるようにする

translation のみを表示すると、誤訳時に原文を確認できない。したがって overlay は translated view を重ねつつ、original English text を参照できる状態を保つ。

- 採用理由:
  - 誤訳や専門用語の確認がしやすい
  - diagnostics や AI result の信頼性を保てる
- 代替案:
  - 翻訳のみ表示する: 監査性が落ちるため不採用

### 4. translation cache は source text と target language と provider context で持つ

同じ text を何度も翻訳すると遅い。source text、target language、provider / model context を key に cache し、繰り返し表示時の latency を下げる。

- 採用理由:
  - auto translate でも UX を保ちやすい
  - provider 切替時の混線を防げる
- 代替案:
  - no cache: 表示のたびに翻訳が走り不安定になるため不採用

### 5. translation overlay 自身は translation target に含めない

overlay によって生成した translated text や、既に非英語と判定できる text を再び translation pipeline に流すと、二重翻訳や無限再評価に近い挙動が起きる。したがって eligibility rule には「overlay generated」「non-English source」「translation in progress」を除外条件として持たせる。

- 採用理由:
  - 二重翻訳と無駄な provider call を防げる
  - original English text を source of truth として保てる
- 代替案:
  - UI render のたびに全 text を再評価する: loop とコスト増を招くため不採用

## Risks / Trade-offs

- **[Risk] 自動翻訳で初回表示が遅くなる**  
  -> Mitigation: cache と lazy hydration を組み合わせる
- **[Risk] 文脈不足で誤訳が起きる**  
  -> Mitigation: original English text を常に参照可能にする
- **[Risk] target inventory が漏れる**  
  -> Mitigation: release 時点で inventory を文書化し、実装時に再点検する
- **[Risk] overlay が自分自身を再翻訳して二重表示やループを起こす**  
  -> Mitigation: eligibility rule に overlay generated / translation in progress / non-English source の除外条件を入れる

## Migration Plan

1. dynamic / external English target の inventory を作成する
2. eligibility rule と translation request / cache contract を定義する
3. local provider を translation path に接続する
4. diagnostics、AI result など対象 UI に overlay を追加する
5. fallback、cache invalidation、original text reveal を整備する

## Open Questions

- translation cache の保存先を memory のみにするか、session をまたいで保持するか
- original English text の見せ方を tooltip、expand、side-by-side のどれにするか
