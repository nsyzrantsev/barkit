use std::borrow::Cow;
use std::ffi::c_int;
use std::hint::unreachable_unchecked;

use crate::{
    err::{BindingErrorCode, ErrorKind, RegexError, Result},
    tre, Regex, RegexecFlags,
};

pub type RegApproxMatchStr<'a> = RegApproxMatch<&'a str, Result<Cow<'a, str>>>;
pub type RegApproxMatchBytes<'a> = RegApproxMatch<&'a [u8], (Cow<'a, [u8]>, usize, usize)>;

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

impl Regex {
    #[inline]
    pub fn regaexec<'a>(
        &self,
        string: &'a str,
        params: &RegApproxParams,
        nmatches: usize,
        flags: RegexecFlags,
    ) -> Result<RegApproxMatchStr<'a>> {
        let data = string.as_bytes();
        let match_results = self.regaexec_bytes(data, params, nmatches, flags)?;

        let mut result: Vec<Option<Result<Cow<'a, str>>>> = Vec::with_capacity(nmatches);
        for pmatch in match_results.get_matches() {
            let Some(pmatch) = pmatch else {
                result.push(None);
                continue;
            };

            #[allow(clippy::match_wildcard_for_single_variants)]
            result.push(Some(match pmatch.0 {
                Cow::Borrowed(pmatch) => match std::str::from_utf8(pmatch) {
                    Ok(s) => Ok(s.into()),
                    Err(e) => Err(RegexError::new(
                        ErrorKind::Binding(BindingErrorCode::ENCODING),
                        &format!("UTF-8 encoding error: {e}"),
                    )),
                },
                // SAFETY: cannot get here, we only have borrowed values.
                _ => unsafe { unreachable_unchecked() },
            }));
        }

        Ok(RegApproxMatchStr::new(
            string,
            result,
            *match_results.get_regamatch(),
        ))
    }

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

        let mut result: Vec<Option<(Cow<'a, [u8]>, usize, usize)>> = Vec::with_capacity(nmatches);
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

            result.push(Some((Cow::Borrowed(&data[start_offset..end_offset]), start_offset, end_offset)));
        }

        Ok(RegApproxMatchBytes::new(data, result, amatch))
    }
}

#[inline]
pub fn regaexec<'a>(
    compiled_reg: &Regex,
    string: &'a str,
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchStr<'a>> {
    compiled_reg.regaexec(string, params, nmatches, flags)
}

#[inline]
pub fn regaexec_bytes<'a>(
    compiled_reg: &Regex,
    data: &'a [u8],
    params: &RegApproxParams,
    nmatches: usize,
    flags: RegexecFlags,
) -> Result<RegApproxMatchBytes<'a>> {
    compiled_reg.regaexec_bytes(data, params, nmatches, flags)
}