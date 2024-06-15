#include "fuzzy_matcher/cxx/fuzzy_matcher.hpp"

namespace fuzzy_matcher {
    std::unique_ptr<std::string> create_regex(rust::Str pattern) {
        std::string pattern_cpp(pattern.data(), pattern.size());
        return std::make_unique<std::string>(std::move(pattern_cpp));
    }

    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(std::unique_ptr<std::string> regex, int8_t max_errors, rust::Str input) {
        return std::make_unique<FuzzyMatcher>(std::move(*regex), max_errors, std::string(input.data(), input.size()));
    }

    bool matches(std::unique_ptr<FuzzyMatcher> matcher) {
        return matcher->matches() > 0;
    }

    uint8_t edits(std::unique_ptr<FuzzyMatcher> matcher) {
        return matcher->edits();
    }

    uint16_t distance(std::unique_ptr<FuzzyMatcher> matcher) {
        return matcher->distance();
    }
}