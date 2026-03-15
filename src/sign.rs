#![allow(dead_code)]

#[derive(Clone)]
pub struct Sign {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub face: i32,
    pub text: String,
}

pub struct SignList {
    pub data: Vec<Sign>,
}

impl SignList {
    pub fn new() -> Self {
        SignList { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        SignList { data: Vec::with_capacity(cap) }
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32, face: i32, text: &str) {
        // Remove existing sign at same position+face
        self.data.retain(|s| !(s.x == x && s.y == y && s.z == z && s.face == face));
        self.data.push(Sign { x, y, z, face, text: text.to_string() });
    }

    pub fn remove(&mut self, x: i32, y: i32, z: i32, face: i32) -> bool {
        let len_before = self.data.len();
        self.data.retain(|s| !(s.x == x && s.y == y && s.z == z && s.face == face));
        self.data.len() != len_before
    }

    pub fn remove_all(&mut self, x: i32, y: i32, z: i32) -> bool {
        let len_before = self.data.len();
        self.data.retain(|s| !(s.x == x && s.y == y && s.z == z));
        self.data.len() != len_before
    }
}
