#pragma once

#include <cstddef>
#include <stdexcept>

#include "uamp_types.hpp"

extern "C" {
#include <libavutil/samplefmt.h>
} // extern "C"

namespace ufd {

static inline std::size_t get_sample_size(SampleFormat fmt) noexcept {
    switch (fmt) {
    case SampleFormat::U8:
    case SampleFormat::I8:
        return 1;
    case SampleFormat::U16:
    case SampleFormat::I16:
        return 2;
    case SampleFormat::I24:
        return 3;
    case SampleFormat::F32:
    case SampleFormat::U32:
    case SampleFormat::I32:
        return 4;
    case SampleFormat::F64:
    case SampleFormat::U64:
    case SampleFormat::I64:
        return 8;
    default:
        return 0;
    }
}

static inline SampleFormat from_av_sample(AVSampleFormat fmt) {
    switch (fmt) {
    case AV_SAMPLE_FMT_U8P:
    case AV_SAMPLE_FMT_U8:
        return SampleFormat::U8;
    case AV_SAMPLE_FMT_S16P:
    case AV_SAMPLE_FMT_S16:
        return SampleFormat::I16;
    case AV_SAMPLE_FMT_S32P:
    case AV_SAMPLE_FMT_S32:
        return SampleFormat::I32;
    case AV_SAMPLE_FMT_FLTP:
    case AV_SAMPLE_FMT_FLT:
        return SampleFormat::F32;
    case AV_SAMPLE_FMT_DBLP:
    case AV_SAMPLE_FMT_DBL:
        return SampleFormat::F64;
    case AV_SAMPLE_FMT_S64P:
    case AV_SAMPLE_FMT_S64:
        return SampleFormat::I64;
    default:
        throw std::runtime_error("Unsupported sample format.");
    }
}

} // namespace ufd