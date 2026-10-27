# 配布証跡ツールの Pillow 更新

## 対象と今回の判断

[Issue 374](https://github.com/HiroyukiFuruno/KatanA/issues/374) で追跡する。通常 push 時の GitHub の警告について全ページの open alerts を取得したところ、41 件すべてが `scripts/release/evidence-requirements.txt` の Pillow に関するものだった。critical が 5 件、high が 23 件、medium が 13 件。要求版は 12.3.0、各 alert が示す最初の修正版は 13.0.0 だった。

製品本体の Rust 依存とは別の Python 証跡ツールである。件数や重要度だけで KatanA 本体の実影響や、新たな公開停止条件を作らない。一方、KatanA が所有する依存なので、公式修正版の API 互換と既存検査を確認して更新する。KDV/KRR の公開待ちや新しい期限付き例外へ置き換えない。

公式の [Pillow 13.0.0 公開ノート](https://pillow.readthedocs.io/en/stable/releasenotes/13.0.0.html) を確認した。版の出所、対象 API、画像入力の出所、既存の契約検査結果は、それぞれ別の証拠として記録する。

## 公開版と互換性の検証

PyPI の 13.0.0 metadata と macOS arm64 / CPython 3.14 wheel を照合した。非 yanked で、ダウンロードした wheel の SHA-256 は公開 metadata の `d9b0a2bee3f3e03e9f29057aedb807dbf1e1d96ee1b9d0f422d9db6c8583271d` と一致した。隔離した CPython 3.14.3 環境へ導入し、Pillow の実 import 版 13.0.0 と `pip check` の成功を確認した。必要な Python は 3.11 以上で、グローバルな Python 環境は変更していない。

変更は要求版の 1 行だけで、既存の API や受入基準を変更していない。新環境で PNG・geometry・証跡の契約 55 件、公開 gate の契約 35 件、公開 artifact identity の契約 9 件、release flow の契約 20 件、計 119 件が成功した。Office の既存 source fixture に含まれる 640×360 の PNG も、production の `verify_render_artifact` と decode、load、RGB 変換、ImageStat の各入口で成功した。画像内容は公開しない。

この画像は source fixture であり、新たな公開配布版の画像を検査した証明ではない。現在の GitHub default branch の警告がすべて閉じられたという主張もしない。通常 commit、review、必須 CI と公開 source への統合は後続工程として維持する。元の警告一覧は `tmp/oct27-dependabot-open-alert-inventory.json`、PyPI artifact と実検査の receipt は `tmp/oct27-pillow13-validation/receipt.json` に保存した。

PNG の signature、IHDR、geometry は先に確認するが、formats allowlist のない `Image.open` の後に画像形式を確認し、artifact の SHA は decode 後に照合している。したがって正常な証跡が PNG であることと、細工された入力で他 plugin を通らないことは同義ではない。入力の出所と hash の束縛を保持しつつ、他形式や polyglot の到達性を断定的に排除しない。各 advisory が実際に成立したとも断定しない。既存の ZIP/Zstd の監査と Issue 354 は別の範囲として維持する。
