# プレビュー生成時の不要な構文辞書初期化

## 所有と変更範囲

同じKatanA repositoryで既に同梱しているCommonMark backendのprivate `SyntaxSet` / `ThemeSet` を、cache生成時ではなくコード描画またはカスタム追加時に初期化する。既定の描画規則、公開API、cacheごとの所有と解放、backendのMSRV 1.88を維持する。新しい依存・sibling override・unsafe・除外・閾値変更はない。

sourceは通常hook・署名Gの `3dc70d94126af15eaba2f40d0ed49f964b3fe73f`、通常検証への接続は `c1957f0b6a5f6025fa1456b7d8ee4897d573951e`。リロード検査の不要な二重参照2か所は別関心の `8228a76c19441b30333958a1e2419e5a14576471`。

## 回収した検査

- root lock・UIとbackendのfeature unionでbackend回帰4件actual exit0、session47787 / `tmp/commonmark-lazy-root-locked-tests-corrected.log`。既定のLayoutJob同一、必要時だけ初期化・同じinstance再利用、描画前後のcustom構文/theme追加、無効入力後の既定とcustom保持。併記されたUI0件はUI受入の証拠にしない。
- 既存実Office worker入口のHTML移動group7件actual exit0、session7960 / `tmp/commonmark-lazy-html-two-thread-tests.log`。元2秒・dirty buffer・実RGB・2threadsを維持。
- 実ファイル変更とsidebar pointer操作からリロードする既存回帰1件actual exit0、session49147 / `tmp/commonmark-lazy-sidebar-regression.log`。本人native受入の完了ではない。
- backendとUIを明示選択したall-targets strict Clippy actual exit0、session26343 / `tmp/commonmark-lazy-selected-clippy-corrected.log`。先行strict検査では既存sidebarテストのneedless_borrow2件が失敗し、上記別commitで正規修正した。
- 公式AST23件actual exit0、session25923 / `tmp/commonmark-lazy-ast-lint.log`。CI/resource契約9、release-flow契約9、lefthook validateもactual exit0。
- vendor変更が通常検査から漏れた旧分類に対するUTはactual exit1・1 assertion failure / `tmp/vendor-routing-original-red.log`。現在の分類は3件actual exit0 / `tmp/vendor-routing-current-green.log`。metadata/diff固定は分類関数のUTであり、文書IT/E2Eの代替ではない。

初回backendへのfeature直指定はCargoがworkspace外指定を拒否しactual101、最初の新UTはegui0.36のPanel API不一致でcompile101。これらを製品のassertion REDや検査成功としない。root UIとの同時選択と実Ui入口へ訂正した結果だけを採用した。

## 元のmacOS CI失敗との境界

公開4a09c195のmac CI112249577372はHTML dirty-target初回frame2149msでFAIL、1127PASS/1FAIL/既存ignore2。実PID29110のsampleはdeadline後の2観測で、隣接testの同期CommonMark初期化とHTML SVG描画を示すが、前2秒の原因を確定しない。

旧sourceのローカル同profile/2threads groupは7PASS・0.50秒、新候補は7PASS・0.46秒。単発group結果を統計的な高速化や元cloud失敗の修復証明へ読み替えない。新HEADの全CI、coverage、実配布、本人受入と公開後処理はまだ未完了。

既存のworkspace全テスト・元coverage・全3OS・受入条件を残し、workspace外backendの回帰だけをCIと通常push経路へ追加した。手動の重複full checkは行わず、通常push hookを正式ゲートとする。KDV0.5.12/KRR0.4.23を維持し、上流のChrome互換性、変換遅延、再起動後artifact再利用は今回修復したと主張しない。
