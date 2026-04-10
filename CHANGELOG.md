# Changelog

All notable changes to KatanA Desktop. This file records the major changes to KatanA Desktop.

## [0.18.4] - 2026-04-10 (UTC)

### 🚀 Features

- **Windows distribution stability:** Centralized command execution logic for Windows to prevent console windows from popping up during background tasks such as diagram rendering and application updates.

### ✨ Improvements

- **PlantUML Download Resilience:** Implemented a robust download mechanism for PlantUML with an automatic PowerShell fallback on Windows, ensuring compatibility even when `curl` is unavailable.

### 🐛 Bug Fixes

- **Mermaid CLI (mmdc) on Windows:** Suppressed unnecessary console windows during Mermaid diagram rendering on Windows.
- **Silent Background Updates:** Fixed an issue where a PowerShell window would briefly appear during the application update process on Windows.

### 🔧 System

- **Headless-by-Default enforcement:** All external process invocations across the codebase (diagram rendering, file reveal, update cleanup, download) are now routed exclusively through `ProcessService`, making the `CREATE_NO_WINDOW` policy on Windows both mandatory and auditable.
- **AST Linter rule `no-direct-process-command`:** Added a compile-time linting rule that prohibits direct use of `std::process::Command::new` outside of `ProcessService`. This prevents future regressions where background processes could inadvertently pop up console windows on Windows.

## [0.18.3] - 2026-04-10 09:52:37 (UTC)

### ✨ Improvements

- Improved the resolution logic of the Mermaid rendering engine to be more robust across multiple platforms.

### 🐛 Bug Fixes

- Fixed an issue where the terms of service modal was incorrectly displayed every time the app was updated.
- Fixed an issue where the application would freeze and become unresponsive during the first launch.
- Fixed an issue where the demo language switching links were incorrectly resolved as relative paths.

### 🔧 System

- Improved internal code structure for greater maintainability and stability.

## [0.18.2] - 2026-04-10 07:25:00 (UTC)

### ✨ Improvements

- Enabled manual diagram re-rendering for Demo files via the Refresh button, allowing users to update diagrams without restarting after installing prerequisite tools like Mermaid CLI or Java.
- Added a PlantUML class diagram example to the rendering features demo.
- Updated Windows documentation to feature the portable ZIP version as the recommended installation method.

### 🐛 Bug Fixes

- Fixed an issue where the Terms of Service agreement modal would incorrectly appear on every minor app version update.
- Fixed a bug causing internal Katana:// virtual URLs (opened from tooltips etc.) to be misidentified as local paths, preventing them from opening correctly.
- Enhanced multi-platform support for the Mermaid CLI (mmdc) resolution, ensuring reliable path discovery on Windows and robust cache validation when reloading diagrams after tool installation.
- Addressed an installation failure in the Windows MSI installer where directories under AppData were not properly initialized during per-user installations.

### 🔧 System

- Improved background stability of the testing environment for Windows builds.

## [0.18.1] - 2026-04-09

### 🚀 Features

- **Windows MSI Installer:** KatanA for Windows is now distributed as a native `.msi` package (built via WiX), automatically creating Start Menu and Desktop shortcuts upon installation.
- The built-in auto-updater seamlessly supports smooth background updates for the new `perUser` installed directory.

## [0.18.0] - 2026-04-08 12:15:00 (UTC)

### 🚀 Features

- **Cross-Platform Support:** KatanA now officially supports Windows and Linux (Ubuntu) distributions. Users on these platforms can now download MSI and DEB installer packages directly from the GitHub Releases page.
- **Smart Update Detection:** The built-in updater is now fully OS-aware. It will securely ignore update files that do not match your current operating system, preventing unsupported cross-platform installations.

### ✨ Improvements

- App installation has become much simpler as the external dependency on the `gh` CLI has been completely removed. Communication with GitHub is now performed securely through an internal HTTP adapter.

### 🔧 System

- Improved CI/CD pipelines to construct and distribute stable releases across multiple operating systems automatically.

## [0.17.2] - 2026-04-08 08:45:00 (UTC)

### 🚀 Features

- Expanded Icon Settings to support Advanced Icon Configuration. Users can now assign custom colors or override default icon packs on a per-icon basis, and save them as reusable presets.

### ✨ Improvements

- Reorganized the icon settings panel to group icons systematically by vendor and category, greatly enhancing discoverability.
- Added a fixed action bar to the bottom of the settings screen, providing immediate access to 'Apply' and 'Restore' functions even when scrolling through long lists.

## [0.17.1-1] - 2026-04-08

### 🐛 Fixed

- Resolved an issue with the feather theme where `folder_open.svg` was incorrectly using the `folder_closed` icon.

## [0.17.1] - 2026-04-07 21:47:42 (UTC)

### ✨ Improvements

- Stabilized UI controls during slideshows to ensure magnification elements remain visible and active on diagram blocks.
- Improved hover highlight visibility across all diagram types to ensure unified preview behavior.
- Unified the source design of system view icons (reset view) for a more consistent visual experience.

### 🐛 Bug Fixes

