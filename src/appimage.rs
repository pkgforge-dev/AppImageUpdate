use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::{Error, Result};

const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ISO_MAGIC: &[u8; 5] = b"CD001";
const APPIMAGE_MAGIC: &[u8; 2] = b"AI";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppImageType {
    Type1,
    Type2,
}

pub struct AppImage {
    path: Box<Path>,
    ty: AppImageType,
}

impl AppImage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::AppImage(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(Error::AppImage(format!("Not a file: {}", path.display())));
        }

        let mut file = File::open(path)?;
        let ty = Self::detect_type(&mut file)?;

        Ok(Self {
            path: path.into(),
            ty,
        })
    }

    pub fn appimage_type(&self) -> AppImageType {
        self.ty
    }

    pub fn read_update_info(&self) -> Result<String> {
        let mut file = File::open(&self.path)?;

        match self.ty {
            AppImageType::Type1 => Self::read_update_info_type1(&mut file),
            AppImageType::Type2 => Self::read_update_info_type2(&mut file),
        }
    }

    fn detect_type(file: &mut File) -> Result<AppImageType> {
        let mut magic = [0u8; 3];
        file.seek(SeekFrom::Start(8))?;
        file.read_exact(&mut magic)?;

        if &magic[0..2] == APPIMAGE_MAGIC {
            let ty = magic[2];
            if ty == 1 {
                return Ok(AppImageType::Type1);
            }
            if ty == 2 {
                return Ok(AppImageType::Type2);
            }
        }

        if Self::has_elf_magic(file)? && Self::has_iso_magic(file)? {
            return Ok(AppImageType::Type1);
        }

        Err(Error::AppImage(
            "Unknown AppImage type or not an AppImage".into(),
        ))
    }

    fn has_elf_magic(file: &mut File) -> Result<bool> {
        let mut magic = [0u8; 4];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut magic)?;
        Ok(&magic == ELF_MAGIC)
    }

    fn has_iso_magic(file: &mut File) -> Result<bool> {
        let mut magic = [0u8; 5];
        file.seek(SeekFrom::Start(32769))?;
        file.read_exact(&mut magic)?;
        Ok(&magic == ISO_MAGIC)
    }

    fn read_update_info_type1(file: &mut File) -> Result<String> {
        const POSITION: u64 = 0x8373;
        const LENGTH: usize = 512;

        file.seek(SeekFrom::Start(POSITION))?;
        let mut buffer = vec![0u8; LENGTH];
        file.read_exact(&mut buffer)?;

        let null_pos = buffer.iter().position(|&b| b == 0).unwrap_or(LENGTH);
        String::from_utf8(buffer[..null_pos].to_vec())
            .map_err(|e| Error::AppImage(format!("Invalid UTF-8 in update info: {}", e)))
    }

    fn read_update_info_type2(file: &mut File) -> Result<String> {
        let (offset, length) = Self::find_elf_section(file, ".upd_info")?;

        if offset == 0 || length == 0 {
            return Err(Error::AppImage(
                "Could not find .upd_info section in AppImage".into(),
            ));
        }

        file.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer)?;

        let null_pos = buffer
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(length as usize);
        String::from_utf8(buffer[..null_pos].to_vec())
            .map_err(|e| Error::AppImage(format!("Invalid UTF-8 in update info: {}", e)))
    }

    fn find_elf_section(file: &mut File, section_name: &str) -> Result<(u64, u64)> {
        let mut e_ident = [0u8; 16];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut e_ident)?;

        if &e_ident[0..4] != ELF_MAGIC {
            return Err(Error::AppImage("Not a valid ELF file".into()));
        }

        let is_64bit = e_ident[4] == 2;

        if is_64bit {
            Self::find_elf_section_64(file, section_name)
        } else {
            Self::find_elf_section_32(file, section_name)
        }
    }

    fn find_elf_section_64(file: &mut File, section_name: &str) -> Result<(u64, u64)> {
        let mut header = [0u8; 64];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut header)?;

        let e_shoff = u64::from_le_bytes(header[40..48].try_into().unwrap());
        let e_shentsize = u16::from_le_bytes(header[58..60].try_into().unwrap());
        let e_shnum = u16::from_le_bytes(header[60..62].try_into().unwrap());
        let e_shstrndx = u16::from_le_bytes(header[62..64].try_into().unwrap());

        if e_shoff == 0 || e_shnum == 0 {
            return Ok((0, 0));
        }

        let shstrtab_offset = e_shoff + (e_shstrndx as u64 * e_shentsize as u64);
        file.seek(SeekFrom::Start(shstrtab_offset))?;
        let mut shstrtab_header = [0u8; 64];
        file.read_exact(&mut shstrtab_header)?;
        let shstrtab_offset = u64::from_le_bytes(shstrtab_header[24..32].try_into().unwrap());
        let shstrtab_size = u64::from_le_bytes(shstrtab_header[32..40].try_into().unwrap());

        let mut strtab = vec![0u8; shstrtab_size as usize];
        file.seek(SeekFrom::Start(shstrtab_offset))?;
        file.read_exact(&mut strtab)?;

        for i in 0..e_shnum {
            let section_offset = e_shoff + (i as u64 * e_shentsize as u64);
            file.seek(SeekFrom::Start(section_offset))?;
            let mut section_header = [0u8; 64];
            file.read_exact(&mut section_header)?;

            let sh_name = u32::from_le_bytes(section_header[0..4].try_into().unwrap()) as usize;
            let sh_offset = u64::from_le_bytes(section_header[24..32].try_into().unwrap());
            let sh_size = u64::from_le_bytes(section_header[32..40].try_into().unwrap());

            if sh_name < strtab.len() {
                let end = strtab[sh_name..]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(strtab.len() - sh_name);
                if let Ok(name) = std::str::from_utf8(&strtab[sh_name..sh_name + end])
                    && name == section_name
                {
                    return Ok((sh_offset, sh_size));
                }
            }
        }

        Ok((0, 0))
    }

    fn find_elf_section_32(file: &mut File, section_name: &str) -> Result<(u64, u64)> {
        let mut header = [0u8; 52];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut header)?;

        let e_shoff = u32::from_le_bytes(header[32..36].try_into().unwrap()) as u64;
        let e_shentsize = u16::from_le_bytes(header[46..48].try_into().unwrap());
        let e_shnum = u16::from_le_bytes(header[48..50].try_into().unwrap());
        let e_shstrndx = u16::from_le_bytes(header[50..52].try_into().unwrap());

        if e_shoff == 0 || e_shnum == 0 {
            return Ok((0, 0));
        }

        let shstrtab_offset = e_shoff + (e_shstrndx as u64 * e_shentsize as u64);
        file.seek(SeekFrom::Start(shstrtab_offset))?;
        let mut shstrtab_header = [0u8; 40];
        file.read_exact(&mut shstrtab_header)?;
        let shstrtab_offset =
            u32::from_le_bytes(shstrtab_header[16..20].try_into().unwrap()) as u64;
        let shstrtab_size = u32::from_le_bytes(shstrtab_header[20..24].try_into().unwrap()) as u64;

        let mut strtab = vec![0u8; shstrtab_size as usize];
        file.seek(SeekFrom::Start(shstrtab_offset))?;
        file.read_exact(&mut strtab)?;

        for i in 0..e_shnum {
            let section_offset = e_shoff + (i as u64 * e_shentsize as u64);
            file.seek(SeekFrom::Start(section_offset))?;
            let mut section_header = [0u8; 40];
            file.read_exact(&mut section_header)?;

            let sh_name = u32::from_le_bytes(section_header[0..4].try_into().unwrap()) as usize;
            let sh_offset = u32::from_le_bytes(section_header[16..20].try_into().unwrap()) as u64;
            let sh_size = u32::from_le_bytes(section_header[20..24].try_into().unwrap()) as u64;

            if sh_name < strtab.len() {
                let end = strtab[sh_name..]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(strtab.len() - sh_name);
                if let Ok(name) = std::str::from_utf8(&strtab[sh_name..sh_name + end])
                    && name == section_name
                {
                    return Ok((sh_offset, sh_size));
                }
            }
        }

        Ok((0, 0))
    }
}
