# Code Input Improvements Design

## Overview

This design outlines enhancements to the markdown editor's input capabilities including rich text controls, clipboard image support, and improved image display.

## Rich Text Controls

### Current State

- No visual controls for rich text features
- Only keyboard shortcuts available

### Proposed Implementation

- Add system SVG-based buttons for formatting
- Support for common markdown features (bold, italic, lists, etc.)
- Context-aware toolbars in editor UI

## Clipboard Image Support

### Current State

- No supported clipboard image pasting
- Feature planned but not implemented

### Proposed Implementation

- Support for command+v paste of images
- Context menu "Paste" option for images
- Automatic saving of pasted images in assets/img/ directory
- Proper naming and handling of image files

## Image Display in Explorer

### Current State

- Images referenced in markdown are not displayed in explorer

### Proposed Implementation

- Parse markdown for image references
- Display referenced images in explorer view
- Implement lazy loading approach to maintain performance
- Indicate image files with appropriate icons

## Drag and Drop Image Support

### Current State

- No drag and drop support

### Proposed Implementation

- Allow drag and drop of images into editor
- Insert at cursor position, append to end if no selection
- Automatic saving of dropped images in assets/img/ directory
- Preserve existing file naming conventions
