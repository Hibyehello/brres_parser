use std::fmt;
use std::mem;

pub trait FromBytes: Sized {
    #[track_caller]
    fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String>;
}

pub struct VariableString {
    pub string: String,
    pub length: usize,
}

impl FromBytes for VariableString {
    #[track_caller]
    fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String> {
        let mut slice = data.get(offset - 0x4..offset).ok_or(format!(
            "{ctx}: Failed to get 0x4 bytes for {ctx} name length: line {}",
            line!()
        ))?;
        let len = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("{ctx}: Unable to convert Slice: line {}", line!()))?,
        ) as usize;
        slice = data.get(offset..offset + len).ok_or(format!(
            "{ctx}: Failed to get {:#02x} bytes for {ctx} Name: line {}",
            len,
            line!()
        ))?;
        Ok(VariableString {
            string: String::from_utf8(slice.to_vec()).map_err(|_| {
                format!("{ctx}: Unable to convert slice to String: line {}", line!())
            })?,
            length: len,
        })
    }
}

impl FromBytes for String {
    #[track_caller]
    fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String> {
        let slice = data
            .get(offset..offset + 0x4)
            .ok_or(format!("{ctx}: Unable to get slice: line {}", line!()))?;
        let magic = String::from_utf8(slice.to_vec())
            .map_err(|_| format!("{ctx}: Unable to convert slice to String: line {}", line!()))?;

        Ok(magic)
    }
}

impl FromBytes for bool {
    #[track_caller]
    fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String> {
        let slice = data.get(offset..offset + 1).ok_or_else(|| {
            format!(
                "{ctx}: Unable to get {:#02x} bytes from offset {:#02x}: line{}",
                1,
                offset,
                line!()
            )
        })?;

        Ok(slice[0] != 0)
    }
}

macro_rules! impl_primitive_FromBytes {
    ($($t:ty), *) => {
        $(
            impl FromBytes for $t {
                #[track_caller]
                fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String> {
                    let size = std::mem::size_of::<$t>();

                    let slice = data.get(offset..offset + size).ok_or_else(|| {
                        format!(
                            "{ctx}: Unable to get {:#02x} bytes from offset {:#02x}",
                            size, offset
                        )
                    })?;

                    Ok(<$t>::from_be_bytes(slice.try_into().map_err(|_| {
                        format!(
                            "{ctx}: Internal error converting bytes to array at offset {:#02x}",
                            offset
                        )
                    })?))

                }
            }
        )*
    };
}

impl_primitive_FromBytes!(u16, i16, u32, i32, u64, i64, f32, f64);

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

impl FromBytes for Vec3 {
    fn from_bytes(data: &[u8], offset: usize, ctx: &str) -> Result<Self, String> {
        Ok(Vec3 {
            x: read(data, offset, ctx)?,
            y: read(data, offset + 4, ctx)?,
            z: read(data, offset + 8, ctx)?,
        })
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

pub fn read<T: FromBytes>(data: &[u8], offset: usize, ctx: &str) -> Result<T, String> {
    T::from_bytes(data, offset, ctx)
}
