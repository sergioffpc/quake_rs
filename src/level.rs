use std::{
    error::Error,
    io::{Cursor, ErrorKind},
};

use byteorder::{LittleEndian, ReadBytesExt};
use int_enum::IntEnum;

use crate::{load_resource, resource::GLOBAL_RESOURCES};

#[derive(Clone, Debug)]
pub struct Bsp {}

impl Bsp {
    pub fn load<S>(name: S) -> Result<Self, Box<dyn Error>>
    where
        S: AsRef<str>,
    {
        debug!("Loading BSP file {}", name.as_ref());

        Bsp::deserialize(&mut Cursor::new(load_resource!(name.as_ref())?))
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let version = reader.read_i32::<LittleEndian>().unwrap();
        if version != 29 {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid version: {}", version),
            )));
        }

        let mut sections = [DEntry { offset: 0, size: 0 }; 15];
        for section in sections.iter_mut() {
            *section = DEntry::deserialize(reader)?;
        }

        let entities_section = sections[SectionId::Entities.int_value()];
        let planes_section = sections[SectionId::Planes.int_value()];
        let textures_section = sections[SectionId::Textures.int_value()];
        let vertices_section = sections[SectionId::Vertices.int_value()];
        let visibility_section = sections[SectionId::Visibility.int_value()];
        let render_nodes_section = sections[SectionId::RenderNodes.int_value()];
        let texture_info_section = sections[SectionId::TextureInfo.int_value()];
        let faces_section = sections[SectionId::Faces.int_value()];
        let lightmaps_section = sections[SectionId::Lightmaps.int_value()];
        let clip_nodes_section = sections[SectionId::ClipNodes.int_value()];
        let leaves_section = sections[SectionId::Leaves.int_value()];
        let face_list_section = sections[SectionId::FaceList.int_value()];
        let edges_section = sections[SectionId::Edges.int_value()];
        let edge_list_section = sections[SectionId::EdgeList.int_value()];
        let models_section = sections[SectionId::Models.int_value()];

        Ok(Self {})
    }
}

#[derive(Clone, Copy, Debug)]
struct DEntry {
    offset: i32,
    size: i32,
}

impl DEntry {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let offset = reader.read_i32::<LittleEndian>().unwrap();
        let size = reader.read_i32::<LittleEndian>().unwrap();

        Ok(Self { offset, size })
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum SectionId {
    Entities = 0,
    Planes = 1,
    Textures = 2,
    Vertices = 3,
    Visibility = 4,
    RenderNodes = 5,
    TextureInfo = 6,
    Faces = 7,
    Lightmaps = 8,
    ClipNodes = 9,
    Leaves = 10,
    FaceList = 11,
    Edges = 12,
    EdgeList = 13,
    Models = 14,
}
