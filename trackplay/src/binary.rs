pub struct BinaryWriter {
    data: Vec<u8>
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new()
        }
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.push((value & 0xFF) as u8);
        self.data.push(((value >> 8) & 0xFF) as u8);
        self.data.push(((value >> 16) & 0xFF) as u8);
        self.data.push((value >> 24) as u8);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.push((value & 0xFF) as u8);
        self.data.push(((value >> 8) & 0xFF) as u8);
    }

    pub fn write_bytes(&mut self, bytes: &mut Vec<u8>) {
        self.data.append(bytes);
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}