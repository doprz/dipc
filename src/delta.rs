use deltae::LabValue;

#[derive(Debug, Clone, Copy)]
pub struct Lab {
    l: f32,
    a: f32,
    b: f32,
}

impl From<[u8; 3]> for Lab {
    fn from(value: [u8; 3]) -> Self {
        let lab::Lab { l, a, b } = lab::Lab::from_rgb(&value);
        Lab { l: l, a: a, b: b }
    }
}

impl From<[u8; 4]> for Lab {
    fn from(value: [u8; 4]) -> Self {
        let lab::Lab { l, a, b } = lab::Lab::from_rgba(&value);
        Lab { l: l, a: a, b: b }
    }
}

impl From<Lab> for LabValue {
    fn from(lab: Lab) -> Self {
        LabValue {
            l: lab.l,
            a: lab.a,
            b: lab.b,
        }
    }
}

impl Lab {
    pub fn to_nearest_pallete(self, pallete: &[Lab]) -> Self {
        let mut min_distance = std::f32::MAX;
        let mut new_color = self;

        for &color in pallete {
            let delta = *deltae::DeltaE::new(self, color, deltae::DEMethod::DE2000).value();

            if delta < min_distance {
                min_distance = delta;
                new_color = color;
            }
        }

        new_color
    }

    pub fn to_rgb(self) -> [u8; 3] {
        let lab = lab::Lab {
            l: self.l,
            a: self.a,
            b: self.b,
        };
        lab.to_rgb()
    }
}

// Implement DeltaEq for Lab
impl<D: deltae::Delta + Copy> deltae::DeltaEq<D> for Lab {}
