use std::ffi::c_int;

use crate::{
    errors::{BindingErrorCode, ErrorKind, RegexError, Result},
    flags::{RegexecFlags, RegcompFlags},
    tre, TreRegex
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Match<'h> {
    matched: &'h [u8],
    start: usize,
    end: usize,
}

impl<'h> Match<'h> {
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn as_bytes(&self) -> &'h [u8] {
        &self.matched
    }
}

pub type FuzzyMatchBytes<'a> = FuzzyMatch<&'a [u8], Match<'a>>;

/// Regex params passed to approximate matching functions such as [`regaexec`]
#[cfg(feature = "approx")]
#[derive(Copy, Clone, Debug)]
pub struct FuzzyRegexParams(tre::regaparams_t);

impl FuzzyRegexParams {
    /// Creates a new empty [`FuzzyRegexParams`] object.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(tre::regaparams_t::default())
    }

    /// Sets the [`cost_insertion`](tre_regex_sys::regaparams_t::cost_ins) element.
    #[must_use]
    #[inline]
    pub const fn cost_insertion(&self, cost_insertion: c_int) -> Self {
        let mut copy: FuzzyRegexParams = *self;
        copy.0.cost_ins = cost_insertion;
        copy
    }

    /// Sets the [`cost_deletion`](tre_regex_sys::regaparams_t::cost_del) element.
    #[must_use]
    #[inline]
    pub const fn cost_deletion(&self, cost_deletion: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_del = cost_deletion;
        copy
    }

    /// Sets the [`cost_substitution`](tre_regex_sys::regaparams_t::cost_subst) element.
    #[must_use]
    #[inline]
    pub const fn cost_substitution(&self, cost_substitution: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_subst = cost_substitution;
        copy
    }

    /// Sets the [`max_cost`](tre_regex_sys::regaparams_t::max_cost) element.
    #[must_use]
    #[inline]
    pub const fn max_cost(&self, max_cost: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_cost = max_cost;
        copy
    }

    /// Sets the [`max_insertion`](tre_regex_sys::regaparams_t::max_ins) element.
    #[must_use]
    #[inline]
    pub const fn max_insertion(&self, max_insertion: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_ins = max_insertion;
        copy
    }

    /// Sets the [`max_deletion`](tre_regex_sys::regaparams_t::max_del) element.
    #[must_use]
    #[inline]
    pub const fn max_deletion(&self, max_deletion: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_del = max_deletion;
        copy
    }

    /// Sets the [`max_substitution`](tre_regex_sys::regaparams_t::max_subst) element.
    #[must_use]
    #[inline]
    pub const fn max_substitution(&self, max_substitution: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_subst = max_substitution;
        copy
    }

    /// Sets the [`max_error`](tre_regex_sys::regaparams_t::max_err) element.
    #[must_use]
    #[inline]
    pub const fn max_error(&self, max_error: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_err = max_error;
        copy
    }

    /// Get an immutable reference to the underlying [`regaparams_t`](tre_regex_sys::regaparams_t) object.
    #[must_use]
    #[inline]
    pub const fn get(&self) -> &tre::regaparams_t {
        &self.0
    }
}

impl Default for FuzzyRegexParams {
    fn default() -> Self {
        Self::new()
    }
}

/// This struct is returned by [regaexec] and related functions.
///
/// The match results from this function are quite complex. See the [TRE documentation] for details
/// on how this works and the corresponding fields, and what they mean.
///
/// This structure should never be instantiated outside the library.
///
/// [TRE documentation]: https://laurikari.net/tre/documentation/regaexec/
#[derive(Clone, Debug)]
pub struct FuzzyMatch<Data, Res> {
    data: Data,
    matches: Vec<Option<Res>>,
    amatch: tre::regamatch_t,
}

impl<Data, Res> FuzzyMatch<Data, Res> {
    pub(crate) fn new(data: Data, matches: Vec<Option<Res>>, amatch: tre::regamatch_t) -> Self {
        Self {
            data,
            matches,
            amatch,
        }
    }

    /// Get the number of substitutions in the match
    pub const fn substitutions_number(&self) -> c_int {
        self.amatch.num_subst
    }

