use chess::{
    between, get_adjacent_files, get_bishop_moves, get_king_moves, get_knight_moves,
    get_pawn_attacks, get_pawn_quiets, get_rank, get_rook_moves, line, BitBoard, CastleRights,
    Color, File, Piece, Rank, Square, EMPTY,
};

use super::retraction_gen::{RetractionList, SourceAndTargets, UnCaptureKind};
use crate::{utils::is_attacked, EnPassantFlag, RetractableBoard};

pub trait PieceType {
    fn into_piece() -> Piece;
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard;

    /// This blanket implementation is for sliding pieces only.
    /// Kings, knights and pawns will reimplement their own [legals] function.
    #[inline(always)]
    fn legals<T>(movelist: &mut RetractionList, board: &RetractableBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let retracting_color = !board.side_to_move();
        let retracting_pieces = board.color_combined(retracting_color);
        let opp_ksq = board.king_square(!retracting_color);

        let pieces = board.pieces(Self::into_piece()) & retracting_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let capture_kind = |src: Square| {
            if BitBoard::from_square(src) & pinned != EMPTY {
                UnCaptureKind::Necessary
            } else {
                UnCaptureKind::Optional
            }
        };

        let mut castling_rooks = EMPTY;
        if board.castle_rights(retracting_color).has_kingside() {
            castling_rooks ^= BitBoard::from_square(Square::make_square(
                retracting_color.to_my_backrank(),
                File::H,
            ))
        }
        if board.castle_rights(retracting_color).has_queenside() {
            castling_rooks ^= BitBoard::from_square(Square::make_square(
                retracting_color.to_my_backrank(),
                File::A,
            ))
        }

        if T::NB_CHECKERS == 0 {
            // the retracting player must not check their opponent after the retraction
            let check_mask = Self::pseudo_legals(opp_ksq, retracting_color, *combined, !EMPTY);
            for src in pieces & !castling_rooks {
                let targets =
                    Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                        & !check_mask;
                if targets != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SourceAndTargets::new(
                            src,
                            targets,
                            capture_kind(src),
                            false,
                        ));
                    }
                }
            }
        }

        if T::NB_CHECKERS == 1 && checkers & pieces != EMPTY {
            // a piece of our own type is checking, thus it must be the retracting piece
            let src = checkers.to_square();
            let check_mask =
                Self::pseudo_legals(opp_ksq, retracting_color, *combined & !checkers, !EMPTY);
            let targets_with_optional_capture =
                Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                    & !check_mask;
            if targets_with_optional_capture != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets_with_optional_capture,
                        capture_kind(src),
                        false,
                    ));
                }
            }

            let check_mask = Self::pseudo_legals(opp_ksq, retracting_color, *combined, !EMPTY);
            let targets_with_necessary_capture = line(opp_ksq, src)
                & Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                & !check_mask;
            if targets_with_necessary_capture != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets_with_necessary_capture,
                        UnCaptureKind::Necessary,
                        false,
                    ));
                }
            }
        }

        if T::NB_CHECKERS == 1 && Self::into_piece() != Piece::Queen && checkers & pieces == EMPTY {
            // a different piece is checking, thus we must have moved from
            // the checking ray (if we are not a queen)
            for src in pieces & !castling_rooks {
                let targets =
                    Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                        & between(checkers.to_square(), opp_ksq);
                if targets != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SourceAndTargets::new(
                            src,
                            targets,
                            capture_kind(src),
                            false,
                        ));
                    }
                }
            }
        }

        // double checks
        if T::NB_CHECKERS == 2 && checkers & pieces != EMPTY && checkers & !pieces != EMPTY {
            let src = (checkers & pieces & !castling_rooks).to_square();
            let targets = between((checkers & !pieces).to_square(), opp_ksq)
                & Self::pseudo_legals(src, retracting_color, *combined, !combined & mask);
            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        capture_kind(src),
                        false,
                    ));
                }
            }
        }
    }
}

