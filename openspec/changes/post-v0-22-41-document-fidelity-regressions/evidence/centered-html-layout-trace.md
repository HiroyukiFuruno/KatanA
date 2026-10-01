# Centered HTML の割当高・間隔の診断

## 結論

42px は画像や本文そのものの高さではなく、CommonMark の親行と開始・終了改行を含むアンカー高。
見出し描画下端から次の centered 本文描画上端までの14pxは `本文行高21 − 上側調整7`。
`6+8` や badge 固有の26.5pxによる補償ではない。レイアウト定数、参照PNG、閾値95は変更していない。

## 同一フレームの実測

診断出力は `target/typography-host.tyoRkw/output-html-layout-trace/capture/`。
`stderr.log` の `frame=743` と `sample-typography-preview-geometry.json` の
`ui_pass_frame_nr=completed_ui_frame_nr=743` を main agent が直接照合した。
本文行高 B=21、item_spacing.y=3、設定font size=14、親は横向き・中央揃え・折返し。
座標は論理px。vertical 内部のカーソル高infは縦向きレイアウトの未制限cursorであり描画高ではない。

| HTML | anchor開始S | callback entry | vertical開始 | top−7後 | renderer後cursor | bottom−3後 | vertical response Y | parent callback後cursor | newline後 | anchor高 |
|---|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|
| H1 | 1242 | 1263 | 1252.5 | 1245.5 | 1280 | 1277 | 1245.5–1277 | 1261.25 | 1287.5 | 45.5 |
| description | 1287.5 | 1308.5 | 1298 | 1291 | 1315 | 1312 | 1291–1312 | 1305 | 1329.5 | 42 |
| badges | 1329.5 | 1350.5 | 1340 | 1333 | 1364 | 1361 | 1333–1361 | 1347 | 1371.5 | 42 |
| language | 1371.5 | 1392.5 | 1382 | 1375 | 1406 | 1403 | 1375–1403 | 1389 | 1413.5 | 42 |

H1 parent cursorはログの小数1桁出力では1261.2。表の1261.25はresponse上下端の中央から導出。
画像ResponseはLicense `(557.5,1337)-(639.5,1357)`、CI `(643.5,1337)-(717.5,1357)`。
画像の高さは20、centered child行は28であり、アンカー高42とは異なる。

## ソース導出

- `egui_commonmark/src/parsers/pulldown.rs`: block anchor開始は `try_insert_start` より前に保存。
- `egui_commonmark_backend/src/elements.rs`: newlineは `ui.label("\n")`。
- egui 0.36.1 `widgets/label.rs`: 横折返しラベルは `first_row_min_height=cursor.height()` を使い各行をallocateする。
- `preview_pane/html.rs`: verticalを作り、上−7、renderer、下−3。下−3は描画後spacing.y=3を戻すが、既存min_rectを縮めない。
- 親行の以前の下端は `S+1.5B`。新しい上端は `S+B/2−7`。renderer高さHとの合成で親行高は `max(B+7,H)`。
- 終了改行後の進行量は `max(B+7,H)+B−7 = max(28,H)+14`。
- H1のH=31.5なら45.5、descriptionのH=21とbadge/languageのH=28はいずれも42。
- H1下端1277からdescription上端1291の差は14で、`B−7` と一致。

## 保全・検証

既存requestをDEBUG=trueで実行し、runner buildとcaptureは成功。mainのscoped diff checkも成功。
新full PNG SHA-256は `2171d6f263be8abda85529bfaec426f51d546999fb4677ec9ab878ca4e0b4776` で旧v2と完全一致。
旧runner SHA `3367b4720c2de6929493685035d5d5d2a3dfa092416d19b35f849ae4fd94278c` は
`output-html-layout-trace/katana-screenshot-before-html-trace` に保全。
新runner SHA `17d5ac7f4bc53cdf22c59fe7039a457cc262e6ac894288e66f33b1342345f2b7`。
fixture SHAはformal/snapshotとも `489360a81d60af20d67f8ea47e251732a194983431ea1ab261627490cc9009e1`。
これは既存候補依存によるin-process診断であり、公開registry-onlyの最終受入・リリース証拠ではない。
