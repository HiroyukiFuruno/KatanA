## Context

`crates/katana-core/src/ai/mod.rs` には `AiProvider` trait と `AiProviderRegistry` があり、provider abstraction 自体は存在する。しかし現在は次の欠落がある。

- user が active provider を設定・永続化する仕組みがない
- local endpoint provider を登録する adapter がない
- model 一覧取得や利用可否確認の導線がない
- diagnostics から autofix request を組み立てる層がない
- AI 未設定時の disabled state はあるが、UI workflow はない

ユーザー要望は「ユーザーが選べる local LLM」「軽くて容量が低いものが良い」であり、最初のスコープは lint 自動修正までである。したがって `v0.22.0` は bundled runtime ではなく local endpoint integration を前提とし、provider 選択と markdownlint autofix を同時に成立させる。

## Goals / Non-Goals

**Goals:**

- user が local LLM provider を設定、切り替え、検証できる
- `Ollama`、`LM Studio`、OpenAI 互換 local endpoint を選択肢として提供する
- provider 未設定でも app 本体の利用性を維持する
- markdownlint diagnostics を入力にした autofix 候補生成を実装する
- autofix は preview / 確認の上で適用できるようにする

**Non-Goals:**

- model runtime や model file のアプリ同梱
- document generation の実装
- translation overlay の実装
- remote commercial provider を `v0.22.0` の主対象にすること

## Decisions

### 1. local LLM は endpoint integration を前提にする

runtime bundling ではなく、local server / local endpoint を provider adapter として扱う。`LM Studio` は OpenAI 互換 preset、`Ollama` は専用 adapter、その他は OpenAI 互換 local endpoint として扱う。
このアダプター層にはプロセスの生死監視（KatanA とのライフサイクル連携）および VRAM/メモリ不足に伴う推論タイムアウト・フォールバック機構を持たせ、バックエンドの不調が本体 UI をフリーズさせないよう隔離する。

- 採用理由:
  - 既存の `AiProvider` abstraction に素直に乗る
  - cross-platform packaging を増やしにくい
  - バックエンドプロセスの不安定さをサンドボックス化できる
- 代替案:
  - llama.cpp などを同梱する: model 配布と runtime 管理が重く不採用

### 2. provider 設定は「種類 + endpoint + model + capability」で保持する

display name だけでなく、provider kind、base URL、selected model、利用可能な能力を settings に保持する。軽量 model の推奨は UI で提示するが、最終選択は user に委ねる。

- 採用理由:
  - provider 切替と再接続を deterministic に扱える
  - `v0.23.0` と `v0.24.0` にそのまま流用できる
- 代替案:
  - provider ごとに別 settings 画面へ分離する: 共通 contract が崩れるため不採用

### 3. autofix は preview 後の明示適用を必須にする

local LLM の返す修正をそのまま silent apply すると Markdown を破壊する危険がある。したがって、`v0.22.0` の autofix は patch preview または修正候補確認の後で apply する。

- 採用理由:
  - 初期リリースでも破壊的誤修正を抑えられる
  - user trust を損ねにくい
- 代替案:
  - diagnostic 単位で即時自動適用する: 修正ミス時の復旧が難しく不採用

### 4. autofix input は official markdownlint diagnostics payload をそのまま使う

`v0.18.0` で整備する official rule code、message、location、file path をそのまま prompt input / structured input に使い、追加 lookup を最小化する。

- 採用理由:
  - diagnostics と autofix の契約が単純になる
  - future generation / translation と入力モデルを揃えやすい
- 代替案:
  - UI 表示文字列だけを prompt に使う: rule location や fix scope が曖昧になり不採用

## Risks / Trade-offs

- **[Risk] local provider の利用可否判定が不安定だと UX が崩れる**  
  -> Mitigation: explicit test connection と disabled state を用意する
- **[Risk] 軽量 model が lint 修正品質を満たさない**  
  -> Mitigation: recommended model guidance と preview review を組み合わせる
- **[Risk] provider ごとの差異が prompt / response shape に表れる**  
  -> Mitigation: adapter ごとに normalized request / response shape を定義する
- **[Risk] autofix が広い範囲を書き換える**  
  -> Mitigation: diagnostic 単位または user-selected batch 単位に fix scope を制限する

## Migration Plan

1. settings schema と provider registry を local endpoint provider 前提に拡張する
2. `Ollama`、`LM Studio`、OpenAI 互換 endpoint の adapter / preset を追加する
3. provider test connection と model 選択 UI を実装する
4. markdownlint diagnostics から autofix request を組み立てる
5. preview / approve / apply / re-lint の flow を実装する

## Open Questions

- lightweight 推奨 model の候補を UI に固定表示するか、provider から列挙したものに限定するか
- batch autofix を `v0.22.0` で含めるか、single diagnostic fix を先行するか