pub struct PawnType;
pub struct BishopType;
pub struct KnightType;
pub struct RookType;
pub struct QueenType;
pub struct KingType;

pub trait CheckType {
    const NB_CHECKERS: u8;
}

pub struct NotInCheck;
pub struct InSimpleCheck;
pub struct InDoubleCheck;

impl CheckType for NotInCheck {
    const NB_CHECKERS: u8 = 0;
}

impl CheckType for InSimpleCheck {
    const NB_CHECKERS: u8 = 1;
}

impl CheckType for InDoubleCheck {
    const NB_CHECKERS: u8 = 2;
}

impl PieceType for PawnType {
    fn into_piece() -> Piece {
        Piece::Pawn
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        (get_pawn_attacks(src, !color, !combined) ^ get_pawn_quiets(src, !color, combined)) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut RetractionList, board: &RetractableBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let retracting_color = !board.side_to_move();
        let retracting_pieces = board.color_combined(retracting_color);
        let opp_ksq = board.king_square(!retracting_color);

        let pieces = board.pieces(Self::into_piece()) & retracting_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        // the retracting player must not check their opponent after the retraction
        let check_mask = get_pawn_attacks(opp_ksq, !retracting_color, !EMPTY);
        let first_rank = get_rank(retracting_color.to_my_backrank());
        let last_rank = get_rank(retracting_color.to_their_backrank());
        let candidate_retractors =
            if T::NB_CHECKERS >= 1 && (checkers & (pieces | last_rank)) != EMPTY {
                checkers & (pieces | last_rank)
            } else if T::NB_CHECKERS <= 1 {
                pieces
                    | (last_rank
                        & retracting_pieces
                        & !BitBoard::from_square(board.king_square(retracting_color)))
            } else {
                EMPTY
            };
        for src in candidate_retractors {
            let other_checker_ray =
                if T::NB_CHECKERS == 0 || (T::NB_CHECKERS == 1 && checkers.to_square() == src) {
                    !EMPTY
                } else {
                    let checker = (checkers & !BitBoard::from_square(src)).to_square();
                    between(checker, opp_ksq)
                };

            // pawn unpushes
            let mut targets = BitBoard::from_square(src.ubackward(retracting_color));
            if src.get_rank() == retracting_color.to_fourth_rank()
                && board.en_passant() == EnPassantFlag::Any
            {
                targets |= BitBoard::from_square(
                    src.ubackward(retracting_color).ubackward(retracting_color),
                );
            }
            targets &= !combined & !check_mask & !first_rank & other_checker_ray & mask;
            if BitBoard::from_square(src) & pinned != EMPTY {
                targets &= line(src, opp_ksq)
            };
            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        UnCaptureKind::Forbidden,
                        src.get_rank() == retracting_color.to_their_backrank(),
                    ));
                }
            }

            // pawn uncaptures
            let targets = get_pawn_attacks(src, !retracting_color, !combined)
                & !check_mask
                & !first_rank
                & other_checker_ray
                & mask;
            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        UnCaptureKind::Necessary,
                        src.get_rank() == retracting_color.to_their_backrank(),
                    ));
                }
            }
        }

        // en-passant retractions
        let ep_rank = match retracting_color {
            Color::White => get_rank(Rank::Sixth),
            Color::Black => get_rank(Rank::Third),
        };
        for src in ep_rank & pieces {
            let reappearing_pawn_square = src.ubackward(retracting_color);
            if BitBoard::from_square(src.uforward(retracting_color)) & combined != EMPTY
                || BitBoard::from_square(reappearing_pawn_square) & combined != EMPTY
            {
                continue;
            }
            let mut targets = get_adjacent_files(src.get_file())
                & get_rank(reappearing_pawn_square.get_rank())
                & !combined
                & !check_mask
                & mask;
            if BitBoard::from_square(src) & pinned != EMPTY
                && BitBoard::from_square(reappearing_pawn_square) & line(src, opp_ksq) == EMPTY
            {
                targets &= line(src, opp_ksq)
            };

            if T::NB_CHECKERS == 1 && checkers.to_square() != src {
                let checking_ray = between(checkers.to_square(), opp_ksq);
                if BitBoard::from_square(reappearing_pawn_square) & checking_ray == EMPTY {
                    targets &= checking_ray;
                }
            } else if T::NB_CHECKERS == 2 {
                if BitBoard::from_square(src) & checkers != EMPTY {
                    let other_checker = (checkers & !BitBoard::from_square(src)).to_square();
                    if BitBoard::from_square(reappearing_pawn_square)
                        & between(other_checker, opp_ksq)
                        != EMPTY
                    {
                        targets &= between(other_checker, opp_ksq);
                    }
                } else {
                    // There are two officers checking, we must block both rays.
                    let mut rays = EMPTY;
                    for checker in *checkers {
                        rays |= between(checker, opp_ksq);
                    }
                    if BitBoard::from_square(reappearing_pawn_square) & rays == EMPTY {
                        targets = EMPTY;
                    }
                    targets &= rays;
                }
            }

            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        UnCaptureKind::UnEnPassant,
                        false,
                    ));
                }
            }
        }
    }
}

