#!/bin/bash
# scripts/create_video.sh

# Exit immediately if a command exits with a non-zero status.
set -e

COVER_ART="docs/logo/rust_beats.png"
OUTPUT_VIDEO="output/youtube_video.mp4"

# Find the most recent MP3 file in the output directory
MP3_FILE=$(ls -t output/*.mp3 2>/dev/null | head -1)

if [ -z "$MP3_FILE" ]; then
  echo "Error: No MP3 file found in output/"
  exit 1
fi

if [ ! -f "$COVER_ART" ]; then
  echo "Error: Cover art not found at $COVER_ART"
  exit 1
fi

echo "Creating video with dynamic effects..."
echo "  Cover: $COVER_ART"
echo "  Audio: $MP3_FILE"
echo "  Output: $OUTPUT_VIDEO"

# Get audio duration for effect calculations
# Use default 180s if ffprobe fails or audio is too short
AUDIO_DURATION=$(ffprobe -i "$MP3_FILE" -show_entries format=duration -v quiet -of csv="p=0" 2>/dev/null || echo "180")
# Ensure duration is a number for bc
if ! [[ "$AUDIO_DURATION" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
    AUDIO_DURATION="180"
fi


# Randomize effects for each video to ensure uniqueness
# Use song filename hash as seed for consistent randomization per song
if command -v md5sum >/dev/null 2>&1; then
  SONG_HASH=$(echo -n "$MP3_FILE" | md5sum | cut -d' ' -f1 | head -c 8)
elif command -v md5 >/dev/null 2>&1; then
  SONG_HASH=$(echo -n "$MP3_FILE" | md5 | cut -d' ' -f4 | head -c 8)
else # Fallback for systems without md5/md5sum (e.g., some minimal environments)
  SONG_HASH=$(echo -n "$MP3_FILE" | shasum -a 256 | cut -d' ' -f1 | head -c 8)
fi
RAND_SEED=$(printf "%d" 0x${SONG_HASH} 2>/dev/null || echo "12345") # Convert hex hash to decimal seed

# Randomize shake parameters for a smooth, subtle vibration
VIBRATION_INTENSITY=$(echo "scale=1; 0.5 + ($RAND_SEED % 10) / 10.0" | bc 2>/dev/null || echo "1.0")  # 0.5-1.5 pixels for smooth vibration
VIBRATION_FREQ=$(echo "scale=2; 2.0 + ($RAND_SEED % 30) / 10.0" | bc 2>/dev/null || echo "3.0") # 2-5 Hz for smooth, slow vibration
SHAKE_TRIGGER_INTERVAL=$(echo "scale=1; 8.0 + ($RAND_SEED % 100) / 10.0" | bc 2>/dev/null || echo "10.0") # Avg 8-18s between bigger "shakes"
SHAKE_DURATION=$(echo "scale=2; 0.10 + ($RAND_SEED % 15) / 100" | bc 2>/dev/null || echo "0.15") # Duration of bigger "shake" burst

# Randomize color enhancement (subtle variations)
BRIGHTNESS=$(echo "scale=3; 0.01 + ($RAND_SEED % 20) / 1000" | bc 2>/dev/null || echo "0.02") # 0.01 to 0.03
CONTRAST=$(echo "scale=3; 1.03 + ($RAND_SEED % 40) / 1000" | bc 2>/dev/null || echo "1.05") # 1.03 to 1.07
SATURATION=$(echo "scale=3; 1.08 + ($RAND_SEED % 40) / 1000" | bc 2>/dev/null || echo "1.1") # 1.08 to 1.12

echo "  Effects: Vibration (intensity=${VIBRATION_INTENSITY}px, freq=${VIBRATION_FREQ}Hz)"
echo "             Occasional Shake (avg interval=${SHAKE_TRIGGER_INTERVAL}s, duration=${SHAKE_DURATION}s)"
echo "             Color (brightness=${BRIGHTNESS}, contrast=${CONTRAST}, saturation=${SATURATION})"


# Calculate total padding needed for all effects (for the outer pad)
# Convert to integer for padding calculation (round up for safety)
TOTAL_PADDING=$(echo "scale=0; ($VIBRATION_INTENSITY + 5) / 1" | bc 2>/dev/null || echo "6")

# Create video with continuous vibration + occasional shakes
# Vibration: Continuous subtle shake using sin/cos at high frequency
# Occasional shakes: Deterministic pseudo-random shakes using mod() and sin/cos combinations

ffmpeg -y -loop 1 -i "$COVER_ART" -i "$MP3_FILE" \
  -filter_complex "\
    [0:v]scale=1280:720:force_original_aspect_ratio=decrease[scaled];\
    [scaled]pad=1280:720:(ow-iw)/2:(oh-ih)/2[centered];\
    [centered]pad=1280+${TOTAL_PADDING}*2:720+${TOTAL_PADDING}*2:${TOTAL_PADDING}:${TOTAL_PADDING}[padded];\
    [padded]crop=1280:720:'${TOTAL_PADDING}+${VIBRATION_INTENSITY}*sin(2*PI*t*${VIBRATION_FREQ})+if(lt(mod(t,${SHAKE_TRIGGER_INTERVAL}),${SHAKE_DURATION}),${VIBRATION_INTENSITY}*3*sin(20*PI*t),0)':'${TOTAL_PADDING}+${VIBRATION_INTENSITY}*cos(2*PI*t*${VIBRATION_FREQ})+if(lt(mod(t+${SHAKE_TRIGGER_INTERVAL}*0.3,${SHAKE_TRIGGER_INTERVAL}),${SHAKE_DURATION}),${VIBRATION_INTENSITY}*3*cos(20*PI*t),0)'[shaken_vibrated];\
    [shaken_vibrated]eq=brightness=${BRIGHTNESS}:contrast=${CONTRAST}:saturation=${SATURATION}[v]" \
  -map "[v]" -map 1:a \
  -c:v libx264 -preset medium -crf 23 -pix_fmt yuv420p \
  -c:a aac -b:a 192k \
  -shortest -r 30 \
  "$OUTPUT_VIDEO"

if [ $? -eq 0 ]; then
  echo "✅ Successfully created $OUTPUT_VIDEO"
  ls -lh "$OUTPUT_VIDEO"
else
  echo "❌ Error creating video"
  exit 1
fi