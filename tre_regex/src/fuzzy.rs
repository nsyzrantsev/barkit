use std::ffi::c_int;

use crate::{
    errors::{BindingErrorCode, ErrorKind, RegexError, Result},
    flags::RegexecFlags,
    tre, TreRegex, Match
};

pub type RegApproxMatchBytes<'a> = RegApproxMatch<&'a [u8], Match<'a>>;

/// Regex params passed to approximate matching functions such as [`regaexec`]
#[cfg(feature = "approx")]
#[derive(Copy, Clone, Debug)]
pub struct RegApproxParams(tre::regaparams_t);

impl RegApproxParams {
    /// Creates a new empty [`RegApproxParams`] object.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(tre::regaparams_t::default())
    }

    /// Sets the [`cost_ins`](tre_regex_sys::regaparams_t::cost_ins) element.
    #[must_use]
    #[inline]
    pub const fn cost_ins(&self, cost_ins: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_ins = cost_ins;
        copy
    }

    /// Sets the [`cost_del`](tre_regex_sys::regaparams_t::cost_del) element.
    #[must_use]
    #[inline]
    pub const fn cost_del(&self, cost_del: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_del = cost_del;
        copy
    }

    /// Sets the [`cost_subst`](tre_regex_sys::regaparams_t::cost_subst) element.
    #[must_use]
    #[inline]
    pub const fn cost_subst(&self, cost_subst: c_int) -> Self {
        let mut copy = *self;
        copy.0.cost_subst = cost_subst;
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

    /// Sets the [`max_ins`](tre_regex_sys::regaparams_t::max_ins) element.
    #[must_use]
    #[inline]
    pub const fn max_ins(&self, max_ins: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_ins = max_ins;
        copy
    }

    /// Sets the [`max_del`](tre_regex_sys::regaparams_t::max_del) element.
    #[must_use]
    #[inline]
    pub const fn max_del(&self, max_del: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_del = max_del;
        copy
    }

    /// Sets the [`max_subst`](tre_regex_sys::regaparams_t::max_subst) element.
    #[must_use]
    #[inline]
    pub const fn max_subst(&self, max_subst: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_subst = max_subst;
        copy
    }

    /// Sets the [`max_err`](tre_regex_sys::regaparams_t::max_err) element.
    #[must_use]
    #[inline]
    pub const fn max_err(&self, max_err: c_int) -> Self {
        let mut copy = *self;
        copy.0.max_err = max_err;
        copy
    }

    /// Get an immutable reference to the underlying [`regaparams_t`](tre_regex_sys::regaparams_t) object.
    #[must_use]
    #[inline]
    pub const fn get(&self) -> &tre::regaparams_t {
        &self.0
    }

    /// Get a mutable reference to the underlying [`regaparams_t`](tre_regex_sys::regaparams_t) object.
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> &mut tre::regaparams_t {
        &mut self.0
    }
}

impl Default for RegApproxParams {
    fn default() -> Self {
        Self::new()
    }
}

/// This struct is returned by [`regaexec`] and friends.
///
/// The match results from this function are very complex. See the [TRE documentation] for details
/// on how this all works and corresponding fields, and what they mean.
///
/// This structure should never be instantiated outside the library.
///
/// [TRE documentation]: <https://laurikari.net/tre/documentation/regaexec/>
#[derive(Clone, Debug)]
pub struct RegApproxMatch<Data, Res> {
    data: Data,
    matches: Vec<Option<Res>>,
    amatch: tre::regamatch_t,
}

impl<Data, Res> RegApproxMatch<Data, Res> {
    pub(crate) fn new(data: Data, matches: Vec<Option<Res>>, amatch: tre::regamatch_t) -> Self {
        Self {
            data,
            matches,
            amatch,
        }
    }

    /// Gets the cost of the match
    pub const fn cost(&self) -> c_int {
        self.amatch.cost
    }

    /// Gets the number of insertions if the match
    pub const fn num_ins(&self) -> c_int {
        self.amatch.num_ins
    }

    /// Gets the number of deletions if the match
    pub const fn num_del(&self) -> c_int {
        self.amatch.num_del
    }

    /// Get the number of substitutions in the match
    pub const fn num_subst(&self) -> c_int {
        self.amatch.num_subst
    }

    /// Gets an immutable reference to the underlying data
    pub const fn get_orig_data(&self) -> &Data {
        &self.data
    }

    /// Gets the matches returned by this, as references to the data
    pub const fn get_matches(&self) -> &Vec<Option<Res>> {
        &self.matches
    }

    /// Gets a reference to the underlying [`regamatch_t`](tre_regex_sys::regamatch_t) object.
    pub const fn get_regamatch(&self) -> &tre::regamatch_t {
        &self.amatch
    }
}

impl TreRegex {
    pub fn regaexec_bytes<'a>(
        &self,
        data: &'a [u8],
        params: &RegApproxParams,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegApproxMatchBytes<'a>> {
        let Some(compiled_reg_obj) = self.get() else {
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
                *params.get(),
                flags.get(),
            )
        };
        if result != 0 {
            return Err(self.regerror(result));
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
                haystack: data,
                start: start_offset,
                end: end_offset
            }));
        }

        Ok(RegApproxMatchBytes::new(data, result, amatch))
    }
}

#[inline]
pub fn regaexec_bytes<'a>(
    compiled_reg: &TreRegex,
    data: &'a [u8],
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchBytes<'a>> {
    compiled_reg.regaexec_bytes(data, params, nmatches, flags)
}

#[cfg(test)]
use crate::flags::RegcompFlags;

#[test]
fn test_regaexec_bytes() {
    let regcomp_flags = RegcompFlags::new()
        .add(RegcompFlags::EXTENDED)
        .add(RegcompFlags::ICASE);
    let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let regaexec_params = RegApproxParams::new()
        .cost_ins(1)
        .cost_del(1)
        .cost_subst(1)
        .max_cost(2)
        .max_del(2)
        .max_ins(2)
        .max_subst(2)
        .max_err(2);

    let compiled_reg = TreRegex::new_bytes(b"^(hello).*(world)$", regcomp_flags).expect("Regex::new");
    let result = compiled_reg
        .regaexec_bytes(
            b"hullo warld",   // String to match against
            &regaexec_params, // Matching parameters
            3,                // Number of matches we want
            regaexec_flags,   // Flags
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