use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_maze(filename: &str) -> Vec<Vec<char>> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    //lee archivo para formar el mapa
    reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect()
}