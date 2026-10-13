# Task 1: Failure Contract Alignment

## 1.1 `settings.save()` 呼び出し箇所の棚卸しと対象の確定

現在、Katana-UI内にて `settings.save()` は多数の箇所で呼び出されていますが、そのほとんどがエラーを握り潰しています（Silent Failure）。

### 現状の呼び出し箇所（`katana-ui/src/` 内）

- **`shell.rs`**:
  - `Line 158`: `if let Err(e) = app.state.config.settings.save()` -> ログ出力のみ (`tracing::warn!`)
- **`app/action.rs`**:
  - `Line 223, 384, 681`: `if let Err(e) = ...save()` -> ログもなし（ブロック内で処理なしと推測）、一部は暗黙的。
  - `Line 401`: `let _ = ...save();` -> 完全に握り潰し
- **`app/workspace.rs`**:
  - `Line 190, 293, 318`: `if let Err(e) = ...save()` -> 処理を握り潰しまたはログ出力のみ
- **各種設定タブ (`settings/tabs/*.rs`)**:
  - `workspace.rs`: 5箇所の `let _ = settings.save();`
  - `theme.rs`: 6箇所の `let _ = settings.save();`
  - `behavior.rs`: 4箇所の `let _ = settings.save();`
  - `updates.rs`: 1箇所の `let _ = settings.save();`
  - `font.rs`: 2箇所の `let _ = settings.save();`
  - `layout.rs`: 7箇所の `let _ = state.config.settings.save();`

**結論**:
ユーザーが設定UIで値を変更した直後や、他のUI操作（ペイン幅変更など）ですべて `save()` が呼ばれますが、**ディスクフルやパーミッションエラーで書き込みに失敗してもユーザーには一切通知されません**。
これは明確な Silent Failure であり、対象は **「UI層でのすべての `settings.save()` 呼び出し」** となります。これを撲滅するために、エラー処理を共通メソッドに委譲するか、通知機構（Notification/Toast）につなげる必要があります。

---

## 1.2 Failure Contract（失敗時の契約）の確定

各領域における「何を守り、どこで止め、何を表示するか」のルールを以下のように定めます。

### A. Settings (設定の保存・読み込み)

- **何を守るか**:
  - 保存時: 既存の正常な設定ファイル（`settings.json`）を不完全な書き込みから守る（書き込み中のクラッシュ等）。
  - 読込時: 不正な（パース失敗する）JSONファイルによって過去の設定がサイレントに消失する（デフォルトで上書きされる）のを防ぐ。
- **どうするか**:
  - 保存時 (**Save**): `temp` ファイルへの書き込み後、`rename` によるアトミック保存を行う。失敗した場合は `tracing::error!` でログを残すとともに、可能であればアプリケーション側のエラー通知機構へ流す（または一元化されたエラーハンドラで処理する）。アプリ自体はクラッシュさせない（インメモリ設定は有効なまま続行）。
  - 読込時 (**Load**): パースエラーなどの破損を検知した場合、既存ファイルをただ読み飛ばしてデフォルト状態にするのではなく、`settings.json.bak`（または `.corrupted`）として退避してからデフォルト設定でフォールバックする。
- **UI表示**:
  - 理想的には `Toast` や `Notification` による一時的かつ目立つ通知。

### B. App Update (アプリのオンラインアップデート)

- **何を守るか**:
  - バージョン入れ替え作業中に失敗した場合でも、既存バージョンのアプリが起動不能（壊れた状態）になることを防ぐ。
- **どうするか**:
  - 破壊的更新（直接の上書き解凍）をやめる。新しいバージョンの展開を別の一時ディレクトリ（`Staging`）で行う。
  - バンドルの展開と完全性の検証が終わったあとにのみディレクトリスワップを行う（Staged Update Swap）。
  - もしスワップ中に失敗した場合（プロセスロック等）は、直ちにエラー・ロールバックする。
- **UI表示**:
  - アップデート準備・スワップ失敗時: 「アップデートに失敗しました（詳細）」というエラーダイアログやトーストを表示し、現在のバージョンで利用を継続できるようにする。

### C. Release (リリースのPreflight)

- **何を守るか**:
  - CHANGELOG・OpenSpec Tasksなどが不完全/不整合な状態のまま、CI上で（あるいはローカルから）タグ付け・Publishされてしまうのを防ぐ。
- **どうするか**:
  - CI (GitHub Actions) と Local (Makefile等) の双方で共通の `preflight` 検証スクリプトまたは Makefile ターゲットを呼び出す。
- **UI/出力**:
  - ターミナル/CI上で非ゼロ（デプロイ阻却）の終了コードと、"Resolve incomplete tasks in tasks.md before release" や "Missing CHANGELOG entry for vX.Y.Z" のようなアクションを促す明示的エラーメッセージを表示する。

---

## 1.3 追加テストケースの洗い出し

これらを保護するため、Task 2, 3, 4 で以下のテストを実装します。

### Settings (Task 2)

1. **Save Atomic Validation Test**:
  - `JsonFileRepository` の `save()` が一時ファイルを経由してリネームするアトミック処理であることを検証する（書き込み途中で異常終了してもファイルが壊れないことの論理的保護）。
1. **Corrupted Settings Backup Test**:
  - 不正なJSON `{"invalid": "json` などを `settings.json` に書き込んだ状態で `load()` を呼び出す。
  - デフォルト設定で起動するだけでなく、元の破損ファイルが `.bak` などの別名で退避（Rename）保存されていることを検証する。

### Update Install Hardening (Task 3)

1. **Staged Swap Mock Test**:
  - `prepare_update` による一時ディレクトリ展開と入れ替え（Swap）処理が、途中の失敗時に元のバンドルを破壊しないことを検証する。
1. **Swap Failure Handling**:
  - スワップ処理がアクセス権等で失敗した場合、元の状態が保たれつつエラー（`Err`）が正しく呼び出し元に返ることをテストする。

### Release Pipeline Consistency (Task 4)

1. **Preflight Failure (Incomplete Tasks/Changelog)**:
  - `tasks.md` 内に未完了 `- [ ]` が残っている場合や、次バージョンの CHANGELOG エントリが存在しない場合に、Preflight が失敗し異常終了（Exit 1等）することを検証する。
1. **Preflight Success**:
  - 全条件を満たしている場合に限り、Preflight が成功することを検証する。
