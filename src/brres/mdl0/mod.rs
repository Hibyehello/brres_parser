use crate::brres::{RawBrres, common::*};

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
        let ctx = "MDL0::Header";
    
        let len_bytes = brres
            .slice(offset, 0x4)
            .ok_or_else(|| format!("{ctx}: Unable to get 4 bytes for header length"))?;
        
        let length = u32::from_be_bytes(
            len_bytes
                .try_into()
                .map_err(|_| format!("{ctx}: Unable to parse length for Header"))?,
        );
    
        let data = brres
            .slice(offset, length as usize)
            .ok_or_else(|| format!("{ctx}: Unable to get {:#02x} bytes", length))?;

        let file_header_offset = read_u32(data, 0x04, ctx)? as usize;
        let scale_mode = read_u32(data, 0x08, ctx)?;
        let tex_mode = read_u32(data, 0x0C, ctx)?;
        let vertex_count = read_u32(data, 0x10, ctx)?;
        let face_count = read_u32(data, 0x14, ctx)?;
        let _unused = read_u32(data, 0x18, ctx)?;
        let matrices_count = read_u32(data, 0x1C, ctx)?;
        let normalize_matrices = read_bool(data, 0x20, ctx)?;
        let need_tex_matrices = read_bool(data, 0x21, ctx)?;
        let bounding_volume = read_bool(data, 0x22, ctx)?;
        let _padding = read_u32(data, 0x23, ctx)?;
        let matrices_offset = read_u32(data, 0x24, ctx)? as usize;
        let bounding_volume_min = read_vec3(data, 0x28, ctx)?;
        let bounding_volume_max = read_vec3(data, 0x34, ctx)?;

        println!("length: {}", length);
        println!("file_header_offset: {}", file_header_offset);
        println!("scale_mode: {}", scale_mode);
    
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