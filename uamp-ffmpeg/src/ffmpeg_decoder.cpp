#include "ffmpeg_decoder.hpp"

#include <algorithm>
#include <stdexcept>

#include <libavutil/rational.h>
#include <libavutil/samplefmt.h>

#include "av_codec_ctx.hpp"
#include "av_fmt_ctx.hpp"
#include "sample_format.hpp"
#include "uamp_types.hpp"

extern "C" {
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/avutil.h>
} // extern "C"

namespace ufd {

FfmpegDecoder::FfmpegDecoder(const char *path) : _ps(path) {
    _ps.find_stream_info();
    _stream = int(_ps.first_audio_stream());
    auto codec =
        avcodec_find_decoder(_ps->streams[_stream]->codecpar->codec_id);
    if (!codec) {
        throw std::runtime_error("Codec is not supported.");
    }

    _avctx = { codec };
    _avctx.parameters_to_context(_ps->streams[_stream]->codecpar);
    _avctx->request_sample_fmt =
        av_get_alt_sample_fmt(_avctx->sample_fmt, false);
    _avctx->time_base = _ps->streams[_stream]->time_base;

    _avctx.open(codec);
    _is_interleaved = !av_sample_fmt_is_planar(_avctx->sample_fmt);
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
    _sample_size = get_sample_size(sample_fmt);
    if (sample_fmt != conf.sample_format) {
        _reformat = conf.sample_format;
        throw std::runtime_error("Reformatting samples is not supported.");
    }
}

DeviceConfig FfmpegDecoder::preferred_config() {
    return {
        .channel_count = std::uint32_t(_avctx->ch_layout.nb_channels),
        .sample_rate = std::uint32_t(_avctx->sample_rate),
        .sample_format = from_av_sample(_avctx->sample_fmt),
    };
}

void FfmpegDecoder::read(std::span<char> buf, std::size_t &written) {
    if (_frame_continue) {
        read_frame(buf, written);
    }
    if (_drained) {
        return;
    }
    while (buf.size() > written) {
        if (!_resend_pkt) {
            if (!_ps.read_frame(_pkt)) {
                // EOF
                _drained = true;
                _avctx.send_packet();
                read_frames(buf, written);
                return;
            }

            if (_pkt->stream_index != _stream) {
                _pkt.unref();
                continue;
            }
        }

        if (_avctx.send_packet(_pkt)) {
            _pkt.unref();
            _resend_pkt = false;
        } else {
            _resend_pkt = true;
        }
        read_frames(buf, written);
    }
}

void FfmpegDecoder::seek(double secs) {
    _pos = av_rescale_q(
        std::int64_t(secs * AV_TIME_BASE),
        AV_TIME_BASE_Q,
        _ps->streams[_stream]->time_base
    );
    _ps.seek_frame(_stream, _pos);
    _avctx.flush_buffers();
}

double FfmpegDecoder::get_pos() {
    return double(_pos) * av_q2d(_ps->streams[_stream]->time_base);
}

double FfmpegDecoder::get_length() {
    return double(_ps->duration) / AV_TIME_BASE;
}

void FfmpegDecoder::read_frames(std::span<char> buf, std::size_t &written) {
    while (buf.size() > written && _avctx.receive_frame(_frame)) {
        _pos = _frame->pts;
        read_frame(buf, written);
    }
}

void FfmpegDecoder::read_frame(std::span<char> buf, std::size_t &written) {
    buf = buf.subspan(written);
    auto offset = _frame_continue.value_or(0);

    auto channel_count = std::size_t(_frame->ch_layout.nb_channels);
    auto sample_count = std::size_t(_frame->nb_samples);
    auto bps = channel_count * _sample_size;
    auto frame_buf_len = bps * sample_count;

    auto len = std::min(buf.size(), frame_buf_len - offset);

    if (_is_interleaved) {
        std::copy_n(_frame->extended_data[0] + offset, len, buf.begin());
        written += len;
        auto next_byte = offset + len;
        if (next_byte < frame_buf_len) {
            _frame_continue = offset;
        } else {
            _frame.unref();
            _frame_continue.reset();
        }
    } else {
        if (len % bps != 0) {
            throw std::runtime_error(
                "Invalid multiple length of buffer with respect to parameters."
            );
        }

        auto bytes_cnt = (offset + len) / channel_count;
        std::size_t pos = offset / channel_count;
        for (; pos < bytes_cnt; pos += _sample_size) {
            for (int i = 0; i < _frame->ch_layout.nb_channels; ++i) {
                std::copy_n(
                    _frame->extended_data[i] + pos, _sample_size, buf.begin()
                );
                buf = buf.subspan(_sample_size);
            }
            written += bps;
        }
        if (pos < sample_count * _sample_size) {
            _frame_continue = pos * channel_count;
        } else {
            _frame.unref();
            _frame_continue.reset();
        }
    }
}

} // namespace ufd