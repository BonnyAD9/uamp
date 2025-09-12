#pragma once

#include <stdexcept>
#include <libavutil/error.h>
namespace ufd {
    
[[noreturn]]
static void throw_av_error(int code) {
    throw std::runtime_error(av_err2str(code));
}

static void check_av_error(int code) {
    if (code < 0) {
        throw_av_error(code);
    }
}
    
} // namspace ufd