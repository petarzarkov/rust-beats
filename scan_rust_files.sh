#!/bin/bash

# Output file
OUTPUT_FILE="./tmp/rust_files_output.txt"

# Remove output file if it exists
rm -f "$OUTPUT_FILE"

# Find all .rs files in src directory, sort them for consistent output
find src -name "*.rs" -type f | sort | while read -r file; do
    # Write the path comment
    echo "// $file" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Write the file contents
    cat "$file" >> "$OUTPUT_FILE"
    
    # Add a separator between files
    echo "" >> "$OUTPUT_FILE"
    echo "---" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

echo "All Rust files from src/ have been written to $OUTPUT_FILE"

