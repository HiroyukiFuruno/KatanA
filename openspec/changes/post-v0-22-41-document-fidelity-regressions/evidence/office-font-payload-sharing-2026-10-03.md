# Officeタブ間のフォント保持重複

## 対象と再現

- PR346 P2 comment4171707858 / threadPRRT_kwDORm09y86oj-ns（review対象04a5cefa）。
- 旧5f8e5fdfの実コードに、Ubuntu-Lightの実bytesを独立owned Vec/Arcへ読み込む回帰を追加。`just JOBS=2 T=independent test-specific`はexit101、2つ目のleaseでepochが1→2になるREDを確認した。ログ: `tmp/font-sharing-old-red.log`、RTK全ログ1791002780_cargo_test.log。
- 原因はpointer identityで同じ書体を別物扱いすること。aliasだけを共有しても、lease.facesに独立payloadを残せばbytesの保持は解消しない。

## 修正境界

- 既存sha2依存で背景resolverの最終選択時にSHA256を一度計算する。描画やUI lock内で全font bytesを走査しない。
- 内容digest、face index、FontData index/tweak、family/weight/styleから共有identityを作る。file pathだけの一致では更新されたbytesを同一扱いしない。
- 既存shared entryへの追加時は、LeaseFacesが保持するpayloadもcanonical Arcへ変更する。resolutionはaccept後に保持されず、duplicate allocationを解放できる。
- 最後のleaseまでreference countを維持し、最後のcloseでalias/payloadを除去する。永続global font cacheや新依存は追加しない。
- identity責務はface_identity.rsへ分離し、既存source200行上限を維持する。

## 検証

- 新規回帰は独立allocationのalias/epoch共有、duplicate Weak解放、全leaseのcanonical Arc共有、最後close後のWeak解放、異なる実font内容/index/style/tweakの非共有を確認する。
- 試作の非共有テストは不正font bytesを実eguiへ渡してpanicしたため、商用コードのfallbackを追加せず、別の実font（Hack）のbytesとidentity直接比較に修正した。
- 修正後font_loader52件成功: `tmp/font-sharing-focused-final.log`。さらに実OS書体を2つの独立resolverで読み込んでlease共有・duplicate Weak解放を確認する回帰を追加した。
- 最終の実OS書体回帰を含むfont_loader53件成功: `tmp/font-sharing-all-font-final.log`、exit0。
- AST23件成功: `tmp/font-sharing-ast-final.log`。初稿のコメント形式・digest長リテラル違反は規則を無効化せず修正した。
- `just JOBS=2 check-full`はexit0: `tmp/font-sharing-check-full.log`。native lint、実Office worker fixture8件、UI1021件（既存ignore2）、parallel143件（既存ignore2）、serial18件、AST、全coverage（意味のある未実行行0、strict document surface100%）が成功。Linux locked workspaceはUI1005件（既存ignore2）・実fixture8件・parallel141件（既存ignore2）・serial18件を含め成功。Windows workspace test-inclusive cross-check、supply-chainのadvisories/bans/licenses/sourcesも成功。閾値・除外・skip・依存graphは変更していない。
- 長時間のMermaid exportもnative13件334.70s、Linux14件340.71sで成功。途中の1秒CPU sample（`tmp/font-sharing-export-cpu.sample.txt`）では435 samples中410がKDVのpaste_rgba_resized配下だった。これは当該debug検査の限定診断であり、配布性能やKRR#95の原因証明ではない。既存KDV担当へ診断を引き継いだ。
- 自己レビューPASS: hash計算の唯一の商用callerは背景font_lookup_worker、UI lock内の全bytes走査なし。accept後にresolution.facesを保持する別ownerなし。内容/index/style/tweakの非共有とduplicate/canonical Weakの解放を実データで確認。既存base/他pane/世代/取消の契約は変更しない。formatter・MD lint・diff check成功。正式commit/push、review reply/resolveは続行する。

## 未完了の製品受入

この修正はタブ間の長期保持重複を扱う。ファイル読込中の一時payload、KDV/KRR資源、allocator、原本HTML性能、全Office fidelity、配布mainでのRSS・通常close、clean-machineの全CPU/OS受入、v0.22.42公開は別の未完了条件である。単体回帰をリリース完了扱いしない。
