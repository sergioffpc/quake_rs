use std::{
    error::Error,
    io::{Cursor, ErrorKind, Read},
    time::Duration,
};

use byteorder::{LittleEndian, ReadBytesExt};
use cgmath::{InnerSpace, Vector3};

use crate::{load_resource, mesh::Vertex1XYZ1N1UV, resource::GLOBAL_RESOURCES};

#[derive(Clone, Debug)]
pub struct Mdl {
    pub skins: Box<[Skin]>,
    pub skin_width: u32,
    pub skin_height: u32,
    pub num_verts: u32,
    pub keyframes: Box<[Keyframe]>,
    skin_coords: Box<[SkinCoord]>,
    triangles: Box<[Triangle]>,
}

impl Mdl {
    pub fn load<S>(name: S) -> Result<Self, Box<dyn Error>>
    where
        S: AsRef<str>,
    {
        debug!("Loading MDL file {}", name.as_ref());

        Mdl::deserialize(&mut Cursor::new(load_resource!(name.as_ref())?))
    }

    pub fn indices(&self) -> Box<[u32]> {
        let mut indices = Vec::with_capacity(self.triangles.len() * 3);
        self.triangles
            .iter()
            .for_each(|a| indices.append(&mut a.indices.to_vec()));
        indices.into_boxed_slice()
    }

