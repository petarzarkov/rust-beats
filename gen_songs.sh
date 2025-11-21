#!/bin/bash

# Generate 7 songs with different dates for testing
echo "Generating 7 test songs..."

# 1. Delete everything in output/*
echo "Cleaning output directory..."
rm -rf output/*

# 2. Delete everything in docs/songs/*
echo "Cleaning docs/songs directory..."
rm -rf docs/songs/*

# 3. Create/cleanup docs/songs.json
echo "[]" > docs/songs.json

# Create directories
mkdir -p output
mkdir -p docs/songs

# STEP 1: Generate all songs first
echo ""
echo "==================================="
echo "Step 1: Generating all songs..."
echo "==================================="

for i in {6..0}; do
    echo ""
    echo "Generating song $((7-i)) of 7..."
    
    # Calculate date (i days ago from today)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        DATE=$(date -v-${i}d +%Y-%m-%d)
    else
        # Linux
        DATE=$(date -d "${i} days ago" +%Y-%m-%d)
    fi
    
    echo "Date: $DATE"
    
    # Run cargo to generate a song with the specific date
    SONG_DATE="$DATE" cargo run --release
done

echo ""
echo "==================================="
echo "All songs generated!"
echo "==================================="
echo ""
ls -lh output/

# STEP 2: Copy all contents from output to docs/songs at once
echo ""
echo "==================================="
echo "Step 2: Copying all songs to docs/songs/..."
echo "==================================="

# Simple approach: find all .json files first (each represents one song)
# Then copy the audio file (prefer .mp3 over .wav) and the .json
for json_file in output/*.json; do
    # Skip if no json files
    [ -e "$json_file" ] || continue
    
    # Get base filename without extension
    base=$(basename "$json_file" .json)
    
    # Copy .mp3 if exists (preferred), otherwise .wav
    if [ -f "output/${base}.mp3" ]; then
        cp "output/${base}.mp3" docs/songs/
        echo "Copied: ${base}.mp3"
    elif [ -f "output/${base}.wav" ]; then
        cp "output/${base}.wav" docs/songs/
        echo "Copied: ${base}.wav"
    fi
    
    # Copy the .json metadata
    cp "$json_file" docs/songs/
    echo "Copied: ${base}.json"
done

echo ""
echo "Files in docs/songs/:"
ls -lh docs/songs/

# STEP 3: Generate songs.json ordered from latest to oldest
echo ""
echo "==================================="
echo "Step 3: Generating songs.json (latest to oldest)..."
echo "==================================="

cd docs/songs

echo "[" > ../songs.json
first=true

# Sort files by date (latest first) - extract date and sort
for file in $(ls *.mp3 *.wav 2>/dev/null | grep -E '[0-9]{4}-[0-9]{2}-[0-9]{2}' | sort -t_ -k1 -r); do
    base="${file%.*}"
    
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> ../songs.json
    fi
    
    # Extract date from filename
    date=$(echo $file | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | head -1)
    size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    
    # Find corresponding metadata file
    metadata_file="${base}.json"
    if [ -f "$metadata_file" ]; then
        name=$(jq -r '.name // ""' "$metadata_file" 2>/dev/null || echo "")
        genre=$(jq -c '.genre // []' "$metadata_file" 2>/dev/null || echo "[]")
        echo "  {\"filename\": \"$file\", \"date\": \"$date\", \"size\": $size, \"name\": \"$name\", \"genre\": $genre}" >> ../songs.json
    else
        echo "  {\"filename\": \"$file\", \"date\": \"$date\", \"size\": $size}" >> ../songs.json
    fi
done

echo "" >> ../songs.json
echo "]" >> ../songs.json

cd ../..

echo ""
echo "==================================="
echo "Done!"
echo "==================================="
echo ""
echo "songs.json content (latest to oldest):"
cat docs/songs.json

