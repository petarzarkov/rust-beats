# rust-beats 🥁

A parallel drum beat generator written in Rust that creates randomized musical beats using real audio samples.

## 🎵 Live Demo

**[Listen to daily generated beats →](https://petarzarkov.github.io/rust-beats/)**

Every day at midnight UTC, a new unique song is automatically generated and deployed to GitHub Pages. The website features an audio player where you can listen to the latest beat and browse through the previous 7 generated songs.

## Overview

`rust-beats` is a command-line application that generates unique drum beats by procedurally combining kick, snare, and hi-hat samples. It leverages Rust's concurrency features to generate multiple beats simultaneously, utilizing all available CPU cores for efficient parallel processing.

## Features

- **Advanced Song Structure**: Intro, Verse, Chorus, Bridge, Outro with dynamic arrangement
- **Complete Synthesis Engine**: All sounds generated procedurally - drums, bass, melody, no samples needed
- **Music Theory Integration**: Real chord progressions, scales, and key-aware melodies
- **Dynamic Intensity**: Sections vary in volume and complexity for natural song flow
- **Configurable via TOML**: Easy customization of tempo, structure, metadata
- **Song Naming System**: Generates funky, jazzy, groovy names automatically
- **WAV Metadata**: Embeds artist, title, genre, copyright in file properties
- **Automated Daily Generation**: GitHub Actions workflow generates new songs daily
- **GitHub Pages Deployment**: Live website with audio player and history of the last 7 songs
- **Free for Content Creators**: All music is CC0/CC BY license

## How It Works

1. **Beat Generation** (`beat_maker.rs`):

   - Creates a sequence of 64 drum events (kick, snare, hi-hat, or rest)
   - Uses position-aware probabilities to maintain musical coherence
   - Emphasizes strong beats (positions 0, 4, 8, 12) with kicks and snares
   - Fills in with hi-hats and occasional variations

2. **Audio Rendering** (`audio_renderer.rs`):

   - Loads sample WAV files from the `samples/` directory
   - Maps each drum event to its corresponding audio sample
   - Renders the beat at 44.1kHz sample rate with 250ms per step
   - Writes the final composition as a WAV file

3. **Concurrent Execution** (`main.rs`):
   - Spawns one thread per logical CPU core
   - Each thread generates a unique random beat
   - All beats are saved to the `output/` directory as `beat_1.wav`, `beat_2.wav`, etc.

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

📝 Song Name: Cosmic Midnight Groove
🎸 Genres: ["Funk", "Groovy"]
🎹 Key: Root MIDI 43, Scale: Dorian
⏱️  Tempo: 112.3 BPM
🥁 Groove: Funk

🎼 Generating 52 bars of music...
   Structure: 7 sections
   Intro: 4 bars
   Verse: 8 bars
   Chorus: 8 bars
   Verse: 8 bars
   Chorus: 8 bars
   Bridge: 8 bars
   Outro: 4 bars

  ├─ Drums (with dynamics)
  ├─ Bass (with sections)
  ├─ Melody (with variation)
  └─ Mixing

✅ Successfully created: output/final_song.wav
   Duration: 143.2s (2:23)
   Samples: 6315600

🎉 Song generation complete!
   Name: Cosmic Midnight Groove
   Style: Funk, Groovy @ 112 BPM
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
│   │   ├── song-*.wav
│   │   └── song-*.json    # Metadata per song
│   └── songs.json         # Song list
├── config.toml            # User configuration
├── output/                # Local generated songs
│   ├── final_song.wav
│   └── song_metadata.json
└── Cargo.toml
```

## Technical Details

- **Language**: Rust (Edition 2024)
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit PCM
- **Step Duration**: 250ms per step
- **Beat Length**: 64 steps (16 seconds)
- **Concurrency**: Thread-per-core using `std::thread::scope`

### Dependencies

- `rand` (0.8) - Random number generation for probabilistic beat creation
- `hound` (3.5) - WAV file reading and writing

## GitHub Actions & Deployment

This project uses GitHub Actions to automatically generate and deploy new beats:

### Workflow Triggers

- **Daily**: Runs at 00:00 UTC every day
- **On Push**: Runs on every push to the `main` branch
- **Manual**: Can be triggered manually from the Actions tab

### Deployment Process

1. Builds the Rust project in release mode
2. Generates a new beat composition
3. Names the output file with the current date (e.g., `song-2025-11-16.wav`)
4. Keeps only the 7 most recent songs
5. Updates the song list metadata
6. Deploys to GitHub Pages

### Viewing the Results

Visit **[petarzarkov.github.io/rust-beats](https://petarzarkov.github.io/rust-beats/)** to:

- Listen to the latest generated beat
- Browse and download the previous 7 songs
- See generation dates and file information

## License

See [LICENSE](LICENSE) file for details.

### Local dev

- if you have nodejs
  - `npx serve docs -p 8000` to serve locally
- or python
  - `python3 -m http.server 8000`
