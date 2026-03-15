#![allow(dead_code)]

// Block type constants
pub const EMPTY: i32 = 0;
pub const GRASS: i32 = 1;
pub const SAND: i32 = 2;
pub const STONE: i32 = 3;
pub const BRICK: i32 = 4;
pub const WOOD: i32 = 5;
pub const CEMENT: i32 = 6;
pub const DIRT: i32 = 7;
pub const PLANK: i32 = 8;
pub const SNOW: i32 = 9;
pub const GLASS: i32 = 10;
pub const COBBLE: i32 = 11;
pub const LIGHT_STONE: i32 = 12;
pub const DARK_STONE: i32 = 13;
pub const CHEST: i32 = 14;
pub const LEAVES: i32 = 15;
pub const CLOUD: i32 = 16;
pub const TALL_GRASS: i32 = 17;
pub const YELLOW_FLOWER: i32 = 18;
pub const RED_FLOWER: i32 = 19;
pub const PURPLE_FLOWER: i32 = 20;
pub const SUN_FLOWER: i32 = 21;
pub const WHITE_FLOWER: i32 = 22;
pub const BLUE_FLOWER: i32 = 23;
pub const COLOR_00: i32 = 32;
pub const COLOR_01: i32 = 33;
pub const COLOR_02: i32 = 34;
pub const COLOR_03: i32 = 35;
pub const COLOR_04: i32 = 36;
pub const COLOR_05: i32 = 37;
pub const COLOR_06: i32 = 38;
pub const COLOR_07: i32 = 39;
pub const COLOR_08: i32 = 40;
pub const COLOR_09: i32 = 41;
pub const COLOR_10: i32 = 42;
pub const COLOR_11: i32 = 43;
pub const COLOR_12: i32 = 44;
pub const COLOR_13: i32 = 45;
pub const COLOR_14: i32 = 46;
pub const COLOR_15: i32 = 47;
pub const COLOR_16: i32 = 48;
pub const COLOR_17: i32 = 49;
pub const COLOR_18: i32 = 50;
pub const COLOR_19: i32 = 51;
pub const COLOR_20: i32 = 52;
pub const COLOR_21: i32 = 53;
pub const COLOR_22: i32 = 54;
pub const COLOR_23: i32 = 55;
pub const COLOR_24: i32 = 56;
pub const COLOR_25: i32 = 57;
pub const COLOR_26: i32 = 58;
pub const COLOR_27: i32 = 59;
pub const COLOR_28: i32 = 60;
pub const COLOR_29: i32 = 61;
pub const COLOR_30: i32 = 62;
pub const COLOR_31: i32 = 63;

pub const ITEMS: &[i32] = &[
    GRASS, SAND, STONE, BRICK, WOOD, CEMENT, DIRT, PLANK, SNOW, GLASS,
    COBBLE, LIGHT_STONE, DARK_STONE, CHEST, LEAVES,
    TALL_GRASS, YELLOW_FLOWER, RED_FLOWER, PURPLE_FLOWER,
    SUN_FLOWER, WHITE_FLOWER, BLUE_FLOWER,
    COLOR_00, COLOR_01, COLOR_02, COLOR_03, COLOR_04, COLOR_05,
    COLOR_06, COLOR_07, COLOR_08, COLOR_09, COLOR_10, COLOR_11,
    COLOR_12, COLOR_13, COLOR_14, COLOR_15, COLOR_16, COLOR_17,
    COLOR_18, COLOR_19, COLOR_20, COLOR_21, COLOR_22, COLOR_23,
    COLOR_24, COLOR_25, COLOR_26, COLOR_27, COLOR_28, COLOR_29,
    COLOR_30, COLOR_31,
];

