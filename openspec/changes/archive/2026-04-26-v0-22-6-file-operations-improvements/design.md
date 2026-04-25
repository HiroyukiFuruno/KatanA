# File Operations Improvements Design

## Overview

This design outlines enhancements to file handling capabilities including system file dialogs, drag and drop support, and improved tab management.

## System File Opening

### Current State

- Limited file opening capabilities

### Proposed Implementation

- Support for opening files via system dialog
- Two opening modes: new workspace (temporary) and current workspace
- System SVG icon for temporary workspace indicator

## Drag and Drop Support

### Current State

- No external file drag and drop support

### Proposed Implementation

- Support for dragging and dropping files to open them
- Default behavior: open in current workspace
- Consideration of existing tab structure and organization

## Tab Management

### Current State

- Limited tab management features

### Proposed Implementation

- Drag and drop of tabs to activate/organize
- Tab position precision for temporary tabs
- Support for adding to existing tab groups
- Default behavior: add to end, precise operations for specific positions

## File Movement

### Current State

- No file movement capabilities

### Proposed Implementation

- Support for moving files via drag and drop in explorer
- Confirmation dialog for file movement operations
- Default setting: confirmation required
- Clear notification of operations (e.g., "Move file from xx to yyy/zzz")
