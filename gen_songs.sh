#!/bin/bash

# Generate 7 songs with different dates for testing
echo "Generating 7 test songs..."

# Create docs/songs directory if it doesn't exist
mkdir -p docs/songs

# Generate songs for the last 7 days
for i in {6..0}; do
    echo ""
    echo "==================================="
    echo "Generating song $((7-i)) of 7..."
    echo "==================================="
    
    # Calculate date (i days ago from today)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        DATE=$(date -v-${i}d +%Y-%m-%d)
    else
        # Linux
        DATE=$(date -d "${i} days ago" +%Y-%m-%d)
    fi
    
    echo "Setting date to: $DATE"
    
    # Run cargo to generate a song with the specific date
    SONG_DATE="$DATE" cargo run --release
    
    # Find the newest MP3 and JSON files in output/ (generated with format: {author}_{song_name}_{date})
    MP3_FILE=$(ls -t output/*.mp3 2>/dev/null | head -1)
    JSON_FILE=$(ls -t output/*.json 2>/dev/null | head -1)
    
    if [ -z "$MP3_FILE" ]; then
        echo "Error: No MP3 file found in output/"
        exit 1
    fi
    
    # Extract filename without path
    MP3_BASENAME=$(basename "$MP3_FILE")
    JSON_BASENAME=$(basename "$JSON_FILE" 2>/dev/null || echo "")
    
    # Copy files preserving the original names
    cp "$MP3_FILE" docs/songs/
    if [ -n "$JSON_BASENAME" ] && [ -f "$JSON_FILE" ]; then
        cp "$JSON_FILE" docs/songs/
    fi
    
    echo "Created: docs/songs/$MP3_BASENAME"
    if [ -n "$JSON_BASENAME" ]; then
        echo "Created: docs/songs/$JSON_BASENAME"
    fi
done

echo ""
echo "==================================="
echo "All 7 songs generated!"
echo "==================================="
ls -lh docs/songs/

# Generate the songs.json file
echo "Generating songs.json..."
cd docs/songs
echo "[" > ../songs.json
first=true
for file in $(ls -t *.mp3 2>/dev/null); do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> ../songs.json
    fi
    # Extract date from filename (format: {author}_{song_name}_{date}.mp3)
    # Date is the last part before .mp3, format YYYY-MM-DD
    date=$(echo $file | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' | tail -1)
    size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    
    # Find corresponding metadata file (same base name but .json)
    metadata_file="${file%.mp3}.json"
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
echo "songs.json generated!"
cat docs/songs.json

