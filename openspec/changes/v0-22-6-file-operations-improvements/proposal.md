# File Operations Improvements (v0.22.5)

## Summary

This change enhances file handling capabilities by supporting file opening from new file dialogs, drag and drop operations, and improved tab management.

## Problem

Current file operations lack support for opening files from the system, external drag and drop, and limited tab management functionality.

## Solution

1. Support opening files from the system's new file dialog
2. Implement drag and drop support for opening files in the current workspace
3. Add tab management features for drag and drop operations
4. Implement file movement with confirmation dialog

## Acceptance Criteria

- System file opening dialog works for single files
- External file drag and drop supported in current workspace
- Tab management with drag and drop operations for activation
- File movement with confirmation dialog works