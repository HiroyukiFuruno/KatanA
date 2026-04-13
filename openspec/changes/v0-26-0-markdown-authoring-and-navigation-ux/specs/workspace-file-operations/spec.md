## ADDED Requirements

### Requirement: user は単一 file を 2 つのモードで開けなければならない

システムは、user が single file open を実行したとき、「temporary workspace として開く」モードと「現在の workspace session で開く」モードを提供しなければならない（SHALL）。

#### Scenario: temporary workspace として単一 file を開く

- **WHEN** user が single file open で temporary workspace モードを選択する
- **THEN** system は system icon と label を持つ一意な temporary workspace context を生成する
- **THEN** system は選択した file をその context の active document として開く
- **THEN** temporary workspace は persisted workspace history へ保存されない

#### Scenario: 現在の workspace session で単一 file を開く

- **WHEN** user が single file open で current workspace session モードを選択する
- **THEN** system は現在の workspace root を置き換えずに file を tab として開く
- **THEN** 開いた file は active document になる

### Requirement: 外部からの file drag-and-drop open を受け付けなければならない

システムは、アプリケーション外部から drop された file を受け付け、既定では現在の workspace session で開かなければならない（SHALL）。

#### Scenario: 現在の workspace session へ外部 file を drop する

- **WHEN** user が外部 file をアプリケーションへ drop し、現在の workspace session が存在する
- **THEN** system はその file を現在の workspace session で開く
- **THEN** drop された file は active document になる

#### Scenario: 現在の workspace session が無い場合は temporary workspace へフォールバックする

- **WHEN** user が外部 file をアプリケーションへ drop し、現在の workspace session が存在しない
- **THEN** system は temporary workspace を生成して file を開く

### Requirement: explorer から tab strip への drop は append と positioned insert を区別しなければならない

システムは、explorer item を tab strip へ drop したとき、casual drop と precise drop を区別して document open 挙動を変えなければならない（SHALL）。

#### Scenario: casual drop は末尾へ追加して active にする

- **WHEN** user が explorer item を tab strip の余白または末尾領域へ casual に drop する
- **THEN** system は document を tab strip の末尾へ追加する
- **THEN** 追加した document を active tab にする

#### Scenario: precise drop は指定位置へ temporary tab として挿入する

- **WHEN** user が explorer item を tab 間インジケーター近傍へ precise に drop する
- **THEN** system は指定位置へ document を挿入する
- **THEN** 挿入した document は temporary tab として扱われる

#### Scenario: 既存 tab group への drop を許可する

- **WHEN** user が explorer item を開かれている tab group の受け入れ位置へ drop する
- **THEN** system はその document を target tab group 内へ追加する

### Requirement: explorer 内 drag-and-drop move は confirmation setting を持たなければならない

システムは、explorer 内で file または directory を drag-and-drop したとき、target directory への move を行えなければならず、confirmation の要否を settings で切り替えられなければならない（SHALL）。

#### Scenario: confirmation default-on で move を確認する

- **WHEN** user が explorer 内で file または directory を別 directory へ drop し、move confirmation setting が enabled である
- **THEN** system は `xx を yyy/zzz に移動しますか？` 相当の確認を表示する
- **THEN** user が承認した場合にのみ move を実行する

#### Scenario: confirmation disabled では直ちに move する

- **WHEN** user が explorer 内で file または directory を別 directory へ drop し、move confirmation setting が disabled である
- **THEN** system は追加確認なしで move を実行する