- Fixed a visual clipping issue where Markdown heading highlights failed to extend fully to the edges of the preview pane.

## [0.17.0] - 2026-04-07 19:40:00 (UTC)

### 🚀 Features

- Introduced an "Icon Theme Pack" feature, allowing users to customize the application's icons from a rich selection of 15 built-in themes (including Lucide, Material Symbols, Feather, and more).

### ✨ Improvements

- Ensured robust UI stability by enforcing strict icon synchronization and fixing display anomalies such as placeholder icons for extensionless files.

### 🔧 System

- Enhanced internal asset validation mechanisms to automatically block broken or visually inconsistent SVGs.

## [0.16.10] - 2026-04-07 15:57:00 (UTC)

### 🚀 Features

- Added a "Help -> Demo" menu to explore read-only demonstration documents (Reference Mode), allowing users to safely view sample features without accidentally saving edits.

### 🐛 Bug Fixes

- Fixed a bug where custom task list markers (e.g., `[/]`, `[-]`) were incorrectly rendered as standard unchecked boxes.

## [0.16.9] - 2026-04-07 12:49:00 (UTC)

### 🔧 System

- Improved background rendering stability for preview panes; preview processing now continues in the background even when switching between tabs.

## [0.16.8-3] - 2026-04-07 11:20:00 (UTC)

### 🔧 System

- Others: Internal system updates

## [0.16.8-2] - 2026-04-07 09:03:00 (UTC)

### 🐛 Bug Fixes

- Fixed a regression where background elements (such as the editor or workspace tree) could be inadvertently interacted with while a modal was active.

## [0.16.8-1] - 2026-04-07 04:10:00 (UTC)

### 🐛 Bug Fixes

- Fixed a critical regression where the UI would freeze when opening modals (Workspace Toggle, Settings, Command Palette, Search) due to Z-order interaction conflicts.

## [0.16.8] - 2026-04-07 01:35:00 (UTC)

### ✨ Improvements

- Stabilized hover highlighting and scroll synchronization for rich blocks (such as diagrams, tables, and alerts) in the Markdown preview. Hovering over a rich block smoothly visualizes and maps precisely to its editor source lines. Split-pane scrolling near these blocks now stays perfectly aligned without jittering.

## [0.16.7] - 2026-04-07 01:45:00 (UTC)

### ✨ Improvements

- Refined the visual presentation of alert blocks (Note, Tip, etc.) by adjusting vertical margins and spacing for a more professional layout rhythm.
- Updated the "Caution" alert icon to a dedicated prohibited/caution sign (🚫) for improved visual clarity and consistency.

## [0.16.6] - 2026-04-06 16:08:00 (UTC)

### ✨ Improvements

- Pinned tabs can now be directly unpinned by clicking the pin icon, without using the context menu.

## [0.16.5] - 2026-04-06 15:30:00 (UTC)

### ✨ Improvements

- Improved UI focus behavior: background elements (such as the editor and workspace tree) no longer react to hovers or clicks while a modal, popup, or overlay window is active.

## [0.16.4] - 2026-04-06 14:35:08 (UTC)

### ✨ Improvements

- Overall UI/UX improvements across the system.

## [0.16.3] - 2026-04-03 09:50:00 (UTC)

### 🐛 Bug Fixes

- **UX Resilience**: Relocated workspace tab & group configuration storage from the transient OS Cache directory to the persistent Application Settings directory (`workspaces/*.json`). This ensures that running common disk cleanup tools or OS cache purges will no longer accidentally delete your carefully organized tab groups and pinned tabs.

## [0.16.2] - 2026-04-03 09:08:06 (UTC)

### ✨ Improvements

- Fixed an issue where "mojibake" (character corruption) could occur across different operating systems by replacing OS-dependent emoji icons (e.g., download, hourglass, update badges) with natively rendered, cross-platform SVG icons.
- Enhanced the UI for toggling between the editor and preview panels by introducing context-aware layout icons that dynamically reflect the horizontal or vertical split direction, making layout management more intuitive.

## [0.16.1] - 2026-04-03 05:10:39 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where pinned tabs and tab groups were not successfully restored upon app restart due to inconsistent cached path resolution.

## [0.16.0] - 2026-04-03 03:59:11 (UTC)

### 🚀 Features

- Introduced a fullscreen slideshow viewer for Markdown documents, complete with pagination controls and seamless document activation.

### 🐛 Bug Fixes

- Fixed a backward compatibility issue where legacy cache paths caused workspace tab groups to be lost upon restart.

## [0.15.1] - 2026-04-03 00:10:00 (UTC)

### 🐛 Bug Fixes

- Resolved an issue where tab groups leaked across different workspaces; tab groups are now correctly isolated and persisted per workspace.

## [0.15.0] - 2026-04-02 23:30:14 (UTC)

### ✨ Improvements

- Implemented interactive hover highlights for Markdown tables and accordion (`<details>`) blocks, ensuring the code editor synchronously highlights the exact source lines corresponding to the UI element.

## [0.14.1] - 2026-04-02 15:10:57 (UTC)

### 🐛 Bug Fixes

