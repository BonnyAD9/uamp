#pragma once

#include <cstddef>
#include <optional>
#include <span>

#include "av_codec_ctx.hpp"
#include "av_fmt_ctx.hpp"
#include "av_frame.hpp"
#include "av_packet.hpp"
#include "uamp_types.hpp"

namespace ufd {

class FfmpegDecoder {
public:
    FfmpegDecoder(const char *path);

    void set_config(const DeviceConfig &conf);

    void read(std::span<char> buf, std::size_t &written);

    DeviceConfig preferred_config();

private:
    void read_frame(std::span<char> buf, std::size_t &written);

    AVFmtCtx _ps;
    AVCodecCtx _avctx;
    AVPacket _pkt;
    AVFrame _frame;
    int _stream;
    std::optional<std::size_t> _frame_continue;

    std::optional<int> _resample;
    std::optional<int> _rechannel;
    std::optional<SampleFormat> _reformat;
    std::size_t _sample_size;
    bool _is_interleaved;
};

} // namespace ufd