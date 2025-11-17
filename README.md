# rust-beats 🥁

A procedural music generator written in Rust that creates unique lofi/chill songs with complete synthesis - no samples needed!

## 🎵 Live Demo

**[Listen to daily generated beats →](https://petarzarkov.github.io/rust-beats/)**

Every day at midnight UTC, a new unique song is automatically generated and deployed to GitHub Pages. The website features an audio player where you can listen to the latest beat and browse through the previous 7 generated songs.

## Overview

`rust-beats` is a command-line application that generates unique, complete songs through procedural synthesis and music theory. Every sound is synthesized from scratch - drums, bass, melody, and pads - with no samples required. Each song features real chord progressions, scales, dynamic arrangements, and varied instrumentation.

## Features

### 🎼 Music Generation

- **Advanced Song Structure**: Intro, Verse, Chorus, Bridge, Outro with dynamic arrangement
- **Complete Synthesis Engine**: All sounds generated procedurally - no samples needed
- **Music Theory Integration**: Real chord progressions, scales (Major, Minor, Dorian, Lydian, etc.)
- **Varied Instrumentation**: Rhodes, acoustic guitar, ukulele, electric guitar, multiple bass types
- **Diverse Percussion**: Tambourine, cowbell, bongo, woodblock, 6 different drum kit styles
- **Dynamic Intensity**: Volume automation across sections for natural song flow
- **Mixing Variations**: 4 mixing styles (Clean, Warm, Punchy, Spacious) for sonic diversity

### 🎵 Output & Quality

- **Dual Format Export**: High-quality WAV (16-bit, 44.1kHz) + compressed MP3 (192kbps)
- **File Size Optimized**: MP3 reduces size by ~85% (10-20MB WAV → 1-3MB MP3)
- **Rich Metadata**: Embeds artist, title, genre, copyright, and date in files
- **Song Naming System**: Generates creative names automatically

### ⚙️ Configuration & Automation

- **Configurable via TOML**: Easy customization of tempo, structure, metadata, author
- **Automated Daily Generation**: GitHub Actions workflow generates new songs daily
- **GitHub Pages Deployment**: Live website with audio player and history of the last 7 songs
- **Free for Content Creators**: All music is CC0/CC BY license

## How It Works

1. **Music Theory** (`composition/`):

   - Generates random key and scale (Major, Minor, Dorian, Lydian, Pentatonic, etc.)
   - Creates chord progressions using music theory rules
   - Defines song arrangement (Intro, Verse, Chorus, Bridge, Outro)
   - Generates drum patterns for different groove styles (Funk, Jazz, Lofi, Hip-Hop, Rock)

2. **Sound Synthesis** (`synthesis/`):

   - **Drums**: Synthesizes kick, snare, hi-hat, clap using oscillators and envelopes
   - **Bass**: Multiple bass types (standard, synth, upright, finger, slap) with varied timbres
   - **Melody**: Rhodes piano, guitar, ukulele with humanized timing and velocity
   - **Pads**: Atmospheric layers with slow attack/release for ambient texture
   - **Effects**: Lofi processing (vinyl crackle, tape saturation, bit crushing)

3. **Mixing & Mastering** (`audio/`):

   - Multi-track mixing with volume, panning, and EQ per track
   - 4 mixing styles with different sonic characteristics
   - Volume automation based on song sections
   - Lofi-style mastering with gentle compression and warmth
   - Stereo-to-mono conversion for final output

4. **Export** (`audio/encoder.rs`):
   - Renders high-quality WAV with embedded metadata (title, artist, genre, date)
   - Encodes to MP3 at 192kbps for smaller file size
   - Both formats saved to output directory

## Usage

### Prerequisites

- Rust toolchain (1.70+)
- No audio samples needed - everything is synthesized!

### Configuration

The project uses a `config.toml` file for customization. Create or edit this file in the project root:

```toml
[audio]
sample_rate = 44100  # Sample rate in Hz (44100 = CD quality)
bit_depth = 16       # Bit depth (16 or 24)

[metadata]
artist = "Your Name"
copyright = "Free to use - CC0 Public Domain"

[composition]
structure = "standard"  # "short" (30-60s) or "standard" (2-3 min full song)
min_tempo = 90.0        # Minimum BPM
max_tempo = 130.0       # Maximum BPM

[generation]
output_dir = "output"
write_metadata_json = true
```

If no config file is found, defaults are used automatically.

### Running

```bash
cargo run --release
```

The application will:

1. Load configuration (or use defaults)
2. Generate a unique song with random parameters
3. Create synthesized drums, bass, and melody
4. Save the final song to the output directory
5. Write metadata JSON for the GitHub workflow

### Output

```
🎵 Rust Beats - Procedural Music Generator
============================================
Artist: Petar Zarkov
Sample Rate: 44100 Hz
Structure: standard

📝 Song Name: Honey Stars
🎸 Genres: ["Jazz"]
🎹 Key: Root MIDI 46, Scale: Major
⏱️  Tempo: 105.1 BPM
🥁 Groove: Lofi
🎸 Lead: Electric, Bass: Standard, Drums: Acoustic
🥁 Percussion: None, Pads: Subtle, Mix: Spacious

🎼 Generating 84 bars of music...
   Structure: 7 sections
   Intro: 8 bars
   Verse: 16 bars
   Chorus: 16 bars
   Verse: 16 bars
   Chorus: 16 bars
   Bridge: 8 bars
   Outro: 4 bars

  ├─ Drums (with dynamics)
  ├─ Bass (with sections)
  ├─ Melody (with variation)
  ├─ Pads (atmospheric)
  ├─ Multi-track mixing (Spacious)
  ├─ Arrangement dynamics (volume automation)
  ├─ Lofi mastering (compression, warmth & limiting)
  └─ Lofi effects (vinyl crackle & tape saturation)

✅ Successfully created: output/final_song.wav
   Duration: 201.0s (3:21)
   Samples: 8,868,900
✅ Successfully created MP3: output/final_song.mp3

🎉 Song generation complete!
   Name: Honey Stars
   Style: Jazz @ 105 BPM
```

## Project Structure

```
rust-beats/
├── .github/
│   └── workflows/
│       └── generate-and-deploy.yml  # CI/CD workflow
├── src/
│   ├── main.rs            # Main orchestrator
│   ├── config.rs          # Configuration system
│   ├── composition/       # Music theory & arrangement
│   │   ├── song_names.rs  # Song name generation
│   │   ├── music_theory.rs# Keys, scales, chords
│   │   ├── beat_maker.rs  # Drum pattern generation
│   │   └── arranger.rs    # Song structure
│   ├── synthesis/         # Sound synthesis
│   │   ├── synthesizer.rs # Core synth engine
│   │   ├── drums.rs       # Drum synthesis
│   │   ├── bass.rs        # Bass synthesis
│   │   └── melody.rs      # Melody synthesis
│   ├── audio/             # Audio rendering
│   │   └── renderer.rs    # WAV with metadata
│   └── audio_renderer.rs  # Legacy renderer
├── docs/                  # GitHub Pages website
│   ├── index.html         # Web player interface
│   ├── songs/             # Generated songs (last 7)
│   │   ├── song-*.wav     # High-quality WAV files
│   │   ├── song-*.mp3     # Compressed MP3 files
│   │   └── song-*.json    # Metadata per song
│   └── songs.json         # Song list
├── config.toml            # User configuration
├── output/                # Local generated songs
│   ├── final_song.wav     # WAV output (10-20MB)
│   ├── final_song.mp3     # MP3 output (1-3MB)
│   └── song_metadata.json
└── Cargo.toml
```

## Technical Details

- **Language**: Rust (Edition 2024)
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit PCM (WAV)
- **MP3 Encoding**: 192 kbps, best quality
- **Song Duration**: 2-3.5 minutes (configurable)
- **Synthesis**: Pure Rust oscillators, filters, envelopes
- **Music Theory**: 7 scale types, 17 chord types, key-aware progressions
- **Instruments**: 10+ synthesized instruments with varied timbres
- **File Sizes**: WAV ~10-20MB, MP3 ~1-3MB (85% reduction)

### Dependencies

- `rand` (0.8) - Random number generation for procedural music
- `hound` (3.5) - WAV file writing with metadata
- `mp3lame-encoder` (0.2) - MP3 encoding for file size optimization
- `serde` (1.0) - Configuration and metadata serialization
- `toml` (0.8) - Configuration file parsing

## GitHub Actions & Deployment

This project uses GitHub Actions to automatically generate and deploy new beats:

### Workflow Triggers

- **Daily**: Runs at 00:00 UTC every day
- **On Push**: Runs on every push to the `main` branch
- **Manual**: Can be triggered manually from the Actions tab

### Deployment Process

1. Builds the Rust project in release mode
2. Generates a new song with random parameters (key, tempo, instruments, etc.)
3. Creates both WAV and MP3 files with the current date (e.g., `song-2025-11-16.wav/.mp3`)
4. Keeps only the 7 most recent songs (automatically removes older files)
5. Updates the song list metadata including file sizes and song information
6. Commits the new files to the repository
7. Deploys to GitHub Pages

### Viewing the Results

Visit **[petarzarkov.github.io/rust-beats](https://petarzarkov.github.io/rust-beats/)** to:

- Listen to the latest generated song
- Browse and download the previous 7 songs (WAV or MP3)
- See song names, genres, generation dates, and file sizes
- Read about the song's key, tempo, and instrumentation

## License

See [LICENSE](LICENSE) file for details.

### Local dev

- if you have nodejs
  - `npx serve docs -p 8000` to serve locally
- or python
  - `python3 -m http.server 8000`