- Resolved a backwards-compatibility issue during workspace switching where tab states (pins and groups) saved in older versions were discarded and appeared lost upon restoration.

## [0.14.0] - 2026-04-02 14:05:45 (UTC)

### 🚀 Features

- Introduced the global Command Palette (accessible via `Cmd+Shift+P` / `Ctrl+Shift+P`), providing unified access to application commands with a fuzzy-search interface.

### ✨ Improvements

- Enhanced command palette usability with dynamic height adjustment, window resizing support, and automatic focus tracking to keep the active selection visible.
- Increased overall rendering stability by ensuring text with script-like characters gracefully falls back to plain text instead of triggering false-positive rendering errors.

### 🔧 System

- Increased 100% UI localization compliance across all 10 supported languages by integrating translation parameters into deep workspace and file-operation dialogs.

## [0.13.0] - 2026-04-02 10:15:00 (UTC)

### 🚀 Features

- Introduced a realtime "Problems Panel", allowing users to quickly identify and navigate to Markdown authoring issues without leaving the editor canvas.
- Added foundational local Markdown diagnostics, currently catching skipped heading levels and broken relative links, which are calculated in the background when documents are opened or saved.

## [0.12.0] - 2026-04-02 04:45:00 (UTC)

### 🚀 Features

- Introduced an integrated in-document search feature. Users can now search for specific text within the active Markdown file using the search bar embedded in the ViewModeBar, complete with hit counts and jumping between matches.
- Added comprehensive keyboard navigation for the new document search. Pressing the ArrowUp or ArrowDown keys while focused in the search input quickly jumps to the previous or next match.

### ✨ Improvements

- Silenced internal testing framework CLI noise to greatly improve developer application stability and reduce memory consumption during test cycles.

### 🐛 Bug Fixes

- Fixed a crash that occurred when searching for specific multi-byte Japanese characters due to invalid character boundaries.
- Repaired regression on scroll synchronization. Searching and navigating through document matches will now consistently scroll both the editor and preview panels to highlight the correct position.

## [0.11.2] - 2026-04-02 00:30:00 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where diagram cache files were not properly cleared from persistent storage.
- Resolved missing localization keys for the new Tab Groups feature across multiple languages.
- Polished Tab Group UI layouts and resolved various internal linter warnings.

## [0.11.0] - 2026-04-01 11:45:00 (UTC)

### 🚀 Features

- Introduced the "Tab Groups" feature, allowing users to visually organize and consolidate multiple tabs. Groups can be assigned custom names and colors (from a curated palette of 7), and can be fully collapsed to maximize the editor screen area.
- Enhanced accident prevention safeguards for "Pinned" tabs. Pinned files are now fully protected from bulk-close operations such as "Close Others" or "Close All".

### ✨ Improvements

- Upgraded the workspace tab restoration (session persistence) logic on application restart, ensuring that advanced states such as tab group affiliations and pinned statuses are seamlessly retained across sessions.

## [0.10.1] - 2026-04-01 09:35:00 (UTC)

### ✨ Improvements

- Ensured that diagram previews immediately reflect theme and color changes without requiring an application restart.
- Prevented rendering glitches where outdated cached diagrams in the previous color scheme would incorrectly display after switching themes.

## [0.10.0] - 2026-04-01 07:34:00 (UTC)

### ✨ Improvements

- Stabilized UI responsiveness and reduced disk I/O load during typing or frequent file navigation by restructuring the persistent caching strategy.

## [0.9.0] - 2026-04-01 13:40:00 (UTC)

### 🚀 Features

- Introduced a permanent Activity Rail on the left side of the workspace, consolidating the workspace toggle, search modal, and recently opened workspaces into a unified navigation bar.
- Added a new "Flat View" mode for the workspace tree, displaying all files in a single list with workspace-relative paths, bypassing deeply nested directory structures.
- Implemented natural sort algorithm across both Tree and Flat views to ensure versioned directories and files (e.g., `v0-9` vs `v0-11`) are ordered intuitively by humans.

### ✨ Improvements

- Supported drag-and-drop reordering for icons within the Activity Rail, naturally persisting layout preferences to user settings.
- Rearranged the workspace header layout based on user feedback to prioritize readability and group action icons (Expand All, Collapse All, Refresh, Filter) more logically.

## [0.8.11] - 2026-04-01 00:06:36 (UTC)

### ✨ Improvements

- Stabilized synchronization between the editor and preview panels to prevent jittering during scroll, particularly at the bottom of long documents and in documents without headings.

## [0.8.10] - 2026-03-31 22:30:00 (UTC)

### ✨ Improvements

- Optimized the rendering flow when files are updated from external editors, reducing unnecessary flickering in the preview and applying updates faster.
- Improved nested task lists formatting by allowing the entire row to be clicked to toggle checks.

### 🐛 Bug Fixes

- Resolved a visual bug where multiple lines would highlight simultaneously on hover; the highlight is now precisely constrained to the row under the cursor.

## [0.8.9] - 2026-03-31 02:17:00 (UTC)

### 🐛 Bug Fixes

