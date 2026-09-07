use crate::brres::{common::*, RawBrres};

pub struct TEX0 {
    pub header: Header,
    pub data: Vec<u8>,
}

impl TEX0 {
    pub fn new(brres: RawBrres, offset: usize, header_length: usize) -> Result<Self, String> {
        let header = Header::new(brres, offset + header_length)?;
        Ok(TEX0 {
            header,
            data: Vec::new(),
        })
    }
}

pub struct Header {
    pub flag: u32,
    pub width: u16,
    pub height: u16,
    pub format: u32,
    pub mipmap_num: u32,
    pub min_mipmap: f32,
    pub max_mipmap: f32,
}

impl Header {
    fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let ctx = "TEX0::Header";

        let data = brres
            .slice(offset, 0x1C)
            .ok_or_else(|| format!("{ctx}: Unable to get TEX0 header"))?;

        let flag = read_u32(data, 0x0, ctx)?;
        let width = read_u16(data, 0x4, ctx)?;
        let height = read_u16(data, 0x6, ctx)?;
        let format = read_u32(data, 0x8, ctx)?;
        let mipmap_num = read_u32(data, 0xC, ctx)?;
        let min_mipmap = read_f32(data, 0x10, ctx)?;
        let max_mipmap = read_f32(data, 0x14, ctx)?;

        println!("flag: {}", flag);
        println!("width: {}", width);
        println!("height: {}", height);
        println!("format: {}", format);
        println!("mipmap_num: {}", mipmap_num);
        println!("min_mipmap: {}", min_mipmap);
        println!("max_mipmap: {}", max_mipmap);        

        Ok(Header {
            flag,
            width,
            height,
            format,
            mipmap_num,
            min_mipmap,
            max_mipmap,
        })
    }
}