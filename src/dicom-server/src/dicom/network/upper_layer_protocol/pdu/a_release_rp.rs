pub(crate) const PDU_TYPE: u8 = 0x06;

pub struct AReleaseRp();

impl AReleaseRp {
    pub fn size(&self) -> usize {
        10
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub fn new() -> Self {
        Self()
    }
}

impl From<AReleaseRp> for Vec<u8> {
    fn from(_val: AReleaseRp) -> Self {
        vec![
            PDU_TYPE, // PDU-type
            0,        // Reserved
            0, 0, 0, 4, // PDU-length
            0, 0, 0, 0, // Reserved
        ]
    }
}

impl Default for AReleaseRp {
    fn default() -> Self {
        Self::new()
    }
}
