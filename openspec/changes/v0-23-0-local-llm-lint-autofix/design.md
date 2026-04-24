## Context

`crates/katana-core/src/ai/mod.rs` には `AiProvider` trait と `AiProviderRegistry` があり、provider abstraction 自体は存在する。しかし現在は次の欠落がある。

- user が active provider を設定・永続化する仕組みがない
- Ollama endpoint を登録する adapter がない
- model 一覧取得や利用可否確認の導線がない
- user が local LLM と直接会話する chat UI がない
- diagnostics から autofix request を組み立てる層がない
- AI 未設定時の disabled state はあるが、UI workflow はない

ユーザー要望は「Ollama 経由の local LLM」「1桁GB級モデルの推奨」「`katana-ui` 内に完全分離した chat UI」である。したがって `v0.23.0` は bundled runtime ではなく Ollama endpoint integration を前提とし、chat foundation と markdownlint autofix を分離可能な実装単位として成立させる。

## Goals / Non-Goals

**Goals:**

- user が local LLM provider を設定、切り替え、検証できる
- 初期 provider は Ollama を主対象にする
- 1桁GB級のモデルを選びやすい推奨導線を提供する
- `katana-ui` 内に、VS Code 風の端アイコンから開閉できる chat サイドパネルを提供する
- MVP では Ollama モデル選択を必須にし、細かい生成設定は後続 task に分離する
- MVP の chat はアプリ起動中の一時的な会話だけを扱い、履歴の保存・一覧・管理は後続 task に分離する
- provider 未設定でも app 本体の利用性を維持する
- markdownlint diagnostics と KML の一括 fix 後 content を入力にした file 単位の autofix 候補生成を実装する
- autofix は差分 preview / 確認の上で適用できるようにする

**Non-Goals:**

- model runtime や model file のアプリ同梱
- document generation の実装
- translation overlay の実装
- Vertex AI、Bedrock、OpenAI または OpenAI-compatible endpoint の実装
- 音声入力の実装
- チャット履歴の永続化、履歴一覧、履歴検索、履歴削除 UI
- model ごとの細かい generation parameters UI

## Decisions

### 1. 初期 local LLM は Ollama endpoint integration を前提にする

runtime bundling ではなく、Ollama が提供する local endpoint を provider adapter として扱う。KatanA 独自の外部 AI IF をこの時点で広げず、Ollama adapter を既存 `AiProvider` abstraction に接続する。
このアダプター層には利用可否判定、明示的な timeout、invalid response の扱いを持たせ、バックエンドの不調が本体 UI をフリーズさせないよう隔離する。

- 採用理由:
  - 既存の `AiProvider` abstraction に素直に乗る
  - cross-platform packaging を増やしにくい
  - 最初の実装境界が Ollama に閉じるため conflict しにくい
- 代替案:
  - llama.cpp などを同梱する: model 配布と runtime 管理が重く不採用
  - 初期から LM Studio / OpenAI-compatible / remote provider を並列対応する: credential 管理と UI 分岐が広がるため後続 milestone に回す

### 2. provider 設定は Ollama 設定を明示しつつ、後続 provider へ拡張できる形で保持する

display name だけでなく、provider kind、base URL、selected model、利用可能な能力を settings に保持する。`v0.23.0` の provider kind は Ollama を主対象にし、軽量 model の推奨は UI で提示するが、最終選択は user に委ねる。

- 採用理由:
  - provider 切替と再接続を deterministic に扱える
  - `v0.24.0` と `v0.25.0` にそのまま流用できる
  - Vertex AI / Bedrock / OpenAI 系の credential 対応を後で追加しても設定境界を壊しにくい
- 代替案:
  - provider ごとに別 settings 画面へ分離する: 共通 contract が崩れるため不採用

### 3. chat UI は VS Code 風のサイドパネルとして扱う

chat は editor 操作や diagnostics panel の内部機能ではなく、画面端のアイコン列から開閉できる独立サイドパネルとして配置する。画像で示された VS Code 風の体験に合わせ、アイコンで chat panel の表示・非表示や固定表示を制御できるようにする。chat state、会話中の message、pending request、error state は専用 module に閉じ、document 変更は明示 action を通じてのみ行う。

- 採用理由:
  - chat foundation を lint autofix や document generation と並行して進めやすい
  - 既存の Explorer / TOC / Problems 的な panel 操作と揃えやすい
  - file mutation の事故を防ぎやすい
  - 将来の音声入力や remote provider 対応を chat 側へ足しやすい
- 代替案:
  - diagnostics panel に chat を埋め込む: lint autofix と会話体験の責務が混ざるため不採用
  - editor inline helper として始める: document mutation 境界が曖昧になるため不採用
  - modal だけで始める: 長い会話や作業中の参照に向かないため不採用

### 4. MVP の chat history はアプリ起動中の一時状態に限定する

MVP では「起動中の chat UI から色々要求できること」を優先する。chat messages は active app session の state として保持するが、アプリ再起動後に復元する永続履歴、履歴一覧、履歴検索、履歴削除などの管理機能は後続 task に分離する。

- 採用理由:
  - MVP の実装範囲を provider 接続、モデル選択、chat request lifecycle に集中できる
  - 永続化 schema と privacy / cleanup policy の議論を後続へ分離できる
