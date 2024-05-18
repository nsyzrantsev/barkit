use std::collections::HashMap;
use crate::bitvector::BitVector;
use bitvec::prelude::*;

/*
PM_t generatePatternBitmaskACGT(int m, char *pattern){
        PM_t pm;
        pm.masks[A] = bitvector::ones();
        pm.masks[C] = bitvector::ones();
        pm.masks[G] = bitvector::ones();
        pm.masks[T] = bitvector::ones();

        for(int bit_idx = 0; bit_idx < m; bit_idx++){
            int j = m - 1 - bit_idx;
            char curChar = pattern[j];
            pm.masks[curChar] = pm.masks[curChar] & bitvector::single_zero_at(bit_idx);
        }

        //print_zero_based(m, pattern);
        //cout << "A: "; pm.masks[A].print();
        //cout << "C: "; pm.masks[C].print();
        //cout << "G: "; pm.masks[G].print();
        //cout << "T: "; pm.masks[T].print();

        return pm;
    }
*/

pub fn generate_pattern_bitmasks(pattern: &str) -> HashMap::<u8, BitVec> {
    let pattern_length = pattern.len();
    let mut pm_masks = HashMap::new(); // [BitVector::new(pattern_length), 4];

    for (i, character) in pattern.bytes().enumerate().rev() {
        let bitvec_with_single_zero = &BitVector::single_zero_at(pattern_length, i).elements;
        pm_masks.entry(character)
                .and_modify(|char_bitmask| *char_bitmask &= bitvec_with_single_zero)
                .or_insert(BitVector::ones(pattern_length).elements & bitvec_with_single_zero);
    }

    pm_masks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pattern_bitmasks() {
        let result = generate_pattern_bitmasks("A");
        assert_eq!(result, HashMap::from([
            (b'A', bitvec![0])
        ]));
    }
}

// FS static BITVECTOR single_one_at(int n) {
//     #if BITVECTOR_ELEMENTS == 1
//         return {(BITVECTOR_ELEMENT_TYPE)(((BITVECTOR_ELEMENT_TYPE)1)<<n)};
//     #else
//         int element_offset = n/BITVECTOR_ELEMENT_BITS;
//         int bit_offset = n%BITVECTOR_ELEMENT_BITS;

//         BITVECTOR res = zeros();
//         res.elements[element_offset] = (BITVECTOR_ELEMENT_TYPE)1<<bit_offset;
//         return res;
//     #endif
// }

// pub fn calculate_score(
//     count_m: usize,
//     count_s: usize,
//     count_open: usize,
//     count_extend: usize,
//     score_m: usize,
//     score_s: usize,
//     score_open: usize,
//     score_extend: usize,
// ) -> usize {
//     (count_m * score_m)
//         + (count_s * score_s)
//         + (count_open * (score_open + score_extend))
//         + (count_extend * score_extend)
// }

// pub fn gen_asm_tb(
//     n: usize,
//     k: usize,
//     count: usize,
//     traceback_matrix: &Vec<Vec<Vec<Vec<u64>>>>,
//     m: usize,
//     min_error: usize,
//     mut mask: u64,
//     text: &str,
// ) -> (
//     usize, String, String, String, usize, usize, usize, usize, usize, usize,
// ) {
//     let mut ed = 0;
//     let mut cigar_str = String::new();
//     let mut cigar_str2 = String::new();
//     let mut md = String::new();
//     let mut char_count = 0;
//     let mut char_count2 = 0;
//     let mut char_count3 = 0;
//     let mut last_char = '0';
//     let mut last_char2 = '0';
//     let mut last_char3 = '0';
//     let mut is_first = true;
//     let mut count_m = 0;
//     let mut count_s = 0;
//     let mut count_d = 0;
//     let mut count_i = 0;
//     let mut count_open = 0;
//     let mut count_extend = 0;
//     let mut cur_pattern = m - 1;
//     let mut cur_text = 0;
//     let mut cur_error = min_error;

//     while cur_pattern > 0 && cur_error > 0 {
//         let ind: usize = count - (cur_pattern / 64) - 1;

//         // affine-insertion
//         if last_char == 'I' && (traceback_matrix[cur_text][cur_error][2][ind] & mask) == 0 {
//             cur_pattern -= 1;
//             cur_error -= 1;
//             mask = 1 << (cur_pattern % 64);
//             if last_char == 'I' {
//                 char_count += 1;
//                 count_extend += 1;
//             } else {
//                 if !is_first {
//                     cigar_str += &format!("{}{}", char_count, last_char);
//                 }
//                 char_count = 1;
//                 last_char = 'I';
//                 count_open += 1;
//             }
//             if last_char2 == 'I' {
//                 char_count2 += 1;
//             } else {
//                 if !is_first {
//                     cigar_str2 += &format!("{}{}", char_count2, last_char2);
//                 }
//                 char_count2 = 1;
//                 last_char2 = 'I';
//             }
//             count_i += 1;
//             ed += 1;
//         }
//         // other cases here ...

//         is_first = false;
//     }

