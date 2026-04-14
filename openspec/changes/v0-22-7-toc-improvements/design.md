# Table of Contents Improvements Design

## Overview

This design outlines enhancements to the table of contents functionality including visual improvements, accordion support, and enhanced navigation controls.

## Visual Design

### Current State

- TOC headings have unnecessary background styling

### Proposed Implementation

- Remove all background styling from TOC headings
- Maintain clean, minimal visual appearance

## Accordion Functionality

### Current State

- Table of contents lacks expand/collapse functionality

### Proposed Implementation

- Replace static TOC with accordion-style interface
- Default state: all sections expanded
- Support for expanding and collapsing sections

## Navigation Controls

### Current State

- No full open/close controls

### Proposed Implementation

- Add toggle controls at the top of the TOC
- Full open/close functionality for all sections
- Consistent with explorer interface design

## Vertical Lines

### Current State

- No visual indicators for section hierarchy

### Proposed Implementation

- Add vertical lines to accordion sections
- Make vertical line display configurable by theme settings
- Default setting: lines are displayed
- Proper persistence of user preferences