- Completely resolved the issue with Homebrew auto-updater conflicts. The application will now securely untap and detach itself from Homebrew package management silently in the background simply by launching the KatanA app normally.

## [0.8.8-4] - 2026-03-31 01:05:00 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where the native auto-updater silently failed to remove the Homebrew link when updating KatanA from within the app. The auto-updater now correctly identifies Homebrew and untaps the cask upon successful transition.

## [0.8.8-3] - 2026-03-31 00:20:00 (UTC)

### 🔧 System

- Improved foundational stability for the internal update mechanism.

## [0.8.8-2] - 2026-03-31 00:11:32 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where hyphenated version numbers (e.g., `0.8.8-1`) were incorrectly identified as older versions, causing the new release details to not open automatically after an update.

## [0.8.8-1] - 2026-03-30 23:00:00 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where the new version update dialog rendered English markdown text instead of localized strings.

## [0.8.8] - 2026-03-30 14:30:00 (UTC)

### 🔧 System

- Optimized internal dependencies for the Markdown preview engine, improving long-term maintainability and stability.

## [0.8.7] - 2026-03-30 13:25:00 (UTC)

### 🔧 System

- Strengthened internal application components and refined the release automation pipeline to guarantee delivery stability for future updates.

## [0.8.6] - 2026-03-30 10:00:00 (UTC)

### 🔧 System

- Restructured internal UI logic and rendering pipelines to significantly improve stability and maintainability for future enhancements.
- Expanded automated test coverage to ensure better quality assurance across updates.

## [0.8.5] - 2026-03-30 02:35:00 (UTC)

### 🐛 Bug Fixes

- Fixed a UI regression where the left vertical guideline in expandable accordion (details) blocks was missing when opened.

## [0.8.4-1] - 2026-03-30 02:35:00 (UTC)

### 🐛 Bug Fixes

- Fixed a UI regression where the left vertical guideline in expandable accordion (details) blocks was missing when opened.

## [0.8.4] - 2026-03-30 02:00:00 (UTC)

### 🔧 System

- Optimized internal processing and improved the foundational stability of the application for future updates.

## [0.8.3] - 2026-03-28 18:56:00 (UTC)

### ✨ Improvements

- Added a mechanism to safely detach Katana Desktop from Homebrew management during auto-updates, ensuring older versions aren't automatically reinstalled by `brew upgrade`.
- Eliminated CDN caching delays inside the ChangeLog viewer by actively appending cache-busting timestamps during version updates.

### 🐛 Bug Fixes

- Fixed an issue where the ChangeLog tab would be incorrectly restored and cached as a zombie tab upon application restart.

## [0.8.2] - 2026-03-28 18:45:00 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where the ChangeLog Viewer tab would disappear immediately on startup when restoring a previously opened workspace.
- Fixed a bug where the accordion sections for the latest update were collapsed by default due to the application inadvertently forgetting its previous version state.

## [0.8.1] - 2026-03-28 18:05:00 (UTC)

### 🐛 Bug Fixes

- Fixed an issue where the ChangeLog Viewer would not display the latest release notes immediately after an application update due to network caching algorithms.

## [0.8.0] - 2026-03-28 17:29:17 (UTC)

### 🚀 Features

- Introduced an integrated ChangeLog Viewer UI, allowing users to conveniently browse recent application updates and release notes directly within the app.

### ✨ Improvements

- Unified the alignment of icons and text across the application interface, specifically improving the vertical centering within the ChangeLog and navigation tabs for a cleaner look.

### 🔧 System

- Strengthened network error handling and internal test coverage specifically around background data fetching to guarantee future stability.

## [0.7.10] - 2026-03-28 04:31:08 (UTC)

### 🐛 Bug Fixes

- Restored the missing UI Contrast logic, ensuring transparent background colors (like hover and active rows) correctly adapt their visibility against dark themes.

## [0.7.9] - 2026-03-28 02:54:09 (UTC)

### ✨ Improvements

- Redesigned the Custom Themes settings layout for better usability and a cleaner interface.

### 🐛 Bug Fixes

- Improved the rendering of underlined text to fix an issue where underlines were not displayed correctly on certain macOS environments.
- Fixed an issue where list markers, footnotes, and collapsible text did not properly update their colors when the theme was changed.
- Fixed an issue where changing the theme could cause unsaved documents to be unexpectedly reloaded or discarded.
- Unified hover and text selection highlight colors to match the active accent color across all themes.

## [0.7.8] - 2026-03-27 13:03:25 (UTC)

### 🚀 Features

- Introduced a UI contrast slider in the Appearance settings, allowing fine-grained control over visual contrast limits.
- Added a dedicated 'Clear HTTP Cache' button within the System settings tab for on-demand cache directory purging.

### ✨ Improvements

- Vertically centered and right-aligned color swatches in the Custom Themes grid, improving visual consistency and scanability across all settings panes.

### 🔧 System

- Improved internal codebase quality by enforcing stricter translation standards.
- Improved the performance of background analysis tools.

## [0.7.7] - 2026-03-27 08:30:00 (UTC)

### ✨ Improvements

