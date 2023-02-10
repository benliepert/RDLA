use serde::{Serialize, Deserialize};
use std::{fs, path::Path, process::Command};
use rand::{Rng, thread_rng};

use crate::config::GridType;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Particle {
    pub filled: bool,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Grid {
    /// 1D Vector representing a 2D grid. It's more efficient this way since vectors are allocated on the heap,
    /// we don't want each row at a different memory location. And the math to go from index <=> (x, y) coord
    /// is simple. Each location is either filled (true) or empty (false)
    pub cells: Vec<Particle>,
    pub width: usize,
    pub height: usize,
}

impl Grid {

    // interface to get if a cell is filled. This allows us to change the underlying representation without changing
    // as much of the backend 
    pub fn filled(&self, idx: usize) -> bool {
        self.cells[idx].filled
    }

    pub fn set_fill(&mut self, idx: usize, filled: bool) {
        self.cells[idx].filled = filled;
    }

    /// Serialize the grid to a file
    pub fn to_file(&self, file_name: &str) {
        println!("Writing grid out to file: {}", file_name);
        // serialize 
        let serialized = bincode::serialize(&self).unwrap();

        // this is where we'll put it
        let file_path: &Path = Path::new(file_name);

        // write it out!
        let write_result = fs::write(file_path, serialized);

        if write_result.is_err() {
            println!("Error encountered writing grid to file: {:?}", write_result);
        }

        // zip it 
        let _output = Self::compress(file_name);
    }

    /// Read in a grid from this file
    pub fn from_file(mut file_name: &str) -> Option<Self> {
        println!("Reading grid in from file: {}", file_name);
        // catch stupid mistakes for now
        let mut recompress: bool = false;
        if file_name.ends_with(".gz") {
            let output = Self::decompress(file_name);

            if output.stderr.len() != 0 {
                println!("Error encountered decompressing file {}: {:?}", file_name, String::from_utf8(output.stderr).unwrap());
                return None;
            }

            file_name = file_name.strip_suffix(".gz").unwrap();
            recompress = true;
        } 

        let file_path = Path::new(&file_name);

        // pull the blob in from file
        let read_result = fs::read(file_path);
        if read_result.is_err() {
            println!("Error encountered reading from filesystem: {:?}", read_result);
            return None;
        }
        let serialized = read_result.unwrap();

        if recompress {
            Command::new("rm")
                    .arg(file_name)
                    .output()
                    .expect("Failed to remove uncompressed input file");
        }

        // deserialize it and return it!
        Some(bincode::deserialize(&serialized).unwrap())
    }

    #[allow(dead_code)]
    fn default() -> Self {
        Grid {cells: Vec::new(), width: 400, height: 400}
    }

    fn compress(file_name: &str) -> std::process::Output {
        Command::new("gzip")
                .arg(file_name)
                .arg("--best") // best compression
                .output()
                .expect("Failed to compress output file")
    }

    fn decompress(file_name: &str) -> std::process::Output {
        Command::new("gzip")
                .arg(file_name)
                .arg("--decompress")
                .arg("--keep") // don't overwrite input file
                .output()
                .expect("Failed to decompress the input file")
    }

    pub fn from(grid_type: GridType, width: u32, height: u32) -> Self {
        let width = width as usize;
        let height = height as usize;
        let cells = match grid_type {
            GridType::BottomEdge => Self::cells_bottom_edge(width, height),
            GridType::AllEdges   => Self::cells_all_edges(width, height),
            GridType::FourDots   => Self::cells_four_dots(width, height),
            GridType::RandFive   => Self::cells_random5(width, height),
            GridType::Circle     => Self::cells_circle(width, height, std::cmp::min(width, height) / 10),
            // default to center
            _ => Self::cells_center(width, height),
        };

        // cells is just a vector of bools. we need a new vector with all of its contents, except stored inside of 
        // Particles
        let mut cells_particle: Vec<Particle> = Vec::new();
        for val in cells.iter() {
            let p= Particle { filled: *val, id: 0 };
            cells_particle.push(p);
        }

        Grid { cells: cells_particle, width: width, height: height }
    }

    fn cells_empty(width: usize, height: usize) -> Vec<bool> {
        assert!(width != 0 && height != 0);
        let size: usize = width.checked_mul(height).expect("Grid size should fit in a usize!");
        // initialize the grid with the middle point occupied
        vec![false; size]
    }

    fn cells_center(width: usize, height: usize) -> Vec<bool> {
        let mut new = Self::cells_empty(width, height);
        let mid_idx = width * height / 2 + (width / 2) as usize;
        new[mid_idx] = true;
        new
    }

    /// Generates a grid where the bottom edge is filled
    fn cells_bottom_edge(width: usize, height: usize) -> Vec<bool> {
        let mut new = Self::cells_empty(width, height);
        for x in 0..width {
            let y = height - 1;
            let idx = Self::get_idx(width, x, y);
            new[idx] = true;
        }
        new
    }

    fn cells_all_edges(width: usize, height: usize) -> Vec<bool> {
        let mut new = Self::cells_bottom_edge(width, height);
        // fill top edge
        for x in 0..width {
            let y = 0;
            let idx = Self::get_idx(width, x, y);
            new[idx] = true;
        }
        // left side
        for y in 0..height {
            let x = 0;
            let idx = Self::get_idx(width, x, y);
            new[idx] = true;
        }
        // right side
        for y in 0..height {
            let x = width - 1;
            let idx = Self::get_idx(width, x, y);
            new[idx] = true;
        }
        new
    }

    fn cells_four_dots(width: usize, height: usize) -> Vec<bool> {
        let mut new = Self::cells_empty(width, height);

        let tl_y = height / 3;
        let tr_y = tl_y;

        let bl_y = tl_y * 2;
        let br_y = bl_y;

        let tl_x = width / 3;
        let bl_x = tl_x;

        let tr_x = tl_x * 2;
        let br_x = tr_x;

        let tl_idx = Self::get_idx(width, tl_x, tl_y);
        let tr_idx = Self::get_idx(width, tr_x, tr_y);
        let bl_idx = Self::get_idx(width, bl_x, bl_y);
        let br_idx = Self::get_idx(width, br_x, br_y);

        new[tl_idx] = true;
        new[tr_idx] = true;
        new[bl_idx] = true;
        new[br_idx] = true;

        new
    }
    
    fn cells_random5(width: usize, height: usize) -> Vec<bool> {
        let mut new = Self::cells_empty(width, height);

        for _ in 0..5 {
            let randx = thread_rng().gen_range(0..width);
            let randy = thread_rng().gen_range(0..height);

            let idx = Self::get_idx(width, randx, randy);
            new[idx] = true;
        }

        new
    }

    fn distance(x: usize, y: usize, x2: usize, y2: usize) -> usize {
        let ix = x as isize;
        let iy = y as isize;
        let ix2 = x2 as isize;
        let iy2 = y2 as isize;
        (((ix-ix2).pow(2) + (iy-iy2).pow(2)) as f32).sqrt() as usize
    }

    pub fn dist_to_center(&self, x: usize, y: usize) -> usize {
        Self::distance(x,y, self.width / 2, self.height / 2)
    }

    fn cells_circle(width: usize, height: usize, radius: usize) -> Vec<bool> {
        let mut new = Self::cells_empty(width, height);
        let midx = width / 2 as usize;
        let midy = height / 2 as usize;

        assert!(radius < width / 2 && radius < height / 2);

        for x in 0..width {
            for y in 0..height {
                if Self::distance(x,y,midx,midy) == radius {
                    new[Self::get_idx(width, x, y)] = true;
                } 
            }
        }
        new
    }

    fn get_idx(width: usize, x: usize, y: usize) -> usize {
        x + y * width
    }

    #[allow(dead_code)]
    fn idx(&self, x: usize, y: usize) -> usize {
        Self::get_idx(self.width, x, y)
    }
    
    pub fn stuck_particles(&self) -> usize {
        // count the cells that are 'true' (ie filled)
        self.cells.iter().filter(|&n| n.filled).count()
    }
}