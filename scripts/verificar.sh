#!/usr/bin/env bash
# Verifica a integridade do projeto antes de rodar `trunk serve`.
# Uso: ./scripts/verificar.sh

set -euo pipefail

if [[ -t 1 ]]; then
    BOLD=$'\033[1m'; GREEN=$'\033[32m'; RED=$'\033[31m'; YEL=$'\033[33m'; RST=$'\033[0m'
else
    BOLD=""; GREEN=""; RED=""; YEL=""; RST=""
fi

titulo() { printf '\n%s== %s ==%s\n' "$BOLD" "$1" "$RST"; }
ok()     { printf '%s✓%s %s\n' "$GREEN" "$RST" "$1"; }
erro()   { printf '%s✗%s %s\n' "$RED" "$RST" "$1"; }

cd "$(dirname "$0")/.."

falhou=0

titulo "1. Formatação"
if cargo fmt --check >/dev/null 2>&1; then
    ok "código formatado"
else
    erro "código não formatado — rode 'cargo fmt'"
    falhou=1
fi

titulo "2. Lint (clippy)"
if cargo clippy --all-targets -- -D warnings >/dev/null 2>&1; then
    ok "clippy limpo"
else
    erro "clippy reportou warnings — rode 'cargo clippy --all-targets'"
    falhou=1
fi

titulo "3. Testes da engine (host, sem Wasm)"
if cargo test --lib >/dev/null 2>&1; then
    ok "engine.rs passou"
else
    erro "testes da engine falharam — rode 'cargo test --lib'"
    falhou=1
fi

titulo "4. Integridade do questoes.json + imagens"
if cargo test --test questoes_integridade -- --nocapture 2>&1 | tee /tmp/quiz_integridade.log | grep -q "test result: ok"; then
    ok "JSON e imagens consistentes"
else
    erro "falha na integridade — detalhes:"
    sed 's/^/   /' /tmp/quiz_integridade.log || true
    falhou=1
fi

titulo "5. Build Wasm (trunk)"
if command -v trunk >/dev/null 2>&1; then
    if trunk build --release >/dev/null 2>&1; then
        ok "trunk build ok (dist/ gerado)"
    else
        erro "trunk build falhou — rode 'trunk build' para ver o erro"
        falhou=1
    fi
else
    printf '%s!%s trunk não instalado — pulando build Wasm\n' "$YEL" "$RST"
fi

echo
if [[ $falhou -eq 0 ]]; then
    printf '%sTudo ok.%s Pronto para '"'"'make dev'"'"'.\n' "$GREEN" "$RST"
    exit 0
else
    printf '%sHá problemas acima.%s Corrija antes de rodar o app.\n' "$RED" "$RST"
    exit 1
fi