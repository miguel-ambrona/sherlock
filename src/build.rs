// This file generates the zobrist_gen.rs, filled with random constants used for
// Zobrist hashing. This code is mainly taken from:
// https://github.com/jordanbray/chess/blob/main/src/gen_tables/zobrist.rs

use std::{env, fs::File, io::Write, path::Path};

use rand::{rngs::SmallRng, RngCore, SeedableRng};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let zobrist_path = Path::new(&out_dir).join("zobrist_gen.rs");
    let mut z = File::create(zobrist_path).unwrap();

    write_zobrist(&mut z);
}

const NUM_COLORS: usize = 2;
const NUM_FILES: usize = 8;
const NUM_PIECES: usize = 6;
const NUM_SQUARES: usize = 64;

/// Write the ZOBRIST_* arrays to a file.
pub fn write_zobrist(f: &mut File) {
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF12345678);

    write!(
        f,
        "use chess::{{NUM_COLORS, NUM_FILES, NUM_PIECES, NUM_SQUARES}};\n\n"
    )
    .unwrap();

    write!(f, "const SIDE_TO_MOVE: u64 = {};\n\n", rng.next_u64()).unwrap();

    writeln!(
        f,
        "const ZOBRIST_PIECES: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS] = [[["
    )
    .unwrap();
    for i in 0..NUM_COLORS {
        for j in 0..NUM_PIECES {
            for _ in 0..NUM_SQUARES {
                writeln!(f, "    {},", rng.next_u64()).unwrap();
            }
            if j != NUM_PIECES - 1 {
                writeln!(f, "   ], [").unwrap();
            }
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "  ]], [[").unwrap();
        }
    }
    write!(f, "]]];\n\n").unwrap();

    writeln!(f, "const ZOBRIST_CASTLES: [[u64; 4]; NUM_COLORS] = [[").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..4 {
            writeln!(f, "    {},", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "  ], [").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();

    writeln!(f, "const ZOBRIST_EP: [[u64; NUM_FILES]; NUM_COLORS] = [[").unwrap();
    for i in 0..NUM_COLORS {
        for _ in 0..NUM_FILES {
            writeln!(f, "    {},", rng.next_u64()).unwrap();
        }
        if i != NUM_COLORS - 1 {
            writeln!(f, "], [").unwrap();
        }
    }
    write!(f, "]];\n\n").unwrap();

    write!(f, "const ZOBRIST_EP_ANY: u64 = {};\n\n", rng.next_u64()).unwrap();
}
