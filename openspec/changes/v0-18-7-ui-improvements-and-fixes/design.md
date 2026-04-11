## Context

v0.18系のフィードバックに基づき、日常的な操作における「ノイズ」の削減と、デスクトップアプリとしての「質感」の向上を目指す。現状のUIは機能的ではあるが、情報密度が低かったり、操作の一貫性が欠けている箇所がある。

## Goals / Non-Goals

**Goals:**

- 検索結果におけるRustアトリビュート（allowなど）のノイズ除去。
- メタ情報表示のmacOS Finder風への刷新。
- ダイアグラム全画面表示の集中度向上。
- サイドバーパネルの共存（エクスプローラーを維持したまま他パネルを表示）。
- サイドバーアイコンからのアニメーションポップアップの実装。
- タブグループの操作性向上（確定操作、コンテキストメニュー統合）。
- リンク自動検出のバグ修正。

**Non-Goals:**

- 検索エンジン自体の大幅な変更（今回は既存ロジックのフィルタリングにとどめる）。
- タブグループの永続化形式の変更（現状のメモリ内管理を維持）。

## Decisions

- **検索フィルタリング**: `katana-core` の `WorkspaceSearchOps` にて、行が `#[allow(...)]` のパターンに完全一致（または主目的がアトリビュート）かつクエリがアトリビュート名そのものでない場合に、結果から除外する。
- **メタ情報UI**: `crates/katana-ui/src/views/modals/meta_info.rs` を刷新。セクション（一般、統計、パス）に分け、ラベルと値を整理して配置。フォントサイズや余白に強弱をつける。
- **全画面図表**: `egui` の背景色（`Window` または `Area`）のアルファ値を `1.0` (不透明) に設定。
- **サイドバー継続性**: `AppAction::ToggleHistoryPanel` 等のディスパッチ処理において、`show_explorer = false` にしている箇所を削除する。画面幅が狭い場合はオーバーレイとしてエクスプローラーの上に重ねる。
- **サイドバーポップアップ**: `Area` を使用し、アイコンの座標からオフセットした位置にポップアップ。表示時にスケール/不透明度のアニメーションを適用する。
- **タブUX**: `TextEdit` の `lost_focus` または `Return` 押下を検知してアクションをコミットする。
- **エクスプローラー連携**: `DirEntry` / `FileEntry` の右クリックメニュー（`context_menu`）に、`AppAction::CreateTabGroup` 等へのディスパッチを追加。
- **リンク検出**: `katana-core/src/markdown/link_resolver.rs` の正規表現を修正し、スキーム名から始まるURLを正しく認識するようにする。

## Risks / Trade-offs

- **検索フィルタリング**: ユーザーが意図的に `allow` を探している場合にヒットしなくなる可能性があるため、クエリが `allow` を含む場合はフィルタを無効化する。
- **サイドバー共存**: パネルが横に並びすぎるとコンテンツエリアが圧迫される。一定幅以下の場合はドロワー形式にする必要がある。

---

## Appendix: Windows Packaging & Winget Research (2026-04-11)

この change には当初 UI 改善が中心だったが、`v0.18.7` で Windows 配布と winget 再申請も同梱する方針になった。ここでは、実装前に確定できた技術事実と推奨方針だけを整理する。

### 1. 現時点で確定している事実

- PR `microsoft/winget-pkgs#357436` の manifest には `Dependencies -> Microsoft.VCRedist.2015+.x64` が入っていた。
- その PR manifest は `Scope: user` の KatanA に対して、`Scope: machine` の `Microsoft.VCRedist.2015+.x64` を dependency として宣言していた。
- 現行 release artifact (`v0.18.6`) に対して `komac analyze KatanA-windows-x86_64.msi` を実行すると、`Scope: user` / `InstallerType: wix` / `ProductCode` などは検出されるが、`Dependencies` は出力されない。
- 現行 release artifact (`v0.18.6`) の `KatanA.exe` を `objdump -p` で見ると、`VCRUNTIME140.dll`、`VCRUNTIME140_1.dll`、`api-ms-win-crt-*` を import している。つまり Windows binary は現状 self-contained ではない。
- `scripts/release/sync-external.sh` は常に `komac update HiroyukiFuruno.katana-desktop --submit` を呼ぶ。
- `.github/workflows/build-and-release.yml` は winget sync 用に `GITHUB_TOKEN: ${{ secrets.WINGET_GH_TOKEN || github.token }}` を渡している。つまり現状は「dedicated secret」と「GitHub Actions の自動 token」を混在させている。
- しかし `komac list HiroyukiFuruno.katana-desktop` は upstream `microsoft/winget-pkgs` に package が存在しないため exit code `1` で失敗する。つまり現状の `sync-external.sh` は「初回申請の再提出」には使えない。
- `komac new` は package 初回作成用だが、少なくとも現行 CLI (`v2.16.0`) では non-TTY 環境で `The input device is not a TTY` となる。GitHub Actions からそのまま叩く前提は置けない。
- 一方で `komac submit <path> --yes --dry-run` は non-TTY で動く。manifest file 一式さえ手元にあれば、submit 自体は CI / script から扱える。
- Komac の公式 docs.rs は、PR 提出には classic GitHub token の `public_repo` scope が必要だと明記している。fine-grained token は manifest 作成や commit までは動いても PR 作成に失敗し得る。

