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

    void seek(double secs);

    double get_pos();

    double get_length();

private:
    void read_frames(std::span<char> buf, std::size_t &written);
    void read_frame(std::span<char> buf, std::size_t &written);

    AVFmtCtx _ps;
    AVCodecCtx _avctx;
    AVPacket _pkt;
    AVFrame _frame;
    int _stream;
    std::optional<std::size_t> _frame_continue;
    bool _drained = false;
    bool _resend_pkt = false;

    std::int64_t _pos = 0;

    std::optional<int> _resample;
    std::optional<int> _rechannel;
    std::optional<SampleFormat> _reformat;
    std::size_t _sample_size;
    bool _is_interleaved;
};

} // namespace ufd