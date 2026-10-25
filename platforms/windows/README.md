# Windows (ARM) 検証環境構築ガイド

macOS (Apple Silicon) 上で、Windows 11 を仮想的に動かし、KatanA の動作検証を行うためのガイドです。

## 前提事項

Windows 11 には ARM 版と x86 版が存在します。Apple Silicon 上の **UTM** で Windows 11 ARM を使用する場合、x86_64 向けの `KatanA.msi` は Windows の x64 エミュレーション上で検証する対象になります。起動・インストーラー・IME等の確認には使えますが、動作成功は実行するまで未確認です。

Windows ARM上での結果を、x64 Windowsネイティブでの動作・cold/warm時間・cache効果の証明に代用しないでください。検証区分は [クロスプラットフォーム検証方針](../../docs/cross-platform-validation.ja.md) に従います。

## 検証結果の区分

| 実行環境 | 確認できる範囲 | 代用できないもの |
| --- | --- | --- |
| `ci/` のLinuxコンテナとcargo-xwin | Windows向けの型・テストコードのコンパイル | Windowsでの実行、UI、cold/warm、packaged起動 |
| Windows 11 ARM上のx64エミュレーション | そのOS・実行方式での起動、インストーラー、IME等 | x64 Windowsネイティブの速度・cache効果・全機種の互換性 |
| Windows x64実機・VM・GitHub Actions | 実際に実行した同一環境・対象の検査 | 未実行のGUI操作や別環境の結果 |

Windows実行結果には、検証したsource SHA・両lockfile・toolchain・OS/CPU・実行方式、コマンド、開始終了時刻、実exit、保持ログを記録します。cacheのcold/warm比較は同一入力・環境で行い、cache hitをテスト成功にせず、通常検査を維持します。VMでの成功を物理PCでの成功とも表現しません。

既存Windows環境がない場合、新規VM構築は以下のOS導入・初期設定を必要とします。Windowsの利用条件や本人のセットアップ操作を代行せず、未構築を実測済みと扱わないでください。既存CIを使う場合も、Draftで重いCIを起動せず、レビュー対応後のReady工程で実行します。

## 手順の詳細

`./platforms/windows/init.sh` を実行して UTM と CrystalFetch のインストールを終えた後、以下の手順でVMを構築します。

### Step 1: ISO イメージのダウンロード

1. `CrystalFetch` を起動する
2. `Windows 11` -> `Apple Silicon` を選択する
3. 言語は `Japanese` を推奨
4. エディションを選択し、「ダウンロード」を実行する（数GBあるため時間がかかります）

### Step 2: UTM での仮想マシン作成

1. `UTM` を起動する
2. 「＋新規仮想マシンを作成」をクリック
3. 「仮想化 (Virtualize)」を選択（エミュレートは遅いため選ばないこと）
4. 「Windows」を選択
5. "Windows をインストール..." にチェックを入れ、`Boot ISO Image` として、先ほどダウンロードした ISO を選択
6. ハードウェアの設定（メモリ 4096MB 以上推奨）を行い、完了する
7. VMを再生（起動ボタン）し、Windows の初期セットアップを完了させる
    - *（ネットワークがないと進めない場合がありますが、UTMのGuest Toolsを後でインポートする指示が出たら従ってください）*

### Step 3: KatanA の検証

Windows のデスクトップが表示されたら、Edge ブラウザを開きます。

1. GitHub の KatanA リポジトリ・リリースページへアクセス
2. 対象バージョンの `KatanA.msi` をダウンロード
3. ダブルクリックしてインストール
4. KatanA を起動し、以下のスモークテストを実施します：
    - 起動してウィンドウが表示されるか
    - 日本語 IME がオンになり、文字入力ができるか
    - 日本語フォントが化けて豆腐（□）になっていないか

この3項目の成功は、そのWindows環境での基本スモーク確認です。全体の受入完了ではありません。配布物のarchitecture・sidecar、DPI・クリップボード・メニュー等を含む最終確認は、上記のクロスプラットフォーム検証方針と対象リリースのOpenSpecに従って別途回収してください。
