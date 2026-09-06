use crate::brres::{RawBrres, Vec3};

// TODO: Implement each section
pub struct MDL0 {
    header: Header,
}

impl MDL0 {
    pub fn new(brres: RawBrres, offset: usize, header_length: usize) -> Result<Self, String> {
        let header = Header::new(brres, offset + header_length)?;

        Ok(MDL0 { header })
    }
}

struct Header {
    length: u32,
    file_header_offset: usize,
    scale_mode: u32,
    tex_mode: u32,
    vertex_count: u32,
    face_count: u32,
    matrices_count: u32,
    normalize_matrices: bool,
    need_tex_matrices: bool,
    bounding_volume: bool,
    matrices_offset: usize,
    bounding_volume_min: Vec3,
    bounding_volume_max: Vec3,
}

impl Header {
    fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let data = brres
            .slice(offset, 0x4)
            .ok_or_else(|| "MDLO::Header: Unable to get 4 bytes for header length")?;

        let length = u32::from_be_bytes(
            data.try_into()
                .map_err(|_| "MDL0::Header: Unable to parse length for Header")?,
        );

        let data = brres
            .slice(offset, length as usize)
            .ok_or_else(|| format!("MDL0::Header: Unable to get {:#02x} bytes", length))?;

        let mut slice = data
            .get(0x4..0x8)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let file_header_offset = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse file header offset")?,
        ) as usize;

        slice = data.get(0x8..0xC).ok_or(format!(
            "MDL0::Header: Unable to get slice: line {}",
            line!()
        ))?;

        let scale_mode = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse scaling mode")?,
        );

        slice = data
            .get(0xC..0x10)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let tex_mode = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse texture matrix mode")?,
        );

        slice = data
            .get(0x10..0x14)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let vertex_count = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse vertex count")?,
        );

        slice = data
            .get(0x14..0x18)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let face_count = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse face count")?,
        );

        slice = data
            .get(0x1C..0x20)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let matrices_count = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse matrices count")?,
        );

        let normalize_matrices: bool = data
            .get(0x20)
            .map(|&b| b != 0)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let need_tex_matrices: bool = data
            .get(0x21)
            .map(|&b| b != 0)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let bounding_volume: bool = data
            .get(0x22)
            .map(|&b| b != 0)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        slice = data
            .get(0x24..0x28)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let matrices_offset = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse matrices offset")?,
        ) as usize;

        slice = data
            .get(0x28..0x34)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let bounding_volume_min = Vec3::raw_new(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse bounding volume minimum")?,
        )?;

        slice = data
            .get(0x34..0x40)
            .ok_or_else(|| format!("MDL0::Header: Unable to get slice: line {}", line!()))?;

        let bounding_volume_max = Vec3::raw_new(
            slice
                .try_into()
                .map_err(|_| "MDL0::Header: Unable to parse bounding volume minimum")?,
        )?;

        Ok(Header {
            length,
            file_header_offset,
            scale_mode,
            tex_mode,
            vertex_count,
            face_count,
            matrices_count,
            normalize_matrices,
            need_tex_matrices,
            bounding_volume,
            matrices_offset,
            bounding_volume_min,
            bounding_volume_max,
        })
    }
}