// blocks[w] = [left, right, top, bottom, front, back] tile indices
pub const BLOCKS: [[i32; 6]; 256] = {
    let mut b = [[0i32; 6]; 256];
    b[1]  = [16, 16, 32, 0, 16, 16];   // grass
    b[2]  = [1, 1, 1, 1, 1, 1];         // sand
    b[3]  = [2, 2, 2, 2, 2, 2];         // stone
    b[4]  = [3, 3, 3, 3, 3, 3];         // brick
    b[5]  = [20, 20, 36, 4, 20, 20];    // wood
    b[6]  = [5, 5, 5, 5, 5, 5];         // cement
    b[7]  = [6, 6, 6, 6, 6, 6];         // dirt
    b[8]  = [7, 7, 7, 7, 7, 7];         // plank
    b[9]  = [24, 24, 40, 8, 24, 24];    // snow
    b[10] = [9, 9, 9, 9, 9, 9];         // glass
    b[11] = [10, 10, 10, 10, 10, 10];   // cobble
    b[12] = [11, 11, 11, 11, 11, 11];   // light stone
    b[13] = [12, 12, 12, 12, 12, 12];   // dark stone
    b[14] = [13, 13, 13, 13, 13, 13];   // chest
    b[15] = [14, 14, 14, 14, 14, 14];   // leaves
    b[16] = [15, 15, 15, 15, 15, 15];   // cloud
    // Colors 32-63
    b[32] = [176, 176, 176, 176, 176, 176];
    b[33] = [177, 177, 177, 177, 177, 177];
    b[34] = [178, 178, 178, 178, 178, 178];
    b[35] = [179, 179, 179, 179, 179, 179];
    b[36] = [180, 180, 180, 180, 180, 180];
    b[37] = [181, 181, 181, 181, 181, 181];
    b[38] = [182, 182, 182, 182, 182, 182];
    b[39] = [183, 183, 183, 183, 183, 183];
    b[40] = [184, 184, 184, 184, 184, 184];
    b[41] = [185, 185, 185, 185, 185, 185];
    b[42] = [186, 186, 186, 186, 186, 186];
    b[43] = [187, 187, 187, 187, 187, 187];
    b[44] = [188, 188, 188, 188, 188, 188];
    b[45] = [189, 189, 189, 189, 189, 189];
    b[46] = [190, 190, 190, 190, 190, 190];
    b[47] = [191, 191, 191, 191, 191, 191];
    b[48] = [192, 192, 192, 192, 192, 192];
    b[49] = [193, 193, 193, 193, 193, 193];
    b[50] = [194, 194, 194, 194, 194, 194];
    b[51] = [195, 195, 195, 195, 195, 195];
    b[52] = [196, 196, 196, 196, 196, 196];
    b[53] = [197, 197, 197, 197, 197, 197];
    b[54] = [198, 198, 198, 198, 198, 198];
    b[55] = [199, 199, 199, 199, 199, 199];
    b[56] = [200, 200, 200, 200, 200, 200];
    b[57] = [201, 201, 201, 201, 201, 201];
    b[58] = [202, 202, 202, 202, 202, 202];
    b[59] = [203, 203, 203, 203, 203, 203];
    b[60] = [204, 204, 204, 204, 204, 204];
    b[61] = [205, 205, 205, 205, 205, 205];
    b[62] = [206, 206, 206, 206, 206, 206];
    b[63] = [207, 207, 207, 207, 207, 207];
    b
};

pub const PLANTS: [i32; 256] = {
    let mut p = [0i32; 256];
    p[17] = 48;  // tall grass
    p[18] = 49;  // yellow flower
    p[19] = 50;  // red flower
    p[20] = 51;  // purple flower
    p[21] = 52;  // sun flower
    p[22] = 53;  // white flower
    p[23] = 54;  // blue flower
    p
};

pub fn is_plant(w: i32) -> bool {
    matches!(w, TALL_GRASS | YELLOW_FLOWER | RED_FLOWER |
               PURPLE_FLOWER | SUN_FLOWER | WHITE_FLOWER | BLUE_FLOWER)
}

pub fn is_obstacle(w: i32) -> bool {
    let w = w.abs();
    if is_plant(w) {
        return false;
    }
    !matches!(w, EMPTY | CLOUD)
}

pub fn is_transparent(w: i32) -> bool {
    if w == EMPTY {
        return true;
    }
    let w = w.abs();
    if is_plant(w) {
        return true;
    }
    matches!(w, EMPTY | GLASS | LEAVES)
}

pub fn is_destructable(w: i32) -> bool {
    !matches!(w, EMPTY | CLOUD)
}
