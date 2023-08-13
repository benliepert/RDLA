use clap::Parser;
use strum_macros::EnumIter;

pub const ABOUT: &str = r"
_____  __      __      ____  __  __  ___  ____ 
(  _ \(  )    /__\    (  _ \(  )(  )/ __)(_  _)
 )(_) ))(__  /(__)\    )   / )(__)( \__ \  )(  
(____/(____)(__)(__)  (_)\_)(______)(___/ (__) ";

#[derive(Parser, Clone)]
#[command(author, version, about = ABOUT, long_about = None)]
pub struct DlaConfig {
    /// The edge length of the square grid
    #[arg(short, long, default_value_t = 400)]
    pub width: u32,

    /// The height of the square grid. Optional
    #[arg(long)]
    pub height: Option<u32>,

    /// The number of particles for the simulation
    #[arg(short, long, default_value_t = 10_000)]
    pub particles: usize,

    /// Determines the intial particle configuration for the grid (case insensitive).
    /// Center (default): a single particle in the center.
    /// BottomEdge: the bottom edge is filled.  
    /// AllEdges: all edges are filled
    /// FourDots: a single particle in each of the 4 quadrants of the grid
    #[arg(short, long)]
    pub grid_type: Option<String>,

    /// Particles will be displayed with this color (case insensitive).
    /// Options are: red, orange, yellow, green, lightblue, purple, pink, coral, seafoam, white, black
    #[arg(short, long)]
    pub color: Option<String>,

    /// The background will be displayed with this color (case insensitive).
    /// Same options as the --color flag
    #[arg(long)]
    pub background_color: Option<String>,

    /// After simulation, write the grid to this file
    #[arg(short, long)]
    pub to_file: Option<String>,

    /// Read a grid in from this file and display it.
    /// Grid size is inferred from the file you read in, so no need to specify it
    #[arg(short, long)]
    pub from_file: Option<String>,

    /// Set the style with which particles will be colored when te simulation is displayed.
    /// Normal: use whatever --color specifies.
    /// Sparkle: pick a random color per particle per frame.
    #[arg(long)]
    pub color_style: Option<String>,

    /// When the grid is displayed, each particle's size will be shown as a factor of the pixel size
    #[arg(long, default_value_t = 1.0)]
    pub scale_factor: f64,

    /// Run a benchmark of the simulation to get an approximation of updates/sec. An update is a single move/spawn of a particle.
    /// All other arguments are ignored if this is set.
    #[arg(short, long, default_value_t = false)]
    pub benchmark: bool,

    /// How you'd like to view the simulation. Options are:
    /// Live (default): watch particles stick in real time
    /// End: Simulate in the background, then show the window at the end (slightly faster)
    /// Skip: Don't show the display at all
    #[arg(long, short)]
    pub view: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum GridType {
    /// A single particle in the center
    Center,
    /// The bottom edge is saturated with particles
    BottomEdge,
    /// All edges are saturated with particles
    AllEdges,

    FourDots,

    RandFive,

    Circle,
}

impl std::fmt::Display for GridType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GridType::Center => write!(f, "Center"),
            GridType::BottomEdge => write!(f, "Bottom Edge"),
            GridType::AllEdges => write!(f, "All Edges"),
            GridType::FourDots => write!(f, "Four Dots"),
            GridType::RandFive => write!(f, "Random Five"),
            GridType::Circle => write!(f, "Circle"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum View {
    /// View the simulation as it's occurring
    Live,
    /// View the simulation only once all particles have stuck
    End,
    /// Don't view the simulation
    Skip,
}

// different types of colors we can have
#[derive(Clone, Debug, PartialEq)]
pub enum ColorStyle {
    Normal,
    Sparkle,
    //Time, // color changes as time passes
}

impl Default for DlaConfig {
    fn default() -> Self {
        // This is the config used for benchmarking
        DlaConfig {
            width: 400,
            height: None,
            scale_factor: 1.0, // meta argument
            particles: 10_000,
            color_style: None,
            benchmark: false, // This is a meta-arg that's only relevant at the top level
            from_file: None,
            to_file: None,
            grid_type: None,
            color: None,
            background_color: None,
            view: None,
        }
    }
}

impl DlaConfig {
    pub fn view(&self) -> View {
        // if a from file is specified, use View::End so that DlaGrid::Show doesn't generate on top of the grid
        if self.from_file.is_some() {
            return View::End;
        }

        let default_view: View = View::Live;
        if let Some(view_type) = &self.view {
            match view_type.to_ascii_lowercase().as_str() {
                "end" => View::End,
                "skip" => View::Skip,
                _ => default_view,
            }
        } else {
            default_view
        }
    }

    // encapsulate grid behavior for grid type. If user doesn't specify a type, we default to "center"
    // yes, this is a workaround to me not knowing how to tell clap to use a default str.
    pub fn grid_type(&self) -> GridType {
        let default_grid_type: GridType = GridType::Center;
        if let Some(grid_type) = &self.grid_type {
            match grid_type.to_ascii_lowercase().as_str() {
                "bottomedge" => GridType::BottomEdge,
                "alledges" => GridType::AllEdges,
                "fourdots" => GridType::FourDots,
                _ => default_grid_type,
            }
        } else {
            default_grid_type
        }
    }

    pub fn color_style(&self) -> ColorStyle {
        const DEFAULT: ColorStyle = ColorStyle::Normal;
        if let Some(style) = &self.color_style {
            match style.to_ascii_lowercase().as_str() {
                "sparkle" => ColorStyle::Sparkle,
                _ => DEFAULT,
            }
        } else {
            DEFAULT
        }
    }
}
