#!/usr/bin/env python3
"""
Create video with dynamic effects from MP3 audio and cover art.
"""

import os
import sys
import subprocess
import hashlib
import glob

COVER_ART = "docs/logo/rust_beats.png"
OUTPUT_VIDEO = "output/youtube_video.mp4"
OUTPUT_DIR = "output"


def find_latest_mp3():
    """Find the most recent MP3 file in output directory."""
    mp3_files = glob.glob(f"{OUTPUT_DIR}/*.mp3")
    if not mp3_files:
        print("Error: No MP3 file found in output/")
        sys.exit(1)
    # Sort by modification time, most recent first
    mp3_files.sort(key=os.path.getmtime, reverse=True)
    return mp3_files[0]


def get_audio_duration(mp3_file):
    """Get audio duration using ffprobe."""
    try:
        result = subprocess.run(
            ["ffprobe", "-i", mp3_file, "-show_entries", "format=duration",
             "-v", "quiet", "-of", "csv=p=0"],
            capture_output=True,
            text=True,
            check=True
        )
        duration = float(result.stdout.strip())
        return duration if duration > 0 else 180.0
    except (subprocess.CalledProcessError, ValueError):
        return 180.0


def generate_random_seed(filename):
    """Generate a deterministic random seed from filename."""
    hash_obj = hashlib.md5(filename.encode())
    hash_hex = hash_obj.hexdigest()[:8]
    try:
        return int(hash_hex, 16)
    except ValueError:
        return 12345


def calculate_effects(rand_seed):
    """Calculate video effects based on random seed."""
    # Vibration parameters
    vibration_intensity = 0.5 + (rand_seed % 10) / 10.0  # 0.5-1.5 pixels
    vibration_freq = 2.0 + (rand_seed % 30) / 10.0  # 2-5 Hz
    
    # Shake parameters
    shake_interval = 8.0 + (rand_seed % 100) / 10.0  # 8-18s
    shake_duration = 0.10 + (rand_seed % 15) / 100  # 0.10-0.25s
    
    # Color enhancement
    brightness = 0.01 + (rand_seed % 20) / 1000  # 0.01-0.03
    contrast = 1.03 + (rand_seed % 40) / 1000  # 1.03-1.07
    saturation = 1.08 + (rand_seed % 40) / 1000  # 1.08-1.12
    
    # Total padding (integer)
    total_padding = int(vibration_intensity + 5)
    
    return {
        "vibration_intensity": vibration_intensity,
        "vibration_freq": vibration_freq,
        "shake_interval": shake_interval,
        "shake_duration": shake_duration,
        "brightness": brightness,
        "contrast": contrast,
        "saturation": saturation,
        "total_padding": total_padding,
    }


def create_video(mp3_file, cover_art, output_video, effects):
    """Create video with ffmpeg."""
    filter_complex = (
        f"[0:v]scale=1280:720:force_original_aspect_ratio=decrease[scaled];"
        f"[scaled]pad=1280:720:(ow-iw)/2:(oh-ih)/2[centered];"
        f"[centered]pad=1280+{effects['total_padding']}*2:720+{effects['total_padding']}*2:"
        f"{effects['total_padding']}:{effects['total_padding']}[padded];"
        f"[padded]crop=1280:720:"
        f"'{effects['total_padding']}+{effects['vibration_intensity']}*sin(2*PI*t*{effects['vibration_freq']})+"
        f"if(lt(mod(t,{effects['shake_interval']}),{effects['shake_duration']}),"
        f"{effects['vibration_intensity']}*3*sin(20*PI*t),0)':"
        f"'{effects['total_padding']}+{effects['vibration_intensity']}*cos(2*PI*t*{effects['vibration_freq']})+"
        f"if(lt(mod(t+{effects['shake_interval']}*0.3,{effects['shake_interval']}),{effects['shake_duration']}),"
        f"{effects['vibration_intensity']}*3*cos(20*PI*t),0)'[shaken_vibrated];"
        f"[shaken_vibrated]eq=brightness={effects['brightness']}:"
        f"contrast={effects['contrast']}:saturation={effects['saturation']}[v]"
    )
    
    cmd = [
        "ffmpeg", "-y",
        "-loop", "1",
        "-i", cover_art,
        "-i", mp3_file,
        "-filter_complex", filter_complex,
        "-map", "[v]",
        "-map", "1:a",
        "-c:v", "libx264",
        "-preset", "medium",
        "-crf", "23",
        "-pix_fmt", "yuv420p",
        "-c:a", "aac",
        "-b:a", "192k",
        "-shortest",
        "-r", "30",
        output_video,
    ]
    
    try:
        subprocess.run(cmd, check=True)
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Error creating video: {e}")
        return False


def main():
    # Check inputs
    if not os.path.exists(COVER_ART):
        print(f"Error: Cover art not found at {COVER_ART}")
        sys.exit(1)
    
    mp3_file = find_latest_mp3()
    
    print("Creating video with dynamic effects...")
    print(f"  Cover: {COVER_ART}")
    print(f"  Audio: {mp3_file}")
    print(f"  Output: {OUTPUT_VIDEO}")
    
    # Get audio duration
    audio_duration = get_audio_duration(mp3_file)
    
    # Generate random seed from filename
    rand_seed = generate_random_seed(mp3_file)
    
    # Calculate effects
    effects = calculate_effects(rand_seed)
    
    print(f"  Effects: Vibration (intensity={effects['vibration_intensity']:.1f}px, "
          f"freq={effects['vibration_freq']:.2f}Hz)")
    print(f"             Occasional Shake (avg interval={effects['shake_interval']:.1f}s, "
          f"duration={effects['shake_duration']:.2f}s)")
    print(f"             Color (brightness={effects['brightness']:.3f}, "
          f"contrast={effects['contrast']:.3f}, saturation={effects['saturation']:.3f})")
    
    # Create video
    if create_video(mp3_file, COVER_ART, OUTPUT_VIDEO, effects):
        print(f"✅ Successfully created {OUTPUT_VIDEO}")
        # Show file size
        if os.path.exists(OUTPUT_VIDEO):
            size = os.path.getsize(OUTPUT_VIDEO)
            size_mb = size / (1024 * 1024)
            print(f"   File size: {size_mb:.1f} MB")
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()

