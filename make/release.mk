###################################
# Release / Packaging
###################################

.PHONY: package-mac
package-mac: ## Build macOS .app bundle (release)
	scripts/build/package-mac.sh $(VERSION)

.PHONY: package-linux
package-linux: ## Build Linux zip artifact
	scripts/build/package-linux.sh

.PHONY: package-windows
package-windows: ## Build Windows MSI and ZIP artifacts
	scripts/build/package-windows.sh

.PHONY: dmg
dmg: package-mac ## Build macOS .dmg installer from .app bundle
	@FORCE=$(FORCE) scripts/build/dmg.sh $(VERSION)

.PHONY: release
release: ## Prepare for release by bumping version in all files (usage: make release VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=x.y.z)
endif
	@scripts/release/bump-version.sh $(VERSION)
	@echo ""
	@echo "✅ Version bump completed!"
	@echo "   Next steps:"
	@echo "   1. Review changes (git diff)"
	@echo "   2. Commit with -S (Verified Commit): git commit -S -m \"release: v$(VERSION)\""
	@echo "   3. Create a PR to master"

.PHONY: release-trigger
release-trigger: ## Manually trigger the release workflow on GitHub Actions (emergency use only)
ifndef VERSION
	$(error VERSION is required. Usage: make release-trigger VERSION=x.y.z)
endif
	@echo "Triggering GitHub Actions release workflow for v$(VERSION)..."
	@BOOL_FORCE="false"; \
	if [ "$(FORCE)" = "1" ]; then \
		BOOL_FORCE="true"; \
		echo "⚠️ Force mode enabled (clobbering existing release)"; \
	fi; \
	gh workflow run build-and-release.yml -f version=$(VERSION) -f target=all -f force=$$BOOL_FORCE
	@echo ""
	@echo "✅ Workflow triggered! Monitor progress with:"
	@echo "   gh run watch --repo HiroyukiFuruno/KatanA"

.PHONY: release-preflight
release-preflight: ## Run preflight release checks without publishing (usage: make release-preflight VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make release-preflight VERSION=x.y.z)
endif
	@scripts/release/preflight.sh $(VERSION)
