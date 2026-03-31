## Context

現在の split scroll sync は `ScrollState.fraction` と `ScrollSource` を共有し、editor / preview がそれぞれ自前の補間で相手側の scroll offset を求めています。コードを確認すると、今回の不具合は少なくとも次の 3 点に分解できます。

1. `crates/katana-ui/src/views/panels/preview.rs` は editor から preview へ同期する際、現在フレームの preview 最大 scroll 量ではなく `prev_max_scroll` を使って target offset を決めている。末尾付近ではこの差分がそのまま追従不足になる。
2. 同ファイルは heading anchor から補間点列を毎回組み立てるが、forward / reverse で別々に補間し、最後の見出し以降は単一の終端点に潰している。そのため round-trip で同じ論理位置へ戻る保証がない。
3. `crates/katana-ui/src/views/panels/editor/logic.rs::update_scroll_sync()` は consumer 側が同期を受けた直後でも現在 offset から共有 `fraction` を更新しうる。つまり「適用された同期位置」と「ユーザー入力による新しい scroll」を区別しておらず、微小差分が逆方向同期として再発火する。

既存テストは `source` が最終的に `Neither` に戻ることは見ているが、末尾まで届くこと、長い tail 区間を含むこと、往復同期が収束することは固定化していません。

## Goals / Non-Goals

**Goals:**

- split mode の既定スクロール同期で editor / preview の双方が文書末尾まで到達できるようにする
- 最後の見出し以降に長い本文があっても tail 区間の同期を崩さない
- 一方の pane が同期を受けた直後に逆方向の corrective scroll を出さず、数フレームで安定収束するようにする
- heading がない文書や anchor が少ない文書でも fallback で破綻しないようにする
- vertical / horizontal split の両方で再発防止テストを追加する

**Non-Goals:**

- PreviewOnly / CodeOnly mode のスクロール挙動全体の再設計
- table of contents や hover highlight の仕様変更
- preview refresh、diagram rerender、外部ファイル再読込の修正
- scroll sync 無効時の個別 pane スクロール挙動の変更

## Decisions

### 1. 共有 state は単一 `fraction` ではなく、segment-aware な論理位置を持つ

現在の `fraction` は「相手 pane へどの位置を適用したいか」を表すには情報が粗すぎます。特に heading anchor を使った補間では、どの segment にいるかが消えるため round-trip で drift しやすいです。  
そのため shared scroll state は、少なくとも「どの補間区間か」と「その区間内の進捗」を表せる logical position を持つ前提に切り替えます。内部表現は `segment_index + progress` でも、同等の概念を表現できる構造でもよいですが、他の AI が bare fraction を再利用してよいとは解釈できないようにします。

- 採用理由:
  - editor -> preview -> editor の往復で同じ論理位置へ戻しやすい
  - 最後の見出し以降の tail 区間を明示的な最終 segment として扱える
  - heading がない文書でも「start -> EOF」の 1 segment として表現できる
- 代替案:
  - 既存の shared `fraction` を維持する: どの segment にいるかが失われ、末尾と収束の不具合が残りやすいため不採用

### 2. 同期写像は editor / preview で別実装せず、1つの共有 mapper に寄せる

現在は preview 側が forward / reverse の両方をその場で組み立てていますが、これでは補間点や終端処理の差異を生みやすいです。  
同期写像は shared utility として切り出し、editor geometry、preview geometry、heading anchors から 1 つの segment table を作り、それを両方向で再利用します。

この table は最低でも以下を含みます。

- 先頭点 `(0, 0)`
- heading anchor に基づく対応点
- 文書末尾を表す EOF anchor
- heading が 0 件のときの fallback segment

- 採用理由:
  - forward / reverse で同じ対応表を使える
  - 末尾区間を implicit な終端補正ではなく明示的な segment として扱える
  - 他の AI が修正箇所を `preview.rs` の片側だけと誤解しにくい
