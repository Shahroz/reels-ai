#!/bin/bash

#! Interactive validation runner for New Relic observability enhancements.
#!
#! This script provides a user-friendly interface for running validation tests
#! against New Relic to verify that observability enhancements are working correctly.
#! Includes environment setup, validation execution, and guided verification steps.
#! Ensures comprehensive testing of each phase before production deployment.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print banner
echo -e "${BLUE}"
echo "🧪 New Relic Observability Validation Suite"
echo "=============================================="
echo -e "${NC}"

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}📋 Checking prerequisites...${NC}"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Check if NEW_RELIC_LICENSE_KEY is set
    if [[ -z "${NEW_RELIC_LICENSE_KEY:-}" ]]; then
        echo -e "${RED}❌ NEW_RELIC_LICENSE_KEY environment variable not set.${NC}"
        echo "Please set your New Relic license key:"
        echo "export NEW_RELIC_LICENSE_KEY='your_license_key_here'"
        exit 1
    fi
    
    # Check if we can build with newrelic feature
    if ! cargo check --features newrelic &> /dev/null; then
        echo -e "${RED}❌ Cannot build with newrelic feature. Check dependencies.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites met${NC}"
}

# Show menu
show_menu() {
    echo ""
    echo -e "${BLUE}Select validation phase to run:${NC}"
    echo "1) Phase 1: Error Attribution & Span Enrichment"
    echo "2) Phase 2: Database Query Instrumentation" 
    echo "3) Phase 3: Custom Business Events & Metrics"
    echo "4) Run All Phases"
    echo "5) Exit"
    echo ""
}

# Run specific validation
run_validation() {
    local phase=$1
    local script_name=$2
    local description=$3
    
    echo -e "${YELLOW}🚀 Running $description...${NC}"
    echo ""
    
    # Run the validation script
    if cargo run --bin "$script_name" --features newrelic; then
        echo ""
        echo -e "${GREEN}✅ $description completed successfully${NC}"
        
        # Provide verification guidance
        echo ""
        echo -e "${BLUE}📊 Manual Verification Steps:${NC}"
        case $phase in
            1)
                echo "1. Open New Relic → APM → Errors Inbox"
                echo "2. Filter by app name 'narrativ-validation'"
                echo "3. Look for test errors with user context"
                echo "4. Check distributed traces for error attribution"
                ;;
            2)
                echo "1. Open New Relic → APM → Distributed Tracing"
                echo "2. Search for database query spans"
                echo "3. Verify query performance metrics are visible"
                echo "4. Check for slow query alerts"
                ;;
            3)
                echo "1. Open New Relic → Insights → Query Your Data"
                echo "2. Run NRQL: SELECT * FROM UserRegistration SINCE 1 hour ago"
                echo "3. Run NRQL: SELECT * FROM BusinessMetrics SINCE 1 hour ago"
                echo "4. Verify custom events contain expected attributes"
                ;;
        esac
        
        echo ""
        echo -e "${YELLOW}⏱️  Allow 2-3 minutes for data to appear in New Relic${NC}"
        
        # Ask user to verify
        echo ""
        read -p "Press Enter after verifying data in New Relic..."
        
    else
        echo ""
        echo -e "${RED}❌ $description failed${NC}"
        echo "Check the error output above for details."
        exit 1
    fi
}

# Main execution
main() {
    check_prerequisites
    
    while true; do
        show_menu
        read -p "Enter your choice (1-5): " choice
        
        case $choice in
            1)
                run_validation 1 "phase1_error_validation" "Phase 1: Error Attribution & Span Enrichment"
                ;;
            2)
                run_validation 2 "phase2_database_validation" "Phase 2: Database Query Instrumentation"
                ;;
            3)
                run_validation 3 "phase3_events_validation" "Phase 3: Custom Business Events & Metrics"
                ;;
            4)
                echo -e "${YELLOW}🔄 Running all validation phases...${NC}"
                run_validation 1 "phase1_error_validation" "Phase 1: Error Attribution & Span Enrichment"
                run_validation 2 "phase2_database_validation" "Phase 2: Database Query Instrumentation"
                run_validation 3 "phase3_events_validation" "Phase 3: Custom Business Events & Metrics"
                echo -e "${GREEN}🎉 All validation phases completed successfully!${NC}"
                ;;
            5)
                echo -e "${BLUE}👋 Goodbye!${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid choice. Please select 1-5.${NC}"
                ;;
        esac
        
        echo ""
        echo -e "${GREEN}✨ Validation phase completed${NC}"
        echo ""
    done
}

# Run main function
main "$@"