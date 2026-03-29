import re
import os

with open("crates/katana-ui/src/settings_window.rs", "r") as f:
    text = f.read()

# Define blocks to extract
blocks = [
    ("theme", r"(fn render_theme_tab.*?)(?=fn render_font_tab|fn render_layout_tab|fn render_workspace_tab|fn render_updates_tab|fn render_behavior_tab|$)"),
    ("font", r"(fn render_font_tab.*?)(?=fn render_layout_tab|fn render_workspace_tab|fn render_updates_tab|fn render_behavior_tab|fn render_theme_tab|$)"),
    ("layout", r"(fn render_layout_tab.*?)(?=fn render_workspace_tab|fn render_updates_tab|fn render_behavior_tab|fn render_theme_tab|fn render_font_tab|$)"),
    ("workspace", r"(fn render_workspace_tab.*?)(?=fn render_updates_tab|fn render_behavior_tab|fn render_theme_tab|fn render_font_tab|fn render_layout_tab|$)"),
    ("updates", r"(fn render_updates_tab.*?)(?=fn render_behavior_tab|fn render_theme_tab|fn render_font_tab|fn render_layout_tab|fn render_workspace_tab|$)"),
    ("behavior", r"(fn render_behavior_tab.*)"),
]

# We need a proper way to extract functions including all their helper functions.
# Let's just find all top-level functions instead, and sort them into tabs manually in the script.