    /// Gets an immutable reference to the underlying data
    pub const fn get_original_data(&self) -> &Data {
        &self.data
    }

    /// Gets the matches returned by this, as references to the data
    pub const fn get_matches(&self) -> &Vec<Option<Res>> {
        &self.matches
    }
}

pub struct FuzzyRegex {
    compiled_regex: TreRegex,
    flags: RegexecFlags,
    params: FuzzyRegexParams
}

impl FuzzyRegex {
    pub fn new(reg: &str, max_substitution: usize, max_deletion: usize, max_insertion: usize) -> Result<FuzzyRegex> {
        let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
        let max_cost = (max_deletion + max_insertion + max_substitution) as i32;
        let regaexec_params = FuzzyRegexParams::new()
            .cost_insertion(1)
            .cost_deletion(1)
            .cost_substitution(1)
            .max_cost(max_cost)
            .max_deletion(max_deletion as i32)
            .max_insertion(max_insertion as i32)
            .max_substitution(max_substitution as i32)
            .max_error(max_cost);
        Ok(Self {
            compiled_regex: TreRegex::new_bytes(reg.as_bytes(), &[RegcompFlags::EXTENDED, RegcompFlags::ICASE])?,
            flags: regaexec_flags,
            params: regaexec_params
        })
    }

    pub fn captures<'a>(
        &self,
        data: &'a [u8],
        nmatches: usize,
    ) -> Result<FuzzyMatchBytes<'a>> {
        let Some(compiled_reg_obj) = self.compiled_regex.get() else {
            return Err(RegexError::new(
                ErrorKind::Binding(BindingErrorCode::REGEX_VACANT),
                "Attempted to unwrap a vacant Regex object",
            ));
        };
        let mut match_vec: Vec<tre::regmatch_t> =
            vec![tre::regmatch_t { rm_so: 0, rm_eo: 0 }; nmatches];
        let mut amatch = tre::regamatch_t {
            nmatch: nmatches,
            pmatch: match_vec.as_mut_ptr(),
            ..Default::default()
        };

        // SAFETY: compiled_reg is a wrapped type (see safety concerns for Regex). data is read-only.
        // match_vec has enough room for everything. flags also cannot wrap around.
        #[allow(clippy::cast_possible_wrap)]
        let result = unsafe {
            tre::tre_reganexec(
                compiled_reg_obj,
                data.as_ptr().cast::<i8>(),
                data.len(),
                &mut amatch,
                *self.params.get(),
                self.flags.get(),
            )
        };
        if result != 0 {
            return Err(self.compiled_regex.regerror(result));
        }

        let mut result: Vec<Option<Match>> = Vec::with_capacity(nmatches);
        for pmatch in match_vec {
            if pmatch.rm_so < 0 || pmatch.rm_eo < 0 {
                result.push(None);
                continue;
            }

            // Wraparound is impossible.
            #[allow(clippy::cast_sign_loss)]
            let start_offset = pmatch.rm_so as usize;
            #[allow(clippy::cast_sign_loss)]
            let end_offset = pmatch.rm_eo as usize;

            result.push(Some(Match {
                matched: &data[start_offset..end_offset],
                start: start_offset,
                end: end_offset
            }));
        }

        Ok(FuzzyMatchBytes::new(data, result, amatch))
    }
}


#[test]
fn test_regaexec_bytes() {
    let compiled_reg = FuzzyRegex::new("^(hello).*(world)$", 2, 2, 2).expect("Regex::new");
    let result = compiled_reg
        .captures(
            b"hullo warld",
            3,
        )
        .expect("regaexec");

    let matched = result.get_matches();

    let matched_0 = matched[0].as_ref();
    assert!(matched_0.is_some());
    assert_eq!(matched_0.unwrap().as_bytes(), b"hullo warld");

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert_eq!(matched_1.unwrap().as_bytes(), b"hullo");

    let matched_2 = matched[2].as_ref();
    assert!(matched_2.is_some());
    assert_eq!(matched_2.unwrap().as_bytes(), b"warld");
}