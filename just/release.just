# ============================================================
# Release / Packaging
# ============================================================

# Build macOS .app bundle (release)
package-mac:
    scripts/build/package-mac.sh {{VERSION}}

# Build Linux zip artifact
package-linux:
    scripts/build/package-linux.sh

# Build Windows MSI and ZIP artifacts
package-windows:
    scripts/build/package-windows.sh

# Build macOS .dmg installer from .app bundle
dmg: package-mac
    @FORCE={{FORCE}} scripts/build/dmg.sh {{VERSION}}

# Prepare for release by bumping version in all files (usage: just VERSION=x.y.z release)
release: version-required
    @scripts/release/bump-version.sh {{VERSION}}
    @echo ""
    @echo "✅ Version bump completed!"
    @echo "   Next steps:"
    @echo "   1. Review changes (git diff)"
    @echo "   2. Commit with -S (Verified Commit): git commit -S -m \"release: v{{VERSION}}\""
    @echo "   3. Create a PR to master"

# Manually trigger the release workflow on GitHub Actions (emergency use only)
release-trigger: version-required
    @echo "Triggering GitHub Actions release workflow for v{{VERSION}}..."
    @BOOL_FORCE="false"; \
    if [ "{{FORCE}}" = "1" ]; then \
        BOOL_FORCE="true"; \
        echo "⚠️ Force mode enabled (clobbering existing release)"; \
    fi; \
    gh workflow run build-and-release.yml -f version={{VERSION}} -f target=all -f force=$BOOL_FORCE
    @echo ""
    @echo "✅ Workflow triggered! Monitor progress with:"
    @echo "   gh run watch --repo HiroyukiFuruno/KatanA"

# Run preflight release checks without publishing (usage: just VERSION=x.y.z release-preflight)
release-preflight: version-required
    @scripts/release/preflight.sh {{VERSION}}
