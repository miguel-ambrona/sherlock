//! We say a set of at least k sets is a k-group iff their union results
//! in at most k elements.
//!
//! The notion of k-group is a powerful concept which is the basis of rules like
//! `refine_origins`.

use chess::{BitBoard, EMPTY};

/// On input `k`, an array of sets (64 sets, one for each square on the board)
/// and a set of (square) indices given in the form of `BitBoard`, this function
/// finds a `k`-group and returns its union set and a set of the indices that
/// *do not* form the k-group.
///
/// This function returns `None` iff no k-group exists in the given sets
/// filtered by the given indices.
pub fn find_k_group(
    k: usize,
    sets: &[BitBoard; 64],
    indices: BitBoard,
) -> Option<(BitBoard, BitBoard)> {
    find_k_group_recursively(k, sets, indices, (EMPTY, 0))
}

fn find_k_group_recursively(
    k: usize,
    sets: &[BitBoard; 64],

    remaining_indices: BitBoard,
    group: (BitBoard, usize),
) -> Option<(BitBoard, BitBoard)> {
    if group.0.popcnt() as usize > k {
        return None;
    }
    if group.1 >= k {
        return Some((group.0, remaining_indices));
    }
    let mut remaining = remaining_indices.into_iter();
    if let Some(square) = remaining.next() {
        let union_set = group.0 | sets[square.to_index()];
        match find_k_group_recursively(k, sets, remaining, (union_set, group.1 + 1)) {
            Some(g) => Some(g),
            None => find_k_group_recursively(k, sets, remaining, group)
                .map(|(group, indices)| (group, indices | BitBoard::from_square(square))),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use chess::EMPTY;

    use super::*;
    use crate::utils::*;

    #[test]
    fn test_find_k_group() {
        let mut sets = [!EMPTY; 64];
        sets[0] = bitboard_of_squares(&[A1, A2]);
        sets[1] = bitboard_of_squares(&[A3]);
        sets[2] = bitboard_of_squares(&[A1, A2]);
        sets[3] = bitboard_of_squares(&[A2]);
        sets[4] = bitboard_of_squares(&[A1, A3, A4]);

        assert_eq!(find_k_group(1, &sets, BitBoard(1)), None);
        assert_eq!(
            find_k_group(1, &sets, BitBoard(63)),
            Some((sets[1], BitBoard(63 - 2)))
        );
        assert_eq!(
            find_k_group(2, &sets, BitBoard(63)),
            Some((sets[0] | sets[2], BitBoard(63 - 1 - 4)))
        );
        assert_eq!(
            find_k_group(3, &sets, BitBoard(63)),
            Some((sets[0] | sets[1] | sets[2], BitBoard(63 - 1 - 2 - 4)))
        );

        sets[0] = bitboard_of_squares(&[B1, B2, B3]);
        sets[1] = bitboard_of_squares(&[B2, B3, B4]);
        sets[2] = bitboard_of_squares(&[B2, B3, B4]);
        sets[3] = bitboard_of_squares(&[B1, H8]);
        sets[4] = bitboard_of_squares(&[B1, B2, B4]);

        assert_eq!(
            find_k_group(4, &sets, BitBoard(63)),
            Some((
                sets[0] | sets[1] | sets[2] | sets[4],
                BitBoard(63 - 1 - 2 - 4 - 16)
            ))
        );
        assert_eq!(
            find_k_group(5, &sets, BitBoard(31)),
            Some((sets[0] | sets[1] | sets[2] | sets[3] | sets[4], BitBoard(0)))
        );

        sets[3] = bitboard_of_squares(&[B1, H8, G8]);
        assert_eq!(find_k_group(5, &sets, BitBoard(31)), None);
    }
}
