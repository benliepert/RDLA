use core::time;
use std::io::Write;
use rand::{Rng, thread_rng};
use thousands::Separable;
use crate::config::{DlaConfig, GridType};
use colored::Colorize;
use crate::grid::Grid;

use std::sync::{Arc, Mutex};
use std::thread;

use crate::colors::{Color, ColorName, Theme, get_gradients};
use crate::gui::{PAUSE_BUTTON_TEXT, UNPAUSE_BUTTON_TEXT};

// so the gui can stay in sync
pub const DEFAULT_PART_CLR: ColorName = ColorName::Seafoam;
pub const DEFAULT_BACK_CLR: ColorName = ColorName::Black;
pub const DEFAULT_THEME: Theme = Theme::Seafoam;

use log::debug;
#[derive(Clone, Debug)]
struct Particle {
    /// Does the particle exist?
    exists: bool,
    /// The current position of the particle (x, y)
    pos: (usize, usize),
}

#[derive(Clone, Debug)]
pub(crate) struct DlaGrid {
    pub grid: Grid, // separated out so we can serialize it independently
    // We need to track the currently moving particle in the grid
    /// Each update will either move this particle, or spawn a new one (if the last move stuck it)
    cur_part: Particle,
    /// The number of particles that have stuck in our simulation so far
    stuck_particles: usize,
    /// Track whether the simulation has completed.
    /// Why not just compare max particles with how many are stuck? because I want to aggregate all the logic about whether
    /// it's done or not in the class, so you don't have to worry about max > get_stuck() or max >= get_stuck(), where
    /// one will hang since there's protection in update against hogging the cpu indefinitely when max particles
    /// is reached
    pub is_complete: bool,
    /// The number of particles the simulation will use
    particles: usize,
    /// Track the total number of particle moves the simulation did. Useful for benchmarking
    updates: u64,
    /// Particles will be this color when displayed
    fill_color: Color,
    /// Color for unfilled cells when displayed
    empty_color: Color,
    /// Is the simulation paused? This is separate from is_complete. Both won't be true at the same time
    pub paused: bool,
    /// Style of the initial starting grid
    grid_type: GridType,
    /// Minimum particle spawn distance from center of grid.
    /// This is an Option because it makes the backend more efficient. Even though disabling the radius is equivalent to
    /// using a radius of 0, there's an additional distance calculation that we can skip by just checking if
    /// this variable is Some() or not.
    spawn_radius: Option<usize>,
    /// When the grid size is changed in the gui, track that a resize is required so that the event loop can
    /// take care of it when it loops back around.
    pub do_resize: bool,

    theme: Option<Theme>,
}

impl Default for DlaGrid {
    fn default() -> Self {
        let (width, height) = (400, 400);
        let grid_type = GridType::Center;
        let grid = Grid::from(grid_type, width, height);
        let stuck_particles = grid.stuck_particles();
        Self {
            grid: grid, 
            cur_part: Particle { exists: false, pos: (0, 0)},
            stuck_particles: stuck_particles,
            is_complete: false,
            particles: 10000,
            updates: 0,
            fill_color: DEFAULT_PART_CLR.get_color(),
            empty_color: DEFAULT_BACK_CLR.get_color(),
            paused: true,
            grid_type: grid_type,
            spawn_radius: None,
            do_resize: false,
            theme: Some(DEFAULT_THEME),
        }
    }
}

impl DlaGrid {
    pub(crate) fn from(config: &DlaConfig) -> Self {
        let width: u32 = config.width; // has a default

        // height is optional. defaults to whatever width is
        let height = if let Some(height) = config.height {
            height
        } else {
            width
        };

        let color: Color = DEFAULT_PART_CLR.get_color();
        let background_color: Color = DEFAULT_BACK_CLR.get_color();

        let grid_type = config.grid_type(); // this will give us a default if user didn't specify

        let grid: Grid = Grid::from(grid_type, width, height);

        // number of stuck particles depends on grid type
        // doing this work up front is slower, but it's a 1 time cost that means we don't need to maintain #particles
        // stuck for each grid type separately
        let stuck_particles = grid.stuck_particles();

        Self {
            grid: grid,
            cur_part: Particle { exists: false, pos: (0, 0)},
            stuck_particles: stuck_particles,
            is_complete: false,
            particles: config.particles,
            updates: 0,
            fill_color: color,
            empty_color: background_color,
            paused: true,
            grid_type: grid_type,
            spawn_radius: None,
            do_resize: false,
            theme: Some(DEFAULT_THEME),
        }
    }

