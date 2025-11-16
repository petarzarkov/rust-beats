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
    
    # Calculate date (i days ago)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        DATE=$(date -v-${i}d +%Y-%m-%d)
    else
        # Linux
        DATE=$(date -d "${i} days ago" +%Y-%m-%d)
    fi
    
    # Run cargo to generate a beat
    cargo run --release
    
    # Copy the final song with the dated filename
    cp output/final_song.wav docs/songs/song-${DATE}.wav
    echo "Created: docs/songs/song-${DATE}.wav"
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
for file in $(ls -t song-*.wav 2>/dev/null); do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> ../songs.json
    fi
    date=$(echo $file | sed 's/song-\(.*\)\.wav/\1/')
    size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    echo "  {\"filename\": \"$file\", \"date\": \"$date\", \"size\": $size}" >> ../songs.json
done
echo "" >> ../songs.json
echo "]" >> ../songs.json

cd ../..
echo "songs.json generated!"
cat docs/songs.json

