#!/bin/bash

# Verification script for Participant implementation
# This script checks that all required components are in place

set -e

echo "🔍 Verifying Participant Implementation..."
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check counter
CHECKS_PASSED=0
CHECKS_FAILED=0

# Function to check if a pattern exists in a file
check_pattern() {
    local file=$1
    local pattern=$2
    local description=$3
    
    if grep -q "$pattern" "$file" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} $description"
        ((CHECKS_PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $description"
        ((CHECKS_FAILED++))
        return 1
    fi
}

# Check Participant struct fields
echo "Checking Participant struct..."
check_pattern "stellar-contract/src/lib.rs" "pub name: soroban_sdk::Symbol" "name field exists"
check_pattern "stellar-contract/src/lib.rs" "pub latitude: i128" "latitude field exists"
check_pattern "stellar-contract/src/lib.rs" "pub longitude: i128" "longitude field exists"
check_pattern "stellar-contract/src/lib.rs" "pub is_registered: bool" "is_registered field exists"
check_pattern "stellar-contract/src/lib.rs" "pub total_waste_processed: u128" "total_waste_processed field exists"
check_pattern "stellar-contract/src/lib.rs" "pub total_tokens_earned: u128" "total_tokens_earned field exists"
echo ""

# Check new functions
echo "Checking new contract functions..."
check_pattern "stellar-contract/src/lib.rs" "fn update_participant_stats" "update_participant_stats function exists"
check_pattern "stellar-contract/src/lib.rs" "fn require_registered" "require_registered function exists"
check_pattern "stellar-contract/src/lib.rs" "pub fn deregister_participant" "deregister_participant function exists"
check_pattern "stellar-contract/src/lib.rs" "pub fn update_location" "update_location function exists"
echo ""

# Check overflow protection
echo "Checking overflow protection..."
check_pattern "stellar-contract/src/lib.rs" "checked_add" "checked_add used for overflow protection"
check_pattern "stellar-contract/src/lib.rs" "expect.*Overflow" "overflow error messages present"
echo ""

# Check registration validation
echo "Checking registration validation..."
check_pattern "stellar-contract/src/lib.rs" "Self::require_registered" "require_registered called in submit_material"
check_pattern "stellar-contract/src/lib.rs" "is_registered" "is_registered checks present"
echo ""

# Check stats updates
echo "Checking stats updates..."
check_pattern "stellar-contract/src/lib.rs" "Self::update_participant_stats" "stats update function called"
if grep -c "Self::update_participant_stats" "stellar-contract/src/lib.rs" | grep -q "[4-9]"; then
    echo -e "${GREEN}✓${NC} stats updated in multiple locations"
    ((CHECKS_PASSED++))
else
    echo -e "${RED}✗${NC} stats may not be updated in all locations"
    ((CHECKS_FAILED++))
fi
echo ""

# Check new tests
echo "Checking new tests..."
check_pattern "stellar-contract/src/lib.rs" "test_participant_persistence" "persistence test exists"
check_pattern "stellar-contract/src/lib.rs" "test_participant_initialization" "initialization test exists"
check_pattern "stellar-contract/src/lib.rs" "test_role_based_access_enforcement" "role access test exists"
check_pattern "stellar-contract/src/lib.rs" "test_participant_stats_update" "stats update test exists"
check_pattern "stellar-contract/src/lib.rs" "test_participant_stats_overflow_protection" "overflow protection test exists"
check_pattern "stellar-contract/src/lib.rs" "test_deregister_participant" "deregister test exists"
check_pattern "stellar-contract/src/lib.rs" "test_update_location" "location update test exists"
check_pattern "stellar-contract/src/lib.rs" "test_submit_material_unregistered_user" "unregistered user test exists"
check_pattern "stellar-contract/src/lib.rs" "test_batch_operations_update_participant_stats" "batch stats test exists"
check_pattern "stellar-contract/src/lib.rs" "test_multiple_participants_independent_stats" "independent stats test exists"
echo ""

# Check documentation
echo "Checking documentation..."
if [ -f "docs/PARTICIPANT_IMPLEMENTATION.md" ]; then
    echo -e "${GREEN}✓${NC} PARTICIPANT_IMPLEMENTATION.md exists"
    ((CHECKS_PASSED++))
else
    echo -e "${RED}✗${NC} PARTICIPANT_IMPLEMENTATION.md missing"
    ((CHECKS_FAILED++))
fi

if [ -f "docs/PARTICIPANT_CHANGES_SUMMARY.md" ]; then
    echo -e "${GREEN}✓${NC} PARTICIPANT_CHANGES_SUMMARY.md exists"
    ((CHECKS_PASSED++))
else
    echo -e "${RED}✗${NC} PARTICIPANT_CHANGES_SUMMARY.md missing"
    ((CHECKS_FAILED++))
fi
echo ""

# Check for common issues
echo "Checking for common issues..."

# Check that register_participant has correct signature
if grep -A 5 "pub fn register_participant" "stellar-contract/src/lib.rs" | grep -q "name: soroban_sdk::Symbol"; then
    echo -e "${GREEN}✓${NC} register_participant has correct signature"
    ((CHECKS_PASSED++))
else
    echo -e "${RED}✗${NC} register_participant signature may be incorrect"
    ((CHECKS_FAILED++))
fi

# Check that all tests are updated
if grep -A 10 "test_register_participant" "stellar-contract/src/lib.rs" | grep -q "Symbol::new"; then
    echo -e "${GREEN}✓${NC} test_register_participant updated"
    ((CHECKS_PASSED++))
else
    echo -e "${YELLOW}⚠${NC} test_register_participant may need updating"
    ((CHECKS_FAILED++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verification Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "Checks passed: ${GREEN}$CHECKS_PASSED${NC}"
echo -e "Checks failed: ${RED}$CHECKS_FAILED${NC}"
echo ""

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All verification checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run tests: cd stellar-contract && cargo test --lib"
    echo "2. Build WASM: ./scripts/build-wasm.sh"
    echo "3. Review documentation in docs/PARTICIPANT_IMPLEMENTATION.md"
    exit 0
else
    echo -e "${RED}❌ Some verification checks failed${NC}"
    echo ""
    echo "Please review the failed checks above and ensure all"
    echo "required components are properly implemented."
    exit 1
fi
