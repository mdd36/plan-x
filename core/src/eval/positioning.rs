use crate::position::{
	attacks::{self, KING_ATTACKS},
	board::{Board, Square, A_FILE, B_FILE, C_FILE, D_FILE, E_FILE, F_FILE, G_FILE, H_FILE},
	Color, Piece, Position,
};

use super::combine_phase_scores;

#[rustfmt::skip]
const PAWN_SQUARE_WEIGHTS: [i16; 64] = [
	0,  0,  0,  0,  0,  0,  0,  0,
	50, 50, 50, 50, 50, 50, 50, 50,
	10, 10, 20, 30, 30, 20, 10, 10,
	 5,  5, 10, 25, 25, 10,  5,  5,
	 0,  0,  0, 20, 20,  0,  0,  0,
	 5, -5,-10,  0,  0,-10, -5,  5,
	 5, 10, 10,-20,-20, 10, 10,  5,
	 0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const ROOK_SQUARE_WEIGHTS: [i16; 64] = [
	0,  0,  0,  0,  0,  0,  0,  0,
  5, 10, 10, 10, 10, 10, 10,  5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
 -5,  0,  0,  0,  0,  0,  0, -5,
  0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_SQUARE_WEIGHTS: [i16; 64] = [
	-50,-40,-30,-30,-30,-30,-40,-50,
	-40,-20,  0,  0,  0,  0,-20,-40,
	-30,  0, 10, 15, 15, 10,  0,-30,
	-30,  5, 15, 20, 20, 15,  5,-30,
	-30,  0, 15, 20, 20, 15,  0,-30,
	-30,  5, 10, 15, 15, 10,  5,-30,
	-40,-20,  0,  5,  5,  0,-20,-40,
	-50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_SQUARE_WEIGHTS: [i16; 64] = [
	-20,-10,-10,-10,-10,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5, 10, 10,  5,  0,-10,
	-10,  5,  5, 10, 10,  5,  5,-10,
	-10,  0, 10, 10, 10, 10,  0,-10,
	-10, 10, 10, 10, 10, 10, 10,-10,
	-10,  5,  0,  0,  0,  0,  5,-10,
	-20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const QUEEN_SQUARE_WEIGHTS: [i16; 64] = [
	-20,-10,-10, -5, -5,-10,-10,-20,
	-10,  0,  0,  0,  0,  0,  0,-10,
	-10,  0,  5,  5,  5,  5,  0,-10,
	 -5,  0,  5,  5,  5,  5,  0, -5,
		0,  0,  5,  5,  5,  5,  0, -5,
	-10,  5,  5,  5,  5,  5,  0,-10,
	-10,  0,  5,  0,  0,  0,  0,-10,
	-20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
const KING_SQUARE_WEIGHTS: [i16; 64] = [
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-30,-40,-40,-50,-50,-40,-40,-30,
	-20,-30,-30,-40,-40,-30,-30,-20,
	-10,-20,-20,-20,-20,-20,-20,-10,
	20, 20,  0,  0,  0,  0, 20, 20,
	20, 30, 10,  0,  0, 10, 30, 20
];

#[rustfmt::skip]
const KING_SQUARE_WEIGHTS_ENDGAME: [i16; 64] = [
	-50,-40,-30,-20,-20,-30,-40,-50,
	-30,-20,-10,  0,  0,-10,-20,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 30, 40, 40, 30,-10,-30,
	-30,-10, 20, 30, 30, 20,-10,-30,
	-30,-30,  0,  0,  0,  0,-30,-30,
	-50,-30,-30,-30,-30,-30,-30,-50
];

#[rustfmt::skip]
const PASSED_PAWN_BONUS: [i16; 8] = [
	0, 30, 20, 12, 5, 5, 0, 0
];
#[rustfmt::skip]
const PASSED_PAWN_BONUS_END_GAME: [i16; 8] = [
	0, 60, 40, 25, 15, 10, 10, 0
];

/// An isolated D pawn is less penalizing since it's
/// useful in controlling the center and is generally
/// defended by the queen in the early to mid game.
#[rustfmt::skip]
const ISOLATED_PAWN_PENALTY: [i16; 8] = [
	-15, -15, -15, -5, -15, -15, -15, -15
];
#[rustfmt::skip]
const ISOLATED_PAWN_PENALTY_END_GAME: [i16; 8] = [
	-30, -30, -30, -30, -30, -30, -30, -30	
];

/// A bonus or penalty to apply based on the weighted
/// number of attackers of the king's zone
#[rustfmt::skip]
const KING_ATTACK_WEIGHTS: [i16; 8] = [
	0, 0, -1, -2, -5, -10, -50, -100
];

lazy_static::lazy_static!(
	static ref PAWN_FLANKS: [Board; 8] = [
		B_FILE,
		A_FILE | C_FILE,
		B_FILE | D_FILE,
		C_FILE | E_FILE,
		D_FILE | F_FILE,
		E_FILE | G_FILE,
		F_FILE | H_FILE,
		G_FILE,
	];
);

pub fn positioning_score(position: &Position, color: Color) -> i16 {
	let mut score = 0;
	let our_pieces = position.get_board_for_color(color);

	let pawn_board = position.get_board_for_piece(Piece::Pawn) & our_pieces;
	score += pawn_board
		.into_iter()
		.map(|sq| PAWN_SQUARE_WEIGHTS[sq.lsb_index()])
		.sum::<i16>();

	let rook_board = position.get_board_for_piece(Piece::Rook) & our_pieces;
	score += rook_board
		.into_iter()
		.map(|sq| ROOK_SQUARE_WEIGHTS[sq.lsb_index()])
		.sum::<i16>();

	let knight_board = position.get_board_for_piece(Piece::Knight) & our_pieces;
	score += knight_board
		.into_iter()
		.map(|sq| KNIGHT_SQUARE_WEIGHTS[sq.lsb_index()])
		.sum::<i16>();

	let bishop_board = position.get_board_for_piece(Piece::Bishop) & our_pieces;
	score += bishop_board
		.into_iter()
		.map(|sq| BISHOP_SQUARE_WEIGHTS[sq.lsb_index()])
		.sum::<i16>();

	let queen_board = position.get_board_for_piece(Piece::Queen) & our_pieces;
	score += queen_board
		.into_iter()
		.map(|sq| QUEEN_SQUARE_WEIGHTS[sq.lsb_index()])
		.sum::<i16>();

	// The king is a little different. We want to encourage it to
	// take shelter in the early and mid game, but come out to help
	// with checkmates in the end game.
	let king_square = Square::from(position.get_board_for_piece(Piece::King) & our_pieces);
	let king_square_index = king_square.lsb_index();
	score += combine_phase_scores(
		position,
		KING_SQUARE_WEIGHTS[king_square_index],
		KING_SQUARE_WEIGHTS_ENDGAME[king_square_index],
	);

	score
}

pub fn pawn_structure(position: &Position, color: Color) -> i16 {
	let our_pieces = position.get_board_for_color(color);
	let their_pieces = position.get_board_for_color(!color);

	let all_pawns = position.get_board_for_piece(Piece::Pawn);
	let our_pawns = our_pieces & all_pawns;
	let their_pawns = their_pieces & all_pawns;

	let mut end_game_score = 0;
	let mut early_game_score = 0;

	for pawn in our_pawns {
		if is_passed(pawn, color, our_pawns, their_pawns) {
			let squares_to_promotion = match color {
				Color::White => 7 - pawn.rank(),
				Color::Black => pawn.rank(),
			} as usize;
			early_game_score += PASSED_PAWN_BONUS[squares_to_promotion];
			end_game_score += PASSED_PAWN_BONUS_END_GAME[squares_to_promotion];
		}

		if is_isolated(pawn, our_pawns) {
			early_game_score += ISOLATED_PAWN_PENALTY[pawn.file() as usize];
			end_game_score += ISOLATED_PAWN_PENALTY_END_GAME[pawn.file() as usize];
		}
	}

	combine_phase_scores(position, early_game_score, end_game_score)
}

fn is_passed(pawn: Square, color: Color, our_pawns: Board, their_pawns: Board) -> bool {
	let forward_mask = match color {
		Color::White => std::u64::MAX << (pawn.rank() + 1),
		Color::Black => std::u64::MAX >> (pawn.rank() - 1),
	};
	let pawn_file = forward_mask & (A_FILE.as_u64() << pawn.file());
	let side_files_masks = forward_mask & PAWN_FLANKS[pawn.file() as usize].as_u64();
	(our_pawns & pawn_file).is_empty() && (their_pawns & (pawn_file | side_files_masks)).is_empty()
}

fn is_isolated(pawn: Square, friendly_pawns: Board) -> bool {
	let flanks = PAWN_FLANKS[pawn.file() as usize];
	(friendly_pawns & flanks).is_empty()
}

pub fn king_safety(position: &Position, color: Color) -> i16 {
	let us = position.get_board_for_color(color);
	let king_square = Square::from(us & position.get_board_for_piece(Piece::King));

	let king_zone = us.as_u64() ^ KING_ATTACKS[king_square.lsb_index()];
	let attacked_squares = attacks::get_attacked_squares(position, !color);
	let num_attacks = (attacked_squares & king_square).count_ones();

	let ratio_attacked = king_zone.count_ones() as f64 / num_attacks as f64;
	let penalty_index = (8.0 * ratio_attacked) as usize;

	KING_ATTACK_WEIGHTS[penalty_index]
}
