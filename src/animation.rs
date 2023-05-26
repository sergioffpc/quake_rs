use std::{collections::HashMap, time::Duration};

use crate::mesh::Vertex1XYZ1N1UV;

pub struct KeyframeAnimationComponent {
    pub animations: HashMap<String, Animation>,
    pub current_animation: Option<String>,
}

impl KeyframeAnimationComponent {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            current_animation: None,
        }
    }

    pub fn animate(&self, time: &Duration) -> Option<Vec<Vertex1XYZ1N1UV>> {
        let k = self.current_animation.as_ref()?;
        self.animations.get(k)?.animate(time)
    }
}

pub struct Animation {
    keyframes: Vec<Keyframe>,
}

impl Animation {
    pub fn new() -> Self {
        Animation {
            keyframes: Vec::new(),
        }
    }

    pub fn add_keyframe(&mut self, vertices: Vec<Vertex1XYZ1N1UV>, duration: Duration) {
        let keyframe = Keyframe { vertices, duration };
        self.keyframes.push(keyframe);
    }

    pub fn animate(&self, time: &Duration) -> Option<Vec<Vertex1XYZ1N1UV>> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Find the current keyframes based on the given time
        let (prev_keyframe, next_keyframe) = self.find_keyframes(time);

        // Interpolate the animation state between the keyframes
        let vertices = self.interpolate(prev_keyframe, next_keyframe, time);

        Some(vertices)
    }

    fn find_keyframes(&self, time: &Duration) -> (&Keyframe, &Keyframe) {
        // Find the previous and next keyframes based on the given time
        // You can use different algorithms to find the keyframes, such as binary search
        // Here, a simple linear search is shown for demonstration purposes
        let mut prev_keyframe = &self.keyframes[0];
        let mut next_keyframe = &self.keyframes[0];

        for keyframe in &self.keyframes {
            if keyframe.duration.as_millis() <= time.as_millis() {
                prev_keyframe = keyframe;
            } else {
                next_keyframe = keyframe;
                break;
            }
        }

        (prev_keyframe, next_keyframe)
    }

    fn interpolate(
        &self,
        prev_keyframe: &Keyframe,
        next_keyframe: &Keyframe,
        time: &Duration,
    ) -> Vec<Vertex1XYZ1N1UV> {
        // Interpolate the animation state between the keyframes based on the time
        // Perform interpolation for each property of the animation state

        // Calculate the interpolation factor (e.g., linear interpolation)
        let t = (time.as_millis() - prev_keyframe.duration.as_millis())
            / (next_keyframe.duration.as_millis() - prev_keyframe.duration.as_millis());

        // Interpolate other properties of the animation state

        // Return the interpolated animation state
        vec![]
    }
}

pub struct Keyframe {
    vertices: Vec<Vertex1XYZ1N1UV>,
    duration: Duration,
}
