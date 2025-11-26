# Algorithmic Brutality: A Comprehensive Research Report on Computational Architectures for Generative Heavy Metal

## 1. Introduction: The Intersection of Metal Aesthetics and Algorithmic Logic

The endeavor to procedurally generate heavy metal music presents a unique and formidable set of challenges that distinguish it markedly from other forms of algorithmic composition. While genres such as ambient, techno, or even classical music often rely on stochastic processes, functional harmony, or repetitive looping structures that are forgiving of dissonance or lack of directional resolution, heavy metal—and its myriad subgenres like death metal, djent, and metalcore—is governed by strict, albeit complex, theoretical frameworks. To create a Rust-based application that not only generates notes but authentically "sounds" like metal, one must bridge the vast gap between high-level music theory and low-level digital signal processing (DSP).

The core problem identified in many rudimentary song generators is their reliance on unconstrained randomness. In the context of heavy metal, "random" is rarely heavy. Heaviness is derived from intention: the harmonic tension of the tritone, the rhythmic locking of the kick drum and the guitar palm mute, and the structural release of a breakdown. Therefore, the "theory" required for your Rust application is not merely an academic understanding of scales, but a computational understanding of idiomatic constraints. A metal riff generator must implicitly understand that a minor second interval is preferable to a major third in a Phrygian context. A drum generator must understand that a blast beat requires specific velocity randomization to sound human, rather than robotic.

This report provides an exhaustive analysis of the music theory required to generate authentic metal, translated into algorithmic logic suitable for implementation in Rust. It covers the harmonic foundations of metal scales and tuning, the mathematical structures of rhythmic complexity (polymeters and Euclidean rhythms), the procedural generation of riffs using Markov chains and weighted probabilities, and the Digital Signal Processing (DSP) techniques required to synthesize the aggressive timbres characteristic of the genre.

### 1.1 The Sonic Ontology of Metal

To program metal, one must define it mathematically. It is a genre defined by extremes: extreme dynamic range compression (loudness), extreme frequency content (down-tuned fundamentals vs. high-gain harmonic saturation), and extreme rhythmic precision. The generator must therefore operate on two distinct planes:

- **Symbolic Generation (MIDI/Event Level)**: The selection of notes, durations, and velocities. This requires music theory, graph traversal algorithms for fretboard logic, and stochastic probability models like Markov chains.

- **Signal Generation (Audio Level)**: The synthesis of tone. This requires DSP knowledge—waveshaping for distortion, convolution for cabinet simulation, and physical modeling for string dynamics.

By leveraging the Rust programming language's performance characteristics—specifically its zero-cost abstractions, memory safety, and growing ecosystem of audio crates like `fundsp`, `rodio`, and `rust-music-theory`—we can construct a system that generates high-fidelity metal in real-time.

## 2. Harmonic Foundations: Scales, Modes, and Tonal Centers

The harmonic language of heavy metal is distinct from the functional harmony of the Common Practice Period in classical music, yet it relies heavily on classical modes. To program a generator that produces "evil," "dark," or "crushing" tonalities, the system must prioritize specific intervals and scales that maximize tension.

### 2.1 The Priority of the Minor Second and the Tritone

At the heart of metal's harmonic identity are two specific intervals: the minor second (one semitone) and the tritone (six semitones). These are not merely frequent occurrences; they are structural pillars.

