# 通常検査の実行環境記録の修復

## 対象

Issue #343の通常test・coverage recorderは、実子へ渡す環境と異なる親環境でCargo/Rustc版を取得していた。`native_test_run_identity.py`の版取得を実子と同じcwd・環境へ揃えた。Cargoは既存のliteral `cargo`を維持し、未使用の`CARGO`環境変数を新たな実行経路へ昇格させない。Rustcは明示`RUSTC`を単一の実行ファイルとして扱い、未設定時だけ`rustc`を使う。

取得失敗は従来どおり`None`という診断値であり、実検査を省略したり成功へ置換したりしない。schema、raw child exit、`complete=false`、`reuse_authorized=false`は変更していない。製品コード・通常test選択・coverage・CI・閾値は不変。

## 検証

- 主査の対象discoverは17件PASS、3.537秒、実session70604/exit0。rawは`tmp/oct20-legacy-identity-target-tests.log`。
- 実PATH上のCargo、空白を含むRUSTC path、cwd、RUSTFLAGS、RUSTUP_TOOLCHAINを識別する回帰を追加。`CARGO`に無関係な文字列を入れてもliteral Cargoを使用する。
- 実tiny Cargo fixtureを既存`record-test-invocation.py`から起動し、不存在RUSTCの診断値が`None`でもCargo自体の失敗101を記録し、manifestも101・complete/reuse falseであることを同時検証した。単に別のPython子を実行するだけの初稿は撤去した。
- 既存の実Cargo成功・失敗とlisting契約も同じ対象実行で成功。これは製品全Rust・全OS・packaged受入の代替ではない。

## 製品shadowとの区別

先行85c01a7dの製品shadowは、worker build0、workspace-lib-binの実1770PASS・既存2skipとphase exit0まで回収した。後続integration compile中に空き約3.3GiBとなり、担当が自分のrunnerを中断してsession4768/exit130となった。最終`shadow.json`はなく、全工程成功・coverage report・実行同等性・重複削減は未証明。

rawは`tmp/oct20-product-85c-nextest-shadow.log`、部分receiptは`tmp/nextest-shadow/product-85c01a7d-20261020`。worker623.776秒、lib-bin工程全体783.175秒、test実行90.974秒を混同しない。KDV担当のidle cache保護回収13.3GiB/actual0後、主査の空き容量は16GiB。KRRの稼働中buildを消すglobal cleanは行わず、次の重検査枠をKDVへ返却した。

再実行は既存生成物を移動して絶対pathを壊したり、古いPASSを新しい検証結果へ流用したりせず計画する。残る製品shadowと元Issue #343のDoDは未完了のまま追跡する。
