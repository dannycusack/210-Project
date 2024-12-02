use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{stdin, Write};
use csv::ReaderBuilder;
use serde::de::{self, Deserializer};

fn parse_bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(de::Error::custom(format!("Invalid boolean value: {}", s))),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Track {
    track_id: String,
    artists: String,
    album_name: String,
    track_name: String,
    popularity: u32,
    duration_ms: u32,
    #[serde(deserialize_with = "parse_bool_from_string")]
    explicit: bool,
    danceability: f32,
    energy: f32,
    tempo: f32,
    valence: f32,
    key: u32,
    loudness: f32,
    mode: u32,
    acousticness: f32,
    instrumentalness: f32,
    liveness: f32,
}

// Function to load tracks from a CSV file
fn load_tracks_from_csv(file_path: &str) -> Result<Vec<Track>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut tracks = Vec::new();
    for result in rdr.deserialize() {
        let record: Track = result?;
        tracks.push(record);
    }
    Ok(tracks)
}

// Function to handle multiple matches and allow user to select one
fn select_track<'a>(tracks: &'a [&'a Track]) -> Option<&'a Track> {
    // Sort tracks by popularity in descending order
    let mut sorted_tracks = tracks.to_vec();
    sorted_tracks.sort_by(|a, b| b.popularity.cmp(&a.popularity));

    // Limit to the top 3 tracks
    let top_tracks = sorted_tracks.iter().take(3).collect::<Vec<_>>();

    println!("Multiple matches found. Please select one of the top 3 most popular songs:");
    for (index, track) in top_tracks.iter().enumerate() {
        println!(
            "{}: \"{}\" by {} (Album: {}, Popularity: {})",
            index + 1,
            track.track_name,
            track.artists,
            track.album_name,
            track.popularity
        );
    }

    print!("Enter the number of the correct song: ");
    std::io::stdout().flush().unwrap();

    let mut selection = String::new();
    stdin().read_line(&mut selection).unwrap();
    let selection = selection.trim().parse::<usize>();

    match selection {
        Ok(num) if num > 0 && num <= top_tracks.len() => Some(top_tracks[num - 1]),
        _ => {
            println!("Invalid selection.");
            None
        }
    }
}

// Function to find songs similar to a given input song
fn find_similar_songs<'a>(
    tracks: &'a [Track],
    input_track: &Track,
    danceability_threshold: f32,
    energy_threshold: f32,
    tempo_threshold: f32,
    valence_threshold: f32,
    popularity_threshold: u32,
) -> Vec<&'a Track> {
    let mut similar_songs = Vec::new();
    let mut unique_names = HashSet::new();

    for track in tracks {
        if (track.danceability - input_track.danceability).abs() <= danceability_threshold
            && (track.energy - input_track.energy).abs() <= energy_threshold
            && (track.tempo - input_track.tempo).abs() <= tempo_threshold
            && (track.valence - input_track.valence).abs() <= valence_threshold
            && track.popularity > popularity_threshold
            && track.track_id != input_track.track_id
        {
            if unique_names.insert(track.track_name.clone()) {
                similar_songs.push(track);
            }
        }
    }

    similar_songs.sort_by(|a, b| b.popularity.cmp(&a.popularity));

    similar_songs
}

// Function to display a clean graph
fn display_clean_subgraph(input_track: &Track, similar_songs: &[&Track]) {
    println!(
        "Top 5 similar songs to \"{}\" by {} [Danceability: {:.2}, Energy: {:.2}, Tempo: {:.2}, Valence: {:.2}, Popularity: {}]:",
        input_track.track_name,
        input_track.artists,
        input_track.danceability,
        input_track.energy,
        input_track.tempo,
        input_track.valence,
        input_track.popularity
    );
    for song in similar_songs {
        println!(
            "  -> \"{}\" by {} [Danceability: {:.2}, Energy: {:.2}, Tempo: {:.2}, Valence: {:.2}, Popularity: {}]",
            song.track_name,
            song.artists,
            song.danceability,
            song.energy,
            song.tempo,
            song.valence,
            song.popularity
        );
    }
}

// Function to build the graph for the input song and similar songs
fn build_song_subgraph<'a>(
    input_track: &'a Track,
    similar_songs: &[&'a Track],
) -> HashMap<String, (String, Vec<(String, f32, f32, u32)>)> {
    let mut graph = HashMap::new();

    // Add the input song as the central node
    graph.insert(
        input_track.track_name.clone(),
        (
            format!(
                "Danceability: {:.2}, Energy: {:.2}, Tempo: {:.2}, Valence: {:.2}, Popularity: {}",
                input_track.danceability,
                input_track.energy,
                input_track.tempo,
                input_track.valence,
                input_track.popularity
            ),
            similar_songs
                .iter()
                .map(|song| {
                    (
                        song.track_name.clone(),
                        song.danceability,
                        song.energy,
                        song.popularity,
                    )
                })
                .collect(),
        ),
    );

    graph
}

// Function to export the graph to a DOT file
fn export_subgraph_to_dot(
    graph: &HashMap<String, (String, Vec<(String, f32, f32, u32)>)>,
    file_path: &str,
) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;

    writeln!(file, "graph {{")?;

    for (node, (details, neighbors)) in graph {
        writeln!(file, "    \"{}\" [label=\"{}\"];", node, details)?;
        for (neighbor, danceability, energy, popularity) in neighbors {
            writeln!(
                file,
                "    \"{}\" -> \"{}\" [label=\"Danceability: {:.2}, Energy: {:.2}, Popularity: {}\"];",
                node, neighbor, danceability, energy, popularity
            )?;
        }
    }

    writeln!(file, "}}")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set the path to your CSV dataset
    let file_path = "spotify.csv"; // Replace with the correct path
    let tracks = load_tracks_from_csv(file_path)?;
    println!("Loaded {} tracks from the dataset.", tracks.len());

    // Set thresholds
    let danceability_threshold = 0.05;
    let energy_threshold = 0.05;
    let tempo_threshold = 50.0;
    let valence_threshold = 0.1;
    let popularity_threshold = 70;

    // Ask the user for a song name
    println!("Enter the name of a song:");
    let mut input_song = String::new();
    stdin().read_line(&mut input_song)?;
    let input_song = input_song.trim();

    // Find all tracks with the input name
    let matching_tracks: Vec<_> = tracks
        .iter()
        .filter(|t| t.track_name.eq_ignore_ascii_case(input_song))
        .collect();

    if matching_tracks.is_empty() {
        println!("No song found with the name '{}'.", input_song);
        return Ok(());
    }

    // If multiple matches are found, show top 3 by popularity and let the user select one
    let selected_track = if matching_tracks.len() > 1 {
        select_track(&matching_tracks)
    } else {
        Some(matching_tracks[0])
    };

    if let Some(input_track) = selected_track {
        // Find similar songs
        let similar_songs = find_similar_songs(
            &tracks,
            input_track,
            danceability_threshold,
            energy_threshold,
            tempo_threshold,
            valence_threshold,
            popularity_threshold,
        );

        // Remove duplicates and limit to top 5 similar songs
        let top_similar_songs = similar_songs.iter().take(5).cloned().collect::<Vec<_>>();

        // Display the cleaned-up graph
        display_clean_subgraph(input_track, &top_similar_songs);

        // Export to DOT file
        let dot_file_path = "graph.dot";
        export_subgraph_to_dot(&build_song_subgraph(input_track, &top_similar_songs), dot_file_path)?;
        println!("Graph exported to '{}'.", dot_file_path);
    }

    Ok(())
}