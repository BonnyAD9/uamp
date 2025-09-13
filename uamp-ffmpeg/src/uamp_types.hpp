#pragma once

#include <cstddef>
#include <cstdint>

extern "C" {

enum class SampleFormat : std::int32_t {
    UNKNOWN = 0,
    I8 = -8,
    I16 = -16,
    I24 = -24,
    I32 = -32,
    I64 = -64,
    U8 = 8,
    U16 = 16,
    U32 = 32,
    U64 = 64,
    F32 = 3200,
    F64 = 6400,
};

struct DeviceConfig {
    std::uint32_t channel_count;
    std::uint32_t sample_rate;
    SampleFormat sample_format;
};

struct Duration {
    std::uint64_t secs;
    std::uint32_t nanos;
};

struct String {
    const char *data;
    std::size_t len;
    void (*free)(const char *, std::size_t);
};

enum class ErrorType : std::int32_t {
    NO_ERROR = 0,
    RECOVERABLE = 1,
    FATAL = 2,
};

struct Error {
    String msg;
    ErrorType typ;
};

struct Timestamp {
    Duration current;
    Duration total;
};

struct VolumeIterator {
    bool linear;
    float base;
    float step;
    std::int32_t cur_count;
    std::int32_t target_count;
    std::size_t channel_count;
    std::size_t cur_channel;
};

enum class PluginType : std::int32_t { DECODER = 1 };

struct PluginConfig {
    std::uint32_t version;
    const char *name;
    PluginType typ;
};

enum class DecoderPluginFlags : std::uint32_t {
    NONE = 0x0,
    VOLUME = 0x1,
    CONFIG = 0x2,
    SEEK = 0x4,
    SEEK_BY = 0x8,
    GET_TIME = 0x10,
};

struct DecoderPluginConfig {
    std::uint32_t version;
    DecoderPluginFlags flags;
};

} // extern "C"