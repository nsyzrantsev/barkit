use bitvec::prelude::*;



#[derive(Debug, PartialEq)]
pub struct BitVector {
    bits_number: usize,
    pub elements: BitVec
}

impl BitVector {
    
    /// returns a bit vector of ones
    pub fn ones(bits_number: usize) -> Self {
        Self { bits_number, elements: bitvec![1; bits_number] }
    }
    
    /// returns a bit vector of zeros
    pub fn zeros(bits_number: usize) -> Self {
        Self { bits_number, elements: bitvec![0; bits_number] }
    }

    pub fn single_one_at(bits_number: usize, position: usize) -> Self {
        let element_offset = position / bits_number;
        let bit_offset = position % bits_number;
        let mut new_bitvector = Self::zeros(bits_number);
        new_bitvector.elements.set(element_offset, (1 << bit_offset) != 0);
        new_bitvector
    }

    pub fn single_zero_at(bits_number: usize, position: usize) -> Self {
        let mut new_bitvector = Self::single_one_at(bits_number, position);
        new_bitvector.elements = !new_bitvector.elements;
        new_bitvector
    }

    /// check if bit vector has an one at the selected position
    pub fn has_one_at(&self, position: usize) -> bool {
        let element_offset = position / self.bits_number;
        let bit_offset = position % self.bits_number;
        self.elements[element_offset] & ((1 << bit_offset) != 0)
    }

    /// check if bit vector has a zero at the selected position
    pub fn has_zero_at(&self, position: usize) -> bool {
        !self.has_one_at(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ones() {
        let result = BitVector::ones(4);
        assert_eq!(result, BitVector { 
            bits_number: 4, 
            elements: bitvec![1, 1, 1, 1] 
        });
    }

    #[test]
    fn test_zeros() {
        let result = BitVector::zeros(4);
        assert_eq!(result, BitVector { 
            bits_number: 4, 
            elements: bitvec![0, 0, 0, 0] 
        });
    }

    #[test]
    fn test_single_one_at() {
        let result = BitVector::single_one_at(4, 0);
        assert_eq!(result, BitVector { 
            bits_number: 4, 
            elements: bitvec![1, 0, 0, 0] 
        });
    }

    #[test]
    fn test_single_zero_at() {
        let result = BitVector::single_zero_at(4, 0);
        assert_eq!(result, BitVector { 
            bits_number: 4, 
            elements: bitvec![0, 1, 1, 1] 
        });
    }

    #[test]
    fn test_has_one_at() {
        let bitvec = BitVector {bits_number: 4, elements: bitvec![1, 0, 0, 0]};
        let result = bitvec.has_one_at(0);
        assert_eq!(result, true);
    }

    #[test]
    fn test_has_zero_at() {
        let bitvec = BitVector {bits_number: 4, elements: bitvec![0, 1, 1, 1]};
        let result = bitvec.has_zero_at(0);
        assert_eq!(result, true);
    }
}