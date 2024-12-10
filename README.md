Project Write-Up
Daniel Cusack

For this project, I chose the Spotify Tracks Dataset off of Kaggle. This dataset included +100,000 songs, including over 20 columns including track_id, artists, track_name, popularity, danceability, energy, tempo, and valence. I decided to experiment with the similarity of songs on this huge dataset with these variables. 

How it works:
After running the code using “cargo run”, it shows that it loaded all of the tracks from the dataset (114000). After this, it prompts you to input the name of a song. If the song is not in the dataset or a song name was inputted incorrectly, an error message appears. After inputting a song name, if there are multiple songs with the same name as the one inputted, it displays the top 3 songs based on popularity to choose from with artists. After choosing a song, it outputs a list of the top 5 similar songs to the inputted song. This list of songs is determined by variables, including the song’s danceability, energy, tempo, and valence. These variables are chosen within a threshold of 0.05 for danceability and energy, 50.0 for tempo (BPM), 0.1 for valence, and 70 for popularity. The graph is then exported to a graph.dot file where you can see a representation of these songs in an organized fashion, showing the details of each song. The songs are displayed as nodes, and the connections between them are displayed as edges.
The code can be tested using "cargo test", testing all of the functions and making sure they do what they are suppose to do.

**Code explanation:**
I used different “use” statements in the beginning of my project for different purposes:

use serde::{Deserialize, Serialize};
This was used so the Track struct derives both Deserialize and Serialize traits to map out the rows of the CSV into data structures

use std::collections::{HashMap, HashSet};
HashMap was used to construct the graph with nodes and edges
HashSet was used to make sure there are unique song names when filtering for similarity

use std::error::Error;
Used for error propagation

use std::fs::File;
Used to read the dataset from the CSV file and write a graph to the DOT file

use std::io::{stdin, Write};
Stdin reads the user input
Write writes to files and the console

use csv::ReaderBuilder;
This allows the CSV reader to handle the columns and rows to the Track struct



I used tested different functions to very the correctness of the project:

**Load_tracks_from_csv**
This uses the csv crate to read the file and map each row in the CSV to the Track struct

**Select_track**
This sorts matching tracks by popularity, as well as displays the top 3 matches to the user and requires a selection and returns the track

**Find_similar_songs**
This iterates through the tracks of the dataset, compares the features of each track to the features of the inputted track, filters out duplicates, and returns a vector of the top matches

**Display_clean_subgraph**
This prints the inputted song’s details and iterates through the similar songs and prints their details

**Build_song_subgraph**
This adds the inputted song as a node with its features and adds edges connecting the inputted song to the similar songs

**Export_subgraph_to_dot**
This writes notes and edges to the graph.dot file and includes song details

**Main**
The main function is used to load the dataset, ask for a song name, find the top 5 similar songs, and export the graph to graph.dot


**Example output:**

CARGO RUN
Loaded 114000 tracks from the dataset.

Enter the name of a song:
Animal 

Multiple matches found. Please select one of the top 3 most popular songs:

1: "Animal" by Neon Trees (Album: Habits (Spotify), Popularity: 66)

2: "Animal" by Def Leppard (Album: Hysteria, Popularity: 64)

3: "Animal" by Vicetone;Jordan Powers;Bekah Novi (Album: Legacy, Popularity: 58)

Enter the number of the correct song: 1

Top 5 similar songs to "Animal" by Neon Trees [Danceability: 0.48, Energy: 0.83, Tempo: 147.99, Valence: 0.74, Popularity: 66]:

  -> "The Nights" by Avicii [Danceability: 0.53, Energy: 0.83, Tempo: 125.98, Valence: 0.65, Popularity: 86]
  
  -> "Prom Queen" by Beach Bunny [Danceability: 0.53, Energy: 0.80, Tempo: 143.79, Valence: 0.75, Popularity: 82]
  
  -> "Summer Of '69" by Bryan Adams [Danceability: 0.51, Energy: 0.83, Tempo: 139.13, Valence: 0.77, Popularity: 82]
  
  -> "Boys Don't Cry" by The Cure [Danceability: 0.46, Energy: 0.84, Tempo: 168.77, Valence: 0.67, Popularity: 77]
  
  -> "Self Esteem" by The Offspring [Danceability: 0.49, Energy: 0.86, Tempo: 104.56, Valence: 0.71, Popularity: 77]
  
Graph exported to 'graph.dot'.

**GRAPH**

Graph.dot:

graph {
    "Animal" [label="Danceability: 0.48, Energy: 0.83, Tempo: 147.99, Valence: 0.74, Popularity: 66"];
    "Animal" -> "The Nights" [label="Danceability: 0.53, Energy: 0.83, Popularity: 86"];
    "Animal" -> "Prom Queen" [label="Danceability: 0.53, Energy: 0.80, Popularity: 82"];
    "Animal" -> "Summer Of '69" [label="Danceability: 0.51, Energy: 0.83, Popularity: 82"];
    "Animal" -> "Boys Don't Cry" [label="Danceability: 0.46, Energy: 0.84, Popularity: 77"];
    "Animal" -> "Self Esteem" [label="Danceability: 0.49, Energy: 0.86, Popularity: 77"];
}

**CARGO TEST**

running 5 tests
test tests::tests::test_find_similar_songs ... ok
test tests::tests::test_build_song_subgraph ... ok
test tests::tests::test_select_track ... ok
test tests::tests::test_export_subgraph_to_dot ... ok
test tests::tests::test_load_tracks_from_csv ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s