- Optimized hover and current-line background transparency across all dark themes to improve text visibility.
- Fixed a visual glitch where semi-transparent highlights would appear overly bright or washed out.
- Overhauled the theme customization system to allow detailed adjustments for system elements, code blocks, and preview areas independently.

### 🐛 Bug Fixes

- Resolved an error that could occur when clearing the image cache on macOS.

### 🔧 System

- Eliminated hardcoded color values from the application to improve maintainability and theme stability.

## [0.7.6] - 2026-03-26 21:20:00 (UTC)

### 🚀 Features

- Provide granular control to adjust the auto-save interval with 0.1-second precision.
- Implement toggles for hiding file extensions and add a fully functioning list-style UI for managing scan exclusion paths easily.
- Support complete lifecycle actions for custom themes, including named saving, duplication, and explicit deletion.

### ✨ Improvements

- Standardized all dropdown menus across the application for improved appearance and hover interactions.
- Transition scroll synchronization configuration to an intuitive toggle switch and streamline layout order.
- Rearrange custom theme color pickers vertically to dramatically improve visibility and usability within constrained side panels.

### 🐛 Bug Fixes

- Fixed potential translation errors that could occur when switching application languages.
- Fixed minor layout glitches and formatting issues.

### 🔧 System

- Expanded UI and integration tests to improve application stability.

## [0.7.5] - 2026-03-26 05:43:00 (UTC)

### 🐛 Bug Fixes

- Resolve an issue where closing a tab would fail to load and render the preview for the newly activated tab if it was previously opened in the background without being rendered.

## [0.7.4] - 2026-03-26 05:15:00 (UTC)

### 🚀 Features

- Integrate a visual progress bar during the update download phase and correct the GitHub API release URL.

### ✨ Improvements

- Redesign link rows to display aligned icons (e.g., External Link) and make the build version commit hash clickable.
- Conditionally hide the markdown preview panel in System settings to optimize space.
- Center accordion icons optically for a more balanced layout.

### 🐛 Bug Fixes

- Resolve a critical degradation where the close tab button was rendered unclickable due to the drag-and-drop interaction overlay.
- Fixed layout regressions where the window stretched horizontally out-of-bounds and the title bar rendered insufficiently short.

## [0.7.3] - 2026-03-26 00:30:00 (UTC)

### 🚀 Features

- Add context menus (Open, Rename, Delete, Copy) to directory and file items via right-click interaction.

### ✨ Improvements

- Improve bidirectional drag-and-drop tab movement by unifying drop points to exact midpoints between tabs.
- Support auto-scrolling when dragging a tab to the edges of the visible scroll area.

### 🐛 Bug Fixes

- Fixed an issue where the background color for syntax highlighting was improperly rendered.

## [0.7.2] - 2026-03-25 21:37:57 (UTC)

### 🐛 Bug Fixes

- Fixed a bug where the update checker dialog would stretch vertically indefinitely in the \"up to date\" state. Added preventive measures against recurrence.

## [0.7.1] - 2026-03-25 20:00:00 (UTC)

### 🐛 Bug Fixes

- Eliminated reliance on rate-limited connections, resolving a fundamental architecture flaw that caused false network errors and blocked updates.
- Repaired a layout calculation bug that caused the update checker modal to unpredictably stretch vertically with blank whitespace.

## [0.7.0] - 2026-03-26 03:00:00 (UTC)

### ✨ Features

- Implement interactive UI for the auto-update release framework, incorporating Markdown-rendered release notes and integrated extraction logic.

## [0.6.4] - 2026-03-25 09:50:20 (UTC)

### 🐛 Bug Fixes

- Manually draw underlines using proportional geometry bounds to bypass macOS CJK font metric corruption, ensuring `<u>` tags are consistently visible across all environments.

### 🔧 System

- Implemented extensive internal unit tests to permanently guarantee inline formatting integrity and prevent visual regressions.

## [0.6.3] - 2026-03-25 08:26:00 (UTC)

### 🐛 Bug Fixes

- Remove trailing space from the Homebrew update command in localized update notification messages.

## [0.6.2] - 2026-03-25 08:05:30 (UTC)

### 🚀 Features

- Support for custom states (`[/]`), context menu interactions, and precision vertical alignment.
- High-fidelity TeX/LaTeX equation rendering leveraging the MathJax pipeline for native-quality formatting.

### ✨ Improvements

- Bidirectional scroll tracking between the editor and preview pane with exact block-level precision.
- Visual highlight of the corresponding markdown structure under the cursor in Split-View mode.

### 🐛 Bug Fixes

- Resolved internal panics related to multi-byte characters and eliminated scroll position drifting in long documents.

## [0.6.1] - 2026-03-24 03:15:14 (UTC)

### ✨ Improvements

- Resolved text clipping (left-side truncation) and misalignment within list items, tuning the positioning for CJK glyph centering.
- Implemented stable, CSS-like centering behavior for tables using native layout calculations.
- Expand clickable regions to cover both icon and name for directories/files, with hover effects and context menu support across the full row.
- Active tab scrolls into view only on navigation button press, preventing unwanted scroll jumps during manual scrolling.
- Apply consistent gray background to all sidebar icons (filter, TOC toggle, etc.) for improved visibility in light theme.
- Remove extraneous padding on preview and main window outer frames, and equalize left/right inner margins.

