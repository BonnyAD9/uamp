#pragma once

#include <memory>

extern "C" {
#include <libavcodec/packet.h>
} // extern "C"

namespace ufd {

namespace del {

struct AVPacket {
    void operator()(::AVPacket *pkt) { av_packet_free(&pkt); }
};

} // namespace del

class AVPacket {
public:
    AVPacket() : _pkt(av_packet_alloc()) {
        if (!_pkt) {
            throw std::runtime_error("Failed to allocate packet.");
        }
    }

    ::AVPacket *get() { return _pkt.get(); }

    void unref() { av_packet_unref(get()); }

    ::AVPacket &operator*() { return *get(); }

    ::AVPacket *operator->() { return get(); }

private:
    std::unique_ptr<::AVPacket, del::AVPacket> _pkt;
};

} // namespace ufd