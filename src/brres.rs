// BRRES Headers
mod mdl0;
mod subfile;
use mdl0::MDL0;
use std::fmt;
use std::fs;
use subfile::SubFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawBrres<'a> {
    data: &'a [u8],
}

impl<'a> RawBrres<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn slice(&self, offset: usize, size: usize) -> Option<&'a [u8]> {
        self.data.get(offset..offset.checked_add(size)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    fn raw_new(data: &[u8; 12]) -> Result<Self, String> {
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
enum SectionType {
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
    fn str_to_type(magic: String) -> Result<SectionType, String> {
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

    fn get_section_count(&self, version: u32) -> u32 {
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

enum Endian {
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

struct FileHeader {
    magic: String,
    byte_order: Endian,
    file_size: u32,
    root_offset: u16,
    num_sections: u16,
}

impl FileHeader {
    fn new(data: &[u8; 0x10]) -> Result<Self, String> {
        let mut slice = data
            .get(0x0..0x4)
            .ok_or(format!("FileHeader: Unable to get slice: line {}", line!()))?;
        let magic = String::from_utf8(slice.to_vec()).map_err(|_| {
            format!(
                "FileHeader: Unable to convert slice to String: line {}",
                line!()
            )
        })?;

        if magic != "bres" {
            return Err("Invalid file header".to_string());
        }

        slice = data
            .get(0x4..0x6)
            .ok_or(format!("FileHeader: Unable to get slice: line {}", line!()))?;
        let byte_order = if slice == [0xFF, 0xFE] {
            Endian::Little
        } else {
            Endian::Big
        };

        slice = data
            .get(0x8..0xc)
            .ok_or(format!("FileHeader: Unable to get slice: line {}", line!()))?;
        let file_size = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("FileHeader: Unable to convert Slice: line {}", line!()))?,
        );

        slice = data
            .get(0xc..0xe)
            .ok_or(format!("FileHeader: Unable to get slice: line {}", line!()))?;
        let root_offset = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("FileHeader: Unable to convert Slice: line {}", line!()))?,
        );

        slice = data
            .get(0xe..0x10)
            .ok_or(format!("FileHeader: Unable to get slice: line {}", line!()))?;
        let num_sections = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("FileHeader: Unable to convert Slice: line {}", line!()))?,
        );

        Ok(FileHeader {
            magic,
            byte_order,
            file_size,
            root_offset,
            num_sections,
        })
    }
}

struct RootHeader {
    magic: String,
    len_sections: u32,
    offset: usize,
}

impl RootHeader {
    fn new(data: &[u8; 0x8], offset: usize) -> Result<Self, String> {
        let mut slice = data
            .get(0x0..0x4)
            .ok_or(format!("RootHeader: Unable to get slice: line {}", line!()))?;
        let magic = String::from_utf8(slice.to_vec()).map_err(|_| {
            format!(
                "RootHeader: Unable to convert slice to String: line {}",
                line!()
            )
        })?;

        if magic != "root" {
            return Err("Invalid root header".to_string());
        }

        slice = data
            .get(0x4..0x8)
            .ok_or(format!("RootHeader: Unable to get slice: line {}", line!()))?;
        let len_sections = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("RootHeader: Unable to convert Slice: line {}", line!()))?,
        );

        Ok(RootHeader {
            magic,
            len_sections,
            offset,
        })
    }
}

struct IndexHeader {
    len_group: u32,
    num_group: u32,
    root: IndexEntry,
    offset: usize,
}

impl IndexHeader {
    fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let data = brres
            .slice(offset, 0x8)
            .ok_or("Failed to get 0x8 bytes for IndexHeader")?;
        let mut slice = data.get(0x0..0x4).ok_or(format!(
            "IndexHeader: Unable to get slice: line {}",
            line!()
        ))?;
        let len_group = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexHeader: Unable to convert Slice: line {}", line!()))?,
        );

        slice = data.get(0x4..0x8).ok_or(format!(
            "IndexHeader: Unable to get slice: line {}",
            line!()
        ))?;
        let num_group = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexHeader: Unable to convert Slice: line {}", line!()))?,
        );

        Ok(IndexHeader {
            len_group,
            num_group,
            root: IndexEntry::new(brres, offset + 0x8, true, 0)?,
            offset,
        })
    }
}

