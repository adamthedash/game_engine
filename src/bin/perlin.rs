use game_engine::world_gen::Perlin;

fn main() {
    let generator = Perlin::new(42, 3, 0.5, 2., 1.);
}