    /// Run the simulation until all particles have stuck
    pub fn run(&mut self) {
        while !self.is_complete{
            self.update();
        }
    }

    fn swap_grid_type(&mut self, new_grid_type: GridType, size: Option<(u32, u32)>) {
        // swapping grid type defaults to use the current width
        let (width, height) = size.unwrap_or((self.grid.width as u32, self.grid.height as u32));
        let new_grid: Grid = Grid::from(new_grid_type, width, height);

        self.grid = new_grid;
        self.cur_part = Particle { exists: false, pos: (0, 0)}; // reset
        self.stuck_particles = self.grid.stuck_particles();
        self.is_complete = false; // reset
        // particles same
        self.updates = 0; // reset
        // fill color same
        // empty color same
        // to_file same
        self.paused = true; // make them restart with the new grid
        self.grid_type = new_grid_type;
    }

    fn swap_particle_color(&mut self, new_color: ColorName) {
        self.fill_color = new_color.get_color();
    }

    fn swap_background_color(&mut self, new_color: ColorName) {
        self.empty_color = new_color.get_color();
    }

    /// Returns a boolean representing whether a particle at (x, y) should 'stick'
    /// 
    /// In DLA, a particle sticks if one of its neighbors is also a particle. Particles
    /// stick to one another. Return true if one of the neighbors of (x, y) is FILLED,
    /// and false otherwise.
    fn should_stick(&self, x: usize, y: usize) -> bool {
        // use isize here so that we can go off grid. the if statement in the update loop makes sure we only look at points
        // that fit in the grid
        let neighbors = self.get_neighbors(x, y);

        for (nx, ny) in neighbors.iter() {
            // make sure the neighbor is a valid point on our grid
            if self.valid_grid_pos((*nx, *ny)) && self.grid.filled(self.get_idx(*nx as usize, *ny as usize)) {
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    fn valid_grid_idx(&self, idx: usize) -> bool {
        idx < self.grid.width * self.grid.height
    }

    fn valid_grid_pos(&self, pos: (isize, isize)) -> bool {
        pos.0 < self.grid.width as isize 
        && pos.0 >= 0
        && pos.1 < self.grid.height as isize
        && pos.1 >= 0
    }

    //// Return the filled value of the cell at position (x, y). Returns false if 'pos' isn't a valid grid pos
    #[allow(dead_code)]
    fn get_filled(&self, pos: (isize, isize)) -> bool {
        let n0_idx = self.get_idx(pos.0 as usize, pos.1 as usize);
        if n0_idx < self.grid.width * self.grid.height {
            self.grid.filled(n0_idx)
        } else {
            false // position is out of bounds, effectively unfilled (this makes algorithm simpler)
        }
    }

    /// Get neighbors. They aren't necessarily all valid grid positions 
    fn get_neighbors(&self, x: usize, y:usize) -> [(isize, isize); 8] {
        let tx: isize = x.try_into().unwrap();
        let ty: isize = y.try_into().unwrap();

        [(tx-1, ty-1), (tx, ty-1), (tx+1, ty-1),
         (tx-1, ty),               (tx+1, ty),
         (tx-1, ty+1), (tx, ty+1), (tx+1, ty+1) 
        ]
    }

    /// Given a particle at (x, y), return its new position after it moves randomly to one of its
    /// (unfilled) neighbors
    /// 
    /// This function will always return a valid position in the grid
    fn random_walk(&mut self, x: usize, y: usize) -> (usize, usize) {
        // build a vector of possible neighbors
        // randomly pick from the vector accordingly
        let neighbors = self.get_neighbors(x, y);
        
        let mut valid_neighbors = Vec::new();
        for (nx, ny) in neighbors.iter() {
            if self.valid_grid_pos((*nx, *ny)) && !self.grid.filled(self.get_idx(*nx as usize, *ny as usize)){
                valid_neighbors.push((nx, ny));
            }
        }

        let num_neighbors = valid_neighbors.len();
        if num_neighbors == 0 {
            panic!("No neighbors found for random walk! This is a bug");
        }

        let neighbor_idx = thread_rng().gen_range(0..num_neighbors);

        (*valid_neighbors[neighbor_idx].0 as usize, *valid_neighbors[neighbor_idx].1 as usize)
    }

    fn flush_stdout() {
        let mut stdout = std::io::stdout();
        stdout.flush().unwrap();
    }

    /// Iterates once on the current grid
    /// 
    /// If there's an active particle, move it one step in its random walk
    /// If there's no active particle (the last one stuck), spawn one at a random (unpopulated) location
    /// Check if the particle we added stuck
    /// Mark the simulation complete if we've reached the desired number of particles
    pub fn update(&mut self) {
        if self.is_complete { // don't let this run if the sim is complete
            return;
        }

        self.updates += 1;
        // This if/else block MUST result in an updated location for cur_part
        if self.cur_part.exists {
            let (oldx, oldy) = self.cur_part.pos;
            let (newx, newy) = self.random_walk(oldx, oldy);

            // the particle is no longer at the old location
            let old_idx: usize = self.get_idx(oldx, oldy);
            self.grid.set_fill(old_idx, false);

            // update cur particle
            self.cur_part.pos = (newx, newy);
        } else {
            // spawn a new particle at a random location
            let (startx, starty) = self.random_loc();
            self.cur_part = Particle { exists: true, pos: (startx, starty)};
        }

        // we either moved, or spawned. In both cases we need to update our state if the particle should stick.
        if self.should_stick(self.cur_part.pos.0, self.cur_part.pos.1) {
            self.cur_part.exists = false;
            self.stuck_particles += 1;
            // get the idx of the current partcle, and mark it as full in the vector
            let idx = self.get_idx(self.cur_part.pos.0, self.cur_part.pos.1);
            self.grid.cells[idx].id = self.stuck_particles + 1;
        }

        // we either moved a particle or spawned a particle. The current particle's location needs to be filled to prep
        // for the next iteration
        let idx = self.get_idx(self.cur_part.pos.0, self.cur_part.pos.1);
        self.grid.set_fill(idx, true);

        if self.stuck_particles >= self.particles {
            self.is_complete = true; // flag us as done so somebody running the simulation knows :D
        }
    }

    pub fn draw(&mut self, screen: &mut [u8]) {
        // both draw functions share a for-loop, but there's enough extra stuff for time-coloring I opted to separate them
        if let Some(theme) = self.theme {
            self.draw_theme(screen, theme);
        } else {
            self.draw_normal(screen);
        }
    }

    fn draw_theme(&mut self, screen: &mut [u8], theme: Theme) {
        let num_colors = 10; // should match the number of gradients we get below
        let bucket_size = self.stuck_particles / num_colors;
        let theme_colors: [Color; 10] = get_gradients(theme);
        for (c, pix) in self.grid.cells.iter_mut().zip(screen.chunks_exact_mut(4)) {
            let color = if c.filled {
                let id = c.id;
                let idx = if bucket_size == 0 {
                    0 // avoid divide by 0 if we don't have any stuck particles yet
                } else {
                    let tmp = (id / bucket_size) as usize;
                    // dividing by 10 to get bucket size is imprecise. It won't divide perfectly, so some particles will
                    // be outside the range. We put them in the last bucket. For ex. Particle 100/100 will map to index 10
                    // when it should be 9.
                    if tmp >= num_colors {
                        num_colors - 1
                    } else {
                        tmp
                    }
                };
                theme_colors[idx]
            } else {
                self.empty_color
            };
            pix.copy_from_slice(&color);
        }
    }

    fn draw_normal(&mut self, screen: &mut [u8]) {
        for (c, pix) in self.grid.cells.iter_mut().zip(screen.chunks_exact_mut(4)) {
            pix.copy_from_slice(if c.filled {
                &self.fill_color
            } else {
                &self.empty_color
            });
        }
    }
    /// Returns a valid (empty) spawn location for a new particle.
    /// 
    /// The location can be directly adjacent to another particle, meaning a particle spawning at this
    /// location will stick instantly.
    /// 
    /// If it can't find a valid spawn location and reaches the retry limit (1000), will mark the simulation as complete
    /// and return a point that was already filled.
    fn random_loc(&mut self) -> (usize, usize) {
        const MAX_RETRIES: usize = 1_000;

        let mut attempt: usize = 0;
        // loop until we find a position that isn't full, or we hit the retry limit
        loop {
            let randx = thread_rng().gen_range(0..self.grid.width);
            let randy = thread_rng().gen_range(0..self.grid.height);

            // update idx so the loop works
            let idx = self.get_idx(randx, randy);

            let outside_radius = if let Some(radius) = self.spawn_radius {
                // need to make sure the distance from randx,randy to the center is > radius
                let dist = self.grid.dist_to_center(randx, randy);
                dist > radius // valid points are those outside the radius
            } else {
                true // if spawn radius isn't set, every point is valid
            };

            if !self.grid.filled(idx) && outside_radius {
                // we found a valid position to move to, carry on
                return (randx, randy);
            }

            attempt += 1;
            if attempt >= MAX_RETRIES {
                println!("Couldn't generate a random location in {} tries! The grid must be nearly full - marking simulation as complete", MAX_RETRIES);
                self.is_complete = true;
                return (randx, randy); // this point is already filled in, or didn't meet the radius criteria
            }
        }
    }

    fn get_idx(&self, x: usize, y: usize) -> usize {
        x + y * self.grid.width
    }

    /// Return avg updates/sec
    #[allow(dead_code)]
    pub fn benchmark() {
        println!("{}",
                 format!("Running benchmark...").bold().yellow()
        );
        let iterations = 10;
        let mut updates_vec = Vec::new();
        for i in 0..iterations {
            let mut sim = DlaGrid::from(&Default::default());
            let now = std::time::Instant::now();
            sim.run();
            let elapsed = now.elapsed().as_secs();
            updates_vec.push((sim.updates / elapsed) as u32);

            print!("\r{}{}{}{}",
                   format!("Finished iteration: ").bold().blue(),
                   format!("{}", i+1).green(),
                   format!(" of ").bold().blue(),
                   format!("{}", iterations).green()
            );
            Self::flush_stdout();
        }
        assert!(updates_vec.len() > 0);
        let avg = updates_vec.iter().sum::<u32>() / updates_vec.len() as u32;
        let avg_str = avg.separate_with_commas();
        println!("\n{}{}",
                 format!("Average updates/sec was: ").bold().blue(),
                 format!("{}", avg_str).green()
        );
    }

    pub fn spawn_worker_thread(shared_data: &Arc<Mutex<DlaGrid>>) {
        debug!("worker thread spawning");
        let run_thread_grid = Arc::clone(&shared_data);
        let _handle = thread::spawn(move || {
            //let guard_data = run_thread_grid.lock().unwrap();
            //std::mem::drop(guard_data);

            loop {
                let mut guard_data = run_thread_grid.lock().unwrap();
                if !guard_data.paused && !guard_data.is_complete{
                    // getting about 60-80fps with this method.
                    // should be slightly more efficient in the backend since we're not using the lock as much
                    for _ in 0..10 {
                        guard_data.update();
                    }
                    std::mem::drop(guard_data);
                } else {
                    // sleep a bit so that we don't hog cpu when paused/complete
                    // This means that starting the simulation after it's paused/complete
                    // will lag by up to 100ms
                    std::mem::drop(guard_data);
                    thread::sleep(time::Duration::from_millis(100));
                }
                // implicitly dropping here doesn't seem to be sufficient...
            }
        });
    }

    // ----- FRAMEWORK (egui) HELPER FUNCTIONS -----
    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn stuck_particles(&self) -> usize {
        self.stuck_particles
    }

    pub fn complete(&self) -> bool {
        self.is_complete
    }

    pub fn grid_type(&self) -> GridType {
        self.grid_type
    }

    pub fn size(&self) -> (usize, usize) {
        (self.grid.width, self.grid.height)
    }

    // ----- FRAMEWORK (egui) HANDLER FUNCTIONS -----
    pub fn handle_particle_color_changed(&mut self, new_color: ColorName) {
        self.swap_particle_color(new_color);
    }

    pub fn handle_theme_changed(&mut self, new_theme: Option<Theme>) {
        self.theme = new_theme;
    }

    pub fn handle_background_color_changed(&mut self, new_color: ColorName) {
        self.swap_background_color(new_color);
    }

    pub fn handle_particles_changed(&mut self, particles: usize) {
            // check some basic assumptions about when this could be called
            assert!(self.paused || self.is_complete);

            // based on dynamic clamping in the gui, get_particles() should always give us a number in
            // [stuck_particles, width*height]
            if self.particles != particles {
                // only update self if something changed

                if self.is_complete && self.particles < particles {
                    // if we were complete, but particles increased, mark as not complete but paused.
                    // this happens if the user changed the desired number of particles
                    self.is_complete = false;
                    self.paused = true;
                } else if self.stuck_particles == particles {
                    // this happens if the user changed (decreased) the desired number of particles 
                    // to match however many are already spawned
                    self.is_complete = true;
                }
                // always update particles to reflect what the gui has told us to use
                self.particles = particles;
            }
    }

    pub fn handle_grid_type_selected(&mut self, new_grid_type: GridType) {
        self.swap_grid_type(new_grid_type, None);
    }

    pub fn handle_pause_button_clicked(&mut self, text: &str) {
        let text_should_be = if !self.paused {
            PAUSE_BUTTON_TEXT
        } else {
            UNPAUSE_BUTTON_TEXT
        };
        if text != text_should_be { // text should be the opposite of our current state
            panic!("Application state is out of sync with GUI! (start/stop button)");
        }
        self.paused = !self.paused;
    }

    pub fn handle_save_button_clicked(&mut self, save_file: &str) {
        // overwrite handling
        let save_file_gz = save_file.to_string() + ".gz";
        if std::path::Path::new(save_file_gz.as_str()).exists() {
            println!("Save Button: File already exists! Ignoring to avoid overwrite.");
            return;
        }
        self.grid.to_file(save_file);
    }

    pub fn handle_from_button_clicked(&mut self, from_file: &str) {
        // loading a file may require a window resize, if the new grid is bigger than the current one
        let old_size = self.grid.width;
        let maybe_grid = Grid::from_file(from_file);
        if let Some(grid) = maybe_grid {
            self.grid = grid;
        } else {
            return; // couldn't read grid in from file. whatever
        }

        // count the stuck particles in the grid we read in
        self.stuck_particles = self.grid.cells.iter()
                                              .filter(|&n| n.filled)
                                              .count();

        // particles: same as stuck particles since we're marking as complete
        self.particles = self.stuck_particles;
        self.is_complete = true;

        if old_size != self.grid.width {
            // we need to resize everything
            self.do_resize = true;
        }

        // we don't need to tell egui to update how many particles it has. The gui will be redrawn in 1 frame at which
        // time it will get the updated value from the backend
    }

    pub fn handle_spawn_radius_changed(&mut self, new_radius: Option<usize>) {
        self.spawn_radius = new_radius;
    }

    /// A reset is just a grid type swap with a grid of the same type
    pub fn handle_reset(&mut self, width: u32, height: u32) {
        let height_option = if width != self.grid.width as u32 
                            || height != self.grid.height as u32 {
                // size changed 
                self.do_resize = true; // the event loop will handle this when it loops back around
                Some((width, height))
            } else {
                // no size change
                None
            };
        self.swap_grid_type(self.grid_type, height_option);
    }
}