    pub fn vertices(&self, frame: &Frame) -> Box<[Vertex1XYZ1N1UV]> {
        let mut vertices = Vec::with_capacity(frame.vertices.len());
        for triangle in self.triangles.iter() {
            let mut face = [[0f32; 3]; 3];
            let mut skin_coords = [[0f32; 2]; 3];
            for (i, index) in triangle.indices.iter().enumerate() {
                face[i] = frame.vertices[*index as usize];

                let skin_coord = &self.skin_coords[*index as usize];
                let s = if !triangle.faces_front && skin_coord.is_on_seam {
                    (skin_coord.s as f32 + self.skin_width as f32 / 2.0) + 0.5
                } else {
                    skin_coord.s as f32 + 0.5
                } / self.skin_width as f32;
                let t = (skin_coord.t as f32 + 0.5) / self.skin_height as f32;
                skin_coords[i] = [s, t];
            }

            let normal = Vector3::cross(
                Vector3::from(face[0]) - Vector3::from(face[1]),
                Vector3::from(face[2]) - Vector3::from(face[1]),
            )
            .normalize();

            for i in 0..3 {
                vertices.push(Vertex1XYZ1N1UV {
                    position: face[i],
                    normal: normal.into(),
                    texcoord: skin_coords[i],
                })
            }
        }
        vertices.into_boxed_slice()
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let ident = reader.read_i32::<LittleEndian>().unwrap();
        if ident != 0x4f504449 {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid signature: {:x}", ident),
            )));
        }

        let version = reader.read_i32::<LittleEndian>().unwrap();
        if version != 6 {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid version: {}", version),
            )));
        }

        let mut scale = [0f32; 3];
        reader.read_f32_into::<LittleEndian>(&mut scale).unwrap();

        let mut origin = [0f32; 3];
        reader.read_f32_into::<LittleEndian>(&mut origin).unwrap();

        let bounding_radius = reader.read_f32::<LittleEndian>().unwrap();

        let mut position = [0f32; 3];
        reader.read_f32_into::<LittleEndian>(&mut position).unwrap();

        let num_skins = reader.read_i32::<LittleEndian>().unwrap();
        let skin_width = reader.read_i32::<LittleEndian>().unwrap();
        let skin_height = reader.read_i32::<LittleEndian>().unwrap();
        let num_verts = reader.read_i32::<LittleEndian>().unwrap();
        let num_tris = reader.read_i32::<LittleEndian>().unwrap();
        let num_frames: i32 = reader.read_i32::<LittleEndian>().unwrap();
        let sync_type = reader.read_i32::<LittleEndian>().unwrap();
        let flags: i32 = reader.read_i32::<LittleEndian>().unwrap();
        let size = reader.read_f32::<LittleEndian>().unwrap();

        debug!("MDL Header:");
        debug!(
            "  Ident:                 {}",
            String::from_utf8(ident.to_le_bytes().to_vec()).unwrap()
        );
        debug!("  Version:               {:?}", version);
        debug!("  Scale:                 {:?}", scale);
        debug!("  Origin:                {:?}", origin);
        debug!("  Bounding Radius:       {:?}", bounding_radius);
        debug!("  Position:              {:?}", position);
        debug!("  Number of Skins:       {:?}", num_skins);
        debug!("  Skin Width:            {:?}", skin_width);
        debug!("  Skin Height:           {:?}", skin_height);
        debug!("  Number of Vertices:    {:?}", num_verts);
        debug!("  Number of Triangles:   {:?}", num_tris);
        debug!("  Number of Frames:      {:?}", num_frames);
        debug!("  Sync Type:             {:?}", sync_type);
        debug!("  Flags:                 {:x}", flags);
        debug!("  Size:                  {:?}", size);

        let mut skins = Vec::with_capacity(num_skins as usize);
        for _ in 0..num_skins {
            skins.push(Skin::deserialize(
                reader,
                (skin_width * skin_height) as usize,
            )?);
        }

        let mut skin_coords = Vec::with_capacity(num_verts as usize);
        for _ in 0..num_verts {
            skin_coords.push(SkinCoord::deserialize(reader)?)
        }

        let mut triangles = Vec::with_capacity(num_tris as usize);
        for _ in 0..num_tris {
            triangles.push(Triangle::deserialize(reader)?);
        }

        let mut keyframes = Vec::with_capacity(num_frames as usize);
        for _ in 0..num_frames {
            keyframes.push(Keyframe::deserialize(reader, num_verts, scale, origin)?);
        }

        Ok(Self {
            skins: skins.into_boxed_slice(),
            skin_width: skin_width as u32,
            skin_height: skin_height as u32,
            skin_coords: skin_coords.into_boxed_slice(),
            num_verts: num_verts as u32,
            triangles: triangles.into_boxed_slice(),
            keyframes: keyframes.into_boxed_slice(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Skin {
    Static(StaticSkin),
    Animated(AnimatedSkin),
}

impl Skin {
    pub fn indices(&self, time: &Duration) -> Box<[u8]> {
        match *self {
            Skin::Static(ref s) => s.0.clone(),
            Skin::Animated(ref s) => {
                let total = s.0.iter().fold(Duration::ZERO, |acc, f| acc + f.duration);
                let mut drift = time.as_millis() - total.as_millis();
                for frame in s.0.iter() {
                    drift -= frame.duration.as_millis();
                    if drift <= 0 {
                        return frame.indices.clone();
                    }
                }

                unreachable!()
            }
        }
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>, size: usize) -> Result<Self, Box<dyn Error>> {
        match reader.read_i32::<LittleEndian>()? {
            0 => Ok(Skin::Static(StaticSkin::deserialize(reader, size)?)),
            1 => Ok(Skin::Animated(AnimatedSkin::deserialize(reader, size)?)),
            ty => Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid skin type: {}", ty),
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StaticSkin(Box<[u8]>);

impl StaticSkin {
    fn deserialize(reader: &mut Cursor<Vec<u8>>, size: usize) -> Result<Self, Box<dyn Error>> {
        let mut indices = vec![0u8; size];
        reader.read_exact(&mut indices)?;

        Ok(Self(indices.into_boxed_slice()))
    }
}

#[derive(Clone, Debug)]
pub struct AnimatedSkin(Box<[AnimatedSkinFrame]>);

impl AnimatedSkin {
    fn deserialize(reader: &mut Cursor<Vec<u8>>, size: usize) -> Result<Self, Box<dyn Error>> {
        let num_skin_frames = reader.read_f32::<LittleEndian>()? as usize;

        let mut durations = Vec::with_capacity(num_skin_frames);
        for _ in 0..num_skin_frames {
            let duration = reader.read_f32::<LittleEndian>()?;
            durations.push(Duration::from_micros((duration * 1_000_000.0) as u64));
        }

        let mut frames = Vec::with_capacity(num_skin_frames);
        for i in 0..num_skin_frames {
            let mut indices = vec![0u8; size];
            reader.read_exact(&mut indices)?;

            frames.push(AnimatedSkinFrame {
                duration: durations[i],
                indices: indices.into_boxed_slice(),
            });
        }

        Ok(Self(frames.into_boxed_slice()))
    }
}

#[derive(Clone, Debug)]
struct AnimatedSkinFrame {
    duration: Duration,
    indices: Box<[u8]>,
}

#[derive(Clone, Debug)]
struct SkinCoord {
    is_on_seam: bool,
    s: i32,
    t: i32,
}

impl SkinCoord {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let is_on_seam = match reader.read_i32::<LittleEndian>()? {
            0x20 => true,
            _ => false,
        };
        let s = reader.read_i32::<LittleEndian>()?;
        let t = reader.read_i32::<LittleEndian>()?;

        Ok(Self { is_on_seam, s, t })
    }
}

#[derive(Clone, Debug)]
struct Triangle {
    faces_front: bool,
    indices: [u32; 3],
}

impl Triangle {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let faces_front = match reader.read_i32::<LittleEndian>()? {
            1 => true,
            _ => false,
        };

        let mut indices = [0; 3];
        for i in 0..3 {
            indices[i] = reader.read_i32::<LittleEndian>()? as u32;
        }

        Ok(Self {
            faces_front,
            indices,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Keyframe {
    Static(StaticKeyframe),
    Animated(AnimatedKeyframe),
}

impl Keyframe {
    fn frame(&self, time: &Duration) -> Box<&Frame> {
        match *self {
            Keyframe::Static(ref kf) => Box::new(&kf.0),
            Keyframe::Animated(ref kf) => {
                let total = kf
                    .subframes
                    .iter()
                    .fold(Duration::ZERO, |acc, f| acc + f.duration);
                let mut drift = time.as_millis() - total.as_millis();
                for frame in kf.subframes.iter() {
                    drift -= frame.duration.as_millis();
                    if drift <= 0 {
                        return Box::new(&frame.frame);
                    }
                }

                unreachable!()
            }
        }
    }

    fn deserialize(
        reader: &mut Cursor<Vec<u8>>,
        num_verts: i32,
        scale: [f32; 3],
        origin: [f32; 3],
    ) -> Result<Self, Box<dyn Error>> {
        match reader.read_i32::<LittleEndian>()? {
            0 => Ok(Keyframe::Static(StaticKeyframe(Frame::deserialize(
                reader, num_verts, scale, origin,
            )?))),
            1 => Ok(Keyframe::Animated(AnimatedKeyframe::deserialize(
                reader, num_verts, scale, origin,
            )?)),
            ty => Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid frame type: {}", ty),
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StaticKeyframe(pub Frame);

#[derive(Clone, Debug)]
pub struct AnimatedKeyframe {
    min: [f32; 3],
    max: [f32; 3],
    subframes: Box<[AnimatedKeyframeFrame]>,
}

impl AnimatedKeyframe {
    fn deserialize(
        reader: &mut Cursor<Vec<u8>>,
        num_verts: i32,
        scale: [f32; 3],
        origin: [f32; 3],
    ) -> Result<Self, Box<dyn Error>> {
        let num_subframes = reader.read_i32::<LittleEndian>()? as usize;

        let min = Vertex1XYZ1N1UV::read_packed_position(reader, scale, origin)?;
        reader.read_u8()?;
        let max = Vertex1XYZ1N1UV::read_packed_position(reader, scale, origin)?;
        reader.read_u8()?;

        let mut durations = Vec::with_capacity(num_subframes);
        for _ in 0..num_subframes {
            let duration = reader.read_f32::<LittleEndian>()?;
            durations.push(Duration::from_micros((duration * 1_000_000.0) as u64));
        }

        let mut subframes = Vec::with_capacity(num_subframes);
        for i in 0..num_subframes {
            let frame = Frame::deserialize(reader, num_verts, scale, origin)?;
            subframes.push(AnimatedKeyframeFrame {
                duration: durations[i],
                frame,
            });
        }

        Ok(Self {
            min,
            max,
            subframes: subframes.into_boxed_slice(),
        })
    }
}

#[derive(Clone, Debug)]
struct AnimatedKeyframeFrame {
    duration: Duration,
    frame: Frame,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub name: String,
    min: [f32; 3],
    max: [f32; 3],
    vertices: Box<[[f32; 3]]>,
}

impl Frame {
    fn deserialize(
        reader: &mut Cursor<Vec<u8>>,
        num_verts: i32,
        scale: [f32; 3],
        origin: [f32; 3],
    ) -> Result<Self, Box<dyn Error>> {
        let min = Vertex1XYZ1N1UV::read_packed_position(reader, scale, origin)?;
        reader.read_u8()?;
        let max = Vertex1XYZ1N1UV::read_packed_position(reader, scale, origin)?;
        reader.read_u8()?;

        let mut name_buf = [0u8; 16];
        reader.read_exact(&mut name_buf)?;

        let name = {
            let len = name_buf.iter().position(|b| *b == 0).unwrap();
            String::from_utf8_lossy(&name_buf[..len])
        }
        .to_string();

        let mut vertices = Vec::with_capacity(num_verts as usize);
        for _ in 0..num_verts {
            vertices.push(Vertex1XYZ1N1UV::read_packed_position(
                reader, scale, origin,
            )?);
            reader.read_u8()?;
        }

        Ok(Self {
            name,
            min,
            max,
            vertices: vertices.into_boxed_slice(),
        })
    }
}

impl Vertex1XYZ1N1UV {
    fn read_packed_position(
        reader: &mut Cursor<Vec<u8>>,
        scale: [f32; 3],
        origin: [f32; 3],
    ) -> Result<[f32; 3], Box<dyn Error>> {
        Ok([
            reader.read_u8()? as f32 * scale[0] + origin[0],
            reader.read_u8()? as f32 * scale[1] + origin[1],
            reader.read_u8()? as f32 * scale[2] + origin[2],
        ])
    }
}
