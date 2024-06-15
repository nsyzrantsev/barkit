#ifndef FUZZY_MATCHER_HPP
#define FUZZY_MATCHER_HPP

#include <memory>
#include <string>
#include <fuzzy_matcher/RE-flex/fuzzy/fuzzymatcher.h>
#include "rust/cxx.h"

namespace fuzzy_matcher {
    using ::reflex::FuzzyMatcher;

    std::unique_ptr<std::string> create_regex(rust::Str pattern);
    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(rust::Str regex, int8_t max_errors, rust::Str input);
    bool matches(std::unique_ptr<FuzzyMatcher> matcher);
}

#endif // FUZZY_MATCHER_HPP