#ifndef FUZZY_MATCHER_HPP
#define FUZZY_MATCHER_HPP

#include <memory>
#include <string>
#include <fuzzy_matcher/RE-flex/fuzzy/fuzzymatcher.h>
#include "rust/cxx.h"

namespace fuzzy_matcher {
    using ::reflex::FuzzyMatcher;
    using ::reflex::Pattern;
    using ::reflex::Matcher;

    std::unique_ptr<Pattern> create_regex(rust::Str pattern);
    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(std::unique_ptr<Pattern> regex, int8_t max_errors, rust::Str input);
    bool matches(std::unique_ptr<FuzzyMatcher> matcher);
    uint8_t edits(std::unique_ptr<FuzzyMatcher> matcher);
    uint16_t distance(std::unique_ptr<FuzzyMatcher> matcher);
}

#endif // FUZZY_MATCHER_HPP
