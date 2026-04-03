## MODIFIED Requirements

### Requirement: アプリケーションアイコン

Katana Desktop のアプリケーションアイコンは、対応 desktop OS の window chrome、task switcher、dock / taskbar 等の OS surface から識別できる状態で表示されなければならない（MUST）。

#### Scenario: macOS の Dock アイコンの表示

- **WHEN** Katana Desktop が macOS 上で起動される
- **THEN** macOS の Dock に Katana Desktop のアイコンが表示される

#### Scenario: Windows のアイコン表示

- **WHEN** Katana Desktop が Windows 上で起動される
- **THEN** window または taskbar から Katana Desktop のアイコンを識別できる

#### Scenario: Linux のアイコン表示

- **WHEN** Katana Desktop が Linux 上で起動される
- **THEN** window または task switcher から Katana Desktop のアイコンを識別できる

#### Scenario: アイコンのデザイン

- **WHEN** アイコンが表示される
- **THEN** 「刀（katana）」をモチーフとした識別性の高いデザインである