impl PieceType for BishopType {
    fn into_piece() -> Piece {
        Piece::Bishop
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_bishop_moves(src, combined) & mask
    }
}

impl PieceType for KnightType {
    fn into_piece() -> Piece {
        Piece::Knight
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut RetractionList, board: &RetractableBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let retracting_color = !board.side_to_move();
        let retracting_pieces = board.color_combined(retracting_color);
        let opp_ksq = board.king_square(!retracting_color);

        let pieces = board.pieces(Self::into_piece()) & retracting_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let capture_kind = |src: Square| {
            if BitBoard::from_square(src) & pinned != EMPTY {
                UnCaptureKind::Necessary
            } else {
                UnCaptureKind::Optional
            }
        };

        if T::NB_CHECKERS == 0 {
            // the retracting player must not check their opponent after the retraction
            let check_mask = Self::pseudo_legals(opp_ksq, retracting_color, *combined, !EMPTY);
            for src in pieces {
                let targets =
                    Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                        & !check_mask;
                if targets != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SourceAndTargets::new(
                            src,
                            targets,
                            capture_kind(src),
                            false,
                        ));
                    }
                }
            }
        }

        if T::NB_CHECKERS == 1 && checkers & pieces != EMPTY {
            // a knight is checking, thus it must be the retracting piece
            let src = checkers.to_square();
            let targets = Self::pseudo_legals(src, retracting_color, *combined, !combined & mask);
            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        UnCaptureKind::Optional,
                        false,
                    ));
                }
            }
        }

        if T::NB_CHECKERS == 1 && checkers & pieces == EMPTY {
            // a different piece is checking, thus we must have moved from
            // the checking ray
            for src in pieces {
                let targets =
                    Self::pseudo_legals(src, retracting_color, *combined, !combined & mask)
                        & between(checkers.to_square(), opp_ksq);
                if targets != EMPTY {
                    unsafe {
                        movelist.push_unchecked(SourceAndTargets::new(
                            src,
                            targets,
                            capture_kind(src),
                            false,
                        ));
                    }
                }
            }
        }

        // double checks
        if T::NB_CHECKERS == 2 && checkers & pieces != EMPTY && checkers & !pieces != EMPTY {
            let src = (checkers & pieces).to_square();
            let targets = between((checkers & !pieces).to_square(), opp_ksq)
                & Self::pseudo_legals(src, retracting_color, *combined, !combined & mask);
            if targets != EMPTY {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        targets,
                        UnCaptureKind::Optional,
                        false,
                    ));
                }
            }
        }
    }
}

impl PieceType for RookType {
    fn into_piece() -> Piece {
        Piece::Rook
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_rook_moves(src, combined) & mask
    }
}

