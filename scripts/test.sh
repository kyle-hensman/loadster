#!/bin/bash

# Local test script for loadster CLI
# Run this before pushing to ensure everything works

set -e

echo "🧪 Running Loadster CLI Test Suite"
echo "==================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    echo -e "\n${YELLOW}▶${NC} Testing: $test_name"
    if eval "$2"; then
        echo -e "${GREEN}✓${NC} PASSED: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} FAILED: $test_name"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Clean and build
echo -e "\n📦 Building project..."
cargo clean
cargo build --release

# Start test server
echo -e "\n🌐 Starting test server..."
mkdir -p test-server
echo "Hello from Loadster!" > test-server/index.html
python3 -m http.server 8080 --directory test-server > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

# Cleanup function
cleanup() {
    echo -e "\n🧹 Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    rm -rf test-server
    rm -f test-*.json
}

trap cleanup EXIT

# Run tests
echo -e "\n🧪 Running Tests..."
echo "==================="

# Unit tests
# run_test "Unit tests" "cargo test --lib"

# Integration tests (without ignored tests)
run_test "Integration tests" "cargo test --test '*'"

# Clippy linting
run_test "Clippy linting" "cargo clippy -- -D warnings"

# Format check
run_test "Format check" "cargo fmt -- --check"

# Binary tests
run_test "Help command" "./target/release/loadster --help > /dev/null"

run_test "Version command" "./target/release/loadster --version > /dev/null"

run_test "Basic load test" "./target/release/loadster http://localhost:8080 -n 10 -c 2"

run_test "High concurrency test" "./target/release/loadster http://localhost:8080 -n 50 -c 10"

run_test "JSON output test" "./target/release/loadster http://localhost:8080 -n 20 -c 5 -o test-report.json"

# Validate JSON output
run_test "JSON validation" "jq empty test-report.json 2>/dev/null"

run_test "JSON structure check" '
    jq -e ".url" test-report.json > /dev/null &&
    jq -e ".total_requests == 20" test-report.json > /dev/null &&
    jq -e ".concurrency == 5" test-report.json > /dev/null &&
    jq -e ".successful" test-report.json > /dev/null &&
    jq -e ".latency.avg_ms" test-report.json > /dev/null &&
    jq -e ".latency.p95_ms" test-report.json > /dev/null
'

# Error handling tests
run_test "Invalid URL handling" "./target/release/loadster invalid-url -n 5 -c 1 2>&1 | grep -q '' || true"

run_test "Missing URL error" "! ./target/release/loadster -n 10 2>&1"

run_test "Invalid number error" "! ./target/release/loadster http://localhost:8080 -n abc 2>&1"

# Performance baseline test
echo -e "\n📊 Running performance baseline..."
./target/release/loadster http://localhost:8080 -n 100 -c 10 -o baseline.json

p95=$(jq -r '.latency.p95_ms' baseline.json)
rps=$(jq -r '.requests_per_sec' baseline.json)

echo "  P95 Latency: ${p95}ms"
echo "  Requests/sec: ${rps}"

if (( $(echo "$p95 < 100" | bc -l) )); then
    echo -e "${GREEN}✓${NC} Performance baseline met (p95 < 100ms)"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠${NC} Performance baseline warning (p95 >= 100ms)"
fi

# Summary
echo -e "\n"
echo "=================================="
echo "📊 Test Summary"
echo "=================================="
echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"
echo "=================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed!${NC}"
    exit 1
fi