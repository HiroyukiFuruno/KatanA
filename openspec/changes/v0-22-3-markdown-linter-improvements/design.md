# Markdown Linter Improvements Design

## Overview

This design outlines the implementation of enhanced markdown linting capabilities including full rule support, visual indicators, and improved diagnostics experience.

## Rule Support

### Current State

- Only MD001 rule is supported
- Requires extension to all official markdownlint rules

### Proposed Implementation

- Integrate full markdownlint rule set
- Support all rules in markdownlint specification
- Maintain backward compatibility

## Visual Indicators

### Current State

- Limited visual feedback for linting issues

### Proposed Implementation

- Add yellow underline indicators for linting issues
- Display visual warnings directly in the editor
- Make warning color configurable by theme

## Diagnostics Experience

### Current State

- Limited popup warnings
- No auto-fix capabilities

### Proposed Implementation

- Popup warnings with detailed linting information
- Auto-fix buttons for simple issues
- Integration with editor's existing warning system

## Technical Design

### Backend

- Extend linting engine to support full markdownlint rule set
- Implement rule categorization for auto-fix functionality
- Store diagnostic information with proper positioning

### Frontend

- Update editor component to display visual indicators
- Implement popup warning system
- Add theme-based color configuration for warning indicators
