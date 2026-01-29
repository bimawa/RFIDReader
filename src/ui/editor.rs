use crate::protocol::st25tb::ChipData;

pub struct ChipEditor {
    pub data: ChipData,
    pub selected_block: usize,
    pub selected_byte: usize,
    pub selected_nibble: usize,
    pub edit_mode: bool,
}

impl ChipEditor {
    pub fn new(data: ChipData) -> Self {
        Self {
            data,
            selected_block: 0,
            selected_byte: 0,
            selected_nibble: 0,
            edit_mode: false,
        }
    }

    pub fn move_up(&mut self) {
        if self.edit_mode {
            self.increment_nibble();
        } else if self.selected_block > 0 {
            self.selected_block -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.edit_mode {
            self.decrement_nibble();
        } else if self.selected_block < self.data.block_count - 1 {
            self.selected_block += 1;
        }
    }

    pub fn toggle_edit_mode(&mut self) {
        if !self.edit_mode {
            self.edit_mode = true;
            self.selected_byte = 0;
            self.selected_nibble = 0;
        } else {
            self.next_byte();
        }
    }

    pub fn exit_edit_mode(&mut self) {
        self.edit_mode = false;
    }

    pub fn next_byte(&mut self) {
        self.selected_byte = (self.selected_byte + 1) % 4;
        self.selected_nibble = 0;
    }

    fn increment_nibble(&mut self) {
        let block = &mut self.data.blocks[self.selected_block];
        let byte = &mut block[self.selected_byte];

        if self.selected_nibble == 0 {
            let high = (*byte >> 4).wrapping_add(1) & 0x0F;
            *byte = (high << 4) | (*byte & 0x0F);
        } else {
            let low = (*byte & 0x0F).wrapping_add(1) & 0x0F;
            *byte = (*byte & 0xF0) | low;
        }
    }

    fn decrement_nibble(&mut self) {
        let block = &mut self.data.blocks[self.selected_block];
        let byte = &mut block[self.selected_byte];

        if self.selected_nibble == 0 {
            let high = (*byte >> 4).wrapping_sub(1) & 0x0F;
            *byte = (high << 4) | (*byte & 0x0F);
        } else {
            let low = (*byte & 0x0F).wrapping_sub(1) & 0x0F;
            *byte = (*byte & 0xF0) | low;
        }
    }
}
