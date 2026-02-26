#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

check_service() {
    local name="$1"
    local cmd="$2"
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}[OK]${NC} $name"
    else
        echo -e "${RED}[FAIL]${NC} $name"
    fi
}

echo -e "${YELLOW}=== NEXUS Health Check ===${NC}"
echo ""

check_service "PostgreSQL" "pg_isready -h localhost -p 5432 -U nexus"
check_service "Neo4j" "curl -sf http://localhost:7474"
check_service "Qdrant" "curl -sf http://localhost:6333/healthz"
check_service "InfluxDB" "curl -sf http://localhost:8086/health"
check_service "Redis" "redis-cli ping"
check_service "Ollama" "curl -sf http://localhost:11434/api/tags"
check_service "Nexus API" "curl -sf http://localhost:3001/health"

echo ""
echo -e "${YELLOW}=== Done ===${NC}"