//     cigar_str += &format!("{}{}", char_count, last_char);
//     cigar_str2 += &format!("{}{}", char_count2, last_char2);
//     if last_char3 == 'M' {
//         md += &format!("{}", char_count3);
//     }

//     (
//         ed,
//         cigar_str,
//         cigar_str2,
//         md,
//         count_m,
//         count_s,
//         count_d,
//         count_i,
//         count_open,
//         count_extend,
//     )
// }


// fn gen_asm_dc(
//     text: &str,
//     pattern: &str,
//     k: usize,
//     score_m: usize,
//     score_s: usize,
//     score_open: usize,
//     score_extend: usize,
// ) {
//     let pattern_length = pattern.len();
//     let text_length = text.len();

//     let max_64bit = u64::MAX;

//     let pattern_bitmasks = generate_pattern_bitmasks(pattern);

//     let count = (pattern_length as f64 / 64.0).ceil() as usize;
//     let rem = pattern_length % 64;
//     let max1 = if rem > 0 { 1 << (rem - 1) } else { 1 << 63 };

//     // Initialize the bit arrays R
//     let mut R = vec![max_64bit; (k + 1) * count];

//     let mut traceback_matrix = vec![
//         vec![
//             vec![
//                 vec![max_64bit; count];
//                 4
//             ];
//             k + 1
//         ];
//         text_length
//     ];

//     let mut old_R = R.clone();

//     // now traverse the text in opposite direction (i.e., forward), generate partial tracebacks at each checkpoint
//     for i in (0..text_length).rev() {
//         let c = text.chars().nth(i).unwrap();

//         // copy the content of R to old_R
//         old_R.copy_from_slice(&R);

//         let cur_bitmask = &pattern_bitmasks[(c as usize) * count..(c as usize) * count + count];

//         // update R[0] by first shifting old_R[0] and then ORing with pattern bitmask
//         R[0] = (old_R[0] << 1) & max_64bit;
//         for a in 1..count {
//             R[a - 1] |= old_R[a] >> 63;
//             R[a] = (old_R[a] << 1) & max_64bit;
//         }

//         for a in 0..count {
//             R[a] |= cur_bitmask[a];
//             traceback_matrix[i][0][0][a] = R[a];
//             traceback_matrix[i][0][1][a] = max_64bit;
//             traceback_matrix[i][0][2][a] = max_64bit;
//             traceback_matrix[i][0][3][a] = max_64bit;
//         }

//         for d in 1..=k {
//             let index = (d - 1) * count;
//             let deletion = &old_R[index..index + count];
//             let mut substitution = vec![(deletion[0] << 1) & max_64bit];
//             let mut insertion = vec![(R[index] << 1) & max_64bit];
//             let mut match_ = vec![(old_R[index] << 1) & max_64bit];

//             for a in 1..count {
//                 substitution.push(
//                     ((deletion[a - 1] >> 63) & max_64bit) | ((deletion[a] << 1) & max_64bit),
//                 );
//                 insertion.push(
//                     ((R[index + a] >> 63) & max_64bit) | ((R[index + a] << 1) & max_64bit),
//                 );
//                 match_.push(
//                     ((old_R[index + a] >> 63) & max_64bit) | ((old_R[index + a] << 1) & max_64bit),
//                 );
//             }

//             match_ = match_.iter().zip(cur_bitmask.iter()).map(|(&x, &y)| x | y).collect();

//             for (a, ((&x, &y), &z)) in deletion.iter().zip(substitution.iter()).zip(insertion.iter()).enumerate() {
//                 R[index + a] = x & y & z & match_[a];
//             }

//             for a in 0..count {
//                 traceback_matrix[i][d][0][a] = match_[a];
//                 traceback_matrix[i][d][1][a] = substitution[a];
//                 traceback_matrix[i][d][2][a] = insertion[a];
//                 traceback_matrix[i][d][3][a] = deletion[a];
//             }
//         }
//     }

//     let mut min_error = -1;
//     let mut mask = max1;

//     for t in 0..=k {
//         if (R[t * count] & mask) == 0 {
//             min_error = t as isize;
//             break;
//         }
//     }

//     if min_error == -1 {
//         println!("No alignment found!");
//         return;
//     }

//     let (
//         ed,
//         cigar_str,
//         cigar_str2,
//         md,
//         count_m,
//         count_s,
//         count_d,
//         count_i,
//         count_open,
//         count_extend,
//     ) = gen_asm_tb(text_length, k, count, &traceback_matrix, pattern_length, min_error as usize, mask, text);

//     let bitmac_score =
//         count_m * score_m - count_s * score_s - count_open * (score_open + score_extend)
//             - count_extend * score_extend;

//     println!("ED:{}\tAS:{}\t{}\t{}\t{}", ed, bitmac_score, cigar_str, cigar_str2, md);
// }


// pub fn genasm_aligner(
//     text: &str,
//     pattern: &str,
//     k: usize,
//     score_m: usize,
//     score_s: usize,
//     score_open: usize,
//     score_extend: usize,
// ) {
//     gen_asm_dc(text, pattern, k, score_m, score_s, score_open, score_extend);
// }