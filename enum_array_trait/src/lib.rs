#![feature(min_generic_const_args)]

pub trait EnumDiscriminant {
    #[type_const]
    const COUNT: usize;
    fn discriminant(&self) -> usize;
}
