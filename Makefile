# ─────────────────────────────────────────────────────────────────────────────
# draupnir — Makefile
# ─────────────────────────────────────────────────────────────────────────────

# ANSI color codes
RESET   := \033[0m
BOLD    := \033[1m
RED     := \033[31m
GREEN   := \033[32m
YELLOW  := \033[33m
CYAN    := \033[36m

# Helpers
OK      := $(GREEN)✓$(RESET)
WARN    := $(YELLOW)⚠$(RESET)
ERR     := $(RED)✗$(RESET)
HDR     := $(CYAN)▸$(RESET)

BINARY         := draupnir
RELEASE_BIN    := target/release/$(BINARY)
INSTALL_DIR    := $(HOME)/.local/bin
INSTALL_PATH   := $(INSTALL_DIR)/$(BINARY)
SKILLS_DIR     := $(or $(CLAUDE_CONFIG_DIR),$(HOME)/.claude)/skills

.DEFAULT_GOAL := help

.PHONY: help build install check fmt fmt-check clippy test lint ci clean audit size smoke uninstall link-skills

# ─────────────────────────────────────────────────────────────────────────────
# help
# ─────────────────────────────────────────────────────────────────────────────

help:
	@printf "\n"
	@printf "  $(BOLD)$(CYAN)██████╗ ██████╗  █████╗ ██╗   ██╗██████╗ ███╗   ██╗██╗██████╗ $(RESET)\n"
	@printf "  $(BOLD)$(CYAN)██╔══██╗██╔══██╗██╔══██╗██║   ██║██╔══██╗████╗  ██║██║██╔══██╗$(RESET)\n"
	@printf "  $(BOLD)$(CYAN)██║  ██║██████╔╝███████║██║   ██║██████╔╝██╔██╗ ██║██║██████╔╝$(RESET)\n"
	@printf "  $(BOLD)$(CYAN)██║  ██║██╔══██╗██╔══██║██║   ██║██╔═══╝ ██║╚██╗██║██║██╔══██╗$(RESET)\n"
	@printf "  $(BOLD)$(CYAN)██████╔╝██║  ██║██║  ██║╚██████╔╝██║     ██║ ╚████║██║██║  ██║$(RESET)\n"
	@printf "  $(BOLD)$(CYAN)╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝$(RESET)\n"
	@printf "\n"
	@printf "  $(BOLD)CLI for the Draupnir vault$(RESET) — session logs, instincts, briefs, health checks\n"
	@printf "\n"
	@printf "  $(CYAN)─────────────────────────────────────────────────────────────────$(RESET)\n"
	@printf "\n"
	@printf "  $(BOLD)Development$(RESET)\n"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "check"     "Syntax check without building"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "fmt"       "Auto-format with cargo fmt"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "fmt-check" "Verify formatting (no writes)"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "clippy"    "Lint with clippy -D warnings"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "lint"      "fmt-check + clippy"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "test"      "Run test suite"
	@printf "\n"
	@printf "  $(BOLD)Build & Deploy$(RESET)\n"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "build"     "cargo build --release"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "install"   "Build + symlink to ~/.local/bin/draupnir"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "uninstall" "Remove symlink from ~/.local/bin"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "link-skills" "Symlink skills/ into ~/.claude/skills (dev)"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "size"      "Show release binary size"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "smoke"     "Quick smoke test (doctor + --help)"
	@printf "\n"
	@printf "  $(BOLD)Maintenance$(RESET)\n"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "clean"     "cargo clean"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "audit"     "cargo audit (requires cargo-audit)"
	@printf "    $(GREEN)%-14s$(RESET)  %s\n" "ci"        "fmt-check + clippy + test (what CI runs)"
	@printf "\n"
	@$(if $(wildcard eval/eval.mk),$(MAKE) -s --no-print-directory help-eval,true)

# ─────────────────────────────────────────────────────────────────────────────
# build
# ─────────────────────────────────────────────────────────────────────────────

build:
	@printf "$(HDR) $(BOLD)Building release binary...$(RESET)\n"
	@cargo build --release
	@SIZE=$$(du -sh $(RELEASE_BIN) 2>/dev/null | cut -f1); \
	  printf "$(OK) Build complete ($(BOLD)$$SIZE$(RESET))\n"

# ─────────────────────────────────────────────────────────────────────────────
# install
# ─────────────────────────────────────────────────────────────────────────────

install: build
	@printf "$(HDR) $(BOLD)Installing to $(INSTALL_PATH)...$(RESET)\n"
	@mkdir -p $(INSTALL_DIR)
	@ln -sf $(CURDIR)/$(RELEASE_BIN) $(INSTALL_PATH)
	@printf "$(OK) Installed: $(BOLD)$(INSTALL_PATH)$(RESET) → $(CURDIR)/$(RELEASE_BIN)\n"
	@printf "   Make sure $(BOLD)$(INSTALL_DIR)$(RESET) is on your PATH.\n"

# ─────────────────────────────────────────────────────────────────────────────
# link-skills — dev only; users run `draupnir skills install`
# ─────────────────────────────────────────────────────────────────────────────

link-skills:
	@mkdir -p $(SKILLS_DIR)
	@for s in skills/*/; do \
	  name=$$(basename $$s); target=$(SKILLS_DIR)/$$name; \
	  if [ -e $$target ] && [ ! -L $$target ]; then \
	    printf "$(WARN) $$target is a real directory — move it away first, skipped\n"; \
	  else \
	    ln -sfn $(CURDIR)/skills/$$name $$target; \
	    printf "$(OK) $$name → $(CURDIR)/skills/$$name\n"; \
	  fi; \
	done

# ─────────────────────────────────────────────────────────────────────────────
# uninstall
# ─────────────────────────────────────────────────────────────────────────────

uninstall:
	@printf "$(HDR) $(BOLD)Removing $(INSTALL_PATH)...$(RESET)\n"
	@if [ -L $(INSTALL_PATH) ]; then \
	  rm $(INSTALL_PATH); \
	  printf "$(OK) Removed $(BOLD)$(INSTALL_PATH)$(RESET)\n"; \
	elif [ -e $(INSTALL_PATH) ]; then \
	  printf "$(WARN) $(INSTALL_PATH) exists but is not a symlink — remove it manually.\n"; \
	else \
	  printf "$(WARN) Nothing to remove — $(INSTALL_PATH) does not exist.\n"; \
	fi

# ─────────────────────────────────────────────────────────────────────────────
# check / fmt / clippy / test
# ─────────────────────────────────────────────────────────────────────────────

check:
	@printf "$(HDR) $(BOLD)Checking...$(RESET)\n"
	@cargo check
	@printf "$(OK) Check passed\n"

fmt:
	@printf "$(HDR) $(BOLD)Formatting...$(RESET)\n"
	@cargo fmt
	@printf "$(OK) Formatted\n"

fmt-check:
	@printf "$(HDR) $(BOLD)Checking formatting...$(RESET)\n"
	@cargo fmt -- --check
	@printf "$(OK) Formatting is clean\n"

clippy:
	@printf "$(HDR) $(BOLD)Running clippy...$(RESET)\n"
	@cargo clippy -- -D warnings
	@printf "$(OK) No clippy warnings\n"

test:
	@printf "$(HDR) $(BOLD)Running tests...$(RESET)\n"
	@cargo test
	@printf "$(OK) All tests passed\n"

# ─────────────────────────────────────────────────────────────────────────────
# lint / ci
# ─────────────────────────────────────────────────────────────────────────────

lint: fmt-check clippy
	@printf "$(OK) Lint clean\n"

ci: fmt-check clippy test
	@printf "\n$(OK) $(BOLD)CI checks passed$(RESET)\n"

# ─────────────────────────────────────────────────────────────────────────────
# clean
# ─────────────────────────────────────────────────────────────────────────────

clean:
	@printf "$(HDR) $(BOLD)Cleaning build artifacts...$(RESET)\n"
	@cargo clean
	@printf "$(OK) Clean\n"

# ─────────────────────────────────────────────────────────────────────────────
# audit
# ─────────────────────────────────────────────────────────────────────────────

audit:
	@printf "$(HDR) $(BOLD)Running cargo audit...$(RESET)\n"
	@if command -v cargo-audit >/dev/null 2>&1 || cargo audit --version >/dev/null 2>&1; then \
	  cargo audit; \
	  printf "$(OK) No known vulnerabilities\n"; \
	else \
	  printf "$(WARN) cargo-audit is not installed — run: $(BOLD)cargo install cargo-audit$(RESET)\n"; \
	  exit 1; \
	fi

# ─────────────────────────────────────────────────────────────────────────────
# size
# ─────────────────────────────────────────────────────────────────────────────

size:
	@printf "$(HDR) $(BOLD)Binary size report...$(RESET)\n"
	@if [ ! -f $(RELEASE_BIN) ]; then \
	  printf "$(WARN) Release binary not found — run $(BOLD)make build$(RESET) first.\n"; \
	  exit 1; \
	fi
	@BYTES=$$(wc -c < $(RELEASE_BIN) | tr -d ' '); \
	  HUMAN=$$(du -sh $(RELEASE_BIN) | cut -f1); \
	  printf "$(OK) $(BOLD)$(RELEASE_BIN)$(RESET)  →  $(BOLD)$$HUMAN$(RESET)  ($$BYTES bytes)\n"

# ─────────────────────────────────────────────────────────────────────────────
# smoke
# ─────────────────────────────────────────────────────────────────────────────

smoke:
	@printf "$(HDR) $(BOLD)Running smoke tests...$(RESET)\n"
	@if [ ! -f $(RELEASE_BIN) ]; then \
	  printf "$(WARN) Release binary not found — run $(BOLD)make build$(RESET) first.\n"; \
	  exit 1; \
	fi
	@printf "\n$(CYAN)── draupnir --help ──────────────────────────────$(RESET)\n"
	@$(RELEASE_BIN) --help
	@printf "\n$(CYAN)── draupnir doctor $(if $(PROJECT),--project $(PROJECT)) ─────────$(RESET)\n"
	@$(RELEASE_BIN) doctor $(if $(PROJECT),--project $(PROJECT)); \
	  STATUS=$$?; \
	  if [ $$STATUS -eq 0 ]; then \
	    printf "\n$(OK) Smoke tests passed\n"; \
	  else \
	    printf "\n$(WARN) doctor exited with status $$STATUS (may be expected)\n"; \
	  fi

# eval targets live in the private eval/ repo; absent in a public checkout
-include eval/eval.mk
