#pragma once

#include <cmath>

#include "uamp_types.hpp"

namespace ufd {

constexpr double NS_IN_S = 1'000'000'000;

static inline double dur_to_secs(Duration dur) {
    return double(dur.secs) + double(dur.nanos) / NS_IN_S;
}

static inline Duration secs_to_dur(double secs) {
    auto s = std::floor(secs);
    auto f = secs - s;
    return {
        .secs = std::uint64_t(s),
        .nanos = std::uint32_t(f * NS_IN_S),
    };
}

} // namespace ufd