- 代替案:
  - 初期から履歴管理を入れる: MVP の焦点がぼやけるため不採用

### 5. MVP ではモデル選択を必須にする

Ollama provider が利用可能でも、どの model を使うかが未選択の状態では chat / autofix request を送信しない。MVP はモデル選択を必須にし、temperature、top-p、context size などの細かい生成パラメータは後続 task とする。

- 採用理由:
  - 実行 model が明示されるため挙動を説明しやすい
  - 軽量 model 推奨導線と自然につながる
- 代替案:
  - default model を暗黙選択する: user が意図しない重い model を使う危険があるため不採用

### 6. autofix は file 単位の一括修正と差分 preview を前提にする

autofix は single diagnostic だけを直す機能ではなく、file 内の diagnostics 全体を対象にする。まず KML (`katana-markdown-linter`) が可能な一括 fix を適用した後の content を作り、その content と残存 diagnostics を LLM context に渡す。LLM は全エラー解消のための file-level proposal を返し、KatanA は元 content と提案 content の差分を preview した上で user confirmation 後に apply する。

- 採用理由:
  - KML の deterministic fix を先に使うことで、LLM が考える範囲を減らせる
  - file 単位で全 diagnostics の解消を提案できる
  - 差分 preview により破壊的誤修正を抑えられる
  - user trust を損ねにくい
- 代替案:
  - diagnostic 単位で即時自動適用する: 一括修正の構想と合わず、修正後の相互作用も見えにくいため不採用
  - KML を通さず diagnostics だけを LLM に渡す: deterministic に解ける修正まで LLM に任せるため不採用

### 7. autofix input は official diagnostics payload と KML 一括 fix result を使う

`v0.19.0` で整備する official rule code、message、location、file path を prompt input / structured input に使う。加えて、KML が一括 fix できる範囲を反映した後の content、元 content、残存 diagnostics、対象 file path を file-level context として LLM へ渡す。

- 採用理由:
  - diagnostics と autofix の契約が単純になる
  - KML の fix 能力を local LLM workflow へ自然に接続できる
  - future generation / translation と入力モデルを揃えやすい
- 代替案:
  - UI 表示文字列だけを prompt に使う: rule location や fix scope が曖昧になり不採用

## Risks / Trade-offs

- **[Risk] Ollama の利用可否判定が不安定だと UX が崩れる**
  -> Mitigation: explicit test connection と disabled state を用意する
- **[Risk] 軽量 model が lint 修正品質を満たさない**
  -> Mitigation: recommended model guidance と preview review を組み合わせる
- **[Risk] KML fix 後 content と LLM proposal の差分が広がりすぎる**
  -> Mitigation: file 単位の diff preview と explicit confirmation を必須にする
- **[Risk] 差分 preview 機能が未実装だと autofix の安全境界が成立しない**
  -> Mitigation: autofix apply pipeline の前に reusable diff preview surface を実装する
- **[Risk] provider ごとの差異が prompt / response shape に表れる**
  -> Mitigation: `v0.23.0` は Ollama に閉じ、remote provider は後続 milestone で credential 管理込みで扱う
- **[Risk] chat UI が editor / diagnostics と結合して実装 conflict を起こす**
  -> Mitigation: chat state と UI module を専用化し、document mutation は action 境界を通す
- **[Risk] autofix が広い範囲を書き換える**
  -> Mitigation: file 単位に fix scope を限定し、差分 preview と explicit confirmation を必須にする

## Migration Plan

1. settings schema と provider registry を Ollama endpoint 前提に拡張する
2. Ollama adapter、model 一覧取得、availability check を追加する
3. `katana-ui` に独立 chat state / chat surface / request lifecycle を追加する
4. VS Code 風の端アイコンから chat サイドパネルを表示・非表示・固定表示できる UI を実装する
5. provider test connection と model 選択 UI を実装する
6. 元 content、KML 一括 fix 後 content、残存 diagnostics から file-level autofix request を組み立てる
7. reusable diff preview surface を実装する
8. preview / approve / apply / re-lint の flow を実装する

## Deferred Expansion Backlog

- チャット履歴の永続化、履歴一覧、履歴検索、履歴削除 UI は MVP 後の別 task とする。
- model ごとの細かい生成パラメータ UI は MVP 後の別 task とする。
- 音声入力は chat surface が安定した後に別 OpenSpec change として扱う。MVP は OS dictation 連携寄りにし、音声入力結果を chat composer に入れ、document mutation は既存の confirmation 境界を通す。
- アプリ内録音、speech-to-text、typeless 的な不要音声・ノイズ・口癖除去は MVP ではなく劣後 task とする。必要なら KatanA 本体から分離した repository として扱う。
- Vertex AI / Bedrock / OpenAI または OpenAI-compatible provider は、Ollama adapter が安定した後に別 OpenSpec change として扱う。API key / secret は OS keychain や既存 settings persistence との責務境界を先に決める。

## Open Questions

- widget 依存の追加許容範囲を egui 系 crate までとするか
- lightweight 推奨 model の候補を UI に固定表示するか、Ollama から列挙したものに限定するか
- Vertex AI / Bedrock / OpenAI 系 provider をどの version milestone に切るか
