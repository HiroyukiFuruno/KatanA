# Windowsの診断分離契約と文字コード

## 実際の失敗

[CI 40452429926のWindows job](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40452429926/job/122018009305)、HEAD `6f2fa8b905be03d34efd1a4541c6cdc45e79bcef` で、`Verify CI resource contract` の新しい6件中5件が失敗した。Python 3.14.9の既定cp1252で日本語を含むUTF-8のworkflow・justfile・Rustソースを読み、`UnicodeDecodeError` が発生した。後続の本体テストとheadless検査は未実行であり、成果物未作成によるupload失敗を別の本体障害とは判定しない。

GitHubコネクターから回収したログの抜粋は `tmp/oct27-6f2fa8b9-windows-encoding-failure.json`。ローカルとLinuxの成功をWindowsの成功へ流用しない。

## 修復と回帰

`scripts/ci/test_native_profiler_job_contract.py` の6つの `Path.read_text` へ `encoding="utf-8"` を明示する。製品ソース、workflow、専用native test、元のhash・PID/Thread・5秒、Windows200ms×10、全OS一括公開条件は変更しない。

新しい単体回帰は、実際の6契約を `io.text_encoding` の既定だけcp1252へ置き換えた条件で実行する。入力ファイル・契約結果を偽装せず、明示encodingはそのまま渡す。`-k` で単独実行しても6件を列挙し、空suiteを合格にしない。

- 旧読込方法＋新回帰の単独実行: exit1、既存5件の同じdecode失敗を検出。`tmp/oct27-native-contract-encoding-red.log`。
- 修復後の既存6件＋新回帰: 7件成功。`tmp/oct27-native-contract-encoding-green.log`。
- `git diff --check`: 成功。

実Windows CI、新HEADのcurrent review、全品質条件と公開は通常commit/push後に回収する。この記録はそれらの成功証明ではない。
