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
        Lab { l, a, b }
    }
}

impl From<[u8; 4]> for Lab {
    fn from(value: [u8; 4]) -> Self {
        let lab::Lab { l, a, b } = lab::Lab::from_rgba(&value);
        Lab { l, a, b }
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
    pub fn to_nearest_palette(self, palette: &[Lab], method: deltae::DEMethod) -> Self {
        let mut min_distance = std::f32::MAX;
        let mut new_color = self;

        for &color in palette {
            // let delta = *deltae::DeltaE::new(self, color, deltae::DEMethod::DE2000).value();
            let delta = *deltae::DeltaE::new(self, color, method).value();

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum CLIDEMethod {
    /// The default DeltaE method
    DE2000,
    // /// An implementation of DeltaE with separate tolerances for Lightness and Chroma
    // DECMC(
    //     /// Lightness tolerance
    //     f32,
    //     /// Chroma tolerance
    //     f32,
    // ),
    /// CIE94 DeltaE implementation, weighted with a tolerance for graphics
    DE1994G,
    /// CIE94 DeltaE implementation, weighted with a tolerance for textiles
    DE1994T,
    /// The original DeltaE implementation, a basic euclidian distance formula
    DE1976,
}

impl Default for CLIDEMethod {
    fn default() -> Self {
        Self::DE2000
    }
}

impl std::fmt::Display for CLIDEMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CLIDEMethod::DE2000 => write!(f, "de2000"),
            // CLIDEMethod::DECMC(l, c) => write!(f, "decmc({}, {})", l, c),
            CLIDEMethod::DE1994G => write!(f, "de1994g"),
            CLIDEMethod::DE1994T => write!(f, "de1994t"),
            CLIDEMethod::DE1976 => write!(f, "de1976"),
        }
    }
}

impl From<CLIDEMethod> for deltae::DEMethod {
    fn from(method: CLIDEMethod) -> Self {
        match method {
            CLIDEMethod::DE2000 => Self::DE2000,
            // CLIDEMethod::DECMC(l, c) => Self::DECMC(l, c),
            CLIDEMethod::DE1994G => Self::DE1994G,
            CLIDEMethod::DE1994T => Self::DE1994T,
            CLIDEMethod::DE1976 => Self::DE1976,
        }
    }
}
