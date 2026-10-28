## ADDED Requirements

### Requirement: focus中のPDF/PPTX surfaceは範囲内の左右ページ移動を受け付ける

KatanA MUST PDF/PPTX document surfaceがkeyboard focusを所有している間に限り、modifierのないleft/right arrow keyを既存のprevious/next page commandへ転送する。commandは先頭pageの前や最終pageの後へ進めてはならない。他format、modifier、または別widgetが所有するfocusを横取りしてはならない。

#### Scenario: focus中のPDF/PPTXが範囲内arrow keyを受け付ける

- **WHEN** focus中のPDF/PPTX surfaceがpage範囲内でmodifierのないleft/right arrow keyを受け取る
- **THEN** KatanAが対応する既存previous/next page commandをdispatchする

#### Scenario: 別focus・他format・modifier・境界では移動しない

- **WHEN** 別widgetがfocusを持つ、active formatがPDF/PPTXでない、modifierが押されている、または現在pageが該当する境界にある
- **THEN** KatanAはpage navigation commandをdispatchしない