### 🐛 Bug Fixes

- Corrected vertical positioning of background fills for inline code and strikethrough text.
- Fix header-row border rendering that was being cut off midway.
- Implement per-column text alignment (left/center/right) as specified in Markdown alignment syntax.
- Explicitly refresh file content from disk and correctly reset the visual state to prevent stale UI artifacts on diagram updates.
- Ensure truly asynchronous/parallel tab loading when opening multiple files from a workspace directory, with the first file activated immediately.

### 🔧 System

- Optimized internal testing environments and centralized resource management for overall quality improvements.

## [0.6.0] - 2026-03-22 21:52:53 (UTC)

### 🐛 Bug Fixes

- Fix 100% idle CPU utilization and spinner UI freeze by optimizing rendering and SVG load logic.
- Fix list item line breaks inside blockquotes and remove unnecessary vertical whitespace around code blocks.
- Transition from SVG icons to direct `Painter` API drawing for reliability. Adjusted button positioning for better visibility and UX.
- Stabilize centrally squeezed layout by enforcing fixed widths on side panels.
- Skip splash screen natively in test harness context without causing false positives.
- Expanded test coverage for image loading fallback logic to maintain code quality standards.

## [0.5.2] - 2026-03-22 12:44:52 (UTC)

### 🚀 Features

- Added a \"Workspace\" settings tab allowing users to configure the maximum depth and ignored directories for scanning.

### ✨ Improvements

- Significant reduction in idle CPU usage (from 75%+ to <5%) by optimizing window title updates, splash screen repaints, and font rebuilding logic.
- Ensure rendering engine resources are properly released on workspace switch to prevent thread proliferation.

### 🐛 Bug Fixes

- Fix persistence and ordering of recently opened workspaces.
- Fix infinite spinner loop caused by unhandled rendering thread panic.

### 🔧 System

- Improved stability by enhancing static code analysis.
- Fix borrow checker errors, synchronize all i18n locale files, and achieve 100% test coverage gate.

## [0.5.1] - 2026-03-22 09:41:24 (UTC)

### 🐛 Bug Fixes

- Fix GitHub release creation by pushing the tag before creating the release

## [0.5.0] - 2026-03-22 09:16:29 (UTC)

### 🚀 Features

- Add Terms of Service agreement with version tracking
- Implement Markdown export (HTML, PDF, PNG, JPG)

### ✨ Improvements

- Polish Terms modal with language ComboBox and better centering
- Workspace sidebar filter icon changed to ∇ (Nabla) for better semantics

## [0.4.0] - 2026-03-21 13:05:00 (UTC)

### 🚀 Features

- Add App Branding (Icon & Splash Screen)
- Implement Check For Updates functionality
- Add native menus for Checking for Updates, Help, and Donations
- Optimize Diagram Texture implementation with cache
- Add Trackpad support (Pan and Zoom) to Preview and Full-screen Viewers

### 🐛 Bug Fixes

- Fix Native Fullscreen on macOS displaying black background
- Support relative image resolution in Markdown
- Fix integration TOC bugs

## [0.3.1] - 2026-03-21 04:32:00 (UTC)

### 🚀 Features

- Add `FORCE=1` option to `make release` to skip all interactive confirmation prompts
- Implement `USE_GITHUB_WORKFLOW` flag to conditionally trigger GitHub Actions release

### 🔧 System

- Skip Git hooks (`--no-verify`) during release push as quality checks are pre-verified
- Enable full local release flow (DMG build, GitHub publication, Homebrew update) as default

- Modularize release logic into independent scripts under `scripts/release/`
- Move main release control script to `scripts/release/release.sh`

## [0.3.0] - 2026-03-21 03:52:24 (UTC)

### 🚀 Features

- Implement Tab Context Menu (Close, Close Others, Close All) and Tab Restoration actions
- Support automatic restoration of previously opened workspace tabs on startup
- Add Editor Table of Contents (TOC) side panel with setting persistence and i18n support
- Enable Editor Line Numbers and Current Line Highlighting features

### 🐛 Bug Fixes

- Resolve Japanese CJK font baseline jitter (ガタツキ) in UI components
- Prevent TOC side panel auto-expansion and ignore YAML frontmatter in outline
- Allow dead_code for macOS specific emoji rendering constants on Linux CI

### 🔧 System

- Restore signed tag generation config after GPG environment setup
- Update dependencies (rustls-webpki)

## [0.2.1] - 2026-03-21 00:53:02 (UTC)

### 🔧 System

- Update Rust dependencies and GitHub Actions plugins
- Fix coverage gap in preview_pane and codify release bypass rules
- Resolve V0.2.0 archive omission and add AI warning block to next tasks

- Rename repository to KatanA, reorganize documents, and support English translation

- Specify language in settings window integration test to stabilize test
- Collect_matches logic extraction and partial setting screen integration test addition for coverage improvement

## [0.2.0] - 2026-03-20 19:16:37 (UTC)

### 🚀 Features

