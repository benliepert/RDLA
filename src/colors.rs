//! RGBA Color constants
pub type Color = [u8; 4];

// normal particle colors
const SEAFOAM     : Color = [0x27, 0xf5, 0x9f, 0xc8];
const WHITE       : Color = [0xff, 0xff, 0xff, 0xff];
const BLACK       : Color = [0,0,0,255];
const LIGHT_BLUE  : Color = [100, 221, 219, 0xff];
const PURPLE      : Color = [100, 106, 221, 0xff];
const PINK        : Color = [219, 100, 221, 0xff];
const CORAL       : Color = [221, 100, 100, 0xff];
const ORANGE      : Color = [240, 134, 81, 0xff];
const YELLOW      : Color = [240, 229, 81, 0xff];
const GREEN       : Color = [124, 240, 81, 0xff];
const RED         : Color = [217, 23, 23, 0xff];

// sefoam
const GRAD1_1 : Color = [67,193,151,0xff];
const GRAD1_2 : Color = [63,174,144,0xff];
const GRAD1_3 : Color = [58,155,136,0xff];
const GRAD1_4 : Color = [54,136,129,0xff];
const GRAD1_5 : Color = [50,117,121,0xff];
const GRAD1_6 : Color = [45,97,114,0xff];
const GRAD1_7 : Color = [41,78,106,0xff];
const GRAD1_8 : Color = [37,59,99,0xff];
const GRAD1_9 : Color = [32,40,91,0xff];
const GRAD1_10 : Color = [28,21,84,0xff];

// lemon
const GRAD2_1 : Color = [244,233,0,0xff];
const GRAD2_2 : Color = [228,227,27,0xff];
const GRAD2_3 : Color = [211,222,54,0xff];
const GRAD2_4 : Color = [195,216,80,0xff];
const GRAD2_5 : Color = [178,210,107,0xff];
const GRAD2_6 : Color = [162,205,134,0xff];
const GRAD2_7 : Color = [145,199,161,0xff];
const GRAD2_8 : Color = [129,193,187,0xff];
const GRAD2_9 : Color = [112,188,214,0xff];
const GRAD2_10 : Color = [96,182,241,0xff];

// forest
const GRAD3_1 : Color = [33,86,0,0xff];
const GRAD3_2 : Color = [53,101,23,0xff];
const GRAD3_3 : Color = [74,115,46,0xff];
const GRAD3_4 : Color = [94,130,69,0xff];
const GRAD3_5 : Color = [115,145,92,0xff];
const GRAD3_6 : Color = [135,159,114,0xff];
const GRAD3_7 : Color = [156,174,137,0xff];
const GRAD3_8 : Color = [176,189,160,0xff];
const GRAD3_9 : Color = [197,203,183,0xff];
const GRAD3_10 : Color = [217,218,206,0xff];

// candy
const GRAD4_1 : Color = [93,224,240,0xff];
const GRAD4_2 : Color = [110,218,241,0xff];
const GRAD4_3 : Color = [127,211,241,0xff];
const GRAD4_4 : Color = [144,205,242,0xff];
const GRAD4_5 : Color = [161,198,242,0xff];
const GRAD4_6 : Color = [179,192,243,0xff];
const GRAD4_7 : Color = [196,185,243,0xff];
const GRAD4_8 : Color = [213,179,244,0xff];
const GRAD4_9 : Color = [230,172,244,0xff];
const GRAD4_10 : Color = [247,166,245,0xff];

// christmas
const GRAD5_1 : Color = [187,9,9,0xff];
const GRAD5_2 : Color = [166,22,8,0xff];
const GRAD5_3 : Color = [145,35,7,0xff];
const GRAD5_4 : Color = [125,48,6,0xff];
const GRAD5_5 : Color = [104,61,5,0xff];
const GRAD5_6 : Color = [83,73,4,0xff];
const GRAD5_7 : Color = [62,86,3,0xff];
const GRAD5_8 : Color = [42,99,2,0xff];
const GRAD5_9 : Color = [21,112,1,0xff];
const GRAD5_10 : Color = [0,125,0,0xff];

// creamsicle
const GRAD6_1 : Color = [244,113,31,0xff];
const GRAD6_2 : Color = [228,116,44,0xff];
const GRAD6_3 : Color = [212,119,57,0xff];
const GRAD6_4 : Color = [196,122,70,0xff];
const GRAD6_5 : Color = [180,125,23,0xff];
const GRAD6_6 : Color = [163,127,97,0xff];
const GRAD6_7 : Color = [147,130,110,0xff];
const GRAD6_8 : Color = [131,133,123,0xff];
const GRAD6_9 : Color = [115,136,136,0xff];
const GRAD6_10 : Color = [99,139,149,0xff];

