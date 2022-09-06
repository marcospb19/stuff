use std::env;

use rand::seq::SliceRandom; // 0.7.2

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();

    let chosen = args.choose(&mut rand::rngs::OsRng);

    if let Some(chosen) = chosen {
        println!("{chosen}");
    } else {
        println!("Argument list is empty.");
    }
}
