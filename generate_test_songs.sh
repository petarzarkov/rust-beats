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
    
    # Run cargo to generate a song
    cargo run --release
    
    # Copy the final song (MP3 only) and metadata with the dated filename
    cp output/final_song.mp3 docs/songs/song-${DATE}.mp3
    cp output/song_metadata.json docs/songs/song-${DATE}.json
    echo "Created: docs/songs/song-${DATE}.mp3"
    echo "Created: docs/songs/song-${DATE}.json"
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
for file in $(ls -t song-*.mp3 2>/dev/null); do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> ../songs.json
    fi
    date=$(echo $file | sed 's/song-\(.*\)\.mp3/\1/')
    size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
    
    metadata_file="song-${date}.json"
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

