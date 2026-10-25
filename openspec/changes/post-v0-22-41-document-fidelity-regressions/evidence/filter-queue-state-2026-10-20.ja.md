# フィルター候補飽和の旧未対応表示を訂正

## 結論

PR346のP2 comment4156681273は、KatanAの独立未実装ではなく、修復済みの指摘が古い`[ ]`のまま残っていた。対象行を実装済み`[/]`へ訂正する。製品全工程・現行HEAD全coverage・全OS・配布受入を完了にするものではない。

## 一次根拠

- `d0e8e910`は候補を最大通常入力16件とは別に有界保持し、通常操作のeviction対象から除外する修復。`63279a37`はClear/Applyも保護する後続修復。主査が両方を現HEADの祖先と確認した。
- 現`render_support.rs`の`push_user`、`evict_oldest_user_if_full`、`is_disposable`を実読。候補の置換は最新要求だけで、通常入力16件の上限、Clear/Applyの順序、満杯時の明示拒否を維持している。
- `render_support_queue_tests.rs`に候補保持、再要求、coalescing、Clear/Apply順序、有界性の回帰があり、`render_support_filter_worker_tests.rs`に実XLSX workerへの到達とLoading解除がある。
- fresh全レビュー取得の`tmp/oct20-85c-current-review.json`で、該当thread `PRRT_kwDORm09y86n_eop`はresolved=true。返信`PRRC_kwDORm09y8731TAD`はd0e8e910の通常push9b0226ba・RED→GREEN・既存coverage等を記録している。Clear/Apply後続thread `PRRT_kwDORm09y86oyJyb`もresolved=true。旧検査成功は当時の証跡であり現HEAD全検査の代わりではない。

今回の85c01a7d製品shadowは容量中断130だが、成功したlib-bin工程のraw `tmp/oct20-product-85c-nextest-shadow.log`に次の5件の実PASSが保存されている。主査が行2436～2442の個別結果を実読した。部分成功を全shadow成功へ昇格しない。

- `candidate_keeps_coalescing_on_each_side_of_its_ordering_barrier`
- `candidate_preservation_keeps_clear_apply_fifo_order`
- `latest_candidate_supersedes_only_pending_request_at_its_new_position`
- `repeated_candidate_requests_remain_bounded_and_clearable`
- `saturated_navigation_preserves_candidate_and_sixteen_latest_commands`

新たな製品ソース変更・重複Rust検査・上流編集は行っていない。残る現行graphの全ゲートと配布受入は既存タスクで追跡する。
