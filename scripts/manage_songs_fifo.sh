#!/bin/bash
# scripts/manage_songs_fifo.sh
set -euo pipefail

SONGS_DIR="docs/songs"
SONGS_JSON="docs/songs.json"
TEMP_KEEP_BASES="/tmp/keep_bases_for_cleanup.txt"

# --- PART 1: Rebuild songs.json from all MP3 files in directory (FIFO: keep latest 8) ---

echo "🎧 Starting song list update and cleanup..."

# 1.1 Find all MP3 files in the songs directory, sorted by modification time (newest first)
# Create a temporary file list to avoid subshell issues
TEMP_MP3_LIST="/tmp/mp3_list_$$.txt"
ls -t "$SONGS_DIR"/*.mp3 2>/dev/null > "$TEMP_MP3_LIST" || true

if [ ! -s "$TEMP_MP3_LIST" ]; then
    rm -f "$TEMP_MP3_LIST"
    echo "Error: No MP3 files found in $SONGS_DIR to process. Exiting."
    exit 1
fi

MP3_COUNT=$(wc -l < "$TEMP_MP3_LIST" | tr -d ' ')
echo "Found $MP3_COUNT MP3 files in directory:"
while IFS= read -r mp3; do
    echo "  $(basename "$mp3")"
done < "$TEMP_MP3_LIST"

# 1.2 Build JSON array from all MP3 files
TEMP_JSON_FILE="/tmp/songs_json_$$.json"
echo "[]" > "$TEMP_JSON_FILE"

while IFS= read -r MP3_FILE; do
    FILENAME=$(basename "$MP3_FILE")
    DATE=$(echo "$FILENAME" | grep -oE '^[0-9]{4}-[0-9]{2}-[0-9]{2}' | head -1)
    
    # Get file size (works on both Linux and macOS)
    if command -v stat >/dev/null 2>&1; then
        if stat -c%s "$MP3_FILE" >/dev/null 2>&1; then
            SIZE=$(stat -c%s "$MP3_FILE" 2>/dev/null || echo "0")
        else
            SIZE=$(stat -f%z "$MP3_FILE" 2>/dev/null || echo "0")
        fi
    else
        SIZE=$(wc -c < "$MP3_FILE" 2>/dev/null || echo "0")
    fi
    
    # Check for corresponding JSON metadata file
    JSON_FILE="${MP3_FILE%.mp3}.json"
    
    if [ -f "$JSON_FILE" ]; then
        NAME=$(jq -r '.name // ""' "$JSON_FILE" 2>/dev/null || echo "")
        GENRE=$(jq -c '.genre // []' "$JSON_FILE" 2>/dev/null || echo "[]")
        
        # Build song JSON object
        if [ -n "$NAME" ] && [ "$NAME" != "null" ] && [ "$NAME" != "" ]; then
            SONG_JSON=$(jq -n \
                --arg filename "$FILENAME" \
                --arg date "$DATE" \
                --argjson size "$SIZE" \
                --arg name "$NAME" \
                --argjson genre "$GENRE" \
                '{filename: $filename, date: $date, size: $size, name: $name, genre: $genre}')
        else
            SONG_JSON=$(jq -n \
                --arg filename "$FILENAME" \
                --arg date "$DATE" \
                --argjson size "$SIZE" \
                '{filename: $filename, date: $date, size: $size}')
        fi
    else
        SONG_JSON=$(jq -n \
            --arg filename "$FILENAME" \
            --arg date "$DATE" \
            --argjson size "$SIZE" \
            '{filename: $filename, date: $date, size: $size}')
    fi
    
    # Add to array (maintaining order - newest first)
    jq --argjson song "$SONG_JSON" '. += [$song]' "$TEMP_JSON_FILE" > "${TEMP_JSON_FILE}.tmp" && mv "${TEMP_JSON_FILE}.tmp" "$TEMP_JSON_FILE"
done < "$TEMP_MP3_LIST"

# Clean up temp MP3 list
rm -f "$TEMP_MP3_LIST"

# 1.3 Keep only the latest 8 songs (FIFO)
NEW_JSON_ARRAY=$(jq '.[0:8]' "$TEMP_JSON_FILE")

echo "$NEW_JSON_ARRAY" > "$SONGS_JSON"
rm -f "$TEMP_JSON_FILE"

echo "✅ $SONGS_JSON updated (keeping latest 8 songs)."

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
# Use a temporary file list to avoid subshell issues
TEMP_FILE_LIST="/tmp/files_to_check_$$.txt"
find "$SONGS_DIR" -maxdepth 1 -type f -print0 2>/dev/null > "$TEMP_FILE_LIST"

while IFS= read -r -d $'\0' file; do
    FILENAME=$(basename "$file")

    # Check if the filename is in the list of files to keep
    if grep -q "^${FILENAME}$" "$TEMP_KEEP_BASES" 2>/dev/null; then
        : # Keep this file
    else
        echo "Deleting old file (FIFO): $FILENAME"
        rm -fv "$SONGS_DIR/$FILENAME"
    fi
done < "$TEMP_FILE_LIST"

# Clean up temporary file list
rm -f "$TEMP_FILE_LIST"

# Clean up temporary file
rm -f "$TEMP_KEEP_BASES"

echo "✅ Physical cleanup complete."
echo "Songs remaining in $SONGS_DIR:"
ls -lh "$SONGS_DIR"