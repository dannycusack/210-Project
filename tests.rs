use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Track {
    pub track_id: String,
    pub artists: String,
    pub album_name: String,
    pub track_name: String,
    pub popularity: u32,
    pub danceability: f32,
    pub energy: f32,
    pub tempo: f32,
    pub valence: f32,
}

#[cfg(test)]
pub fn load_tracks_from_csv(file_path: &str) -> Result<Vec<Track>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut tracks = Vec::new();
    for result in rdr.deserialize() {
        let record: Track = result?;
        tracks.push(record);
    }
    Ok(tracks)
}

#[cfg(test)]
pub fn select_track<'a>(tracks: &'a [&'a Track], user_input: Option<usize>) -> Option<&'a Track> {
    let mut sorted_tracks = tracks.to_vec();
    sorted_tracks.sort_by(|a, b| b.popularity.cmp(&a.popularity));
    let top_tracks = sorted_tracks.iter().take(3).collect::<Vec<_>>();

    if let Some(input) = user_input {
        if input > 0 && input <= top_tracks.len() {
            return Some(top_tracks[input - 1]);
        }
    }
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
    std::io::stdin().read_line(&mut selection).unwrap();
    let selection = selection.trim().parse::<usize>();

    match selection {
        Ok(num) if num > 0 && num <= top_tracks.len() => Some(top_tracks[num - 1]),
        _ => {
            println!("Invalid selection.");
            None
        }
    }
}

#[cfg(test)]
pub fn find_similar_songs<'a>(
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

#[cfg(test)]
pub fn build_song_subgraph<'a>(
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

#[cfg(test)]
pub fn export_subgraph_to_dot(
    graph: &HashMap<String, (String, Vec<(String, f32, f32, f32, f32, u32)>)>,
    file_path: &str,
) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_load_tracks_from_csv() {
        let file_path = "test_data/test_data.csv";
        std::fs::create_dir_all("test_data").unwrap();
        let mut file = File::create(file_path).unwrap();
        writeln!(
            file,
            "track_id,artists,album_name,track_name,popularity,danceability,energy,tempo,valence"
        )
        .unwrap();
        writeln!(file, "1,Artist A,Album A,Song A,85,0.8,0.9,120.0,0.7").unwrap();
        writeln!(file, "2,Artist B,Album B,Song B,75,0.75,0.85,122.0,0.68").unwrap();

        let result = load_tracks_from_csv(file_path);
        assert!(result.is_ok(), "Failed to load tracks from CSV.");
        let tracks = result.unwrap();
        assert_eq!(tracks.len(), 2, "Expected 2 tracks in the CSV file.");

        std::fs::remove_file(file_path).unwrap();
        std::fs::remove_dir("test_data").unwrap();
    }

    #[test]
    fn test_select_track() {
        let track1 = Track {
            track_id: "1".to_string(),
            artists: "Artist A".to_string(),
            album_name: "Album A".to_string(),
            track_name: "Song A".to_string(),
            popularity: 85,
            danceability: 0.8,
            energy: 0.9,
            tempo: 120.0,
            valence: 0.7,
        };

        let track2 = Track {
            track_id: "2".to_string(),
            artists: "Artist B".to_string(),
            album_name: "Album B".to_string(),
            track_name: "Song B".to_string(),
            popularity: 95,
            danceability: 0.75,
            energy: 0.88,
            tempo: 122.0,
            valence: 0.68,
        };

        let tracks = vec![&track1, &track2];
        let selected = select_track(&tracks, Some(1));
        assert!(selected.is_some(), "No track was selected.");
        assert_eq!(
            selected.unwrap().track_name, "Song B",
            "Expected Song B to be selected."
        );
    }

    #[test]
    fn test_find_similar_songs() {
        let input_track = Track {
            track_id: "1".to_string(),
            artists: "Artist A".to_string(),
            album_name: "Album A".to_string(),
            track_name: "Song A".to_string(),
            popularity: 85,
            danceability: 0.8,
            energy: 0.9,
            tempo: 120.0,
            valence: 0.7,
        };

        let similar_track = Track {
            track_id: "2".to_string(),
            artists: "Artist B".to_string(),
            album_name: "Album B".to_string(),
            track_name: "Song B".to_string(),
            popularity: 75,
            danceability: 0.79,
            energy: 0.91,
            tempo: 121.0,
            valence: 0.72,
        };

        let dissimilar_track = Track {
            track_id: "3".to_string(),
            artists: "Artist C".to_string(),
            album_name: "Album C".to_string(),
            track_name: "Song C".to_string(),
            popularity: 65,
            danceability: 0.5,
            energy: 0.5,
            tempo: 90.0,
            valence: 0.3,
        };

        let tracks = vec![input_track.clone(), similar_track.clone(), dissimilar_track.clone()];
        let similar_songs = find_similar_songs(&tracks, &input_track, 0.05, 0.05, 5.0, 0.05, 70);
        assert_eq!(similar_songs.len(), 1, "Expected 1 similar song.");
        assert_eq!(
            similar_songs[0].track_name, "Song B",
            "Expected Song B to be the similar song."
        );
    }

    #[test]
    fn test_build_song_subgraph() {
        let input_track = Track {
            track_id: "1".to_string(),
            artists: "Artist A".to_string(),
            album_name: "Album A".to_string(),
            track_name: "Song A".to_string(),
            popularity: 85,
            danceability: 0.8,
            energy: 0.9,
            tempo: 120.0,
            valence: 0.7,
        };

        let similar_track = Track {
            track_id: "2".to_string(),
            artists: "Artist B".to_string(),
            album_name: "Album B".to_string(),
            track_name: "Song B".to_string(),
            popularity: 75,
            danceability: 0.79,
            energy: 0.91,
            tempo: 121.0,
            valence: 0.72,
        };

        let graph = build_song_subgraph(&input_track, &[&similar_track]);
        assert!(graph.contains_key("Song A"), "Graph should contain the input track.");
        let node = graph.get("Song A").unwrap();
        assert_eq!(node.1.len(), 1, "Expected 1 similar song in the graph.");
        assert_eq!(node.1[0].0, "Song B", "Expected Song B in the graph.");
    }

    #[test]
    fn test_export_subgraph_to_dot() {
        let input_track = Track {
            track_id: "1".to_string(),
            artists: "Artist A".to_string(),
            album_name: "Album A".to_string(),
            track_name: "Song A".to_string(),
            popularity: 85,
            danceability: 0.8,
            energy: 0.9,
            tempo: 120.0,
            valence: 0.7,
        };

        let similar_track = Track {
            track_id: "2".to_string(),
            artists: "Artist B".to_string(),
            album_name: "Album B".to_string(),
            track_name: "Song B".to_string(),
            popularity: 75,
            danceability: 0.79,
            energy: 0.91,
            tempo: 121.0,
            valence: 0.72,
        };

        let graph = build_song_subgraph(&input_track, &[&similar_track]);
        let dot_file_path = "test_output.dot";
        let result = export_subgraph_to_dot(&graph, dot_file_path);
        assert!(result.is_ok(), "Failed to export the graph to DOT file.");

        let content = std::fs::read_to_string(dot_file_path).unwrap();
        assert!(content.contains("graph {"), "DOT file should contain 'graph {{'");
        assert!(content.contains("\"Song A\""), "DOT file should contain 'Song A'");
        assert!(content.contains("\"Song B\""), "DOT file should contain 'Song B'");

        std::fs::remove_file(dot_file_path).unwrap();
    }
}