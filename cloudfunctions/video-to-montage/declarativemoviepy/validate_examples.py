#!/usr/bin/env python3

import json
import jsonschema
from pathlib import Path
import sys

def load_schema(schema_path: str = "declarativemoviepy/composition_schema.json"):
    """Load the JSON schema from file."""
    with open(schema_path, 'r', encoding='utf-8') as f:
        return json.load(f)

def validate_composition(composition_data, schema):
    """Validate a composition against the schema."""
    try:
        jsonschema.validate(composition_data, schema)
        return True, None
    except jsonschema.ValidationError as e:
        return False, str(e)
    except Exception as e:
        return False, f"Unexpected error: {str(e)}"

def validate_examples():
    """Validate all example JSON files against the schema."""
    schema = load_schema()
    examples_dir = Path("declarativemoviepy/examples")
    
    if not examples_dir.exists():
        print(f"Examples directory not found: {examples_dir}")
        return
    
    example_files = list(examples_dir.glob("*.json"))
    if not example_files:
        print("No JSON example files found")
        return
    
    print(f"Validating {len(example_files)} example files against schema...")
    print("=" * 60)
    
    valid_count = 0
    total_count = len(example_files)
    
    for example_file in sorted(example_files):
        print(f"\nValidating: {example_file.name}")
        print("-" * 40)
        
        try:
            with open(example_file, 'r', encoding='utf-8') as f:
                composition_data = json.load(f)
            
            is_valid, error_msg = validate_composition(composition_data, schema)
            
            if is_valid:
                print("✅ VALID")
                valid_count += 1
            else:
                print("❌ INVALID")
                print(f"Error: {error_msg}")
                
        except json.JSONDecodeError as e:
            print("❌ INVALID JSON")
            print(f"JSON Error: {e}")
        except FileNotFoundError:
            print("❌ FILE NOT FOUND")
    
    print("\n" + "=" * 60)
    print(f"Summary: {valid_count}/{total_count} examples are valid")
    
    if valid_count == total_count:
        print("🎉 All examples pass schema validation!")
        return True
    else:
        print(f"⚠️  {total_count - valid_count} examples have validation errors")
        return False

def main():
    """Main function to run validation."""
    print("JSON Schema Validation for Declarative MoviePy Examples")
    print("=" * 60)
    
    success = validate_examples()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()