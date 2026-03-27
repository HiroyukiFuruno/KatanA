import os
import re

def rgb_to_luminance(r, g, b):
    a = [c / 255.0 for c in (r, g, b)]
    for i in range(3):
        a[i] = a[i] / 12.92 if a[i] <= 0.03928 else ((a[i] + 0.055) / 1.055) ** 2.4
    return 0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2]

def contrast_ratio(rgb1, rgb2):
    l1 = rgb_to_luminance(*rgb1)
    l2 = rgb_to_luminance(*rgb2)
    return (max(l1, l2) + 0.05) / (min(l1, l2) + 0.05)

themes_dir = "crates/katana-platform/src/theme/presets"
results = []
fails = 0

for root, _, files in os.walk(themes_dir):
    for f in files:
        if not f.endswith(".rs"): continue
        path = os.path.join(root, f)
        with open(path, "r") as r: content = r.read()
        
        bg_match = re.search(r'background:\s*Rgb\s*\{\s*r:\s*(\d+),\s*g:\s*(\d+),\s*b:\s*(\d+)', content)
        text_match = re.search(r'text:\s*Rgb\s*\{\s*r:\s*(\d+),\s*g:\s*(\d+),\s*b:\s*(\d+)', content)
        sec_match = re.search(r'text_secondary:\s*Rgb\s*\{\s*r:\s*(\d+),\s*g:\s*(\d+),\s*b:\s*(\d+)', content)
        
        if bg_match and text_match and sec_match:
            bg = (int(bg_match.group(1)), int(bg_match.group(2)), int(bg_match.group(3)))
            txt = (int(text_match.group(1)), int(text_match.group(2)), int(text_match.group(3)))
            sec = (int(sec_match.group(1)), int(sec_match.group(2)), int(sec_match.group(3)))
            
            main_ratio = contrast_ratio(bg, txt)
            sec_ratio = contrast_ratio(bg, sec)
            results.append(f"{f}: Main Contrast={main_ratio:.2f}, Sec Contrast={sec_ratio:.2f}")
            if main_ratio < 4.0: fails += 1

print("\n".join(results))
print(f"Total Fails (<4.0): {fails}")
