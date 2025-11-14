#!/bin/bash

# Script para testar CI/CD localmente usando 'act'
# Requer: https://github.com/nektos/act

set -e

echo "🧪 Teste Local do Sistema CI/CD"
echo "================================"

# Verifica se act está instalado
if ! command -v act &> /dev/null; then
    echo "❌ 'act' não está instalado!"
    echo ""
    echo "Instale com:"
    echo "  curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    echo "ou"
    echo "  brew install act  # macOS"
    exit 1
fi

# Menu de opções
echo ""
echo "Escolha o que testar:"
echo "1) CI completo (build e testes)"
echo "2) Validação de version bump"
echo "3) Build de binários"
echo "4) Release completa (simulada)"
echo ""
read -p "Opção (1-4): " option

case $option in
    1)
        echo "🔨 Testando CI completo..."
        act -W .github/workflows/ci.yml
        ;;
    2)
        echo "📋 Testando validação de version bump..."
        # Simula PR com mudança no Cargo.toml
        act pull_request -W .github/workflows/validate-version-bump.yml
        ;;
    3)
        echo "📦 Testando build de binários..."
        act workflow_dispatch -W .github/workflows/test-release.yml
        ;;
    4)
        echo "🚀 Testando release completa..."
        # Simula todo o fluxo
        act pull_request -W .github/workflows/validate-version-bump.yml
        act pull_request -W .github/workflows/auto-release.yml --event-path test-events/pr-merged.json
        ;;
    *)
        echo "❌ Opção inválida"
        exit 1
        ;;
esac

echo ""
echo "✅ Teste concluído!"