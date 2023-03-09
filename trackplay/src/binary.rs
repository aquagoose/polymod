pub struct BinaryWriter {
    data: Vec<u8>,
    position: usize,
}

impl BinaryWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            position: 0
        }
    }

    pub fn write_u32(&mut self, value: u32) {
        self.ensure_size(self.position + 4);
        self.data[self.position + 0] = (value & 0xFF) as u8;
        self.data[self.position + 1] = ((value >> 8) & 0xFF) as u8;
        self.data[self.position + 2] = ((value >> 16) & 0xFF) as u8;
        self.data[self.position + 3] = (value >> 24) as u8;
        self.position += 4;
    }

    pub fn write_u16(&mut self, value: u16) {
        self.ensure_size(self.position + 2);
        self.data[self.position + 0] = (value & 0xFF) as u8;
        self.data[self.position + 1] = ((value >> 8) & 0xFF) as u8;
        self.position += 2;
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.ensure_size(self.position + bytes.len());
        // TODO: This is garbage!
        for value in bytes {
            self.data[self.position] = *value;
            self.position += 1;
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.ensure_size(position);
        self.position = position;
    }

    fn ensure_size(&mut self, position: usize) {
        if position >= self.data.len() {
            for _ in 0..position - self.data.len() {
                self.data.push(0);
            }
        }
    }
}