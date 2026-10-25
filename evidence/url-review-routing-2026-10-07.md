# URL完了先と形式別タブのレビュー回帰

対象: PR346、公開HEAD `04e596f03c11b9b3cb0206ffc41700b789d81ccd` のレビュー5431243615本文P2二件。

## 変更と検証

- 閉じた文書のpending応答を実close時に取消し、pollでも閉鎖済みtargetを拒否する。dirty確認中・まだ開いているpinned文書・targetを持たないユーザー入力は保持する。同pathを再openしても取消済み旧応答を適用しない。
- HTMLからPDF/Officeへ移る場合、検出形式に合うvirtual pathへタブを移行する。順序・pin・split・groupを保持し、refreshの取得URLを新identityへ接続する。未保存HTMLは元タブを保持し別binaryタブを開く。既存binary targetとの衝突では文書を複製しない。
- 元製品分岐で同回帰を実行: `tmp/url-review-old-product-red.log`、session90844 actual101、14PASS/7assertFAIL。先行24017のE0425は作業中の誤sender名によるcompile失敗であり製品REDではない。
- 最初の候補34438は22PASS/1FAIL。UTが既存ForceCloseのpin仕様を誤認していたため、開いたpinned文書のcleanup保持を検証するよう訂正。製品の強制close仕様は変更していない。
- 最終候補37652: `tmp/url-review-current-green-final.log`、actual0、URL全23PASS。実worker入口で既存HTTP/redirect/manual reloadの回帰も実行した。
- 公式AST/impacted Clippy/fmt-check3159: `tmp/url-review-current-static.log`、actual0、AST23PASS・strict Clippy・fmt成功。`git diff --check` actual0。

## 証明しない事項

形式移行UTのbinary bytesはintake/identity分類用であり、合法コンテナの表示・本人実機確認・packaged全OS受入・95点の証明ではない。新HEADの通常push hook、全CI/current review、coverage、配布成果物と公開後処理は別途必要。上流KDV/KRR不備は次期の既存Issueで扱う。

レビュー本文の指摘には対応するinline threadが存在しない。修正公開後に本文の各指摘へ根拠を返信し、全レビュー本文とthreadを再取得する。manual-target方針二件の未決選択とは別である。
