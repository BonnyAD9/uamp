#include "ffmpeg_decoder.hpp"
#include <algorithm>
#include <stdexcept>
#include <string>
#include <libavutil/error.h>
#include <libavutil/samplefmt.h>
#include "av_codec_ctx.hpp"
#include "av_fmt_ctx.hpp"
#include "sample_format.hpp"
#include "uamp_types.hpp"

extern "C" {

#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libavutil/avutil.h>

} // extern "C"

namespace ufd {
    
FfmpegDecoder::FfmpegDecoder(const char *path) : _ps(path) {
    _ps.find_stream_info();
    _stream = int(_ps.first_audio_stream());
    auto codec = avcodec_find_decoder(_ps->streams[_stream]->codecpar->codec_id);
    if (!codec) {
        throw std::runtime_error("Codec is not supported.");
    }
    
    _avctx = {codec};
    _avctx.parameters_to_context(_ps->streams[_stream]->codecpar);
    _avctx->request_sample_fmt = av_get_alt_sample_fmt(_avctx->sample_fmt, false);
    _avctx.open(codec);
    if (av_sample_fmt_is_planar(_avctx->sample_fmt)) {
        throw std::runtime_error("Planar is not supported.");
    }
}

void FfmpegDecoder::set_config(const DeviceConfig &conf) {
    auto sample_rate = int(conf.sample_rate);
    if (_avctx->sample_rate != sample_rate) {
        _resample = sample_rate;
        throw std::runtime_error("Resampeling is not supported.");
    }
    
    auto channel_cnt = int(conf.channel_count);
    if (_avctx->ch_layout.nb_channels != channel_cnt) {
        _rechannel = channel_cnt;
        throw std::runtime_error("Rechanneling is not supported.");
    }
    
    auto sample_fmt = from_av_sample(_avctx->sample_fmt);
    if (sample_fmt != conf.sample_format) {
        _reformat = conf.sample_format;
        throw std::runtime_error("Reformatting samples is not supported.");
    }
}

void FfmpegDecoder::read(std::span<char> buf, std::size_t &written) {
    // TODO use remainder of _frame->extended_buf
    while (!buf.empty()) {
        if (!_ps.read_frame(_pkt)) {
            return; // EOF
        }
        
        if (_pkt->stream_index != _stream) {
            _pkt.unref();
            continue;
        }
        
        _avctx.send_packet(_pkt); // TODO: handle eagain
        
        while (!buf.empty() && _avctx.receive_frame(_frame)) {
            auto len = std::min(buf.size(), std::size_t(_frame->nb_extended_buf));
            std::copy_n(_frame->extended_data[0], len, buf.begin());
            buf = buf.subspan(len);
            written += len;
            // TODO: len < _frame->nb_extended_buf
        }
    }
}
 
} // namespace ufd