struct IndexEntry {
    id: u16,
    flag: u16, // Always 0?
    left_idx: u16,
    left: Option<Box<IndexEntry>>,
    right_idx: u16,
    right: Option<Box<IndexEntry>>,
    name: String,
    len_name: u32, // 4 bytes before Offset of Name
    data_ptr: u32,
    root: bool, // Root node of the index groups
    current_idx: u16,
    group_offset: u32,
    sub_index: Option<Box<IndexHeader>>,
    sub_file: Option<SubFile>,
}

impl IndexEntry {
    fn new(
        brres: RawBrres,
        root_offset: usize,
        root: bool,
        current_idx: u16,
    ) -> Result<Self, String> {
        let group_offset = root_offset - 0x8;
        let offset = if root {
            root_offset
        } else {
            root_offset + (0x10 * current_idx as usize)
        };
        let data = brres
            .slice(offset, 0x10)
            .ok_or("Failed to get 0x10 bytes for IndexEntry")?;

        let mut slice = data
            .get(0x0..0x2)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let id = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );

        slice = data
            .get(0x2..0x4)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let flag = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );

        slice = data
            .get(0x4..0x6)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let left_idx = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );
        slice = data
            .get(0x6..0x8)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let right_idx = u16::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );

        let is_child = |child_idx: u16| -> bool {
            if child_idx == 0 {
                return false;
            }
            let child_off = root_offset + (0x10 * child_idx as usize);
            if let Some(slice) = brres.slice(child_off, 2) {
                if let Ok(bytes) = slice.try_into() {
                    u16::from_be_bytes(bytes) < id
                } else {
                    false
                }
            } else {
                false
            }
        };

        let left = if is_child(left_idx) {
            Some(Box::new(IndexEntry::new(
                brres,
                root_offset,
                false,
                left_idx,
            )?))
        } else {
            None
        };

        let right = if is_child(right_idx) {
            Some(Box::new(IndexEntry::new(
                brres,
                root_offset,
                false,
                right_idx,
            )?))
        } else {
            None
        };

        slice = data
            .get(0x8..0xC)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let rel_name_off = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );

        let (name, len_name) = if root {
            ("ROOT".to_string(), 4)
        } else {
            let name_off = group_offset + rel_name_off as usize;
            slice = brres
                .slice((name_off as usize) - 0x4, 0x4)
                .ok_or("Failed to get 0x4 bytes for IndexEntry name length")?;
            let len =
                u32::from_be_bytes(slice.try_into().map_err(|_| {
                    format!("IndexEntry: Unable to convert Slice: line {}", line!())
                })?);
            slice = brres.slice(name_off as usize, len as usize).ok_or(format!(
                "Failed to get {:#02x} bytes for IndexEntry Name",
                len
            ))?;
            (
                String::from_utf8(slice.to_vec()).map_err(|_| {
                    format!(
                        "IndexEntry: Unable to convert slice to String: line {}",
                        line!()
                    )
                })?,
                len,
            )
        };

        slice = data
            .get(0xC..0x10)
            .ok_or(format!("IndexEntry: Unable to get slice: line {}", line!()))?;
        let data_ptr = u32::from_be_bytes(
            slice
                .try_into()
                .map_err(|_| format!("IndexEntry: Unable to convert Slice: line {}", line!()))?,
        );

        let mut sub_index = None;
        let mut sub_file = None;

        if root == false {
            let parsed_data = parse_data(brres, data_ptr, group_offset as u32)?;
            sub_index = parsed_data.0;
            sub_file = parsed_data.1;
        }

        Ok(IndexEntry {
            id,
            flag,
            left_idx,
            left,
            right_idx,
            right,
            name,
            len_name,
            data_ptr,
            root,
            current_idx,
            group_offset: group_offset as u32,
            sub_index,
            sub_file,
        })
    }

    fn create_subfiles<'a>(&'a mut self, sub_files: &mut Vec<&'a mut SubFile>) {
        if self.root == false {
            if let Some(sub_file) = self.sub_file.as_mut() {
                sub_files.push(sub_file);
            }
        }

        if let Some(sub_index) = self.sub_index.as_mut() {
            sub_index.root.create_subfiles(sub_files);
        }

        if let Some(left) = self.left.as_mut() {
            left.create_subfiles(sub_files);
        }
        if let Some(right) = self.right.as_mut() {
            right.create_subfiles(sub_files);
        }
    }

    fn print_entry_names(&self) {
        println!("Node Name: {}", self.name,);

        if let Some(sub) = &self.sub_index {
            println!("--> Entering Sub-Folder: {}", self.name);
            sub.root.print_entry_names();
        }

        if let Some(left) = &self.left {
            left.print_entry_names();
        }
        if let Some(right) = &self.right {
            right.print_entry_names();
        }
    }
}

