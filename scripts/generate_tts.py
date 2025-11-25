#!/usr/bin/env python3
"""
Generate TTS audio using gTTS (Google Text-to-Speech).
This script is called by Rust to generate voice narration.

gTTS provides high-quality, natural-sounding voices that work
identically on macOS and Linux, unlike pyttsx3/espeak-ng.

Usage:
    python3 generate_tts.py <text> <voice_type> <output_wav>

Example:
    python3 generate_tts.py "Hello world" "male" "/tmp/output.wav"
"""

import sys
import os


def generate_tts(text: str, voice_type: str, output_path: str) -> None:
    """
    Generate TTS audio and save as WAV file using gTTS.

    Args:
        text: Text to synthesize
        voice_type: "male" or "female" (or path to model for compatibility)
        output_path: Where to save the WAV file
    """
    try:
        from gtts import gTTS
        import subprocess

        # Generate TTS using gTTS (default voice is natural female)
        # Note: gTTS already has natural prosody, no need for special characters
        tts = gTTS(text=text, lang='en', slow=False)

        # Save to temporary MP3 file (gTTS outputs MP3)
        temp_mp3 = output_path + ".mp3"
        tts.save(temp_mp3)

        # Verify file was created
        if not os.path.exists(temp_mp3):
            raise Exception("TTS engine did not create output file")

        # Convert MP3 to WAV using ffmpeg (mono, 22050 Hz, 16-bit PCM)
        result = subprocess.run(
            ['ffmpeg', '-i', temp_mp3, '-ar', '22050', '-ac', '1', '-acodec', 'pcm_s16le', '-y', output_path],
            capture_output=True,
            timeout=10
        )

        if result.returncode != 0:
            stderr = result.stderr.decode('utf-8', errors='replace')
            raise Exception(f"ffmpeg conversion failed: {stderr}")

        # Clean up temporary MP3 file
        if os.path.exists(temp_mp3):
            os.remove(temp_mp3)

        print(f"✓ Generated TTS audio with melodic voice", file=sys.stderr)

    except ImportError as e:
        print(f"Error: Required packages not installed. Run: pip install gtts pydub", file=sys.stderr)
        print(f"Details: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error generating TTS: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


def main():
    if len(sys.argv) != 4:
        print("Usage: generate_tts.py <text> <voice_type_or_model_path> <output_wav>", file=sys.stderr)
        sys.exit(1)

    text = sys.argv[1]
    voice_type = sys.argv[2]  # Can be "male"/"female" or model path
    output_path = sys.argv[3]

    # Validate inputs
    if not text:
        print("Error: Text cannot be empty", file=sys.stderr)
        sys.exit(1)

    generate_tts(text, voice_type, output_path)


if __name__ == "__main__":
    main()
