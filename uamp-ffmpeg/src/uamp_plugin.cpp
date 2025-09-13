#include <algorithm>
#include <exception>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#include "duration.hpp"
#include "ffmpeg_decoder.hpp"
#include "sample_format.hpp"
#include "uamp_types.hpp"

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
        auto msg = std::unique_ptr<char[]>(new char[size]);

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

static constexpr DecoderPluginFlags operator|(
    DecoderPluginFlags l, DecoderPluginFlags r
) {
    return DecoderPluginFlags(std::uint32_t(l) | std::uint32_t(r));
}

extern "C" {

PluginConfig uamp_plugin_config{
    .version = 0x00'001'000,
    .name = "ffmpeg-decoders",
    .typ = PluginType::DECODER,
};

DecoderPluginConfig uamp_plugin_decoder_config{
    .version = 0x00'001'000,
    .flags = DecoderPluginFlags::CONFIG | DecoderPluginFlags::SEEK |
             DecoderPluginFlags::SEEK_BY | DecoderPluginFlags::GET_TIME,
};

void *uamp_decoder_open(const char *path, std::size_t path_len) {
    ufd::State *res = nullptr;
    try {
        try {
            res = new ufd::State;
            const std::string path2{ std::string_view{ path, path_len } };
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
    } catch (...) {
    }

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
                throw std::runtime_error(
                    "Cannot set config: Decoder not initialized."
                );
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
    } catch (...) {
    }
}

std::size_t uamp_decoder_read(
    void *d, void *b, std::size_t count, SampleFormat fmt
) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    std::size_t written = 0;
    auto sample_size = ufd::get_sample_size(fmt);
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error(
                    "Cannot read: Decoder not initialized."
                );
            }
            state->decoder->read(
                { reinterpret_cast<char *>(b), count * sample_size }, written
            );
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
    } catch (...) {
    }

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

DeviceConfig uamp_decoder_preferred_config(void *d) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    DeviceConfig res{};
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error(
                    "Cannot get preferred config: Decoder not initialized."
                );
            }
            res = state->decoder->preferred_config();
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to get preffered config.");
            }
        }
        // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {
    }

    return res;
}

Timestamp uamp_decoder_seek(void *d, Duration time) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    Timestamp res{};
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error(
                    "Cannot get preferred config: Decoder not initialized."
                );
            }
            state->decoder->seek(ufd::dur_to_secs(time));
            res.current = ufd::secs_to_dur(state->decoder->get_pos());
            res.total = ufd::secs_to_dur(state->decoder->get_length());
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to get preffered config.");
            }
        }
        // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {
    }

    return res;
}

Timestamp uamp_decoder_seek_by(void *d, Duration time, bool forward) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    Timestamp res{};
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error(
                    "Cannot get preferred config: Decoder not initialized."
                );
            }
            auto pos = state->decoder->get_pos();
            auto len = state->decoder->get_length();
            auto chn = ufd::dur_to_secs(time);
            pos = std::clamp(forward ? pos + chn : pos - chn, 0., len);
            state->decoder->seek(pos);
            res.current = ufd::secs_to_dur(state->decoder->get_pos());
            res.total = ufd::secs_to_dur(len);
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to get preffered config.");
            }
        }
        // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {
    }

    return res;
}

Timestamp uamp_decoder_get_time(void *d) {
    auto *state = reinterpret_cast<ufd::State *>(d);
    Timestamp res{};
    try {
        try {
            if (!state || !state->decoder) {
                throw std::runtime_error(
                    "Cannot get preferred config: Decoder not initialized."
                );
            }
            res.current = ufd::secs_to_dur(state->decoder->get_pos());
            res.total = ufd::secs_to_dur(state->decoder->get_length());
        } catch (std::exception &ex) {
            if (state) {
                state->errors.emplace_back(ex.what());
            }
        } catch (...) {
            if (state) {
                state->errors.emplace_back("Failed to get preffered config.");
            }
        }
        // NOLINTNEXTLINE(bugprone-empty-catch)
    } catch (...) {
    }

    return res;
}
}