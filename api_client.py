#!/usr/bin/env python3
"""
Simple Python script to test the Rust PostgreSQL API
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
    
    # # Save as YAML
    # yaml_path = os.path.join(OUTPUT_DIR, f"{filename}.yaml")
    # with open(yaml_path, 'w') as f:
    #     yaml.dump(data_with_meta, f, default_flow_style=False, sort_keys=False)
    # print(f"   💾 Saved to: {yaml_path}")

def print_response(name, response):
    """Pretty print and save the response"""
    print(f"\n{'='*60}")
    print(f"{name}")
    print(f"{'='*60}")
    data = response.json()
    print(json.dumps(data, indent=2))
    
    # Save to files
    save_response(name, data)

# # Example 1: Health check
# print("\n1. Health Check")
# response = requests.get(f"{BASE_URL}/health")
# print_response("1_health_check", response)

# # Example 2: Get all data
# print("\n2. Get all sample_data")
# response = requests.get(f"{BASE_URL}/sample_data")
# print_response("2_get_all_sample_data", response)

# # Example 3: Filter by date
# print("\n3. Filter by date=2025-10-10")
# response = requests.get(f"{BASE_URL}/sample_data/date=2025-10-10")
# print_response("3_filter_by_date", response)

# # Example 4: Filter by ID >= 3
# print("\n4. Filter by id>=3")
# response = requests.get(f"{BASE_URL}/sample_data/id>=3")
# print_response("4_filter_by_id_gte_3", response)

# Example 5: Filter by value > 150
# print("\n5. Filter by value>150")
# response = requests.get(f"{BASE_URL}/sample_data/value>150")
# print_response("5_filter_by_value_gt_150", response)

# Example 6: Multiple filters
# print("\n6. Multiple filters (date=2025-10-10 AND value>100)")
# response = requests.get(f"{BASE_URL}/sample_data/date=2025-10-10&value>100")
# print_response("6_multiple_filters", response)

# Example 7: Get all users
print("\n7. Get all users")
response = requests.get(f"{BASE_URL}/users")
print_response("7_get_all_users", response)

# Example 8: Filter by name with spaces (URL encoded)
print("\n8. Filter by name='Alice Johnson'")
response = requests.get(f"{BASE_URL}/users/name=Alice%20Johnson")
print_response("8_filter_by_name", response)

# Example 9: Pagination
print("\n9. Pagination (page=1, page_size=2)")
response = requests.get(f"{BASE_URL}/sample_data?page=1&page_size=2")
print_response("9_pagination", response)

# Example 10: Sorting
print("\n10. Sort by value descending")
response = requests.get(f"{BASE_URL}/sample_data?sort=value&order=desc")
print_response("10_sort_by_value_desc", response)

# Example 11: Complex query
print("\n11. Complex: id>1 AND value<300, sorted by date")
response = requests.get(f"{BASE_URL}/sample_data/id>1&value<300?sort=date&order=asc")
print_response("11_complex_query", response)

print("\n" + "="*60)
print("✅ All tests completed!")
print(f"📁 All responses saved to: {OUTPUT_DIR}/")
print("="*60)