#![feature(min_generic_const_args)]

use std::ops::{Index, IndexMut};

pub use enum_array_derive;
pub use enum_array_trait;

use enum_array_trait::EnumDiscriminant;

#[derive(Debug)]
pub struct EnumArray<T, E: EnumDiscriminant>(pub [T; E::COUNT])
where
    [(); E::COUNT]:;

impl<T, E: EnumDiscriminant> IndexMut<&E> for EnumArray<T, E>
where
    [(); E::COUNT]:,
{
    fn index_mut(&mut self, index: &E) -> &mut Self::Output {
        &mut self.0[index.discriminant()]
    }
}

impl<T, E: EnumDiscriminant> Index<&E> for EnumArray<T, E>
where
    [(); E::COUNT]:,
{
    type Output = T;

    fn index(&self, index: &E) -> &Self::Output {
        &self.0[index.discriminant()]
    }
}

#[cfg(test)]
mod test {

    use std::fmt::Display;

    use crate::{
        EnumArray, EnumDiscriminant, enum_array_derive::EnumDiscriminant as EnumDiscriminantMacro,
    };

    #[derive(EnumDiscriminantMacro, Debug)]
    enum Test {
        One,
        Two(i32),
        Three { a: i32, b: usize },
    }

    #[test]
    fn test_disc() {
        let x = Test::One;
        assert_eq!(x.discriminant(), 0);
        let x = Test::Three { a: 1, b: 0 };
        assert_eq!(x.discriminant(), 2);
    }

    #[test]
    fn test_array() {
        struct MyArr(EnumArray<Vec<Box<dyn Display>>, Test>);

        let arr = EnumArray::<usize, Test>([0, 1, 2]);
        println!("{:?}", arr);

        let x = arr[Test::Three { a: 1, b: 0 }];
        assert_eq!(x, 2);
    }
}
