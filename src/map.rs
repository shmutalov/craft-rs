#![allow(dead_code)]

#[derive(Clone, Copy, Default)]
pub struct MapEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}

impl MapEntry {
    pub fn is_empty(&self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0 && self.w == 0
    }
}

pub struct Map {
    pub dx: i32,
    pub dy: i32,
    pub dz: i32,
    pub mask: usize,
    pub size: usize,
    pub data: Vec<MapEntry>,
}

fn hash_int(mut key: i32) -> i32 {
    key = !key.wrapping_add(key << 15);
    key ^= key >> 12;
    key = key.wrapping_add(key << 2);
    key ^= key >> 4;
    key = key.wrapping_mul(2057);
    key ^= key >> 16;
    key
}

fn hash(x: i32, y: i32, z: i32) -> usize {
    let x = hash_int(x);
    let y = hash_int(y);
    let z = hash_int(z);
    (x ^ y ^ z) as usize
}

impl Map {
    pub fn new(dx: i32, dy: i32, dz: i32, mask: usize) -> Self {
        Map {
            dx,
            dy,
            dz,
            mask,
            size: 0,
            data: vec![MapEntry::default(); mask + 1],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, w: i32) -> bool {
        let mut index = hash(x, y, z) & self.mask;
        let rx = x - self.dx;
        let ry = y - self.dy;
        let rz = z - self.dz;
        loop {
            let entry = &self.data[index];
            if entry.is_empty() {
                break;
            }
            if entry.x == rx && entry.y == ry && entry.z == rz {
                // overwrite
                if self.data[index].w != w {
                    self.data[index].w = w;
                    return true;
                }
                return false;
            }
            index = (index + 1) & self.mask;
        }
        if w != 0 {
            self.data[index] = MapEntry { x: rx, y: ry, z: rz, w };
            self.size += 1;
            if self.size * 2 > self.mask {
                self.grow();
            }
            return true;
        }
        false
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        let mut index = hash(x, y, z) & self.mask;
        let rx = x - self.dx;
        let ry = y - self.dy;
        let rz = z - self.dz;
        if rx < 0 || rx > 255 { return 0; }
        if ry < 0 || ry > 255 { return 0; }
        if rz < 0 || rz > 255 { return 0; }
        loop {
            let entry = &self.data[index];
            if entry.is_empty() {
                return 0;
            }
            if entry.x == rx && entry.y == ry && entry.z == rz {
                return entry.w;
            }
            index = (index + 1) & self.mask;
        }
    }

    fn grow(&mut self) {
        let new_mask = (self.mask << 1) | 1;
        let mut new_map = Map::new(self.dx, self.dy, self.dz, new_mask);
        for entry in &self.data {
            if !entry.is_empty() {
                let x = entry.x + self.dx;
                let y = entry.y + self.dy;
                let z = entry.z + self.dz;
                new_map.set(x, y, z, entry.w);
            }
        }
        self.mask = new_map.mask;
        self.size = new_map.size;
        self.data = new_map.data;
    }

    pub fn copy_from(&mut self, other: &Map) {
        self.dx = other.dx;
        self.dy = other.dy;
        self.dz = other.dz;
        self.mask = other.mask;
        self.size = other.size;
        self.data = other.data.clone();
    }

    /// Iterate over all entries, yielding (world_x, world_y, world_z, w)
    pub fn iter(&self) -> MapIter<'_> {
        MapIter { map: self, index: 0 }
    }
}

impl Clone for Map {
    fn clone(&self) -> Self {
        Map {
            dx: self.dx,
            dy: self.dy,
            dz: self.dz,
            mask: self.mask,
            size: self.size,
            data: self.data.clone(),
        }
    }
}

pub struct MapIter<'a> {
    map: &'a Map,
    index: usize,
}

impl<'a> Iterator for MapIter<'a> {
    type Item = (i32, i32, i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index <= self.map.mask {
            let entry = &self.map.data[self.index];
            self.index += 1;
            if !entry.is_empty() {
                return Some((
                    entry.x + self.map.dx,
                    entry.y + self.map.dy,
                    entry.z + self.map.dz,
                    entry.w,
                ));
            }
        }
        None
    }
}
