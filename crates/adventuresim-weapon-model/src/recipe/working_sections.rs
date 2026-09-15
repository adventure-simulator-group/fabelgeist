//! Default working sections shared by construction and contact mechanics.
use super::*;
impl BeakParameters {
    pub fn working_root(&self) -> Metres {
        self.root_section
            .unwrap_or_else(|| Metres::new(self.radius.get() * 1.5).expect("validated beak radius"))
    }
    pub fn working_thickness(&self) -> Metres {
        self.thickness.unwrap_or_else(|| {
            Metres::new(self.radius.get() * 0.75).expect("validated beak radius")
        })
    }
}
impl ForkParameters {
    pub fn working_tine_width(&self) -> Metres {
        self.tine_width
            .unwrap_or_else(|| Metres::new(self.width.get() * 0.18).expect("validated fork width"))
    }
}
impl PickParameters {
    pub fn working_diameter(&self) -> Metres {
        Metres::new(self.radius.get() * 2.0).expect("validated pick radius")
    }
}
