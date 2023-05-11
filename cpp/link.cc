#include "from_bytes/fast_float.h"

#include "to_bytes/dragonbox.h"
#include "to_bytes/dragonbox_to_chars.h"

extern "C" {
    float from_bytes_f32(const char *input, size_t input_len) {
        float out;
        fast_float::from_chars(input, input + input_len, out);
        return out;
    }
    double from_bytes_f64(const char *input, size_t input_len) {
        double out;
        fast_float::from_chars(input, input + input_len, out);
        return out;
    }

    unsigned int to_bytes_f32(char *buf, float f) {
        auto *end = jkj::dragonbox::to_chars(f, buf);
        return end - buf;
    }
    unsigned int to_bytes_f64(char *buf, double f) {
        auto *end = jkj::dragonbox::to_chars(f, buf);
        return end - buf;
    }
}
