#!/usr/bin/env python3
"""
Python script to test the Rust PostgreSQL API with Customers and Loans data
Saves each response to JSON and YAML files
"""

import requests
import json
import yaml
import os
from datetime import datetime

# Base URL
BASE_URL = "http://localhost:8080"

# Create output directory
OUTPUT_DIR = "api_responses"
os.makedirs(OUTPUT_DIR, exist_ok=True)

def save_response(name, data):
    """Save response to both JSON and YAML files"""
    # Create safe filename from the name
    filename = name.lower().replace(" ", "_").replace(":", "").replace("=", "").replace(">", "gt").replace("<", "lt")
    
    # Add timestamp to the data
    data_with_meta = {
        "timestamp": datetime.now().isoformat(),
        "query_name": name,
        "response": data
    }
    
    # Save as JSON
    json_path = os.path.join(OUTPUT_DIR, f"{filename}.json")
    with open(json_path, 'w') as f:
        json.dump(data_with_meta, f, indent=2)
    print(f"   💾 Saved to: {json_path}")

def print_response(name, response):
    """Pretty print and save the response"""
    print(f"\n{'='*70}")
    print(f"{name}")
    print(f"{'='*70}")
    data = response.json()
    
    # Show summary
    print(f"Page: {data.get('page', 'N/A')} | Page Size: {data.get('page_size', 'N/A')}")
    print(f"Records: {data.get('count', 0)} | Total: {data.get('total_count', 'N/A')}")
    print(f"{'-'*70}")
    
    # Show first 3 records
    if data.get('data'):
        for i, record in enumerate(data['data'][:3]):
            print(json.dumps(record, indent=2))
            if i < min(2, len(data['data']) - 1):
                print()
        
        if len(data['data']) > 3:
            print(f"... and {len(data['data']) - 3} more records")
    else:
        print("No data found")
    
    # Save to files
    save_response(name, data)

print("\n" + "="*70)
print("🏦 Testing Rust PostgreSQL API - Banking Database")
print("="*70)

# 1. Health check
print("\n1. Health Check")
response = requests.get(f"{BASE_URL}/health")
print_response("1_health_check", response)

# 2. Get first page of customers (10 per page)
print("\n2. Get first page of customers (10 per page)")
response = requests.get(f"{BASE_URL}/customers?page=1&page_size=10")
print_response("2_customers_page_1", response)

# 3. Get customers from USA
print("\n3. Get customers from USA")
response = requests.get(f"{BASE_URL}/customers/country=USA?page=1&page_size=10")
print_response("3_customers_usa", response)

# 4. Get customers by full name with space (e.g., "John Smith")
print("\n4. Get customer by full_name='John Smith'")
response = requests.get(f"{BASE_URL}/customers/full_name=John%20Smith")
print_response("4_customer_by_name", response)

# 5. Get customers born after 1990
print("\n5. Get customers born after 1990-01-01")
response = requests.get(f"{BASE_URL}/customers/birth_date>1990-01-01?sort=birth_date&order=desc&page=1&page_size=10")
print_response("5_customers_born_after_1990", response)

# 6. Get all loans (paginated)
print("\n6. Get first page of loans (10 per page)")
response = requests.get(f"{BASE_URL}/loans?page=1&page_size=10&sort=debt_amount&order=desc")
print_response("6_loans_page_1", response)

# 6. Get loans with debt > 30000
print("\n7. Get loans with debt_amount > 30000")
response = requests.get(f"{BASE_URL}/loans/debt_amount>30000?sort=debt_amount&order=desc&page=1&page_size=10")
print_response("7_loans_high_debt", response)

# 7. Get loans with overdue amount
print("\n8. Get loans with overdue_debt_amount > 0")
response = requests.get(f"{BASE_URL}/loans/overdue_debt_amount>0?page=1&page_size=20")
print_response("8_loans_overdue", response)

# 8. Get active loans
print("\n9. Get active loans")
response = requests.get(f"{BASE_URL}/loans/loan_status=active?page=1&page_size=15")
print_response("9_loans_active", response)

# 9. Get loans for specific customer
print("\n10. Get loans for customer_id = 10")
response = requests.get(f"{BASE_URL}/loans/customer_id=10")
print_response("10_loans_customer_10", response)

# 10. Get loans by date range and status
print("\n11. Get active loans reported after 2024-01-01")
response = requests.get(f"{BASE_URL}/loans/report_date>2024-01-01&loan_status=active?page=1&page_size=10")
print_response("11_loans_recent_active", response)

# 11. Get mortgage loans
print("\n12. Get mortgage loans")
response = requests.get(f"{BASE_URL}/loans/loan_type=Mortgage?page=1&page_size=10")
print_response("12_loans_mortgage", response)

# 12. Complex query: High debt overdue loans
print("\n13. Complex: Debt > 20000 AND overdue > 1000")
response = requests.get(f"{BASE_URL}/loans/debt_amount>20000&overdue_debt_amount>1000?sort=overdue_debt_amount&order=desc&page=1&page_size=10")
print_response("13_loans_high_risk", response)

# 13. Get customers from multiple countries (Germany)
print("\n14. Get customers from Germany with pagination")
response = requests.get(f"{BASE_URL}/customers/country=Germany?page=1&page_size=20&sort=birth_date&order=asc")
print_response("14_customers_germany", response)

# 14. Get customers with ID range
print("\n15. Get customers with customer_id >= 100 and <= 200")
response = requests.get(f"{BASE_URL}/customers/customer_id>=100&customer_id<=200?page=1&page_size=10")
print_response("15_customers_id_range", response)

# 15. Get loans by contract number pattern
print("\n16. Get specific loan by contract number (shows debt over time)")
response = requests.get(f"{BASE_URL}/loans/contract_number=LOAN-00000001?sort=report_date&order=asc")
print_response("16_loan_debt_history", response)

# 16. Show another contract's debt history
print("\n17. Another contract's debt progression")
response = requests.get(f"{BASE_URL}/loans/contract_number=LOAN-00000050?sort=report_date&order=asc")
print_response("17_loan_debt_progression", response)

print("\n" + "="*70)
print("✅ All tests completed!")
print(f"📁 All responses saved to: {OUTPUT_DIR}/")
print("="*70)

# Print summary
print("\n📊 Summary of Available Queries:")
print("-" * 70)
print("Customers Table:")
print("  - Filter by country: /customers/country=USA")
print("  - Filter by full_name: /customers/full_name=John%20Smith")
print("  - Filter by birth_date: /customers/birth_date>1990-01-01")
print("  - Filter by status: /customers/status=active")
print("  - Sort by any column: ?sort=birth_date&order=desc")
print()
print("Loans Table:")
print("  - Filter by customer: /loans/customer_id=10")
print("  - Filter by debt: /loans/debt_amount>30000")
print("  - Filter by overdue: /loans/overdue_debt_amount>0")
print("  - Filter by status: /loans/loan_status=active")
print("  - Filter by type: /loans/loan_type=Mortgage")
print("  - Filter by date: /loans/report_date>2024-01-01")
print("  - Multiple filters: /loans/debt_amount>20000&loan_status=active")
print()
print("Pagination: ?page=1&page_size=10")
print("Sorting: ?sort=column_name&order=asc/desc")
print()
print("💡 Tip: URL encode spaces in values (use %20 or +)")
print("   Example: /customers/full_name=John%20Smith")
print("="*70)