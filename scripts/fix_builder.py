path = "crates/katana-platform/src/theme/builder.rs"
with open(path, "r") as f:
    content = f.read()

# Replace literal backslash+n with actual newline character
new_content = content.replace("\\\\n", "\\n")

with open(path, "w") as f:
    f.write(new_content)
