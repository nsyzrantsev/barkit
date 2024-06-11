#include "fuzzy_matcher/cxx/fuzzy_matcher.hpp"

namespace fuzzy_matcher {
    std::unique_ptr<std::string> create_regex(rust::Str pattern) {
        std::string pattern_cpp(pattern.data(), pattern.size());
        return std::make_unique<std::string>(reflex::Matcher::convert(pattern_cpp, reflex::convert_flag::unicode));
    }

    std::unique_ptr<FuzzyMatcher> create_fuzzy_matcher(rust::Str regex, int8_t threshold, rust::Str input) {
        std::string regex_cpp(regex.data(), regex.size());
        std::string input_cpp(input.data(), input.size());
        reflex::FuzzyMatcher matcher(regex_cpp, threshold, input_cpp);
        return std::make_unique<reflex::FuzzyMatcher>(matcher);
    }
}