- Add workspace persistence and tab restoration logic (Task 1)
- Implement CacheFacade and stabilize all integration tests
- Implement recursive expansion of workspaces and "Open All", and improve usability (Task 3, 5)
- Localize metadata tooltips and apply to file items

### 🐛 Bug Fixes

- Enforce strict lazy loading and restrict folder auto-expand on Open All
- Abolish redundant filename tooltip and fix ast linter coverage
- Restore missing absolute path in metadata tooltip and apply TDD

### 🔧 System

- Refactor RwLock usage and fix external image caching on force reload

## [0.1.6] - 2026-03-19 23:57:28 (UTC)

### 🚀 Features

- Implement workspace search and filter functionality
- Add internationalized text for search modal Include/Exclude options
- Add inclusion/exclusion filter functionality to search modal and place search button in UI

### 🐛 Bug Fixes

- Automatically inject version into Info.plist during DMG build
- Automatically sync internal version files during releases.

### 🔧 System

- Prepare for v0.1.7 release
- Prepare for v0.1.6 release

## [0.1.5] - 2026-03-19 21:12:34 (UTC)

### 🚀 Features

- Apply v0.1.5 changes and bump version to 0.1.5

### 🔧 System

- Bump version to 0.1.4

- Unify HashMap and fixed-length arrays into Vec, and apply collectively including AST rules and migration functionality

- Fix tests broken by workspace methods renaming
- Add missing tests to meet 100% coverage gate

## [0.1.4] - 2026-03-19 21:03:35 (UTC)

## [0.1.3] - 2026-03-19 19:59:23 (UTC)

### 🚀 Features

- Expand theme presets from 10 to 30 (added OneDark/TokyoNight/CatppuccinMocha etc.)
- Migrate i18n to type-safe structs (I18nMessages) and add 8 languages (zh-CN/zh-TW/ko/pt/fr/de/es/it)
- Add 8 language tags to macOS native menu and dynamically translate menu strings according to language switching
- Update entire UI for i18n/settings hierarchization, and implement OS language detection, theme expansion, and Show more/less toggle in settings screen

### 🐛 Bug Fixes

- Recovery of missed v0.1.3 version update
- Fix flaky tests where curl failed to start due to environment variable pollution during parallel test execution

### 🔧 System

- Hierarchize settings.json structure (ThemeSettings/FontSettings/LayoutSettings) and add migration mechanism
- Fix coverage gate and improve code quality

- Update tests according to i18n type-safety, settings hierarchization, and theme expansion (integration/i18n/theme/diagram_rendering tests)

## [0.1.2] - 2026-03-19 16:54:57 (UTC)

### 🚀 Features

- Add i18n tooltips to tab navigation and slider

### 🐛 Bug Fixes

- Fix left alignment of workspace file entries
- Fix issue where font size slider becomes invisible in light theme
- Add selection color border to slider to ensure visibility in all themes
- Modify markdown preview tables to use available width
- Fix bugs in table layout and vertical split scroll

### 🔧 System

- Prepare for v0.1.2 release
- Turn warnings into errors and remove unused code
- Prepare for v0.1.2 release

## [0.1.1] - 2026-03-19 10:54:34 (UTC)

### 🚀 Features

- Support hidden directory display in workspace tree and add directory refresh button

### 🐛 Bug Fixes

- Add error handling to Homebrew Cask update step
- Prevent contamination of cached old DMG files

## [0.1.0] - 2026-03-19 09:33:46 (UTC)

### 🚀 Features