impl PieceType for QueenType {
    fn into_piece() -> Piece {
        Piece::Queen
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, combined: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, combined) ^ get_bishop_moves(src, combined)) & mask
    }
}
impl PieceType for KingType {
    fn into_piece() -> Piece {
        Piece::King
    }

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _combined: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(movelist: &mut RetractionList, board: &RetractableBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let combined = board.combined();
        let retracting_color = !board.side_to_move();
        let src = board.king_square(retracting_color);
        let opp_ksq = board.king_square(!retracting_color);
        let pinned = board.pinned();
        let my_pieces = board.color_combined(retracting_color);

        if board.castle_rights(retracting_color) != CastleRights::NoRights {
            return;
        }

        let mut targets = Self::pseudo_legals(src, retracting_color, *combined, mask)
            & !Self::pseudo_legals(opp_ksq, !retracting_color, *combined, mask)
            & !combined;

        if T::NB_CHECKERS == 1 {
            targets &= between(board.checkers().to_square(), opp_ksq);
        }

        let targets_with_optional_uncapture = if BitBoard::from_square(src) & pinned != EMPTY {
            targets & between(src, opp_ksq)
        } else {
            targets
        };
        let targets_with_necessary_uncapture = targets & !targets_with_optional_uncapture;

        if targets_with_optional_uncapture != EMPTY {
            unsafe {
                movelist.push_unchecked(SourceAndTargets::new(
                    src,
                    targets_with_optional_uncapture,
                    UnCaptureKind::Optional,
                    false,
                ));
            }
        }

        if targets_with_necessary_uncapture != EMPTY {
            unsafe {
                movelist.push_unchecked(SourceAndTargets::new(
                    src,
                    targets_with_necessary_uncapture,
                    UnCaptureKind::Necessary,
                    false,
                ));
            }
        }

        // We may uncastle iff:
        //  * we are at an after-castle position
        //  * the squares between the king and future rook (inclusive) are empty
        //  * the current rook square is not attacked
        //  * the future king square is not attacked
        //  * the future rook will not check the opponent after uncastle
        //  * the opponent is not in check except possibly by the uncastling rook
        if src.get_rank() == retracting_color.to_my_backrank() {
            let my_rooks = my_pieces & board.pieces(Piece::Rook);
            // short uncastle
            if src.get_file() == File::G
                && BitBoard::from_square(src.uleft()) & my_rooks != EMPTY // F
                && BitBoard::from_square(src.uleft().uleft()) & combined == EMPTY // E
                && BitBoard::from_square(src.uright()) & combined == EMPTY // H
                && !is_attacked(board, src.uleft(), !retracting_color) // F
                && !is_attacked(board, src.uleft().uleft(), !retracting_color) // E
                && get_rook_moves(opp_ksq, *combined) & BitBoard::from_square(src.uright()) == EMPTY // H
                && (T::NB_CHECKERS == 0
                    || board.checkers() & BitBoard::from_square(src.uleft()) != EMPTY)
            {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        BitBoard::from_square(src.uleft().uleft()),
                        UnCaptureKind::Forbidden,
                        false,
                    ));
                }
            }
            // long uncastle
            else if src.get_file() == File::C
                && BitBoard::from_square(src.uright()) & my_rooks != EMPTY // D
                && BitBoard::from_square(src.uright().uright()) & combined == EMPTY // E
                && BitBoard::from_square(src.uleft()) & combined == EMPTY // B
                && BitBoard::from_square(src.uleft().uleft()) & combined == EMPTY // A
                && !is_attacked(board, src.uright(), board.side_to_move()) // D
                && !is_attacked(board, src.uright().uright(), board.side_to_move()) // E
                && get_rook_moves(opp_ksq, *combined) & BitBoard::from_square(src.uleft().uleft())
                    == EMPTY // A
                && (T::NB_CHECKERS == 0
                    || board.checkers() & BitBoard::from_square(src.uright()) != EMPTY)
            {
                unsafe {
                    movelist.push_unchecked(SourceAndTargets::new(
                        src,
                        BitBoard::from_square(src.uright().uright()),
                        UnCaptureKind::Forbidden,
                        false,
                    ));
                }
            }
        }
    }
}
