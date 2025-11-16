# rust-beats 🥁

A parallel drum beat generator written in Rust that creates randomized musical beats using real audio samples.

## 🎵 Live Demo

**[Listen to daily generated beats →](https://petarzarkov.github.io/rust-beats/)**

Every day at midnight UTC, a new unique song is automatically generated and deployed to GitHub Pages. The website features an audio player where you can listen to the latest beat and browse through the previous 7 generated songs.

## Overview

`rust-beats` is a command-line application that generates unique drum beats by procedurally combining kick, snare, and hi-hat samples. It leverages Rust's concurrency features to generate multiple beats simultaneously, utilizing all available CPU cores for efficient parallel processing.

## Features

- **Parallel Beat Generation**: Automatically detects system CPU cores and generates multiple beats concurrently
- **Probabilistic Rhythm Engine**: Uses weighted random selection to create musically coherent 16-step drum patterns
- **Real Audio Samples**: Works with actual WAV audio files for authentic drum sounds
- **Multi-threaded WAV Rendering**: Each thread independently generates and writes a complete beat to disk
- **Configurable Beat Length**: Generates 64-step beats (4 measures at 16 steps per measure)
- **Automated Daily Generation**: GitHub Actions workflow generates new beats daily and on every commit
- **GitHub Pages Deployment**: Live website with audio player and history of the last 7 songs

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
- Audio sample files in `samples/` directory:
  - `kick.wav`
  - `snare.wav`
  - `hihat.wav`

### Running

```bash
cargo run --release
```

The application will:

1. Detect the number of CPU cores
2. Generate that many beats concurrently
3. Save all beats to the `output/` folder
4. Print progress for each thread

### Output

```
System has 8 logical cores.
Generating 8 beats concurrently...
Thread 0: Generating beat...
Thread 1: Generating beat...
...
Thread 0: Successfully wrote output/beat_1.wav
Thread 1: Successfully wrote output/beat_2.wav
...
All beats generated! Check the 'output' folder.
```

## Project Structure

```
rust-beats/
├── .github/
│   └── workflows/
│       └── generate-and-deploy.yml  # CI/CD workflow for daily generation
├── src/
│   ├── main.rs            # Entry point and concurrent execution
│   ├── beat_maker.rs      # Probabilistic beat generation logic
│   └── audio_renderer.rs  # WAV file processing and rendering
├── samples/               # Input drum samples (WAV format)
│   ├── kick.wav
│   ├── snare.wav
│   └── hihat.wav
├── docs/                  # GitHub Pages website
│   ├── index.html         # Web player interface
│   ├── songs/             # Generated songs (last 7 songs)
│   │   └── song-*.wav
│   └── songs.json         # Song metadata
├── output/                # Generated beats (created automatically)
│   └── beat_*.wav
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
