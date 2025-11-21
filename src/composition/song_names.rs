use rand::Rng;

/// Generates funky, jazzy, groovy song names with personality
pub fn generate_song_name() -> String {
    let mut rng = rand::thread_rng();

    let adjectives = vec![
        // Funky vibes
        "Funky",
        "Groovy",
        "Swingin'",
        "Jumpin'",
        "Bumpin'",
        "Soulful",
        "Smooth",
        "Slick",
        "Cool",
        "Hot",
        // Jazz vibes
        "Blue",
        "Mellow",
        "Sweet",
        "Sassy",
        "Jazzy",
        "Velvet",
        "Silky",
        "Liquid",
        "Dreamy",
        "Moody",
        // Electro-swing vibes
        "Electric",
        "Cosmic",
        "Neon",
        "Digital",
        "Vintage",
        "Retro",
        "Urban",
        "Downtown",
        "Midnight",
        "Sunset",
        // Energy vibes
        "Wild",
        "Crazy",
        "Wicked",
        "Radical",
        "Ultimate",
        "Supreme",
        "Golden",
        "Silver",
        "Diamond",
        "Platinum",
        // Time/place vibes
        "Late Night",
        "Early Morning",
        "Twilight",
        "Moonlight",
        "Starlight",
        "Underground",
        "Uptown",
        "Street",
        "Avenue",
        "Boulevard",
    ];

    let nouns = vec![
        // Music/groove terms
        "Groove",
        "Rhythm",
        "Beat",
        "Pulse",
        "Funk",
        "Soul",
        "Vibes",
        "Flow",
        "Motion",
        "Movement",
        // Jazz terms
        "Blues",
        "Swing",
        "Bop",
        "Cats",
        "Notes",
        "Riff",
        "Lick",
        "Solo",
        "Jam",
        "Session",
        // Urban/city vibes
        "Streets",
        "Lights",
        "City",
        "Dreams",
        "Nights",
        "Days",
        "Tales",
        "Stories",
        "Memories",
        "Echoes",
        // Cosmic vibes
        "Galaxy",
        "Cosmos",
        "Stars",
        "Space",
        "Universe",
        "Orbit",
        "Nebula",
        "Planet",
        "Moon",
        "Sun",
        // Smooth vibes
        "Velvet",
        "Silk",
        "Satin",
        "Honey",
        "Sugar",
        "Whisper",
        "Breeze",
        "Wave",
        "River",
        "Ocean",
        // Party vibes
        "Party",
        "Celebration",
        "Fiesta",
        "Carnival",
        "Festival",
        "Dance",
        "Shuffle",
        "Strut",
        "Glide",
        "Slide",
    ];

    let suffixes = vec![
        "",
        " Delight",
        " Express",
        " Special",
        " Supreme",
        " Connection",
        " Experience",
        " Adventure",
        " Journey",
        " Sensation",
        " Fever",
        " Magic",
        " Spell",
        " Charm",
    ];

    // Randomly choose structure
    let structure_roll = rng.gen_range(0..100);

    if structure_roll < 70 {
        // Adjective + Noun structure (70% of the time)
        let adj = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        let suffix = suffixes[rng.gen_range(0..suffixes.len())];
        format!("{} {}{}", adj, noun, suffix)
    } else if structure_roll < 90 {
        // Double noun (20% of the time) - like "Groove City"
        let noun1 = nouns[rng.gen_range(0..nouns.len())];
        let noun2 = nouns[rng.gen_range(0..nouns.len())];
        format!("{} {}", noun1, noun2)
    } else {
        // Triple word (10% of the time) - like "Cosmic Midnight Groove"
        let adj1 = adjectives[rng.gen_range(0..adjectives.len())];
        let adj2 = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        format!("{} {} {}", adj1, adj2, noun)
    }
}

use crate::composition::Genre;

/// Generates a genre/style tag for the song based on the actual genre
pub fn generate_genre_tags(genre: Genre) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut tags = Vec::new();

    // Add primary genre name
    let primary = match genre {
        Genre::Lofi => "Lofi",
        Genre::Rock => "Rock",
        Genre::Dubstep => "Dubstep",
        Genre::DnB => "Drum & Bass",
        Genre::Jazz => "Jazz",
        Genre::Funk => "Funk",
        Genre::HipHop => "Hip Hop",
        Genre::ElectroSwing => "Electro Swing",
    };
    tags.push(primary.to_string());

    // Genre-specific secondary tags
    let secondary_options = match genre {
        Genre::Lofi => vec!["Chill", "Study", "Relaxing", "Nostalgic", "Sleep", "Beats"],
        Genre::Rock => vec!["Alternative", "Indie", "Electric", "Guitar", "Energy"],
        Genre::Dubstep => vec!["Bass", "Electronic", "Heavy", "Wobble", "Drop"],
        Genre::DnB => vec!["Liquid", "Fast", "Electronic", "Breakbeat", "Energy"],
        Genre::Jazz => vec!["Smooth", "Fusion", "Improv", "Lounge", "Cafe"],
        Genre::Funk => vec!["Groove", "Soul", "Rhythm", "Dance", "Classic"],
        Genre::HipHop => vec!["Boom Bap", "Urban", "Rap", "Beats", "Old School"],
        Genre::ElectroSwing => vec!["Vintage", "Swing", "Dance", "Retro", "Upbeat"],
    };

    // Add 1-2 secondary tags
    let num_secondary = rng.gen_range(1..=2);
    for _ in 0..num_secondary {
        let tag = secondary_options[rng.gen_range(0..secondary_options.len())];
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }

    // Global stylistic tags
    if rng.gen_range(0..100) < 30 {
        tags.push("Instrumental".to_string());
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_name_generation() {
        // Generate a bunch of names to make sure they all work
        for _ in 0..20 {
            let name = generate_song_name();
            assert!(!name.is_empty());
            assert!(name.len() > 3);
            println!("Generated: {}", name);
        }
    }

    #[test]
    fn test_genre_tags() {
        for _ in 0..10 {
            let tags = generate_genre_tags(Genre::Lofi);
            assert!(!tags.is_empty());
            assert!(tags.contains(&"Lofi".to_string()));
            println!("Tags: {:?}", tags);
        }
    }
}
