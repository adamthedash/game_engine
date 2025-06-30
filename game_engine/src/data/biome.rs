use enum_map::Enum;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Enum)]
pub enum Biome {
    DirtLand,
    StoneLand,
    DenseCaves,
}
