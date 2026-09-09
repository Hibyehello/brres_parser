use crate::brres::{
    RawBrres,
    common::SectionType,
    index_group::IndexHeader,
    mdl0::{vertices::EntryHeader, *},
    tex0::TEX0,
};

pub(crate) struct Header {
    pub header_length: usize,
    pub section_type: SectionType,
    length: u32,
    version: u32,
    out_brres_off: u32,
    num_section_offsets: u32,
    section_offsets: Vec<u32>, // These offsets point to an Index Group
    name: String,
    file_off: usize,
}

impl Header {
    pub(crate) fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let data = brres
            .slice(offset, 0x10)
            .ok_or("Subfile::Header: Failed to get 0x10 bytes")?;

        let mut slice = data.get(0x0..0x4).ok_or(format!(
            "SubFile::Header: Unable to get slice: line {}",
            line!()
        ))?;

        let section_type =
            SectionType::str_to_type(String::from_utf8(slice.to_vec()).map_err(|_| {
                format!(
                    "SubFile::Header: Unable to convert slice to String: line {}",
                    line!()
                )
            })?)?;

        slice = data.get(0x4..0x8).ok_or(format!(
            "SubFile::Header: Unsable to get slice: line {}",
            line!()
        ))?;

        let length =
            u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);

        slice = data.get(0x8..0xC).ok_or(format!(
            "SubFile::Header: Unable to get slice: line {}",
            line!()
        ))?;

        let version =
            u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);

        slice = data.get(0xC..0x10).ok_or(format!(
            "SubFile::Header: Unable to get slice: line {}",
            line!()
        ))?;

        let out_brres_off =
            u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);

        let num_section_offsets = section_type.get_section_count(version);

        let mut section_offsets: Vec<u32> = Vec::new();

        let section_data_size: usize = (num_section_offsets as usize * 4) + 4;

        let data = brres
            .slice(offset + 0x10, section_data_size)
            .ok_or_else(|| {
                format!(
                    "Subfile::Header: Failed to get {:#02x} bytes",
                    section_data_size
                )
            })?;

        for n in 0..num_section_offsets {
            slice = data
                .get((n * 4) as usize..((n * 4) + 4) as usize)
                .ok_or(format!(
                    "SubFile::Header: Unable to get slice: line {}",
                    line!()
                ))?;
            let off = u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);
            section_offsets.push(off);
        }

        slice = data
            .get((num_section_offsets * 4) as usize..((num_section_offsets * 4) + 4) as usize)
            .ok_or(format!(
                "SubFile::Header: Unable to get slice: line {}",
                line!()
            ))?;

        let rel_name_off =
            u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);

        let name_off = offset + rel_name_off as usize;
        slice = brres
            .slice((name_off as usize) - 0x4, 0x4)
            .ok_or("SubFile::Header: Failed to get 0x4 bytes for name length")?;
        let len =
            u32::from_be_bytes(slice.try_into().map_err(|_| {
                format!("SubFile::Header: Unable to convert Slice: line {}", line!())
            })?);

        println!("Name offset?: {:#02x}", name_off);

        slice = brres.slice(name_off as usize, len as usize).ok_or(format!(
            "SubFile::Header: Failed to get {:#02x} bytes for Name",
            len
        ))?;

        let name = String::from_utf8(slice.to_vec()).map_err(|_| {
            format!(
                "SubFile::Header: Unable to convert slice to String: line {}",
                line!()
            )
        })?;

        println!("SubFile::Header: Name: {}", name);

        Ok(Header {
            header_length: (num_section_offsets as usize * 4) + 0x14,
            section_type,
            length,
            version,
            out_brres_off,
            num_section_offsets,
            section_offsets,
            name,
            file_off: offset,
        })
    }
}

pub enum SubFileData {
    Mdl0(MDL0),
    Tex0(TEX0),
    Unsupported,
}

pub struct SubFile {
    pub header: Header,
    pub file: Option<SubFileData>,
}

impl SubFile {
    pub fn new(brres: RawBrres, offset: usize) -> Result<Self, String> {
        let header = Header::new(brres, offset)?;

        Ok(SubFile { header, file: None })
    }

    fn create_mdl0(&mut self, brres: RawBrres) -> Result<(), String> {
        let mdl0 = MDL0::new(brres, self.header.file_off, self.header.header_length)?;

        self.file = Some(SubFileData::Mdl0(mdl0));

        /*  TODO: This needs to be a shallow new, since I don't believe sections use a folder-like structure
        Potentially can have a flat map parsing rather than using recursion */
        let verts_index_group = IndexHeader::new(
            brres,
            self.header.file_off + self.header.section_offsets[2] as usize,
        )?;

        Ok(())
    }

    fn create_tex0(&mut self, brres: RawBrres) -> Result<(), String> {
        let tex0 = TEX0::new(brres, self.header.file_off, self.header.header_length)?;

        self.file = Some(SubFileData::Tex0(tex0));

        Ok(())
    }

    pub fn generate_subfile(&mut self, brres: RawBrres) -> Result<(), String> {
        match (self.header.section_type) {
            SectionType::MDL0 => self.create_mdl0(brres),
            SectionType::TEX0 => self.create_tex0(brres),
            _ => {
                println!(
                    "Unsupported subfile encountered: {}",
                    self.header.section_type
                );
                self.file = Some(SubFileData::Unsupported);
                Ok(())
            }
        }
    }
}
