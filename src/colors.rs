//! RGBA Color constants
pub type Color = [u8; 4];

// normal particle colors
const SEAFOAM: Color = [0x27, 0xf5, 0x9f, 0xc8];
const WHITE: Color = [0xff, 0xff, 0xff, 0xff];
const BLACK: Color = [0, 0, 0, 255];
const LIGHT_BLUE: Color = [100, 221, 219, 0xff];
const PURPLE: Color = [100, 106, 221, 0xff];
const PINK: Color = [219, 100, 221, 0xff];
const CORAL: Color = [221, 100, 100, 0xff];
const ORANGE: Color = [240, 134, 81, 0xff];
const YELLOW: Color = [240, 229, 81, 0xff];
const GREEN: Color = [124, 240, 81, 0xff];
const RED: Color = [217, 23, 23, 0xff];

// standard colors go in here
pub const COLORS: [Color; 11] = [
    RED, LIGHT_BLUE, PURPLE, PINK, CORAL, ORANGE, YELLOW, GREEN, SEAFOAM, WHITE, BLACK,
];

const SEAFOAM_GRADIENT: [Color; 10] = [
    [67, 193, 151, 0xff],
    [63, 174, 144, 0xff],
    [58, 155, 136, 0xff],
    [54, 136, 129, 0xff],
    [50, 117, 121, 0xff],
    [45, 97, 114, 0xff],
    [41, 78, 106, 0xff],
    [37, 59, 99, 0xff],
    [32, 40, 91, 0xff],
    [28, 21, 84, 0xff],
];

const LEMON_GRADIENT: [Color; 10] = [
    [244, 233, 0, 0xff],
    [228, 227, 27, 0xff],
    [211, 222, 54, 0xff],
    [195, 216, 80, 0xff],
    [178, 210, 107, 0xff],
    [162, 205, 134, 0xff],
    [145, 199, 161, 0xff],
    [129, 193, 187, 0xff],
    [112, 188, 214, 0xff],
    [96, 182, 241, 0xff],
];

const FOREST_GRADIENT: [Color; 10] = [
    [33, 86, 0, 0xff],
    [53, 101, 23, 0xff],
    [74, 115, 46, 0xff],
    [94, 130, 69, 0xff],
    [115, 145, 92, 0xff],
    [135, 159, 114, 0xff],
    [156, 174, 137, 0xff],
    [176, 189, 160, 0xff],
    [197, 203, 183, 0xff],
    [217, 218, 206, 0xff],
];

const CANDY_GRADIENT: [Color; 10] = [
    [93, 224, 240, 0xff],
    [110, 218, 241, 0xff],
    [127, 211, 241, 0xff],
    [144, 205, 242, 0xff],
    [161, 198, 242, 0xff],
    [179, 192, 243, 0xff],
    [196, 185, 243, 0xff],
    [213, 179, 244, 0xff],
    [230, 172, 244, 0xff],
    [247, 166, 245, 0xff],
];

const CHRISTMAS_GRADIENT: [Color; 10] = [
    [187, 9, 9, 0xff],
    [166, 22, 8, 0xff],
    [145, 35, 7, 0xff],
    [125, 48, 6, 0xff],
    [104, 61, 5, 0xff],
    [83, 73, 4, 0xff],
    [62, 86, 3, 0xff],
    [42, 99, 2, 0xff],
    [21, 112, 1, 0xff],
    [0, 125, 0, 0xff],
];

const CREAMSICLE_GRADIENT: [Color; 10] = [
    [244, 113, 31, 0xff],
    [228, 116, 44, 0xff],
    [212, 119, 57, 0xff],
    [196, 122, 70, 0xff],
    [180, 125, 23, 0xff],
    [163, 127, 97, 0xff],
    [147, 130, 110, 0xff],
    [131, 133, 123, 0xff],
    [115, 136, 136, 0xff],
    [99, 139, 149, 0xff],
];

const VIBRANT_GRADIENT: [Color; 10] = [
    [84, 71, 140, 0xff],
    [44, 105, 154, 0xff],
    [4, 139, 168, 0xff],
    [13, 179, 158, 0xff],
    [22, 219, 147, 0xff],
    [131, 227, 119, 0xff],
    [185, 231, 105, 0xff],
    [239, 234, 90, 0xff],
    [241, 196, 83, 0xff],
    [242, 158, 76, 0xff],
];

use strum_macros::EnumIter; // allows the enum to be iterated over

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum ColorName {
    Seafoam,
    White,
    Black,
    LightBlue,
    Purple,
    Pink,
    Coral,
    Orange,
    Yellow,
    Green,
    Red,
}

impl ColorName {
    pub fn get_color(&self) -> Color {
        match self {
            ColorName::Seafoam => SEAFOAM,
            ColorName::White => WHITE,
            ColorName::Black => BLACK,
            ColorName::LightBlue => LIGHT_BLUE,
            ColorName::Purple => PURPLE,
            ColorName::Pink => PINK,
            ColorName::Coral => CORAL,
            ColorName::Orange => ORANGE,
            ColorName::Yellow => YELLOW,
            ColorName::Green => GREEN,
            ColorName::Red => RED,
        }
    }
}

// need a way to convert from Color to ColorEnum
pub fn get_color_name(color: Color) -> ColorName {
    match color {
        SEAFOAM => ColorName::Seafoam,
        WHITE => ColorName::White,
        BLACK => ColorName::Black,
        LIGHT_BLUE => ColorName::LightBlue,
        PURPLE => ColorName::Purple,
        PINK => ColorName::Pink,
        CORAL => ColorName::Coral,
        ORANGE => ColorName::Orange,
        YELLOW => ColorName::Yellow,
        GREEN => ColorName::Green,
        RED => ColorName::Red,
        _ => {
            panic!(
                "get_color_name(): could not convert color '{:?}' to a known enum!",
                color
            );
        }
    }
}

// so that the gui can print these. Also implicitly adds a to_string() method
impl std::fmt::Display for ColorName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorName::Seafoam => write!(f, "Seafoam"),
            ColorName::White => write!(f, "White"),
            ColorName::Black => write!(f, "Black"),
            ColorName::LightBlue => write!(f, "Light Blue"),
            ColorName::Purple => write!(f, "Purple"),
            ColorName::Pink => write!(f, "Pink"),
            ColorName::Coral => write!(f, "Coral"),
            ColorName::Orange => write!(f, "Orange"),
            ColorName::Yellow => write!(f, "Yellow"),
            ColorName::Green => write!(f, "Green"),
            ColorName::Red => write!(f, "Red"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum Theme {
    Seafoam,
    Lemon,
    Forest,
    Candy,
    Christmas,
    Creamsicle,
    Vibrant,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Theme::Seafoam => write!(f, "Seafoam"),
            Theme::Lemon => write!(f, "Lemon"),
            Theme::Forest => write!(f, "Forest"),
            Theme::Candy => write!(f, "Candy"),
            Theme::Christmas => write!(f, "Christmas"),
            Theme::Creamsicle => write!(f, "Creamsicle"),
            Theme::Vibrant => write!(f, "Vibrant"),
        }
    }
}

pub fn get_gradients(theme: Theme) -> [Color; 10] {
    match theme {
        Theme::Seafoam => SEAFOAM_GRADIENT,
        Theme::Lemon => LEMON_GRADIENT,
        Theme::Forest => FOREST_GRADIENT,
        Theme::Candy => CANDY_GRADIENT,
        Theme::Christmas => CHRISTMAS_GRADIENT,
        Theme::Creamsicle => CREAMSICLE_GRADIENT,
        Theme::Vibrant => VIBRANT_GRADIENT,
    }
}
