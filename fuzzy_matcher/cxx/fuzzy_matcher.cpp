#include "fuzzy_matcher.hpp"

namespace fuzzy_matcher {
    std::unique_ptr<Pattern> create_regex(rust::Str pattern) {
        std::string pattern_cpp(pattern.data(), pattern.size());
        std::string regex(reflex::Matcher::convert(pattern_cpp, reflex::convert_flag::unicode));
        return std::make_unique<reflex::Pattern>(regex, "mr");
    }

    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(std::unique_ptr<Pattern> regex, int8_t max_errors, rust::Str input) {
        reflex::FuzzyMatcher matcher(std::move(*regex), max_errors | reflex::FuzzyMatcher::SUB, std::string(input.data(), input.size()));
        return std::make_unique<FuzzyMatcher>(matcher);
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