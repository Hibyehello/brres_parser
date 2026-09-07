use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn raw_new(data: &[u8; 12]) -> Result<Self, String> {
        let x_slice = data.get(0x0..0x4).ok_or("Failed to get X slice")?;
        let y_slice = data.get(0x4..0x8).ok_or("Failed to get Y slice")?;
        let z_slice = data.get(0x8..0xC).ok_or("Failed to get Z slice")?;

        let x = f32::from_be_bytes(
            x_slice
                .try_into()
                .map_err(|_| "Vec3: Failed to convert X bytes")?,
        );
        let y = f32::from_be_bytes(
            y_slice
                .try_into()
                .map_err(|_| "Vec3: Failed to convert Y bytes")?,
        );
        let z = f32::from_be_bytes(
            z_slice
                .try_into()
                .map_err(|_| "Vec3: Failed to convert Z bytes")?,
        );

        Ok(Vec3 { x, y, z })
    }
}

#[derive(Clone, Copy)]
pub enum SectionType {
    MDL0,
    TEX0,
    SRT0,
    CHR0,
    PAT0,
    CLR0,
    SHP0,
    SCN0,
    PLT0,
    VIS0,
}

impl SectionType {
    pub fn str_to_type(magic: String) -> Result<SectionType, String> {
        match magic.as_str() {
            "MDL0" => Ok(SectionType::MDL0),
            "TEX0" => Ok(SectionType::TEX0),
            "SRT0" => Ok(SectionType::SRT0),
            "CHR0" => Ok(SectionType::CHR0),
            "PAT0" => Ok(SectionType::PAT0),
            "CLR0" => Ok(SectionType::CLR0),
            "SHP0" => Ok(SectionType::SHP0),
            "SCN0" => Ok(SectionType::SCN0),
            "PLT0" => Ok(SectionType::PLT0),
            "VIS0" => Ok(SectionType::VIS0),
            &_ => Err("Invalid Section Magic recieved".to_string()),
        }
    }

    pub fn get_section_count(&self, version: u32) -> u32 {
        match self {
            SectionType::MDL0 => match version {
                8 => 11,
                11 => 14,
                _ => 0,
            },
            SectionType::TEX0 => match version {
                1 => 1,
                2 => 2,
                3 => 1,
                _ => 0,
            },
            SectionType::SRT0 => match version {
                4 => 1,
                5 => 2,
                _ => 0,
            },
            SectionType::CHR0 => match version {
                3 => 1,
                5 => 2,
                _ => 0,
            },
            SectionType::PAT0 => match version {
                4 => 6,
                _ => 0,
            },
            SectionType::CLR0 => match version {
                4 => 2,
                _ => 0,
            },
            SectionType::SHP0 => match version {
                4 => 3,
                _ => 0,
            },
            SectionType::SCN0 => match version {
                4 => 6,
                5 => 7,
                _ => 0,
            },
            SectionType::PLT0 => match version {
                _ => 0,
            },
            SectionType::VIS0 => match version {
                _ => 0,
            },
        }
    }
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SectionType::MDL0 => write!(f, "MDL0"),
            SectionType::TEX0 => write!(f, "TEX0"),
            SectionType::SRT0 => write!(f, "SRT0"),
            SectionType::CHR0 => write!(f, "CHR0"),
            SectionType::PAT0 => write!(f, "PAT0"),
            SectionType::CLR0 => write!(f, "CLR0"),
            SectionType::SHP0 => write!(f, "SHP0"),
            SectionType::SCN0 => write!(f, "SCN0"),
            SectionType::PLT0 => write!(f, "PLT0"),
            SectionType::VIS0 => write!(f, "VIS0"),
        }
    }
}

//TODO: Actually use this to determine from_xx_bytes calls
pub enum Endian {
    Big,
    Little,
}

impl fmt::Display for Endian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Endian::Big => write!(f, "Big"),
            Endian::Little => write!(f, "Little"),
        }
    }
}
