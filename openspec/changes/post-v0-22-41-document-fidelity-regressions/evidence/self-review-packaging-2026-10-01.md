# 配布検査差分の自己レビュー

対象はuniversal macOS生成、全platformの展開後起動、binary architecture、main/sidecar identity、heartbeat、公開前検査と資産収集。通常release sourceの検証閾値は変更していない。

- P1: checkoutをcwdにした起動はresource漏れを隠すため、macOS/Linuxは展開先cwdとabsolute smoke script、WindowsはStart-ProcessのWorkingDirectoryを展開先に固定した。
- P1: partial dispatchでskipped jobを成功扱いにする公開条件を、全5needsのsuccessへ変更。新契約assertionは修正前に失敗し修正後成功。
- P1: asset収集の欠損許容を廃止し、非emptyの5成果物とexact1 DMG、既存出力の衝突拒否を実filesystem契約で検証。追加レビューで既知名symlink/hidden outputの見落としを修正。checksums全5件を再計算照合した。
- macOS main/worker arm64+x86_64、minimum OS、ad-hoc署名を維持。有償Apple Secret、公証、quality bypassは追加していない。
- architecture7件、startup/実child identity/heartbeat負例、ZIP path-preservation、collection8ケースが成功。mainとdistinctな観測sidecar PID/path/hashを必須にするchecker20件、release guard13件も成功。
- 実sidecar observerはkernel PID/path/hash/descendantを観測し、clean_machine/normal_closeをnullとして診断と受入を区別する。

残DoD: 実Actions、全配布CPU/OSのclean-machine起動、実Office入力/通常終了とsidecar identity。empty-workspace worker0の成功をOffice sidecar実行受入と扱わない。配布manifest、公開、全gateの完了は主張しない。
