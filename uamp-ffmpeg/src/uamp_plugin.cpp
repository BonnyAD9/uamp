#include "ffmpeg_decoder.hpp"
#include "sample_format.hpp"
#include "uamp_types.hpp"
#include <algorithm>
#include <exception>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string_view>
#include <vector>
#include <string>

namespace ufd {

extern "C" void uamp_unique_error_free_string(const char *str, std::size_t) {
    delete[] str;
}

static Error no_error() noexcept {
    return {
        .msg{
            .data = nullptr,
            .len = 0,
            .free = uamp_unique_error_free_string,
        },
        .typ = ErrorType::NO_ERROR,
    };
}

struct State {
    std::vector<std::string> errors;
    std::optional<FfmpegDecoder> decoder;
    
    Error pop_error() {
        if (errors.empty()) {
            return no_error();
        }
        auto err = errors.rbegin();
        
        auto size = err->size();
        // NOLINTNEXTLINE(modernize-avoid-c-arrays)
        auto msg = std::unique_ptr<char []>(new char[size]);

        std::copy_n(err->begin(), size, msg.get());
        errors.pop_back();
        
        return {
            .msg{
                .data = msg.release(),
                .len = size,
                .free = uamp_unique_error_free_string,
            },
            .typ = ErrorType::FATAL,
        };
    }
};

} // namespace ufd 


extern "C" {

PluginConfig uamp_plugin_config{
    .version = 0x00'001'000,
    .name = "ffmpeg-decoders",
    .typ = PluginType::DECODER,
};

DecoderPluginConfig uamp_plugin_decoder_config{
    .version = 0x00'001'000,
    .flags = DecoderPluginFlags::NONE,
};

void *uamp_decoder_open(const char *path, std::size_t path_len) {
    ufd::State *res = nullptr;
    try {
        try {
            res = new ufd::State;
            std::string path2{std::string_view{path, path_len}};
            res->decoder = ufd::FfmpegDecoder(path2.c_str());
        } catch (std::exception &ex) {
            if (res) {
                res->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (res) {
                res->errors.emplace_back("Failed to create decoder.");
            }
        }
    // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {}
    
    return res;
}

void uamp_decoder_free(void *d) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    delete state;
}

void uamp_decoder_set_config(void *d, const DeviceConfig *conf) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error("Cannot set config: Decoder not initialized.");
            }
            state->decoder->set_config(*conf);
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to set confugration.");
            }
        }
    // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {}
}

std::size_t uamp_decoder_read(void *d, void *b, std::size_t count, SampleFormat fmt) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    std::size_t written = 0;
    auto sample_size = ufd::get_sample_size(fmt);
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error("Cannot read: Decoder not initialized.");
            }
            state->decoder->read({ reinterpret_cast<char *>(b), count * sample_size }, written);
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to read.");
            }
        }
    // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {}
    
    return written / sample_size;
}

Error uamp_decoder_err(void *d) {
    auto state = reinterpret_cast<ufd::State *>(d);
    if (!state) {
        return ufd::no_error();
    }
    try {
        return state->pop_error();
    } catch (...) {
        return ufd::no_error();
    }
}

}