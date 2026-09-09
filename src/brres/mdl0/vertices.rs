use crate::brres::{self, RawBrres, common::*};

pub(crate) struct EntryHeader {
    pub length: u32,
    mdl0_offset: i32,
    data_offset: i32,
    name_offset: i32,
    pub name: VariableString,
    index: u32,
    is_3d: bool,
    format: u32,
    divisor: u8,
    stride: u8,
    vertex_count: u16,
    bounding_volume_min: Vec3,
    bounding_volume_max: Vec3,
}

impl EntryHeader {
    pub fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let ctx = "MDL0::Vertices::EntryHeader:";
        let data = brres
            .slice(offset, 0x38)
            .ok_or_else(|| format!("{ctx}: Unable to get {:#02x} bytes: {}", 0x38, line!()))?;

        let length = read(data, 0x0, ctx)?;
        let mdl0_offset = read(data, 0x4, ctx)?;
        let data_offset = read(data, 0x8, ctx)?;
        let name_offset = read(data, 0xC, ctx)?;
        println!("Name_offset: {:#02x}", name_offset);
        let name = read(data, name_offset as usize, ctx)?;
        let index = read(data, 0x10, ctx)?;
        let is_3d = read::<u32>(data, 0x14, ctx)? == 0x1;
        let format = read(data, 0x18, ctx)?;
        let divisor = read(data, 0x1C, ctx)?;
        let stride = read(data, 0x1D, ctx)?;
        let vertex_count = read(data, 0x1E, ctx)?;
        let bounding_volume_min = read(data, 0x20, ctx)?;
        let bounding_volume_max = read(data, 0x2C, ctx)?;

        Ok(EntryHeader {
            length,
            mdl0_offset,
            data_offset,
            name_offset,
            name,
            index,
            is_3d,
            format,
            divisor,
            stride,
            vertex_count,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}
