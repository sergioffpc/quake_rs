use std::{
    collections::HashMap,
    fs::File,
    io::{Error, ErrorKind},
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

lazy_static! {
    pub static ref GLOBAL_RESOURCES: OnceCell<Mutex<Pak>> = OnceCell::new();
    pub static ref GLOBAL_PALETTE: OnceCell<Box<[[u8; 3]; 256]>> = OnceCell::new();
}

pub fn init<P>(path: P)
where
    P: AsRef<Path>,
{
    GLOBAL_RESOURCES.get_or_init(|| Mutex::new(Pak::open(path).unwrap()));
    GLOBAL_PALETTE.get_or_init(|| {
        let palette = GLOBAL_RESOURCES
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .read("gfx/palette.lmp")
            .unwrap();
        let mut rgb = [[0u8; 3]; 256];
        for index in 0..256 {
            for channel in 0..3 {
                rgb[index][channel] = palette[index * 3 + channel];
            }
        }
        Box::new(rgb)
    });
}

pub fn palette_index_to_rgba(indices: &Box<[u8]>) -> Box<[u8]> {
    let palette = GLOBAL_PALETTE.get().unwrap();
    let mut rgba = Vec::with_capacity(indices.len() * 4);
    for color_index in indices.iter() {
        match *color_index {
            0xff => {
                for _ in 0..4 {
                    rgba.push(0u8);
                }
            }
            i => {
                for color_channel in 0..3 {
                    rgba.push(palette[i as usize][color_channel]);
                }
                rgba.push(0xffu8);
            }
        }
    }
    rgba.into_boxed_slice()
}

#[macro_export]
macro_rules! load_resource {
    ($name: expr) => {
        GLOBAL_RESOURCES.get().unwrap().lock().unwrap().read($name)
    };
}

#[derive(Debug)]
pub struct Pak {
    file: File,
    directory: HashMap<String, (i32, i32)>,
}

impl Pak {
    pub fn open<P>(path: P) -> Result<Pak, Error>
    where
        P: AsRef<Path>,
    {
        debug!("Opening PAK file {}", path.as_ref().to_string_lossy());

        let mut file = File::open(path.as_ref())?;

        let mut header = [0u8; 12];
        file.read_exact(&mut header)?;

        if &header[0..4] != b"PACK" {
            return Err(Error::new(ErrorKind::InvalidData, "invalid signature"));
        }

        let mut offset = i32::from_le_bytes(header[4..8].try_into().unwrap());
        let num_files = i32::from_le_bytes(header[8..12].try_into().unwrap()) / 64;

        debug!("PACK Header:");
        debug!(
            "  Ident:            {}",
            String::from_utf8(header[0..4].to_vec()).unwrap()
        );
        debug!("  Directory Offset: {}", offset);
        debug!("  Number of files:  {}", num_files);

        debug!("PACK Content:");
        let mut directory = HashMap::with_capacity(num_files as usize);
        for _ in 0..num_files {
            file.seek(SeekFrom::Start(offset as u64))?;

            let mut entry_buf = [0u8; 64];
            file.read_exact(&mut entry_buf)?;

            let name = {
                let len = match entry_buf.iter().position(|&c| c == 0) {
                    Some(value) => value,
                    None => 0,
                };
                String::from_utf8_lossy(&entry_buf[..len])
            };
            let file_offset = i32::from_le_bytes(entry_buf[56..60].try_into().unwrap());
            let file_size = i32::from_le_bytes(entry_buf[60..64].try_into().unwrap());

            debug!(
                "  {:<64} {:<16} {:<16}",
                name.to_string(),
                file_offset,
                file_size
            );

            directory.insert(name.to_string(), (file_offset, file_size));
            offset += 64;
        }

        Ok(Self { file, directory })
    }

    pub fn read<S>(&mut self, name: S) -> Result<Vec<u8>, Error>
    where
        S: AsRef<str>,
    {
        match self.directory.get(name.as_ref()) {
            Some((offset, size)) => {
                let mut bytes = vec![0; *size as usize];
                self.file.seek(SeekFrom::Start(*offset as u64))?;
                self.file.read_exact(&mut bytes)?;

                Ok(bytes)
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("file not found: {}", name.as_ref()),
            )),
        }
    }
}
