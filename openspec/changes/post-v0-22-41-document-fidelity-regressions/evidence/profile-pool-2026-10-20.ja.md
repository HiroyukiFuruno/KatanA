# shadow profile pool の限定修復（2026-10-20）

## 実装

- `LLVM_PROFILE_FILE_NAME`をUUID・phase名付き`%4m.profraw`へ変更した。
- UUID、phase分離、通常coverageのRUSTFLAGS、false policyは変更していない。
- PID依存の`%p`を除去し、同一targetを使うseed/retryでもprofile名をrunごとに分離する。

## 回帰

- 初回は旧期待値を検出してRED。期待値修正後、`just test-nextest-shadow-contract`は42件PASS、1.918秒、actual0。
- c実験の12 old/new runは各actual0。profileは12 files/4896 bytesから4 files/1632 bytesへ縮小し、counter merge完全一致を確認した。これはtiny実験の観測であり製品全体の品質・速度改善を主張しない。
- 由来未確定のroot profileは削除せず、`tmp/oct20-profile-pool-probe-20261020/unattributed-default-11448.profraw`へ移動しSHAを保存した。

## 残件

製品shadowは容量急減によりactual130で中断した。修復後の製品全4phase、通常coverage、全OS、配布受入は未完了である。
