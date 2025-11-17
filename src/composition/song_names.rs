use rand::Rng;

/// Generates funky, jazzy, groovy song names with personality
pub fn generate_song_name() -> String {
    let mut rng = rand::thread_rng();
    
    let adjectives = vec![
        // Funky vibes
        "Funky", "Groovy", "Swingin'", "Jumpin'", "Bumpin'",
        "Soulful", "Smooth", "Slick", "Cool", "Hot",
        
        // Jazz vibes
        "Blue", "Mellow", "Sweet", "Sassy", "Jazzy",
        "Velvet", "Silky", "Liquid", "Dreamy", "Moody",
        
        // Electro-swing vibes
        "Electric", "Cosmic", "Neon", "Digital", "Vintage",
        "Retro", "Urban", "Downtown", "Midnight", "Sunset",
        
        // Energy vibes
        "Wild", "Crazy", "Wicked", "Radical", "Ultimate",
        "Supreme", "Golden", "Silver", "Diamond", "Platinum",
        
        // Time/place vibes
        "Late Night", "Early Morning", "Twilight", "Moonlight", "Starlight",
        "Underground", "Uptown", "Street", "Avenue", "Boulevard",
    ];
    
    let nouns = vec![
        // Music/groove terms
        "Groove", "Rhythm", "Beat", "Pulse", "Funk",
        "Soul", "Vibes", "Flow", "Motion", "Movement",
        
        // Jazz terms
        "Blues", "Swing", "Bop", "Cats", "Notes",
        "Riff", "Lick", "Solo", "Jam", "Session",
        
        // Urban/city vibes
        "Streets", "Lights", "City", "Dreams", "Nights",
        "Days", "Tales", "Stories", "Memories", "Echoes",
        
        // Cosmic vibes
        "Galaxy", "Cosmos", "Stars", "Space", "Universe",
        "Orbit", "Nebula", "Planet", "Moon", "Sun",
        
        // Smooth vibes
        "Velvet", "Silk", "Satin", "Honey", "Sugar",
        "Whisper", "Breeze", "Wave", "River", "Ocean",
        
        // Party vibes
        "Party", "Celebration", "Fiesta", "Carnival", "Festival",
        "Dance", "Shuffle", "Strut", "Glide", "Slide",
    ];
    
    let suffixes = vec![
        "", " Delight", " Express", " Special", " Supreme",
        " Connection", " Experience", " Adventure", " Journey",
        " Sensation", " Fever", " Magic", " Spell", " Charm",
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

/// Generates a genre/style tag for the song
pub fn generate_genre_tags() -> Vec<String> {
    let mut rng = rand::thread_rng();
    
    let primary_genres = vec!["Funk", "Jazz", "Electro-Swing", "Soul", "Rock"];
    let secondary_styles = vec![
        "Groovy", "Jazzy", "Funky", "Smooth", "Upbeat", 
        "Chill", "Energetic", "Retro", "Modern", "Experimental"
    ];
    
    let mut tags = Vec::new();
    
    // Always add a primary genre
    tags.push(primary_genres[rng.gen_range(0..primary_genres.len())].to_string());
    
    // 70% chance to add a secondary style
    if rng.gen_range(0..100) < 70 {
        tags.push(secondary_styles[rng.gen_range(0..secondary_styles.len())].to_string());
    }
    
    // 30% chance to add "Instrumental"
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
            let tags = generate_genre_tags();
            assert!(!tags.is_empty());
            println!("Tags: {:?}", tags);
        }
    }
}

