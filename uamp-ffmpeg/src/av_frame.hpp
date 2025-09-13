#pragma once

#include <memory>
#include <stdexcept>

extern "C" {
#include <libavutil/frame.h>
} // extern "C"

namespace ufd {

namespace del {

struct AVFrame {
    void operator()(::AVFrame *frame) { av_frame_free(&frame); }
};

} // namespace del

class AVFrame {
public:
    AVFrame() : _frame(av_frame_alloc()) {
        if (!_frame) {
            throw std::runtime_error("Failed to allcoate frame.");
        }
    }

    ::AVFrame *get() { return _frame.get(); }

    void unref() { av_frame_unref(get()); }

    ::AVFrame &operator*() { return *get(); }

    ::AVFrame *operator->() { return get(); }

private:
    std::unique_ptr<::AVFrame, del::AVFrame> _frame;
};

} // namespace ufd