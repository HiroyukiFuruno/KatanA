# Changelog

All notable changes to KatanA Desktop. This file records the major changes to KatanA Desktop.

## [0.22.1] - 2026-04-12 18:59:00 (UTC)

### ✨ Improvements

- **Standardized Search UI**: Unified the search bar design across the explorer, global search, and document search.
- **Enhanced Search Inputs**: Integrated search icons and clear buttons directly inside search input fields for a more premium look and feel.
- **Advanced Search Toggles**: Added support for Case Sensitivity, Whole Word matches, and Regular Expressions across all search interfaces.
- **Layout Stability**: Resolved layout regressions where the search bar could cause the sidebar to stretch horizontally.
- **Extended Localization**: Completed native localization for "Official Website" links and About dialog entries across all 11 supported languages.
- **macOS Menu Optimization**: Simplified the macOS-specific application menu by removing redundant "Hide" and "Show All" items for a cleaner user experience.

### 🐛 Bug Fixes

- **Explorer Filter Behavior**: Fixed an issue where hidden directories starting with "." (e.g., .git) were not excluded when filtering.
- **Improved UI Usability**: Improved the explorer to automatically focus the search input field when the filter is enabled.

### 🔧 System

- **Refactored Search Components**: Consolidated redundant search UI logic into a centralized `SearchBar` widget for better maintainability and consistency.

## [0.22.0] - 2026-04-12 13:10:00 (UTC)

### 🚀 Features

- Implemented Markdown Authoring Commands (Bold, Italic, Strikethrough, etc.) and integrated them into the command inventory.
- Added comprehensive Image Ingest Pipeline UI and fallback integrations.
- Replaced missing images with an aesthetic UI fallback component (`ImageFallbackOps`) for better user experience.
- Persisted image ingestion settings to allow custom paths for image asset management.
- Standardized file system scanning and tree entry logic for improved stability.
