#include "fuzzy_matcher/cxx/fuzzy_matcher.hpp"

namespace fuzzy_matcher {
    std::unique_ptr<std::string> create_regex(rust::Str pattern) {
        return std::make_unique<std::string>(pattern.data(), pattern.size());
    }

    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(rust::Str regex, int8_t max_distance, rust::Str input) {
        return std::make_unique<reflex::FuzzyMatcher>(std::string(regex.data(), regex.size()), max_distance, std::string(input.data(), input.size()));
    }
}