- Add Homebrew Cask support
- Implement 10 theme presets and ThemeColors foundation (Task 1) (#23)
- Implement foundation for font size and family settings (Task 2)
- Implement theme linking and settings screen, and update snapshots (WIP)
- Add dynamic acquisition of OS fonts and reflection in UI
- Implement Task 4: editor/preview layout settings
- Implement Task 5: OS theme linking (initial default auto-selection)
- Implement Task 6: font setting expansion (search function + Apple Color Emoji)
- Add strict quality checks to linter (prohibit use of todo! macro, etc.)
- Improve UI functions such as font search, emoji support, and preview
- Implement emoji inline rendering foundation and separate SVG/HTTP cache loaders
- Enhanced internal mechanisms for early detection of potential implementation issues.

### 🐛 Bug Fixes

- Improve .app signing (abolish --deep, specify runtime/timestamp, DMG remains unsigned)
- Fix regression where workspace is not restored on startup
- Apply emoji support patch to vendored egui_commonmark

### 🔧 System

- Add glob filters to pre-push hook and skip CI execution for pushes without code changes
- Prepare for v0.0.4 release
- Exclude unnecessary old backup images (.old.png) when updating snapshots from Git tracking
- Expand coverage gate exclusion rules (return None/false/display/Pending)
- Prepare for v0.1.0 release (update version number)

- Optimized codebase structure and brought comments up to international development standards.

- Add tests for coverage improvement

## [0.0.3] - 2026-03-18 02:50:23 (UTC)

### 🐛 Bug Fixes

- Unify Coverage job with local make coverage
- Improve DrawIo diagram text visibility in dark theme
- Expand mmdc resolution from .app bundle to 6-layer fallback
- Skip diagram snapshot tests in CI environment
- Fix startup from GUI apps by supplementing node PATH when executing mmdc
- Add margins above and below HTML blocks to resolve layout tightness

### 🔧 System

- Update coverage exclusion reasons to accurate technical grounds
- Prepare for v0.0.3 release
- Change release notes to be extracted from CHANGELOG.md

- Constantize magic numbers and expand AST linter tests
- Unify Ignore tags to limited_local

- Fix CI environment dependent errors in snapshot tests
- Fix global state conflict errors in multiple i18n tests
- Add integration tests for diagram rendering and sample fixtures

## [0.0.2] - 2026-03-17 09:20:28 (UTC)

### 🐛 Bug Fixes

- Resolve linux cross-compilation errors for github actions
- Resolve markdown rendering, i18n label update, and CI coverage flakiness
- Support CenteredMarkdown for raw HTML alignment reproduction
- Fix CenteredMarkdown alignment, image path resolution, and badge display

### 🔧 System

- Kick ci to retry integration tests
- Release v0.0.2
- Include Cargo.lock and CHANGELOG.ja.md in release v0.0.2

- Update integration test snapshot
- Increase snapshot tolerance to 4000 to absorb CI/local macOS text rendering differences

## [0.0.1] - 2026-03-16 23:16:22 (UTC)

### 🚀 Features

- Bootstrap Katana macOS MVP — implementation of Rust project foundation and all core modules
- Task 3.2 — native Markdown preview pane implementation
- I18n support, language setting, appAction expansion, bin rename
- Improve diagram rendering (Draw.io arrow support, Mermaid PNG output, CommandNotFound distinction)
- Extend filesystem service (workspace tree and file monitoring improvements)
- Tab-specific preview management, scroll synchronization, macOS native menu, workspace panel control
- Enhance verification — introduction of lefthook, adding tests, tightening Clippy, defining quality gates
- Improved stability by enhancing static code analysis.
- Apply Katana app icon and version for native About panel (#15)
- Implement settings persistence foundation (JsonFileRepository + SettingsService)
- Auto-save settings when workspace or language changes
- Restore saved settings (workspace, language) on startup
- Improve preview functionality (image path resolution, section splitting starting fence support, diagram renderer improvements)
- Improve About screen and unify app display name to KatanA
- Add macOS app bundle (.app) packaging (#18)
- Add macOS DMG installer generation (#19)
- Automate release (git-cliff + make release) (#20)
- Create new Release CD workflow (.github/workflows/release.yml) (#22)
- Add GitHub Sponsors URL settings and Japanese version of README

### 🐛 Bug Fixes

- Fix Clippy warnings, formatting, and 30-line limit
- Fix issues confirmed via screenshots
- Stabilize tests by making PLANTUML_JAR an exclusive override
- Fix 3 issues — lazy loading, Mermaid fonts, and forced desktop move
- Fix flaky issues in snapshot tests
- De-indent code blocks in lists during preprocessing to avoid egui layout constraints
- Change Info.plist update to Perl for macOS sed compatibility
- Add ad-hoc code signing to Release CD
- Fix CI trigger branch name to master and update Cargo.lock
- Fix sccache-action SHA and organize English/Japanese versions of CHANGELOG
- Temporarily disable sccache during cargo install (conflict avoidance)
- Disable RUSTC_WRAPPER in CI Lint job (clippy compatibility)

### 🔧 System

- Bootstrap katana repository
- Remove opsx prompt files
- Align gitignore with official templates
- Mark Task 6.2 as completed — bootstrap-katana-macos-mvp all tasks completed
- Exclude openspec directory from git control
- Update gitignore (openspec, obsidian settings, katana-core .gitignore integration)
- Delete unnecessary document templates and README
- Add CI coverage job and document quality gates
- Tighten CI requirements for desktop-viewer-polish and delete unnecessary assets
- Integrate lefthook validation commands into Makefile and automate fixes
- Update dependencies (dirs-sys 0.5.0, rfd 0.17.2, egui_commonmark features added)
- Add GitHub Sponsors URL settings and Japanese version of README
- Add configuration to exclude CI bot commits from cliff.toml
- Prepare for v0.0.1 release

- Fix clippy warnings in drawio_renderer
- Migrate tests from src/ to tests/ directory and tighten Clippy
- Refactor katana-ui into lib/binary structure and extract logic
- Extract magic numbers into named constants with clear purpose
- Externalize language definitions to locales/languages.json
- Unify span_location duplication into free functions (self-review fix)
- Separate egui rendering logic and event routing
- Translate Japanese comments and strings in source code and tests to English
- Improve UI layout and add linter module

- Task 6.2 — add preview synchronization tests
- Add app state unit tests and fix java headless mode for plantuml
- Add unit tests for preview synchronization (Task 3.2 completed)
- Tighten coverage — removed ignore-filename-regex, abolished #[coverage(off)], enforced 100% Regions
- Address differences in LLVM coverage calculation and tighten 100% test gate
- Add integration tests for persistence round-trip
