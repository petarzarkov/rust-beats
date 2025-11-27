# rust-beats 🤘

A procedural heavy metal music generator written in Rust that creates authentic metal songs with complete synthesis - no samples needed! Generates physically playable riffs, humanized drums, and authentic metal tones through advanced DSP.

**[GitHub Repository](https://github.com/petarzarkov/rust-beats)** | **[Live Website](https://petarzarkov.github.io/rust-beats/)** | **[YouTube Channel](https://www.youtube.com/@RustBeats)**

## 🎵 Live Demo

**[Listen to daily generated beats →](https://petarzarkov.github.io/rust-beats/)** | **[Watch on YouTube →](https://www.youtube.com/@RustBeats)**

Every day at midnight UTC, a new unique metal song is automatically generated and deployed to GitHub Pages and uploaded to YouTube. The website features an audio player where you can listen to the latest track and browse through the previous 7 generated songs.

## Overview

`rust-beats` is a command-line application that generates authentic heavy metal music through procedural composition and advanced digital signal processing. The system combines music theory, algorithmic rhythm generation, physical modeling synthesis, and professional-grade DSP to create complete metal songs with:

- **Physically playable guitar riffs** using fretboard pathfinding algorithms
- **Authentic metal tones** via Karplus-Strong string synthesis, tube distortion, and cabinet simulation
- **Humanized drum programming** with velocity randomization and micro-timing
- **Subgenre-specific generation** for Heavy Metal, Thrash, Death Metal, Doom, and Progressive Metal

Every sound is synthesized from scratch - drums, distorted guitars, bass - with no samples required.

## Features

### 🎸 Metal Composition Engine

- **5 Metal Subgenres**: Heavy Metal, Thrash Metal, Death Metal, Doom Metal, Progressive Metal
- **Advanced Music Theory**: Phrygian, Locrian, Double Harmonic Major scales with dissonance-aware interval weighting
- **Euclidean Rhythms**: Bjorklund's algorithm for generating syncopated, groovy metal patterns
- **Polymetric Sequences**: Complex time signatures and displaced riffs (Djent-style)
- **Breakdown Patterns**: Metalcore/deathcore breakdown generation with halftime feel
- **Pedal Point Logic**: Markov chain-based riff generation with root note anchoring
- **Fretboard Pathfinding**: Ensures all generated riffs are physically playable on guitar
- **Complete Song Structure**: Intro → Verse → Chorus → Breakdown → Solo → Outro

### 🔊 Advanced DSP Chain

- **Karplus-Strong Synthesis**: Physical modeling of plucked guitar/bass strings
  - Palm muting simulation for authentic metal "chugs"
  - Open string sustain for melodic passages
  - Harmonic techniques for lead sections
- **Noise Gate**: Aggressive gating to eliminate hum between staccato riffs
- **Tube Distortion**: Hyperbolic tangent (tanh) waveshaping with asymmetric clipping
  - 4x and 8x oversampling to prevent aliasing
  - Metal and high-gain presets
- **Cabinet Simulation**: Frequency-domain filtering for authentic speaker coloration
  - Metal 4x12, Combo 2x12, and Vintage presets
  - High-pass filtering (removes subsonic mud)
  - Low-pass filtering (speaker rolloff)
  - Resonance modeling (cabinet characteristic peaks)

### 🥁 Humanized Drum Programming

- **Velocity Randomization**: ±5 variance for round-robin simulation
- **Micro-timing**: ±10 ticks variance with configurable bias (rush/drag)
- **Blast Beat Patterns**: Traditional, Hammer, Euro, and Gravity blast styles
- **Subgenre-Specific Presets**: Blast Beat, Breakdown, and Thrash humanization styles
- **Accent Logic**: First beat emphasis with velocity boosts

### 🎼 Guitar Tuning System

Supports all common metal tunings:
- E Standard (Thrash, Heavy Metal)
- Drop D (Metalcore, Nu-Metal)
- D Standard (Death Metal)
- C Standard (Doom, Stoner)
- Drop C (Metalcore)
- B Standard 7-string (Deathcore)
- Drop A 7-string (Deathcore, Djent)
- F# Standard 8-string (Djent, Progressive)
- Drop E 8-string (Extreme Djent/Thall)

### ⚙️ Configuration & Automation

- **Configurable via TOML**: Easy customization of tempo, structure, metadata, author
- **Automated Daily Generation**: GitHub Actions workflow generates new songs daily
- **GitHub Pages Deployment**: Live website with audio player and history of the last 7 songs
- **YouTube Upload**: Automatically uploads videos with dynamic animations to YouTube
- **Python Scripts**: Video creation and YouTube upload handled by Python scripts with `.env` support
- **Free for Content Creators**: All music is CC0/CC BY license

## How It Works

### 1. Music Theory & Composition (`composition/`)

- **Scale Selection**: Chooses appropriate scale based on subgenre (Phrygian for Thrash, Locrian for Death Metal, etc.)
- **Tuning Selection**: Automatically selects guitar tuning based on subgenre
- **Riff Generation**: Uses Markov chains with pedal point logic to generate memorable riffs
- **Fretboard Validation**: Pathfinding algorithm ensures riffs are physically playable
- **Rhythm Generation**: Euclidean rhythms and polymetric sequences for complex patterns
- **Song Structure**: Creates complete songs with Intro, Verse, Chorus, Breakdown, Solo, Outro

### 2. Sound Synthesis (`synthesis/`)

- **Karplus-Strong String Synthesis**: Generates realistic guitar/bass tones from scratch
  - White noise excitation (the "pluck")
  - Feedback loop with averaging and decay
  - Low-pass filtering for damping
  - Palm mute simulation for metal chugs
- **Drum Synthesis**: Generates kick, snare, hi-hat using oscillators and envelopes
- **DSP Chain**: Processes guitar through Noise Gate → Tube Distortion → Cabinet Simulator

### 3. Audio Rendering (`synthesis/metal_audio_renderer.rs`)

- **Section Rendering**: Converts MIDI riffs to audio samples
- **Drum Generation**: Creates humanized drum patterns per section
- **Mixing**: Combines guitar (60%) and drums (40%) with appropriate levels
- **Normalization**: Prevents clipping with 0.95 max amplitude headroom

### 4. Export (`audio/encoder.rs`)

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

[metadata]
artist = "Your Name"
copyright = "Free to use - CC0 Public Domain"

[generation]
output_dir = "output"
write_metadata_json = true
encode_mp3 = true
```

If no config file is found, defaults are used automatically.

### Running

```bash
cargo run --release
```

The application will:

1. Load configuration (or use defaults)
2. Randomly select a metal subgenre (Heavy Metal, Thrash, Death Metal, Doom, Progressive)
3. Generate a complete song with riffs, drums, and structure
4. Render audio through the full DSP chain
5. Save WAV and MP3 files to the output directory
6. Write metadata JSON for the GitHub workflow

### Output

```
🤘 RUST BEATS - METAL MUSIC GENERATOR 🤘
=========================================

Artist: Petar Zarkov
Sample Rate: 44100 Hz

🎸 Generating DeathMetal song...

📝 Song Details:
   Name: Frost Marsh
   Genre: Swamp Metal
   Subgenre: DeathMetal
   Key: D Phrygian
   Tempo: 180 BPM
   Tuning: DStandard
   Sections: 8

🎼 Song Structure:
   1. Intro - 16 notes
   2. Verse - 32 notes
   3. Chorus - 24 notes
   4. Verse - 32 notes
   5. Chorus - 24 notes
   6. Breakdown - 20 notes
   7. Solo - 40 notes
   8. Outro - 12 notes

🔊 Rendering audio...
   Duration: 32.0s
   Samples: 1,411,200

💾 Saving audio...
✅ Successfully created: output/2025-11-26_petar_zarkov_frost_marsh.wav
✅ Successfully created: output/2025-11-26_petar_zarkov_frost_marsh.mp3
✅ Successfully created: output/2025-11-26_petar_zarkov_frost_marsh.json

🎉 Metal song generation complete!
   Name: Frost Marsh
   Artist: Petar Zarkov
   Style: DeathMetal
   Tempo: 180 BPM
   Duration: 32.0s
```

## Project Structure

```
rust-beats/
├── .github/
│   └── workflows/
│       └── generate-and-deploy.yml  # CI/CD workflow
├── src/
│   ├── main.rs                      # Main orchestrator
│   ├── config.rs                    # Configuration system
│   ├── composition/                 # Music theory & composition
│   │   ├── music_theory.rs          # Keys, scales, intervals, dissonance
│   │   ├── tuning.rs                # Guitar tunings (E Standard through Drop E 8-string)
│   │   ├── rhythm.rs                # Euclidean rhythms, polymeters, breakdowns
│   │   ├── riff_generator.rs        # Markov chains, pedal point logic
│   │   ├── fretboard.rs             # Pathfinding for playable riffs
│   │   ├── drum_humanizer.rs        # Velocity randomization, micro-timing, blast beats
│   │   ├── bass_generator.rs        # Bass line generation
│   │   ├── metal_song_generator.rs  # Complete song generation
│   │   └── song_names.rs            # Song name generation
│   ├── synthesis/                   # Sound synthesis & DSP
│   │   ├── karplus_strong.rs        # Physical string modeling
│   │   ├── metal_dsp.rs             # Tube distortion, noise gate
│   │   ├── cabinet.rs               # Cabinet simulation (IR convolution)
│   │   ├── metal_audio_renderer.rs  # Complete audio rendering pipeline
│   │   ├── synthesizer.rs           # Core synth utilities
│   │   ├── drums.rs                 # Drum synthesis
│   │   └── mixing.rs                # Audio mixing utilities
│   ├── audio/                       # Audio encoding
│   │   ├── encoder.rs               # MP3 encoding
│   │   └── voice.rs                 # Voice utilities
│   └── utils.rs                     # Utility functions
├── docs/                            # GitHub Pages website
│   ├── index.html                   # Web player interface
│   ├── songs/                       # Generated songs (last 7)
│   │   ├── song-*.wav               # High-quality WAV files
│   │   ├── song-*.mp3               # Compressed MP3 files
│   │   └── song-*.json              # Metadata per song
│   └── songs.json                   # Song list
├── config.toml                      # User configuration
├── output/                          # Local generated songs
│   ├── YYYY-MM-DD_artist_song.wav   # WAV output
│   ├── YYYY-MM-DD_artist_song.mp3   # MP3 output
│   └── YYYY-MM-DD_artist_song.json  # Metadata JSON
├── METAL_MIGRATION_PROGRESS.md      # Migration progress log
├── metal_generation_research.md     # Research documentation
└── Cargo.toml
```

## Technical Details

### Music Theory Implementation

- **Scales**: Aeolian (Natural Minor), Phrygian, Phrygian Dominant, Locrian, Double Harmonic Major
- **Interval Analysis**: Dissonance detection and weighting (minor second, tritone prioritization)
- **Tunings**: 9 common metal tunings from E Standard to Drop E 8-string
- **Register Logic**: Bass offset and unison mode for extreme low tunings

### Rhythm Generation

- **Euclidean Rhythms**: Bjorklund's algorithm for even pulse distribution
- **Polymeters**: LCM-based resolution calculation for complex time signatures
- **Breakdown Patterns**: Halftime feel, syncopated chugs, call-and-response structures

### Procedural Riff Generation

- **Markov Chains**: Weighted transition matrices for pitch sequences
- **Pedal Point**: High-probability return to root note (65% default)
- **IRVD Framework**: Introduction, Repetition, Variation, Destruction
- **Fretboard Pathfinding**: Greedy algorithm for optimal fingering
- **Playability Scoring**: 0.0-1.0 score based on movement difficulty

### DSP Chain

- **Karplus-Strong**: Physical string modeling with configurable decay and filtering
- **Noise Gate**: Envelope follower with attack/release (metal preset: aggressive gating)
- **Tube Distortion**: Tanh waveshaping with asymmetric clipping, 4x/8x oversampling
- **Cabinet Simulation**: Frequency-domain filtering with synthetic IR support

### Drum Humanization

- **Velocity Variance**: ±5 units for round-robin simulation
- **Timing Variance**: ±10 ticks with configurable bias
- **Blast Beat Logic**: Reduced velocity for high-speed patterns (max 110)
- **Accent System**: First beat emphasis (+15 velocity)

- **Language**: Rust (Edition 2021)
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit PCM (WAV)
- **MP3 Encoding**: 192 kbps
- **Synthesis**: Pure Rust DSP (no external audio libraries required)
- **Tests**: 109/109 passing ✅

### Dependencies

- `rand` (0.8) - Random number generation for procedural music
- `mp3lame-encoder` (0.2) - MP3 encoding for file size optimization
- `serde` (1.0) - Configuration and metadata serialization
- `toml` (0.8) - Configuration file parsing

## Research & Documentation

This project is based on extensive research into metal music theory, algorithmic composition, and digital signal processing. Key documents:

- **`metal_generation_research.md`**: Comprehensive research report covering:
  - Harmonic foundations (scales, modes, intervals)
  - Rhythmic algorithms (Euclidean rhythms, polymeters)
  - Procedural riff construction (Markov chains, pedal point)
  - DSP techniques (distortion, cabinet simulation, Karplus-Strong)
  - Subgenre-specific techniques

- **`METAL_MIGRATION_PROGRESS.md`**: Complete migration log documenting:
  - Phase-by-phase implementation progress
  - All 109 passing tests
  - Component integration details
  - Complete end-to-end pipeline

## GitHub Actions & Deployment

This project uses GitHub Actions to automatically generate and deploy new metal songs:

### Workflow Triggers

- **Daily**: Runs at 00:00 UTC every day
- **On Push**: Runs on every push to the `main` branch
- **Manual**: Can be triggered manually from the Actions tab

### Deployment Process

1. Builds the Rust project in release mode
2. Generates a new metal song with random subgenre selection
3. Creates both WAV and MP3 files with the current date (e.g., `2025-11-26_petar_zarkov_frost_marsh.mp3`)
4. Creates video with dynamic animations using Python/ffmpeg
5. Uploads video to YouTube (if video creation succeeds)
6. Keeps only the 7 most recent songs (automatically removes older files)
7. Updates the song list metadata including file sizes and song information
8. Commits the new files to the repository
9. Deploys to GitHub Pages

### Viewing the Results

- **Website**: Visit **[petarzarkov.github.io/rust-beats](https://petarzarkov.github.io/rust-beats/)** to:
  - Listen to the latest generated metal song
  - Browse and download the previous 7 songs (MP3 format)
  - See song names, genres, generation dates, and file sizes
  - Read about the song's key, tempo, tuning, and subgenre

- **YouTube**: Watch videos on **[youtube.com/@RustBeats](https://www.youtube.com/@RustBeats)** with:
  - Dynamic video animations
  - Full song playback
  - Download links in descriptions

## License

See [LICENSE](LICENSE) file for details.

## Local Development

### Testing the Website Locally

```bash
# Serve the docs directory
python3 -m http.server 8000 --directory docs
# Then visit http://localhost:8000
```

### Testing YouTube Upload

1. **Set up Python environment**:

   ```bash
   python3 -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   pip install -r scripts/requirements.txt
   ```

2. **Create `.env` file** (copy from `.env.example`):

   ```bash
   cp .env.example .env
   # Edit .env and add your YouTube API credentials
   ```

3. **Generate a video**:

   ```bash
   python3 scripts/create_video.py
   ```

4. **Upload to YouTube**:
   ```bash
   python3 scripts/upload_youtube.py
   ```

The script will automatically:

- Load credentials from `.env` file
- Read metadata from `output/*.json` files
- Upload the video with proper title and description

### Python Scripts

- **`scripts/create_video.py`**: Creates MP4 video with dynamic animations from MP3 + cover art
- **`scripts/upload_youtube.py`**: Uploads video to YouTube with metadata from JSON files
- **`.env`**: Local configuration file (gitignored) for YouTube API credentials

## Testing

Run the full test suite:

```bash
cargo test --release
```

All 109 tests should pass, covering:
- Music theory (scales, intervals, tunings)
- Rhythm generation (Euclidean, polymeters, breakdowns)
- DSP components (distortion, noise gate, cabinet)
- Karplus-Strong synthesis
- Riff generation (Markov chains, pedal point)
- Fretboard pathfinding
- Drum humanization
- Complete song generation and audio rendering

## Contributing

Contributions are welcome! Areas for improvement:

- Additional metal subgenres (Black Metal, Djent, Metalcore)
- More sophisticated riff generation algorithms
- Additional DSP effects (reverb, delay, modulation)
- Performance optimizations for real-time generation
- Export to additional audio formats (FLAC, OGG)
