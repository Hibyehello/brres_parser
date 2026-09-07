// BRRES Headers
mod common;
mod index_group;
mod mdl0;
mod subfile;
use common::Endian;
use index_group::IndexHeader;
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
