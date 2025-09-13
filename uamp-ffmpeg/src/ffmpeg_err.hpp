#pragma once

#include <stdexcept>

extern "C" {
#include <libavutil/error.h>
} // extern "C"

namespace ufd {

[[noreturn]]
static inline void throw_av_error(int code) {
    throw std::runtime_error(av_err2str(code));
}

static inline void check_av_error(int code) {
    if (code < 0) {
        throw_av_error(code);
    }
}

} // namespace ufd