use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{stdin, Write};
use csv::ReaderBuilder;
#[cfg(test)]
mod tests;

//represents a track from the dataset
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Track {
    track_id: String,
    artists: String,
    album_name: String,
    track_name: String,
    popularity: u32,
    danceability: f32,
    energy: f32,
    tempo: f32,
    valence: f32,
}
//loads tracks from CSV into a vector of Track structs
fn load_tracks_from_csv(file_path: &str) -> Result<Vec<Track>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut tracks = Vec::new();
    for result in rdr.deserialize() {
        let record: Track = result?;
        tracks.push(record); //add track to vector
    }
    Ok(tracks)
}
//allows user to enter one track
fn select_track<'a>(tracks: &'a [&'a Track]) -> Option<&'a Track> {
    let mut sorted_tracks = tracks.to_vec();
    sorted_tracks.sort_by(|a, b| b.popularity.cmp(&a.popularity));
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
    //return selected track, or None if input invalid
    match selection {
        Ok(num) if num > 0 && num <= top_tracks.len() => Some(top_tracks[num - 1]),
        _ => {
            println!("Invalid selection.");
            None
        }
    }
}
//find similar songs based on thresholds for track features
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
    //checks if track is similar based on thresholds
    for track in tracks {
        if (track.danceability - input_track.danceability).abs() <= danceability_threshold
            && (track.energy - input_track.energy).abs() <= energy_threshold
            && (track.tempo - input_track.tempo).abs() <= tempo_threshold
            && (track.valence - input_track.valence).abs() <= valence_threshold
            && track.popularity > popularity_threshold
            && track.track_id != input_track.track_id
        {
            //avoids duplicates
            if unique_names.insert(track.track_name.clone()) {
                similar_songs.push(track);
            }
        }
    }
    similar_songs.sort_by(|a, b| b.popularity.cmp(&a.popularity));
    similar_songs
}
//displays input track and top 5 similar songs
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
            song.popularity,
        );
    }
}
//builds graph representation of input track and similar songs
fn build_song_subgraph<'a>(
    input_track: &'a Track,
    similar_songs: &[&'a Track],
) -> HashMap<String, (String, Vec<(String, f32, f32, f32, f32, u32)>)> {
    let mut graph = HashMap::new();
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
                        song.tempo,
                        song.valence,
                        song.popularity,
                    )
                })
                .collect(),
        ),
    );
    graph
}

//exports graph to DOT file
fn export_subgraph_to_dot(
    graph: &HashMap<String, (String, Vec<(String, f32, f32, f32, f32, u32)>)>,
    file_path: &str,
) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    writeln!(file, "graph {{")?;
    for (node, (details, neighbors)) in graph {
        writeln!(file, "    \"{}\" [label=\"{}\"];", node, details)?;
        for (neighbor, danceability, energy, tempo, valence, popularity) in neighbors {
            writeln!(
                file,
                "    \"{}\" -> \"{}\" [label=\"Danceability: {:.2}, Energy: {:.2}, Tempo: {:.2}, Valence: {:.2}, Popularity: {}\"];",
                node, neighbor, danceability, energy, tempo, valence, popularity
            )?;
        }
    }
    writeln!(file, "}}")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "spotify.csv";
    let tracks = load_tracks_from_csv(file_path)?;
    println!("Loaded {} tracks from the dataset.", tracks.len());
    //thresholds
    let danceability_threshold = 0.05;
    let energy_threshold = 0.05;
    let tempo_threshold = 50.0;
    let valence_threshold = 0.1;
    let popularity_threshold = 70;
    println!("Enter the name of a song:");
    let mut input_song = String::new();
    stdin().read_line(&mut input_song)?;
    let input_song = input_song.trim();
    //tracks that match input song name
    let matching_tracks: Vec<_> = tracks
        .iter()
        .filter(|t| t.track_name.eq_ignore_ascii_case(input_song))
        .collect();
    if matching_tracks.is_empty() {
        println!("No song found with the name '{}'.", input_song);
        return Ok(());
    }

    let selected_track = if matching_tracks.len() > 1 {
        select_track(&matching_tracks)
    } else {
        Some(matching_tracks[0])
    };

    if let Some(input_track) = selected_track {
        let similar_songs = find_similar_songs(
            &tracks,
            input_track,
            danceability_threshold,
            energy_threshold,
            tempo_threshold,
            valence_threshold,
            popularity_threshold,
        );
        let top_similar_songs = similar_songs.iter().take(5).cloned().collect::<Vec<_>>();
        display_clean_subgraph(input_track, &top_similar_songs);

        let dot_file_path = "graph.dot";
        export_subgraph_to_dot(&build_song_subgraph(input_track, &top_similar_songs), dot_file_path)?;
        println!("Graph exported to '{}'.", dot_file_path);
    }
    Ok(())
}