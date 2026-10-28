# Windows subclass APIのactivation補正

## 失敗の切り分け

- source aefa0c55b22d25d4e82bb04cf36ba45edfa2df28のCI40603722343、Windows job122537918992。
- UI lib1353成功/0失敗/2ignoredの後、mainテストEXEがテスト開始前に0xc0000139 STATUS_ENTRYPOINT_NOT_FOUNDで終了。追加Win32テスト3件は未実行。
- ログ `tmp/oct28-aefa-windows-ci.log`。worker panicは意図した既存回帰テストで成功しており、この終了の原因と混同しない。
- 新しいsubclass APIはComctl32の名前付きimport。既存build.rsはiconだけをwinresへ渡し、manifestは指定していなかった。
- Microsoftの[GetWindowSubclass契約](https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-getwindowsubclass)はCommon Controls v6をmanifestで指定するよう求める。この契約欠落を補正する。ロード失敗の実解消は次のWindows CIで確認する。

## 補正と検証範囲

- Windows対象のwinresへ `windows.manifest` を渡す。指定する共有assemblyはMicrosoft.Windows.Common-Controls6.0.0.0/publicKeyToken6595b64144ccf1df/architectureと言語は各*。権限昇格やOS保護の無効化は含めない。
- manifest変更をbuild rerun対象とし、resource生成エラーを既存の黙殺から明示的なビルド失敗へ変える。欠けたまま配布しない。
- 実manifestのXML構文と共有assembly全属性を検査し成功。独立監査でroot assemblyIdentity欠落も検出し、先頭へ実行ファイルidentityを追加。新契約2件は実ee9 manifestで1失敗を再現し、現manifestで2成功。assembly identityの1.0.0.0はactivation識別子であり、製品file/versionリソースは従来どおりCargoの版を使う。既存fmtとdiff check成功。
- 追加済み実Win32テストは引き続きWindows CIで実行する。ローカルXML成功やMSVC metadataチェックを実行結果の代用にしない。
- 現PRはDraftへ戻し、補正の通常commit/pushと新HEADレビュー/CIを継続。旧aefaの3OS Lint/供給網/Readiness成功を次HEADへ流用しない。
- 元Windows200ms×10、latency1、メモリ/フォント/identity/終了、全OS公開条件は変更しない。正式配布起動と公開は未完。

## 正式履歴

- source ee9c27b1は通常commit10467実exit0。root identity/新契約はbff6f290、通常commit14941実exit0。hook迂回なし。独立再レビューの残P0/P1は0件。
- 通常push、新HEAD current review/必須CI/Windows実行、正式全OS公開・公開後処理は継続する。

## クロス検査でのresource生成

- 8cbff96bの通常push54589は実exit1。Windows本体ではなく、Linuxのcargo xwin checkでrc.exeが見つからずFAIL101。並列preflightも終端まで回収した。
- Windows nativeまたはGNU targetはwinresの既存compilerを使い、失敗を明示する。non-Windows host/MSVCだけ、既存LLVMrcで同じwinres-generated resourceを生成して同じlink directivesを出す。manifestを省略した成功やfallbackにはしない。
- 公式just check-windows35433 actual0、Windows本体の実行とは区別する。実winres/icon/version/manifestからMacと既存Docker LLVMでMSVC.resを生成し、type24/id1内のmanifest575bytesが実sourceと完全一致した。prototypeとproduction helperのSHAも一致する。
- 手動style-fmt-check/docs-formatは現Justfileに無く、過去の複合shell最終exitを個別成功と取り違えた。これらのログは成功証拠にしない。正式fmt-check24088 actual0と対象2文書KML actual0を取り直した。通常commit/push hooksの結果とは分離する。
- 新しいWindows loader/実Win32テスト、元200ms基準、公開全5成果物は引き続き未完。今回も品質条件とOS保護は変更しない。

- cross helperは通常署名commit6535a7c1（77079実exit0）へ統合。正規pre-commitを使用し、新HEADの通常push・current review・全CIを継続する。
