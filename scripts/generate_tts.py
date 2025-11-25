#!/usr/bin/env python3
"""
Generate TTS audio using pyttsx3 (cross-platform offline TTS).
This script is called by Rust to generate voice narration.

Usage:
    python3 generate_tts.py <text> <voice_type> <output_wav>

Example:
    python3 generate_tts.py "Hello world" "male" "/tmp/output.wav"
"""

import sys
import wave
import os


def generate_tts(text: str, voice_type: str, output_path: str) -> None:
    """
    Generate TTS audio and save as WAV file using pyttsx3.

    Args:
        text: Text to synthesize
        voice_type: "male" or "female" (or path to model for compatibility)
        output_path: Where to save the WAV file
    """
    try:
        import pyttsx3

        # Initialize TTS engine
        engine = pyttsx3.init()

        # Determine voice type from parameter
        # If parameter looks like a path (contains / or models/), extract gender
        if "/" in voice_type or voice_type.startswith("models"):
            # Extract gender from model path (e.g., "models/en_US-joe-medium.onnx" -> male)
            is_female = "amy" in voice_type.lower() or "female" in voice_type.lower()
        else:
            is_female = voice_type.lower() == "female"

        # Get available voices
        voices = engine.getProperty('voices')

        # Try to find appropriate voice
        selected_voice = None
        for voice in voices:
            voice_name = voice.name.lower()
            voice_id = voice.id.lower()

            # macOS voice selection
            if is_female:
                # Prefer female voices: Samantha, Victoria, Karen, etc.
                if any(name in voice_name or name in voice_id for name in ['samantha', 'victoria', 'karen', 'fiona', 'female']):
                    selected_voice = voice.id
                    break
            else:
                # Prefer male voices: Alex, Daniel, Tom, etc.
                if any(name in voice_name or name in voice_id for name in ['alex', 'daniel', 'tom', 'male']):
                    selected_voice = voice.id
                    break

        # Fallback: use first female/male voice found
        if not selected_voice:
            for voice in voices:
                voice_lower = (voice.name + voice.id).lower()
                if is_female and 'female' in voice_lower:
                    selected_voice = voice.id
                    break
                elif not is_female and 'male' in voice_lower:
                    selected_voice = voice.id
                    break

        # Final fallback: use first available voice
        if not selected_voice and voices:
            selected_voice = voices[0 if not is_female else (1 if len(voices) > 1 else 0)].id

        if selected_voice:
            engine.setProperty('voice', selected_voice)

        # Set speech rate (words per minute) - slightly slower for clarity
        engine.setProperty('rate', 150)

        # Save to temporary AIFF file (macOS pyttsx3 outputs AIFF)
        temp_output = output_path + ".aiff"
        engine.save_to_file(text, temp_output)
        engine.runAndWait()

        # Verify file was created
        if not os.path.exists(temp_output):
            raise Exception("TTS engine did not create output file")

        # Convert AIFF to WAV using ffmpeg (if available) or try direct conversion
        try:
            import subprocess
            # Try using ffmpeg for conversion
            result = subprocess.run(
                ['ffmpeg', '-i', temp_output, '-acodec', 'pcm_s16le', '-ar', '22050', '-ac', '1', '-y', output_path],
                capture_output=True,
                timeout=10
            )
            if result.returncode != 0:
                # Fallback: read AIFF and write WAV manually
                import aifc
                with aifc.open(temp_output, 'rb') as aiff_file:
                    params = aiff_file.getparams()
                    frames = aiff_file.readframes(params.nframes)

                    import wave
                    with wave.open(output_path, 'wb') as wav_file:
                        wav_file.setnchannels(params.nchannels)
                        wav_file.setsampwidth(params.sampwidth)
                        wav_file.setframerate(params.framerate)
                        wav_file.writeframes(frames)
        except FileNotFoundError:
            # ffmpeg not found, use aifc fallback
            import aifc
            with aifc.open(temp_output, 'rb') as aiff_file:
                params = aiff_file.getparams()
                frames = aiff_file.readframes(params.nframes)

                import wave
                with wave.open(output_path, 'wb') as wav_file:
                    wav_file.setnchannels(params.nchannels)
                    wav_file.setsampwidth(params.sampwidth)
                    wav_file.setframerate(params.framerate)
                    wav_file.writeframes(frames)
        finally:
            # Clean up temporary AIFF file
            if os.path.exists(temp_output):
                os.remove(temp_output)

        print(f"✓ Generated TTS audio with voice: {selected_voice if selected_voice else 'default'}", file=sys.stderr)

    except ImportError as e:
        print(f"Error: pyttsx3 not installed. Run: pip install pyttsx3", file=sys.stderr)
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