### 2. 調査から導かれる判断

#### 2.1 初回の winget 再申請は `update` ではなく bootstrap path

`HiroyukiFuruno.katana-desktop` はまだ upstream に存在しないため、`komac update` を呼ぶ設計は誤りである。`v0.18.7` 時点では次のどちらかを採る。

- maintainer のローカル TTY で `komac new` を使って manifest を生成し、その生成物を `komac submit` で提出する
- あるいは manifest を template / script で生成し、`komac submit` だけを non-interactive に実行する

結論として、`sync-external.sh` の責務は少なくとも次のどちらかに分けるべきである。

- package 未存在時は fail-fast して「bootstrap が必要」と明示する
- package 存在時だけ `komac update` を使う

#### 2.2 `Dependencies` が消えても VC++ 問題は解決したことにならない

`komac analyze` と `komac new` の現在の挙動を見る限り、同じ種別の MSI を現在の toolchain で処理した場合、`Microsoft.VCRedist.2015+.x64` dependency が常に manifest に自動注入されるわけではない。これは「自動生成される dependency 情報は tool version や analysis path に依存しうる」ことを示している。

ただし binary 本体は依然として `VCRUNTIME140*.dll` / `api-ms-win-crt-*` を必要としている。したがって、

- manifest dependency を単に消す
- `komac` が dependency を出さないことに期待する

だけでは、「clean Windows machine で起動できる」という品質保証にはならない。`v0.18.7` では build policy 側で self-contained 化を優先する。

補足:

- Microsoft Learn の repository validation docs には `Validation-VCRuntime-Dependency` があり、「missing components を package に含めるか、dependency を manifest に追加して再提出する」よう案内されている。
- 今回は manifest dependency を足した PR が失敗しているため、`v0.18.7` では「component を package 側に含める」方向、すなわち self-contained 化を優先する。

#### 2.3 `crt-static` が第一候補、WiX / Burn への拡張は後回し

今回の配布形は `Scope: user` の WiX MSI であり、PR では `Validation-VCRuntime-Dependency` 相当の失敗が出ている。`Microsoft.VCRedist.2015+.x64` 側は `Scope: machine` manifest であり、WinGet は scope requirement によって applicable installer をフィルタし、空なら失敗する。これらを合わせると、scope 不整合が失敗要因の一つである可能性が高い。

ただし、ここは validation service 内部の全設定が見えているわけではないため、**直接原因としては「VC++ runtime dependency が解決できなかった」までを事実とし、scope 不整合は強い仮説として扱う**。

代替として Burn bundle 化や redist 同梱もあるが、installer 種別・署名・保守対象が一気に広がる。

そのため、`v0.18.7` の第一候補は Windows target (`x86_64-pc-windows-msvc`) に対して `-C target-feature=+crt-static` を有効化し、配布 EXE から `VCRUNTIME140*.dll` 依存を外す方針とする。

推奨理由:

- `cargo build` と `cargo wix` の両方に同じ policy を適用しやすい
- winget manifest に machine-scope dependency を持ち込まずに済む
- installer 技術 (`InstallerType: wix`, `Scope: user`) を変えなくてよい

### 3. 他エージェント向けの実装指針

#### 3.1 Windows build policy

推奨 touch point:

- 新規 `/.cargo/config.toml`
- または Windows target 専用の build env

ただし、workflow step ごとの `RUSTFLAGS` だと `cargo build` と `cargo wix` の両方へ確実に伝播させる必要がある。`cargo wix` も同じ target policy を使わせるため、target-specific な `.cargo/config.toml` に寄せる方が実装ミスが少ない。

推奨 verify:

- Windows runner 上で packaged `KatanA.exe` の import table を確認し、`VCRUNTIME140.dll` / `VCRUNTIME140_1.dll` / `api-ms-win-crt-*` が消えていることを確認する
- `komac analyze KatanA-windows-x86_64.msi` で installer metadata を再確認する

注意:

- macOS 上の `cargo check --target x86_64-pc-windows-msvc` は、この repo では `ring` の C compile が Windows SDK header 不足で失敗した。したがって Windows packaging の成否判定は Windows runner を正とし、macOS cross-check に時間を使いすぎないこと

#### 3.2 winget submission flow

推奨 flow:

1. `komac list HiroyukiFuruno.katana-desktop` で upstream 存在確認
2. 未存在なら bootstrap path に分岐
3. 既存 package がある場合のみ `komac update`

token 前提:

- `komac update` / `komac submit` で PR を作る時は classic GitHub PAT (`public_repo`) を使う
- `komac` 自体は `GITHUB_TOKEN` 環境変数も読めるが、automation では `WINGET_GH_TOKEN` を dedicated secret 名として持ち、`--token "${WINGET_GH_TOKEN}"` で明示的に渡す
- GitHub Actions の自動 `github.token` を fallback として期待しない。docs.rs の前提を満たす token source を 1 つに固定する