- 代替案:
  - 現状どおり preview 側で都度 point 配列を組み立てる: 対称性と検証性が弱いため不採用

### 3. consumer 側の write-back には echo suppression を入れる

同期で scroll を適用された pane は、その直後のフレームで自分の offset を新しいユーザー入力として報告してはいけません。  
そのため scroll state には「最後に適用した target pane / target offset / generation」相当の情報を持たせ、consumer 側はそれと実測 offset の差が pixel epsilon 以内である限り、新しい source として再送しないようにします。

dead zone も fraction ベースではなく pixel ベースの比較を優先します。fraction ベースの閾値は文書長に依存して挙動がぶれやすく、末尾での小さな drift を隠したり、逆に長文書で大きすぎる差分を許したりします。

- 採用理由:
  - 「同期を受けたことによる scroll」と「新しいユーザー scroll」を分離できる
  - 上下にガタつく往復同期を止められる
  - pane サイズや全文長による dead zone のぶれを減らせる
- 代替案:
  - 既存の `source = Neither` だけで抑える: 次フレームの write-back を止められないため不採用
  - fraction dead zone を大きくする: 症状を隠すだけで末尾精度が悪化するため不採用

### 4. preview への適用は「現在有効な geometry snapshot」に対して行う

現状の `prev_max_scroll` だけを使う方式では、pane resize や async render 後の高さ変化に追随しづらく、末尾で不足または overshoot が起きます。  
そのため同期適用は `preview_max` だけでなく、対応する heading anchor snapshot と一緒に扱います。最新 snapshot が未確定なフレームでは、最後に安定して観測された snapshot を使って適用し、render 後に geometry が変わった場合は 1 回だけ再評価できる状態を残します。

- 採用理由:
  - `prev_max_scroll` 単体よりも末尾位置の再現性が高い
  - diagram / image / panel resize による geometry 変化を扱いやすい
- 代替案:
  - 毎回 `prev_max_scroll` だけで適用する: 末尾不整合が残るため不採用

### 5. 回帰テストは「末尾到達」と「収束」の両方を固定化する

今回のバグは単に source flag が戻るかどうかでは不十分です。  
テストでは最低でも以下を固定化します。

- editor -> preview で末尾まで達する
- preview -> editor で末尾まで達する
- 最後の heading 以降に長い tail がある文書でも末尾同期が崩れない
- heading がない文書でも fallback で末尾同期できる
- vertical / horizontal split の両方で数フレーム後に offset が収束し、source が往復反転し続けない

## Risks / Trade-offs

- [Risk] shared scroll state が複雑になり、既存の簡単な `fraction` ベース実装より理解コストが上がる
  - Mitigation: mapper と state responsibility を module 境界で分離し、pane 側は「観測」と「適用」に専念させる

- [Risk] pixel epsilon の設定が小さすぎると jitter が残り、大きすぎると同期精度が落ちる
  - Mitigation: テストで tail / convergence を固定し、閾値は fraction ではなく pixel ベースで調整する

- [Risk] preview geometry が async render で変化する文書では、1 フレーム遅れの再同期が入る
  - Mitigation: geometry change を検出したときだけ 1 回再評価し、連続 write-back は suppress する

- [Risk] heading anchor が sparse な文書では segment が粗くなる
  - Mitigation: start/EOF fallback を必須にし、heading 0 件でも full-range sync を保証する

## Migration Plan

1. 既存 `ScrollState` と scroll sync helper の責務を棚卸しし、shared mapper と logical position の契約を追加する
2. editor / preview 両 pane を shared mapper 利用へ寄せ、consumer write-back suppression を導入する
3. geometry snapshot と EOF anchor を含む tail mapping を実装する
4. vertical / horizontal split の regression test に末尾到達と収束判定を追加する

## Open Questions

- pixel epsilon の既定値を何 px にするかは実装時にテストで詰めるが、契約としては「同期待ち offset に十分近い間は write-back しない」を優先する
