#![feature(min_generic_const_args)]
#![allow(incomplete_features)]

pub trait EnumDiscriminant {
    #[type_const]
    const COUNT: usize;
    fn discriminant(&self) -> usize;
}
