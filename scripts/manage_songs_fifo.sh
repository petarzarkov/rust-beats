#!/bin/bash
# scripts/manage_songs_fifo.sh
set -euo pipefail

SONGS_DIR="docs/songs"
SONGS_JSON="docs/songs.json"
TEMP_KEEP_BASES="/tmp/keep_bases_for_cleanup.txt"

# --- PART 1: Update songs.json (Adds new song, truncates to 7, FIFO) ---

echo "🎧 Starting song list update and cleanup..."

# 1.1 Find the newest MP3 and its metadata JSON in the songs directory (just copied)
NEW_MP3=$(ls -t "$SONGS_DIR"/*.mp3 2>/dev/null | head -1)
NEW_JSON=$(ls -t "$SONGS_DIR"/*.json 2>/dev/null | head -1)

if [ -z "$NEW_MP3" ]; then
    echo "Error: No new MP3 file found in $SONGS_DIR to process. Exiting."
    exit 1
fi

NEW_FILENAME=$(basename "$NEW_MP3")
echo "New file found: $NEW_FILENAME"

# 1.2 Extract metadata for the new song
NEW_DATE=$(echo "$NEW_FILENAME" | grep -oE '^[0-9]{4}-[0-9]{2}-[0-9]{2}' | head -1)
NEW_SIZE=$(stat -c%s "$NEW_MP3" 2>/dev/null || echo "0")

if [ -n "$NEW_JSON" ] && [ -f "$NEW_JSON" ]; then
    NEW_NAME=$(jq -r '.name // ""' "$NEW_JSON" 2>/dev/null || echo "")
    NEW_GENRE=$(jq -c '.genre // []' "$NEW_JSON" 2>/dev/null || echo "[]")
    ESCAPED_NAME=$(echo "$NEW_NAME" | sed 's/\"/\\\"/g')
    NEW_SONG_JSON="  {\"filename\": \"$NEW_FILENAME\", \"date\": \"$NEW_DATE\", \"size\": $NEW_SIZE, \"name\": \"$ESCAPED_NAME\", \"genre\": $NEW_GENRE}"
else
    NEW_SONG_JSON="  {\"filename\": \"$NEW_FILENAME\", \"date\": \"$NEW_DATE\", \"size\": $NEW_SIZE}"
fi

# 1.3 Update songs.json (FIFO: Add new, remove oldest)
if [ -f "$SONGS_JSON" ]; then
    # Insert the new song JSON at the beginning of the array
    TEMP_JSON_ARRAY=$(jq --argjson new_song "$NEW_SONG_JSON" '. | [ $new_song ] + .' "$SONGS_JSON")
else
    TEMP_JSON_ARRAY="[ $NEW_SONG_JSON ]"
fi

NEW_JSON_ARRAY=$(echo "$TEMP_JSON_ARRAY" | jq '.[0:8]')

echo "$NEW_JSON_ARRAY" > "$SONGS_JSON"
echo "✅ $SONGS_JSON updated (keeping latest 8 songs temporarily)."

# --- PART 2: Clean up files based on the new docs/songs.json list ---

# 2.1 Generate list of filenames to KEEP from the new JSON
# Extract MP3/WAV filenames
jq -r '.[].filename' "$SONGS_JSON" > "$TEMP_KEEP_BASES"

# Extract corresponding JSON metadata filenames
jq -r '.[].filename | sub("\\.(mp3|wav)$"; ".json")' "$SONGS_JSON" >> "$TEMP_KEEP_BASES"

# Add special files
echo ".gitkeep" >> "$TEMP_KEEP_BASES"
echo "songs.json" >> "$TEMP_KEEP_BASES"

# Remove duplicates
sort -u -o "$TEMP_KEEP_BASES" "$TEMP_KEEP_BASES"

echo "Files to KEEP (base names):"
cat "$TEMP_KEEP_BASES"

# 2.2 Iterate and DELETE files not on the keep list
cd "$SONGS_DIR"

# Loop through ALL files in docs/songs/
find . -maxdepth 1 -type f -print0 2>/dev/null | while IFS= read -r -d $'\0' file; do
    FILENAME=$(basename "$file")
    
    # Check if the filename is in the list of files to keep
    if grep -q "^${FILENAME}$" "$TEMP_KEEP_BASES" 2>/dev/null; then
        : # Keep this file
    else
        echo "Deleting old file (FIFO): $FILENAME"
        rm -f "$FILENAME"
    fi
done

# Clean up temporary file
rm -f "$TEMP_KEEP_BASES"

echo "✅ Physical cleanup complete."
echo "Songs remaining in $SONGS_DIR:"
ls -lh