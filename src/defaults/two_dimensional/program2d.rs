use crate::{
    gfx::{
        Location,
        Program,
    },
};

use super::Locations;

//

pub struct Program2d {
    pub(crate) prog: Program,
    pub(crate) locs: Locations,
}

impl Program2d {
    fn from(prog: Program) -> Self {
        Self {
            locs: Locations::new(&prog),
            prog,
        }
    }

    // TODO rename
    pub fn new_normal(v_effect: Option<&str>, f_effect: Option<&str>) -> Result<Self, String> {
        super::default_program(v_effect, f_effect).map(Self::from)
    }

    pub fn default_program() -> Self {
        Self::from(super::default_program(None, None).unwrap())
    }

    pub fn default_spritebatch_program() -> Self {
        Self::from(super::default_spritebatch_program(None, None).unwrap())
    }

    pub fn get_location(&self, name: &str) -> Location {
        Location::new(&self.prog, name)
    }
}