bootstrap path の推奨:

- CI 内で `komac new` を直接呼ばない
- manifest file を作って `komac submit <path> --yes` を使う
- もしくは maintainer がローカル TTY で `komac new --output ...` を実行し、生成物を review してから `komac submit` する

この方針にしておくと、初回再申請と将来の version update を分離できる。`sync-external.sh` は「未存在なら update を試さず明示的に止まる」だけでも価値がある。

`sync-external.sh` に入れる分岐条件は、次の順序を推奨する。

```sh
PACKAGE_ID="HiroyukiFuruno.katana-desktop"
MSI_URL="https://github.com/HiroyukiFuruno/KatanA/releases/download/${TAG}/KatanA-windows-x86_64.msi"

if [ -z "${WINGET_GH_TOKEN:-}" ]; then
  echo "⚠️ WINGET_GH_TOKEN (classic PAT with public_repo) is not set, skipping winget sync."
  exit 0
fi

if [ ! -f "${ARTIFACTS_DIR}/KatanA-windows-x86_64.msi" ]; then
  echo "⚠️ Windows MSI artifact not found, skipping winget sync."
  exit 0
fi

if ! command -v komac >/dev/null 2>&1; then
  echo "⚠️ komac not found, skipping winget sync."
  exit 0
fi

if komac list "${PACKAGE_ID}" --token "${WINGET_GH_TOKEN}" >/dev/null 2>&1; then
  komac update "${PACKAGE_ID}" \
    --version "${VERSION}" \
    --urls "${MSI_URL}" \
    --release-notes-url "https://github.com/HiroyukiFuruno/KatanA/releases/tag/${TAG}" \
    --submit \
    --token "${WINGET_GH_TOKEN}"
else
  echo "⚠️ ${PACKAGE_ID} does not exist in microsoft/winget-pkgs yet."
  echo "⚠️ Initial winget bootstrap is required; skipping automated update flow."
  exit 0
fi
```

要点:

- `package absent` は release 全体の failure にはせず、explicit warning で skip
- `package exists` の時だけ `komac update`
- 初回再申請の bootstrap path は別手順として切り出す
- token source は `WINGET_GH_TOKEN` に固定し、workflow 側でも `github.token` fallback を残さない

#### 3.3 installer UI refresh

`crates/katana-ui/wix/main.wxs` は現在 `WixUI_FeatureTree` を使っている。`FeatureTree` 自体を別 dialog set へ大きく置き換えるより、`v0.18.7` では次の low-risk 変更を推奨する。

- `Product` / `Package` / `Feature` / `Shortcut` 周辺の文言整理
- `WixUIBannerBmp` / `WixUIDialogBmp` の追加
- `Product.ico` と整合する branding 画像の追加

既存ファイルのコメントにもある通り、asset size は次を前提にする。

- banner: `493 x 58`
- dialog: `493 x 312`

これなら installer 種別も flow も変えずに「古い既定画面」感を減らせる。`WixUI_Advanced` への切替は follow-up とし、`v0.18.7` では scope を広げない。

### 4. 推奨 verification checklist

- `KatanA-windows-x86_64.zip` の `KatanA.exe` から MSVC runtime DLL import が消えている
- `KatanA-windows-x86_64.msi` の `komac analyze` 結果が `Scope: user` / `InstallerType: wix` のまま崩れていない
- bootstrap manifest を生成した場合、installer manifest に `Dependencies: Microsoft.VCRedist.2015+.x64` が入っていない
- `sync-external.sh` は package 未存在時に `komac update` を実行せず、bootstrap が必要だと明示する
- workflow と helper が `WINGET_GH_TOKEN` のみを使い、`github.token` fallback を残していない
- installer UI の確認証跡（最低 1 枚のスクリーンショット）が残っている

### 5. 一次情報

- winget PR: `https://github.com/microsoft/winget-pkgs/pull/357436`
- Komac docs.rs (`update` は pre-existing package 向け、PR 提出には classic token が必要):  
  `https://docs.rs/crate/komac/latest`
- Microsoft Learn (`repository`): `Validation-VCRuntime-Dependency` は missing components を package に含めるか dependency を manifest に追加して再提出するよう案内している  
  `https://learn.microsoft.com/en-us/windows/package-manager/package/repository`
- Microsoft Learn (`winget settings`): user scope requirement は applicable installer を絞り、空なら install failure になる  
  `https://learn.microsoft.com/en-us/windows/package-manager/winget/settings`
- FireGiant (`WixUI_FeatureTree` と branding bitmap 置換):  
  `https://docs.firegiant.com/wix3/wixui/dialog_reference/wixui_featuretree/`
  `https://docs.firegiant.com/wix/tools/wixext/wixui/`
- Rust Reference (`crt-static` target feature):  
  `https://doc.rust-lang.org/reference/conditional-compilation.html`