**The Minor Second (b2)**: This interval creates an immediate sense of unease and dissonance. In a computational context, transition probabilities should weigh movement by a semitone (e.g., E to F) significantly higher than movement by a whole tone (E to F#) when the goal is an aggressive or "dark" sound. This interval is the driving force behind the "Jaws" theme effect—an oscillation that implies impending doom.

**The Tritone (Diabolus in Musica)**: Historically avoided in sacred music, the tritone is essential in metal. It splits the octave perfectly in half and creates a sense of instability that demands resolution—or, in metal, is sustained to create atmosphere. Bands like Black Sabbath utilized this interval to define the genre.

In a Rust implementation using the `rust-music-theory` crate or custom structs, the generator must effectively "know" the distance between notes in semitones. A `Note` struct should have methods to calculate intervals, and the generation algorithm should have a "dissonance weight" parameter that increases the probability of selecting notes at these specific intervals from the root.

### 2.2 Essential Scales and Modes for Procedural Generation

Simply constraining a generator to a "Minor Scale" is insufficient for modern metal. The specific flavor of subgenres dictates the mode. The application should support a `Mode` enum that alters the available note pool.

#### 2.2.1 The Aeolian Mode (Natural Minor)

The Aeolian mode is the foundational scale for traditional heavy metal (e.g., Iron Maiden, Judas Priest). It is minor but retains a degree of melodic consonance.

- **Formula**: 1, 2, b3, 4, 5, b6, b7.
- **Algorithmic Use**: Best for generating "epic" or melodic riffs. It allows for harmonization in thirds, a staple of dual-guitar harmonies in power metal. When generating dual guitar lines, the second voice should track the first voice at a diatonic third interval within this scale.

#### 2.2.2 The Phrygian Mode

The Phrygian mode is arguably the most important scale for thrash, death, and metalcore. Its defining characteristic is the flattened second degree (b2), which creates the minor second interval with the root.

- **Formula**: 1, b2, b3, 4, 5, b6, b7.
- **Algorithmic Use**: When generating a riff in Phrygian, the algorithm should frequently return to the root note (pedal point) and alternate it with the b2. This creates the classic "chug" sound found in bands like Slayer or Metallica.

#### 2.2.3 Phrygian Dominant

A variation of the Phrygian mode with a major third (1, b2, 3, 4, 5, b6, b7). This scale is ubiquitous in technical death metal and neoclassical metal (e.g., Yngwie Malmsteen, Necrophagist).

- **Harmonic Implication**: The gap between the b2 and the natural 3 is an augmented second (3 semitones), which gives the scale an "exotic" or Middle Eastern timbre.
- **Rust Implementation**: To generate Phrygian Dominant leads, the algorithm must handle the wider interval jumps smoothly, ensuring they don't sound like errors. The weighting for the jump from b2 to 3 should be high to accentuate the exotic flavor.

#### 2.2.4 Locrian Mode

The most unstable mode, containing both a b2 and a b5 (tritone).

- **Formula**: 1, b2, b3, 4, b5, b6, b7.
- **Algorithmic Use**: Rarely used for entire songs due to the lack of a perfect fifth stability, but excellent for generating dissonant bridges or breakdowns.

#### 2.2.5 Double Harmonic Major

Also known as the Byzantine scale, this scale features two augmented seconds.

- **Formula**: 1, b2, 3, 4, 5, b6, 7.
- **Algorithmic Use**: Used in specific subgenres like "Djent" or progressive metal to create highly unstable, exotic melodic lines.

### 2.3 Tuning and Pitch Space Logic

Metal guitarists rarely play in standard E tuning (E2). To sound "heavy," the generator must operate in the correct pitch register. The frequency range of the guitar riff interacts physically with the bass guitar and kick drum. Lower tunings reduce string tension, changing the envelope of the sound (less attack, more bloom), which must be simulated in DSP.

#### Table 1: Common Metal Tunings and MIDI Mapping

| Tuning Name         | String Intervals (Low to High) | Lowest Note (MIDI) | Frequency (Hz) | Subgenre Application             |
| ------------------- | ------------------------------ | ------------------ | -------------- | -------------------------------- |
| E Standard          | E A D G B E                    | E2 (40)            | 82.41          | Thrash, Heavy Metal, Power Metal |
| Drop D              | D A D G B E                    | D2 (38)            | 73.42          | Metalcore, Alt-Metal, Nu-Metal   |
| D Standard          | D G C F A D                    | D2 (38)            | 73.42          | Death Metal (Death, Gojira)      |
| C Standard          | C F Bb Eb G C                  | C2 (36)            | 65.41          | Stoner Doom, Melodic Death       |
| Drop C              | C G C F A D                    | C2 (36)            | 65.41          | Metalcore (Killswitch Engage)    |
| B Standard (7-Str)  | B E A D G B E                  | B1 (35)            | 61.74          | Deathcore, Death Metal           |
| Drop A (7-Str)      | A E A D G B E                  | A1 (33)            | 55.00          | Deathcore (Whitechapel), Djent   |
| F# Standard (8-Str) | F# B E A D G B E               | F#1 (30)           | 46.25          | Djent (Meshuggah), Prog Metal    |
| Drop E (8-Str)      | E B E A D G B E                | E1 (28)            | 41.20          | Extreme Djent/Thall              |

**Algorithmic Insight: Automatic Register Management** The "heaviness" is not just pitch; it is the relationship between the fundamental frequency and the guitar's scale length. However, in a MIDI/synth context, simply lowering the pitch can result in muddiness. The app must implement register logic:

- **Bass Offset**: The bass guitar should generally track the guitar riffs an octave lower. However, if the guitar is in Drop A (55 Hz), the bass at A0 (27.5 Hz) approaches the limit of human hearing and reproduction systems. In these ranges, the bass generator should switch to Unison Mode (playing the same octave as the guitar) or focus on rhythmic counterpoint rather than sub-harmonic support.

- **Voice Leading Rules**: When generating chords in low tunings, close voicings (intervals of thirds or seconds) in the low register cause "mud" due to critical band interactions in the ear. The algorithm must enforce Open Voicings (fifths and octaves) for the lowest strings (power chords) and reserve complex intervals for the higher strings.

## 3. Algorithmic Rhythm: From Euclidean Pulses to Polyrhythmic Djent

Rhythm in metal is arguably more complex than its harmony. A song generator that outputs straight 4/4 quantization will sound like a generic rock drum machine. To sound like metal, it must implement sophisticated rhythmic algorithms that mimic the mathematical precision of modern subgenres.

### 3.1 Euclidean Rhythms

Euclidean rhythms, derived from Bjorklund's algorithm (originally used for neutron accelerators), are a powerful method for generating metal riffs and drum patterns. The core concept is distributing k pulses (hits) as evenly as possible over n steps (time slots).

- **Application in Metal**: Many iconic metal rhythms, such as the "Bleed" pattern by Meshuggah, can be approximated or generated using Euclidean distribution. It creates patterns that are catchy yet syncopated.

- **Rust Implementation**: The `euclidian-rythms` or `rhythms` crates can be utilized.

- **Parameters**: `steps` (total 16th notes in a bar, usually 16), `pulses` (number of pick strikes).

- **Rotation**: Shifting the pattern (rotation) allows the generator to start the riff on an off-beat or a "1", drastically changing the feel.

**Example Logic**: To generate a groovy metalcore kick drum pattern:

1. Set `steps = 16` (one bar of 16th notes).
2. Set `pulses = 5` or `7` (prime numbers create more interesting syncopation).
3. Generate the Euclidean bitmap: `[x.. x.. x.. x. x...]`
4. Map this bitmap to the Kick Drum MIDI note and the Guitar "Open Chug" note simultaneously.

### 3.2 Polymeters and Truncated Loops (The "Djent" Algorithm)

Djent and progressive metal rely on polymeters: playing a rhythmic phrase of length X over a time signature of length Y.

**Concept**: A guitar riff might repeat every 5/16 notes, while the drums keep a steady 4/4 (16/16 notes). This causes the riff to "displace" across the bar lines, aligning with the "1" only after 5 bars (5×16=80 steps, 80/16=5 bars).

**Implementation Strategy**:

1. **Riff Generator**: Create a Sequence of length L (e.g., 5, 7, or 9 sixteenth notes).
2. **Container**: Create a Measure of length 16 (4/4).
3. **Filling Logic**: Loop the Sequence into the Measure. The algorithm must decide whether to truncate the riff at the end of the bar (common in accessible metal) or wrap it into the next bar (common in Meshuggah-style metal).
4. **Resolution**: The generator must calculate the "Least Common Multiple" (LCM) of the riff length and the bar length to determine when the phrase resolves. It should ensure a musical change (e.g., a chorus) occurs at a resolution point.

### 3.3 The Physics of the Breakdown

The breakdown is a structural staple of metalcore and deathcore. It is not just "slow"; it is rhythmically sparse and structurally distinct.

**Algorithmic Rules for Breakdown Generation**:

- **Halftime Feel**: The snare must move from beats 2 and 4 to beat 3 (in 4/4 time). This instantly halves the perceived tempo without changing the BPM.

- **Sub-Bass Drops**: Insert a sine wave drop (808 style) at the beginning of the breakdown (beat 1) to maximize impact.

- **Syncopated Chugs**: Use a Grid-Based Step Sequencer approach.

  - Divide the bar into 16th or 32nd note triplets.
  - Place kick/guitar hits on "random" steps but enforce clustering. Metal breakdowns often use "bursts" of notes followed by silence (rests).
  - **Rule**: A rest must be at least as long as the burst to create the necessary "space" for the heavy impact.

- **The "Call and Response"**: A common pattern is to have a complex rhythmic burst in the first half of the bar and a sustained chord or silence in the second half.

## 4. Procedural Riff Construction: The IRVD Framework

Generating a stream of notes is easy; generating a "riff" is hard. A riff implies memorability and structure. The generator should employ an architectural framework such as IRVD: Introduction, Repetition, Variation, Destruction.

### 4.1 Pedal Point Logic

A vast majority of metal riffs use a Pedal Point technique: returning to a static bass note (usually the open low string) between melodic ideas.

**Generative Algorithm**:

1. **Define Pedal Note**: usually MIDI note 40 (E2) or 38 (D2).
2. **Define Melodic Pool**: A set of notes from the chosen Mode (e.g., Phrygian).
3. **Sequence Generation**:
   - State 0 (Pedal): Play low string (Palm Muted).
   - State 1 (Melodic): Play note from Melodic Pool (Open or Vibrato).
   - **Transition**: Create a Markov Chain where the probability of transitioning from a Melodic note back to the Pedal note is high (>60%). This anchors the riff.

### 4.2 Markov Chains for Pitch Transition

To avoid random melody generation, train a Markov Chain on existing metal TABs or MIDI data.

- **Transition Matrix**: Create a matrix where rows/columns are scale degrees.

  - In a "Heavy Metal" matrix, the transition 1→b2 (minor second) should have a high probability.
  - In a "Power Metal" matrix, the transition 1→4 or 1→5 (perfect intervals) should be higher.

- **Higher-Order Chains**: A simple (1st order) Markov chain only looks at the current note. A 2nd order chain looks at the previous two notes. This is crucial for metal phrases that often rely on 3- or 4-note motifs (e.g., 0-3-5 in Smoke on the Water, or 0-1-4 in death metal).

### 4.3 Tablature-Centric Generation

Unlike piano or synth music, guitar music is constrained by biomechanics. A generator must not produce "impossible" riffs (e.g., spanning 6 frets instantly).

- **Graph Traversal**: The fretboard can be modeled as a graph where nodes are (string, fret) tuples. Edges represent feasible finger movements.

- **Cost Function**: Assign a "cost" to movements.

  - Moving 1 fret on the same string = Low Cost.
  - Moving to an adjacent string on the same fret = Low Cost.
  - Moving 5 frets up = High Cost.
  - String Skipping (String 6 to String 4) = Medium Cost.

- **Optimization**: Use Dijkstra's algorithm or A\* search to find the most "playable" path for a generated sequence of notes. This ensures the output can be played by a human or sounds realistic when synthesized.

## 5. Drum Programming: Humanization and Articulation

Robotic drums are the bane of programmed metal. To sound "actual," the app must implement Humanization and Dynamic Articulation logic.

### 5.1 Velocity Dynamics

MIDI velocity (0-127) is not just volume; in sample libraries (like Superior Drummer or GGD), it triggers different samples.

- **The "Max Velocity" Myth**: Beginners program metal drums at velocity 127 for everything. Real drummers cannot hit 127 continuously.

- **Blast Beat Logic**:

  - In a high-speed blast beat, the snare hits should be lower velocity (e.g., 90-110) because the range of motion is smaller.
  - **Accents**: The first beat of the measure should be accented (higher velocity).

- **The "Machine Gun" Effect**: Repeatedly triggering the same sample at the exact same velocity sounds fake.

- **Solution**: Round Robin Simulation. Even if the sampler doesn't support round robin, the generator should randomize velocity by ±3 to ±5 units for every hit to trigger different sample layers.

### 5.2 Micro-Timing (Grid Offset)

Metal drummers playing fast often push or pull the beat.

- **Pushing (Rushing)**: In frantic sections (thrash), the snare often lands slightly before the grid (e.g., -5 to -10 ticks). This creates urgency.

- **Dragging**: In breakdowns or "sludge" riffs, the snare lands slightly after the grid (+5 to +15 ticks). This creates "weight."

**Rust Implementation**: The sequencer should have a `timing_bias` parameter. When generating the MIDI event timestamp:

```rust
event_time = grid_time + rand::thread_rng().gen_range(bias - variance, bias + variance);
```

### 5.3 Blast Beat Patterns

The generator should include presets for standard metal patterns:

- **Traditional Blast**: Kick and Snare hit simultaneously on 8th or 16th notes.
- **Hammer Blast**: Kick and Snare unison.
- **Euro Blast**: Kick and Snare alternate (Kick-Snare-Kick-Snare).
- **Gravity Blast**: Uses MIDI rimshot articulations to simulate the stick pivoting on the rim.

## 6. Digital Signal Processing (DSP): The Tone Engine

Since the user asks for the app to "sound like metal," symbolic generation (MIDI) is only half the battle. The audio synthesis engine (written in Rust) must process these notes to create the distorted texture of high-gain amplifiers.

### 6.1 Waveshaping for Distortion

Distortion is mathematically the process of waveshaping—applying a non-linear transfer function to an input signal (the dry guitar string sound).

- **Hard Clipping**: `f(x) = sign(x)` if `|x| > threshold`. This sounds harsh and digital (fuzz).

- **Soft Clipping (Tube Emulation)**: The hyperbolic tangent function (tanh) is the industry standard for emulating tube saturation.

  - **Formula**: `y = tanh(k·x)` where k is the input gain (drive).

- **Asymmetric Clipping**: Real tube amps often clip the positive and negative cycles differently.

**Rust Logic**:

```rust
fn process_sample(sample: f32, drive: f32) -> f32 {
    (sample * drive).tanh()
}
```

- **Oversampling**: Non-linear operations like `tanh` introduce infinite harmonics. If these harmonics exceed the Nyquist frequency (half the sample rate), they fold back as aliasing (non-harmonic noise). The Rust DSP chain must implement Oversampling (up-sampling the signal 4x or 8x, processing, then down-sampling) to keep the distortion sounding "creamy" rather than "harsh".

### 6.2 Impulse Response (IR) Convolution

A raw distorted signal sounds like a "wasp in a jar." The characteristic "roar" of metal comes from the guitar cabinet (speaker). This is modeled using Convolution.

- **Concept**: An Impulse Response (IR) is a `.wav` file recording of a real cabinet (e.g., Mesa Boogie 4x12) struck by a click.

- **Process**: The distorted signal is convolved with the IR. Mathematically, this multiplies the frequency spectrum of the signal by the spectrum of the cabinet.

**Rust Implementation**:

- Use the `fft-convolver` or `impulse_response` crate. Real-time convolution is computationally expensive (O(N²)), so FFT-based convolution (O(NlogN)) is required.

- **Partitioned Convolution**: To reduce latency (crucial for real-time play), the IR is split into small blocks. The first blocks are processed with zero latency, while later blocks are computed in the background.

### 6.3 Karplus-Strong Synthesis for Guitar Strings

If the app synthesizes the guitar sound from scratch (rather than using samples), the Karplus-Strong algorithm is the most efficient method to simulate a plucked string.

**Algorithm**:

1. Fill a buffer of length L with white noise (the "pluck").
2. Output the first sample.
3. Take the average of the first two samples, multiply by a decay factor (0.99... for sustain, 0.90 for palm mute), and append to the end of the buffer.
4. Repeat.

**Metal Modification (Palm Muting)**: To simulate a palm mute, the decay factor must be aggressively lowered, and a Low Pass Filter (LPF) should be applied in the feedback loop. This damps the high frequencies faster, mimicking the flesh of the hand on the bridge.

## 7. Architecture of the Rust Application

To integrate these theories into a cohesive app, the architecture should follow a pipeline approach.

### 7.1 Data Structures

- **Riff Struct**: Contains a vector of `NoteEvents`.
- **NoteEvent**: `{ pitch: u8, duration: u32, velocity: u8, technique: Technique }`
- **Technique Enum**: `PalmMute`, `Open`, `Vibrato`, `Slide`, `Harmonic`.

### 7.2 The Generation Pipeline

1. **Constraint Solver**: User inputs inputs (e.g., "Djent", "140 BPM", "Drop A").
2. **Rhythm Engine**: Generates the skeletal rhythm using Euclidean or Polymetric algorithms.
3. **Pitch Engine**: Maps the rhythm to pitches using Markov Chains weighted by the selected Mode (e.g., Phrygian).
4. **Articulation Engine**: Analyzes the rhythm. High-density notes (16th notes) are tagged `PalmMute`. Longer notes are tagged `Open`.
5. **Humanizer**: Applies micro-timing and velocity randomization.

### 7.3 The Audio Pipeline (Real-Time)

Using `cpal` for audio output and `rodio` or `fundsp` for processing:

1. **Synthesizer**: Karplus-Strong oscillator generates raw string audio based on `NoteEvent`.
2. **Pre-FX**: Noise Gate (essential for metal to stop hum between staccato riffs).
3. **Amp Sim**: `tanh` distortion + Tone Stack (EQ).
4. **Cab Sim**: FFT Convolution with a metal IR (e.g., Celestion V30).
5. **Post-FX**: Room Reverb (for "glue").

## 8. Specific Techniques for Subgenres

To make the app versatile, it should recognize "Subgenre Profiles."

- **Thrash Metal**: High tempo (180+ BPM). Key: E Minor or Eb. Focus on "Gallop" rhythms (Eighth + two Sixteenths). High probability of chromatic movement (1↔b2).

- **Death Metal**: Low tuning (C Standard or lower). Tremolo picking (32nd notes). Chromaticism and Diminished 5ths. Drum generator must prioritize Blast Beats.

- **Black Metal**: High tempo. High-pitched chords (minor triads on top strings). "Lo-fi" production (reduce bass in the EQ). Drum generator focuses on continuous 16th note double bass.

- **Djent**: Polymetric rhythms. Low tuning (Drop A/F#). Tone requires a "Gate" that cuts off sound instantly after a note to create the "staccato" machine-like effect. Heavy boost at 1.4kHz (the "djent" frequency) before the distortion stage.

## 9. Conclusion

Improving a Rust song generator to sound like metal requires moving beyond basic randomization and into the realm of idiomatic simulation. By implementing the music theory of modes (specifically Phrygian and Locrian), the mathematical rigor of Euclidean and polymetric rhythms, and the acoustic physics of distortion and cabinet convolution, the application can generate music that respects the cultural and sonic expectations of the genre. The code must essentially "think" like a metal musician: prioritizing dissonance, locking rhythmically to the grid while humanizing the performance, and shaping the tone to maximize aggression and clarity.

## 10. Sources

- [metalmastermind.com](https://metalmastermind.com) - Metal Music Theory For Beginner: Scales, Chords, and Songwriting
- [scholarworks.sfasu.edu](https://scholarworks.sfasu.edu) - MODES IN HEAVY METAL MUSIC | SFA ScholarWorks
- [reddit.com](https://reddit.com) - What intervals make metal sound metal? : r/guitarlessons - Reddit
- [crates.io](https://crates.io) - rust-music-theory - crates.io: Rust Package Registry
- [youtube.com](https://youtube.com) - Metal Scales & Modes - YouTube
- [reddit.com](https://reddit.com) - What is the metal scale? : r/metalguitar - Reddit
- [youngcomposers.com](https://youngcomposers.com) - Composing metal - Advice and Techniques
- [overdriven.fr](https://overdriven.fr) - Guitar tunings - Overdriven.fr
- [reddit.com](https://reddit.com) - What's everyone's favorite tuning for metal songs? : r/metalguitar - Reddit
- [quora.com](https://quora.com) - When writing bass lines, do people normally start with the guitar line, bass line, or from scratch? - Quora
- [reddit.com](https://reddit.com) - Metal Basslines that differ from the guitar riff? : r/Bass - Reddit
- [nailthemix.com](https://nailthemix.com) - How to Build a Killer Metal Mix Template for a Faster Workflow - Nail The Mix
- [en.wikipedia.org](https://en.wikipedia.org) - Euclidean rhythm - Wikipedia
- [ampedstudio.com](https://ampedstudio.com) - Euclidean rhythm in music - Amped Studio
- [medium.com](https://medium.com) - Euclidean Rhythms. 'Euclidean' rhythms are one of the few… | by Jeff Holtzkener | Code/Music/Noise | Medium
- [crates.io](https://crates.io) - rhythms - crates.io: Rust Package Registry
- [lib.rs](https://lib.rs) - euclidian-rythms - Lib.rs
- [reddit.com](https://reddit.com) - Progressive metal subgenres (djent, math, technical) : r/musictheory - Reddit
- [youtube.com](https://youtube.com) - Writing Complex Prog / Djent Rhythms that GROOVE- Truncated Polymeters [RHYTHM - YouTube
- [signalsmusicstudio.com](https://signalsmusicstudio.com) - Writing Complex Prog / Djent Rhythms that GROOVE- Truncated Polymeters [RHYTHM – WRITING] - Signals Music Studio
- [riffhard.com](https://riffhard.com) - How to Write a Metalcore Breakdown - Riffhard
- [reddit.com](https://reddit.com) - Metalcore breakdown patterns? - Reddit
- [youtube.com](https://youtube.com) - How To Write a Metalcore Breakdown - YouTube
- [youtube.com](https://youtube.com) - How to Write Metal Guitar Riff From a Raw Idea | Guitar Lesson with TAB - YouTube
- [guitarmetal.com](https://guitarmetal.com) - How Musicians Use Pedal Points In Their Music - Guitar Metal
- [reddit.com](https://reddit.com) - Pedal tone/point riffs : r/metalguitar - Reddit
- [ultimate-guitar.com](https://ultimate-guitar.com) - Pedal-Point Riffs: Metal Rhythm Guitar Basics
- [ufdcimages.uflib.ufl.edu](https://ufdcimages.uflib.ufl.edu) - Modern Improvisational Melody Generation Using Markov chains - UFDC Image Array 2
- [medium.com](https://medium.com) - Markov Chain for music generation | by Alexander Osipenko | TDS Archive | Medium
- [mdpi.com](https://mdpi.com) - A Transformational Modified Markov Process for Chord-Based Algorithmic Composition
- [medium.com](https://medium.com) - Using Linear Algebra and Markov Chains to Algorithmically Generate Music Compositions | by Vanessa Seto | Medium
- [arxiv.org](https://arxiv.org) - GOAT: A Large Dataset of Paired Guitar Audio Recordings and Tablatures - arXiv
- [toontrack.com](https://toontrack.com) - 21st Century Metal MIDI - Toontrack
- [metalrecording.net](https://metalrecording.net) - How to Program Realistic Drum Tracks for Extreme Metal - MetalRecording.net
- [steinberg.net](https://steinberg.net) - How To Make MIDI Drums Sound Real For Heavy Metal - Steinberg
- [youtube.com](https://youtube.com) - The Drum Programming Tricks Every Metal Producer Needs - YouTube
- [midiremap.com](https://midiremap.com) - MidiRemap | Convert your drums to any library
- [reddit.com](https://reddit.com) - What humanize timings do you recommend for tight (metal) programmed drums? - Reddit
- [youtube.com](https://youtube.com) - How to Programme Blast Beats with MIDI - PART 1 | Brickwall Sounds - YouTube
- [github.com](https://github.com) - SamiPerttu/fundsp: Library for audio processing and synthesis - GitHub
- [docs.rs](https://docs.rs) - synfx_dsp - Rust - Docs.rs
- [github.com](https://github.com) - acaala/rustone: A guitar amp simulator written in Rust. - GitHub
- [forum.juce.com](https://forum.juce.com) - Step by step saturation/distortion effect math notation to reasonable c++ code - JUCE Forum
- [gdsp.hf.ntnu.no](https://gdsp.hf.ntnu.no) - Waveshaping - gDSP - Online Course | Distortion
- [juce.com](https://juce.com) - Tutorial: Add distortion through waveshaping and convolution - JUCE
- [reddit.com](https://reddit.com) - Learning Audio DSP with Rust with a Practical Project: Should i build or use an existing Audio DSP library? - Reddit
- [crates.io](https://crates.io) - impulse_response - crates.io: Rust Package Registry
- [github.com](https://github.com) - holoplot/fft-convolution: Audio convolution algorithm in Rust for real-time audio processing
- [blog.demofox.org](https://blog.demofox.org) - Synthesizing a Plucked String Sound With the Karplus-Strong Algorithm
- [youtube.com](https://youtube.com) - Explaining Karplus-Strong Synthesis (Simple) - YouTube
- [amid.fish](https://amid.fish) - Karplus-Strong String Synthesis - Amid Fish
- [reddit.com](https://reddit.com) - How to achieve a better sound using the Karplus-Strong synthesis? - Reddit
- [youtube.com](https://youtube.com) - Programming MIDI Guitars for Metal - The Ultimate Guide - YouTube