// vibrant
const GRAD7_1 : Color = [84,71,140,0xff];
const GRAD7_2 : Color = [44,105,154,0xff];
const GRAD7_3 : Color = [4,139,168,0xff];
const GRAD7_4 : Color = [13,179,158,0xff];
const GRAD7_5 : Color = [22,219,147,0xff];
const GRAD7_6 : Color = [131,227,119,0xff];
const GRAD7_7 : Color = [185,231,105,0xff];
const GRAD7_8 : Color = [239,234,90,0xff];
const GRAD7_9 : Color = [241,196,83,0xff];
const GRAD7_10 : Color = [242,158,76,0xff];

// standard colors go in here
pub const COLORS: [Color; 11] = [RED, LIGHT_BLUE, PURPLE, PINK, CORAL, ORANGE, YELLOW, GREEN, SEAFOAM, WHITE, BLACK];

const SEAFOAM_GRADIENT: [Color; 10] = [GRAD1_1, GRAD1_2, GRAD1_3, GRAD1_4, GRAD1_5, GRAD1_6, GRAD1_7, GRAD1_8, GRAD1_9, GRAD1_10];
const LEMON_GRADIENT: [Color; 10] = [GRAD2_1, GRAD2_2, GRAD2_3, GRAD2_4, GRAD2_5, GRAD2_6, GRAD2_7, GRAD2_8, GRAD2_9, GRAD2_10];
const FOREST_GRADIENT: [Color; 10] = [GRAD3_1, GRAD3_2, GRAD3_3, GRAD3_4, GRAD3_5, GRAD3_6, GRAD3_7, GRAD3_8, GRAD3_9, GRAD3_10];
const CANDY_GRADIENT: [Color; 10] = [GRAD4_1, GRAD4_2, GRAD4_3, GRAD4_4, GRAD4_5, GRAD4_6, GRAD4_7, GRAD4_8, GRAD4_9, GRAD4_10];
const CHRISTMAS_GRADIENT: [Color; 10] = [GRAD5_1, GRAD5_2, GRAD5_3, GRAD5_4, GRAD5_5, GRAD5_6, GRAD5_7, GRAD5_8, GRAD5_9, GRAD5_10];
const CREAMSICLE_GRADIENT: [Color; 10] = [GRAD6_1, GRAD6_2, GRAD6_3, GRAD6_4, GRAD6_5, GRAD6_6, GRAD6_7, GRAD6_8, GRAD6_9, GRAD6_10];
const VIBRANT_GRADIENT: [Color; 10] = [GRAD7_1, GRAD7_2, GRAD7_3, GRAD7_4, GRAD7_5, GRAD7_6, GRAD7_7, GRAD7_8, GRAD7_9, GRAD7_10];

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
            ColorName::Seafoam   => SEAFOAM,
            ColorName::White     => WHITE,
            ColorName::Black     => BLACK,
            ColorName::LightBlue => LIGHT_BLUE,
            ColorName::Purple    => PURPLE,
            ColorName::Pink      => PINK,
            ColorName::Coral     => CORAL,
            ColorName::Orange    => ORANGE,
            ColorName::Yellow    => YELLOW,
            ColorName::Green     => GREEN,
            ColorName::Red       => RED,
        }
    }
}

// need a way to convert from Color to ColorEnum
pub fn get_color_name(color: Color) -> ColorName {
    match color {
        SEAFOAM    => ColorName::Seafoam,
        WHITE      => ColorName::White,
        BLACK      => ColorName::Black,
        LIGHT_BLUE => ColorName::LightBlue,
        PURPLE     => ColorName::Purple,
        PINK       => ColorName::Pink,
        CORAL      => ColorName::Coral,
        ORANGE     => ColorName::Orange,
        YELLOW     => ColorName::Yellow,
        GREEN      => ColorName::Green,
        RED        => ColorName::Red,
        _ => {
            panic!("get_color_name(): could not convert color '{:?}' to a known enum!", color);
        }
    }
}

// so that the gui can print these. Also implicitly adds a to_string() method
impl std::fmt::Display for ColorName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorName::Seafoam   => write!(f, "Seafoam"),
            ColorName::White     => write!(f, "White"),
            ColorName::Black     => write!(f, "Black"),
            ColorName::LightBlue => write!(f, "Light Blue"),
            ColorName::Purple    => write!(f, "Purple"),
            ColorName::Pink      => write!(f, "Pink"),
            ColorName::Coral     => write!(f, "Coral"),
            ColorName::Orange    => write!(f, "Orange"),
            ColorName::Yellow    => write!(f, "Yellow"),
            ColorName::Green     => write!(f, "Green"),
            ColorName::Red       => write!(f, "Red"),
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
           Theme::Seafoam    => write!(f, "Seafoam"),
           Theme::Lemon      => write!(f, "Lemon"),
           Theme::Forest     => write!(f, "Forest"),
           Theme::Candy      => write!(f, "Candy"),
           Theme::Christmas  => write!(f, "Christmas"),
           Theme::Creamsicle => write!(f, "Creamsicle"),
           Theme::Vibrant    => write!(f, "Vibrant"),
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

