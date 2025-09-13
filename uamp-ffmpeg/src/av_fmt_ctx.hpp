#pragma once

#include <cstdint>
#include <memory>
#include <stdexcept>

#include "av_packet.hpp"
#include "ffmpeg_err.hpp"

extern "C" {
#include <libavformat/avformat.h>
#include <libavutil/avutil.h>
} // extern "C"

namespace ufd {

namespace del {

struct AVFormatContext {
    void operator()(::AVFormatContext *ps) { avformat_close_input(&ps); }
};

} // namespace del

class AVFmtCtx {
public:
    AVFmtCtx() = default;
    AVFmtCtx(AVFmtCtx &&) = default;
    AVFmtCtx &operator=(AVFmtCtx &&) = default;

    AVFmtCtx(const char *path) {
        AVFormatContext *ps = nullptr;
        auto res = avformat_open_input(&ps, path, nullptr, nullptr);
        _ps = std::unique_ptr<AVFormatContext, del::AVFormatContext>(ps);
        check_av_error(res);
    }

    AVFormatContext *get() { return _ps.get(); }

    void find_stream_info() {
        check_av_error(avformat_find_stream_info(get(), nullptr));
    }

    std::size_t first_audio_stream() {
        for (std::size_t i = 0; i < get()->nb_streams; ++i) {
            if (get()->streams[i]->codecpar->codec_type ==
                AVMEDIA_TYPE_AUDIO) {
                return i;
            }
        }
        throw std::runtime_error("No audio stream.");
    }

    bool read_frame(AVPacket &pkt) {
        auto res = av_read_frame(get(), &*pkt);
        if (res == AVERROR_EOF) {
            return false;
        }
        check_av_error(res);
        return true;
    }

    void seek_frame(
        int stream, std::int64_t timestamp, int flags = AVSEEK_FLAG_BACKWARD
    ) {
        check_av_error(av_seek_frame(get(), stream, timestamp, flags));
    }

    AVFormatContext &operator*() { return *get(); }

    AVFormatContext *operator->() { return get(); }

private:
    std::unique_ptr<AVFormatContext, del::AVFormatContext> _ps;
};

} // namespace ufd