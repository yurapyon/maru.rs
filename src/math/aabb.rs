use nalgebra_glm as glm;
use nalgebra::{
    Scalar,
    ClosedAdd,
    ClosedSub,
};
use num_traits::{
    ToPrimitive,
};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// An AABB rectangle.
pub struct AABB<T: Scalar> {
    pub c1: glm::TVec2<T>,
    pub c2: glm::TVec2<T>,
}

impl<T: Scalar> AABB<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Self {
        Self {
            c1: glm::vec2(x1, y1),
            c2: glm::vec2(x2, y2),
        }
    }
}

impl<T: Scalar + ClosedSub + Copy> AABB<T> {
    // TODO height and width should do absolute value ?
    #[inline]
    pub fn width(&self) -> T {
        self.c2.x - self.c1.x
    }

    #[inline]
    pub fn height(&self) -> T {
        self.c2.y - self.c1.y
    }
}

impl<T: Scalar + ToPrimitive> AABB<T> {
    pub fn normalized(&self, vec: glm::TVec2<T>) -> AABB<f32> {
        let x = vec.x.to_f32().unwrap();
        let y = vec.y.to_f32().unwrap();
        let x1 = self.c1.x.to_f32().unwrap() / x;
        let y1 = self.c1.y.to_f32().unwrap() / y;
        let x2 = self.c2.x.to_f32().unwrap() / x;
        let y2 = self.c2.y.to_f32().unwrap() / y;
        AABB::new(x1, y1, x2, y2)
    }
}

impl<T: Scalar + PartialOrd> AABB<T> {
    pub fn reorient(&mut self) {
        use std::mem;

        if self.c1.x > self.c2.x {
            mem::swap(&mut self.c1.x, &mut self.c2.x);
        };

        if self.c1.y > self.c2.y {
            mem::swap(&mut self.c1.y, &mut self.c2.y);
        }
    }
}

impl<T: Scalar + ClosedAdd + Copy> AABB<T> {
    pub fn displace(&mut self, offset: glm::TVec2<T>) {
        self.c1 += offset;
        self.c2 += offset;
    }
}

/*
/// Slices to a vec as to be less complex
impl<T: Scalar + ClosedAdd + PartialOrd + Copy> AABB<T> {
    pub fn slice_up(&self, width: T, height: T) -> Vec<AABB<T>> {
        if self.c2.y <= self.c1.y ||
            self.c2.x <= self.c1.x {
            panic!("AABB must be normalized to take make slices of it");
        }

        let mut regions = Vec::new();

        let mut y = self.c1.y;
        while y <= self.c2.y {
            let mut x = self.c1.x;
            while x <= self.c2.x {
                regions.push(AABB::new(x, y,
                                       x + width,
                                       y + height));
                x += width;
            }
            y += height;
        }

        // TODO panic of slices are night aligned properly
        //   right now just ignores them

        regions
    }
}
*/
