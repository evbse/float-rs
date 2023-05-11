#include "from_bytes/fast_float.h"

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
}