fn parse_data(
    brres: RawBrres,
    data_ptr: u32,
    group_offset: u32,
) -> Result<(Option<Box<IndexHeader>>, Option<SubFile>), String> {
    let magic_raw = brres
        .slice((data_ptr + group_offset) as usize, 0x4)
        .ok_or("Failed to get 0x4 bytes for IndexEntry")?;

    let magic_alpha = magic_raw.iter().all(|b| b.is_ascii_alphanumeric());

    if !magic_alpha {
        println!(
            "Creating IndexHeader using offset: {:#02x}",
            data_ptr + group_offset
        );
        let sub_index = Some(Box::new(IndexHeader::new(
            brres,
            (data_ptr + group_offset) as usize,
        )?));

        if let Some(sub) = &sub_index {
            println!("Sub-Index has {} entries", sub.num_group);
        }

        Ok((sub_index, None))
    } else {
        Ok((
            None,
            Some(SubFile::new(brres, (data_ptr + group_offset) as usize)?),
        ))
    }
}

pub fn parse_file(file: &str) -> std::io::Result<()> {
    let data = fs::read(file)?;

    let brres_file = RawBrres::new(&data);

    let file_header_data: &[u8; 0x10] = brres_file
        .slice(0, 0x10)
        .and_then(|d| d.try_into().ok())
        .ok_or(std::io::Error::other(
        "Did not get 0x10 bytes for the file header",
    ))?;

    let file_header = FileHeader::new(file_header_data).map_err(std::io::Error::other)?;
    println!("got magic: {}", file_header.magic);
    println!("Endianess: {}", file_header.byte_order);
    println!("File Size: {}", file_header.file_size);
    println!("Root Offset: {:#02x}", file_header.root_offset);
    println!("Number of sections: {}", file_header.num_sections);

    let root_header_data: &[u8; 0x8] = brres_file
        .slice(file_header.root_offset.into(), 0x8)
        .and_then(|d| d.try_into().ok())
        .ok_or(std::io::Error::other(
            "Did not get 0x8 bytes for the root header",
        ))?;

    let root_header = RootHeader::new(root_header_data, file_header.root_offset.into())
        .map_err(std::io::Error::other)?;
    println!("Root Magic: {}", root_header.magic);

    let mut index_header =
        IndexHeader::new(brres_file, root_header.offset + 0x8).map_err(std::io::Error::other)?;
    println!("Root Index Group has {} entries", index_header.num_group);

    index_header.root.print_entry_names();

    let mut sub_files: Vec<&mut SubFile> = Vec::new();

    index_header.root.create_subfiles(&mut sub_files);

    sub_files
        .iter_mut()
        .try_for_each(|file| -> std::io::Result<()> {
            println!("File SectionType is `{}`", file.header.section_type);
            file.generate_subfile(brres_file)
                .map_err(|e| std::io::Error::other(e))?;
            Ok(())
        })?;

    Ok(())
}
