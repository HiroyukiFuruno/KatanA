## Context

Local LLM の土台は provider 設定、chat state、autofix request / diff preview の3つに分かれている。現状の残り課題は、ユーザーが「どこでモデルを選ぶか」「なぜ送信できないか」「diagnostics から修正に進むには何が足りないか」を UI 上で判断できる状態にすること。

この change は機能追加というより、既存の local LLM MVP をユーザー操作として閉じる作業である。provider や autofix pipeline の内部仕様は大きく変えず、設定と復旧導線を整える。

## Goals / Non-Goals

**Goals:**

- settings、chat、Problems panel の間に明確な導線を作る。
- provider unavailable の理由を UI 表示と状態で説明できるようにする。
- model 未選択時は request を送らず、設定画面へ戻れるようにする。
- diagnostics autofix の入口を、利用可否が分かる形で表示する。
- UI 確認結果を tasks に残せるようにする。

**Non-Goals:**

- chat 履歴の永続化。
- OpenAI / Vertex AI / Bedrock provider の追加。
- 音声入力。
- document generation。
- model ごとの細かい生成パラメータ UI。

## Decisions

### 1. Settings を local LLM の起点にする

chat や autofix から provider 未設定を検出した場合、settings の AI セクションへ遷移する。各機能に個別の設定フォームを持たせず、Ollama endpoint、model、接続確認は settings に集約する。

代替案として chat panel 内に接続設定を埋め込む方法があるが、autofix からも同じ設定を使うため採用しない。

### 2. Disabled state は理由つきで表示する

単にボタンを無効化するだけではなく、model 未選択、接続未確認、接続失敗、timeout などの理由を近くに表示する。復旧操作がある場合は settings へ移動する action を添える。

### 3. Diagnostics autofix は file 単位の入口として扱う

Problems panel の file header または同等の file-level surface から autofix を開始する。single diagnostic の即時修正とは別にし、file-level diff preview の安全境界を維持する。

### 4. UI 検証は画像だけに依存しない

UI screenshot はユーザーレビュー用に使えるが、回帰検知は state、button enabled 状態、i18n key、action dispatch などの semantic assertions を中心にする。visual snapshot test は導入しない。

## Risks / Trade-offs

- [Risk] settings と chat / Problems の state が二重管理になる → provider 状態は既存 settings と専用 state から読み、表示だけを UI 側へ分離する。
- [Risk] UI 導線が増えて Problems panel が重くなる → file header の主操作に限定し、詳細説明は settings 側に寄せる。
- [Risk] provider 接続確認が UI を止める → request lifecycle は background task と pending state を使う。

## Migration Plan

1. settings の AI セクションを local LLM の起点として整理する。
2. chat / autofix の disabled reason を統一モデルで表現する。
3. Problems panel の autofix entry point と recovery 導線を調整する。
4. semantic UI tests を追加する。
5. UI snapshot または操作確認結果をユーザーに提示し、feedback を tasks に記録する。

## Open Questions

- 接続確認済み状態を session state のみにするか、settings に保存するか。
- Ollama model list が空の場合に lightweight model 推奨文をどの位置へ出すか。
