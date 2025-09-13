#pragma once

#include <memory>
#include <stdexcept>

#include "av_frame.hpp"
#include "av_packet.hpp"
#include "ffmpeg_err.hpp"

extern "C" {
#include <libavcodec/avcodec.h>
#include <libavcodec/codec.h>
#include <libavformat/avformat.h>
} // extern "C"

namespace ufd {

namespace del {

struct AVCodecContext {
    void operator()(::AVCodecContext *avctx) { avcodec_free_context(&avctx); }
};

} // namespace del

class AVCodecCtx {
public:
    AVCodecCtx() = default;
    AVCodecCtx(AVCodecCtx &&) = default;
    AVCodecCtx &operator=(AVCodecCtx &&) = default;

    AVCodecCtx(const AVCodec *codec) : _avctx(avcodec_alloc_context3(codec)) {
        if (!_avctx) {
            throw std::runtime_error("Unsupported codec.");
        }
    }

    AVCodecContext *get() { return _avctx.get(); }

    void parameters_to_context(const AVCodecParameters *par) {
        check_av_error(avcodec_parameters_to_context(get(), par));
    }

    void open(const AVCodec *codec) {
        check_av_error(avcodec_open2(get(), codec, nullptr));
    }

    bool send_packet(AVPacket &pkt) {
        auto res = avcodec_send_packet(get(), &*pkt);
        if (res == AVERROR(EAGAIN)) {
            return false;
        }
        check_av_error(res);
        return true;
    }

    void send_packet() { check_av_error(avcodec_send_packet(get(), nullptr)); }

    bool receive_frame(AVFrame &frame) {
        auto res = avcodec_receive_frame(get(), &*frame);
        if (res == AVERROR_EOF || res == AVERROR(EAGAIN)) {
            return false;
        }
        check_av_error(res);
        return true;
    }

    void flush_buffers() { avcodec_flush_buffers(get()); }

    AVCodecContext &operator*() { return *get(); }

    AVCodecContext *operator->() { return get(); }

private:
    std::unique_ptr<AVCodecContext, del::AVCodecContext> _avctx;
};

} // namespace ufd