#include "wl_Lanimate.h"

#include <float.h>
#include <math.h>

#define WL_LANIMATE_PI 3.14159265358979323846

static double wl_Lanimate_clamp01(double value)
{
    if (value != value) return 0.0;
    if (value < 0.0) return 0.0;
    if (value > 1.0) return 1.0;
    return value;
}

static double wl_Lanimate_lerp(double from_value, double to_value, double progress)
{
    return from_value + (to_value - from_value) * wl_Lanimate_clamp01(progress);
}

double wl_Lanimate_ease(wl_Lanimate_easing easing, double progress)
{
    double t;
    double inverse;
    double angle;
    t = wl_Lanimate_clamp01(progress);
    inverse = 1.0 - t;
    switch (easing) {
    case WL_LANIMATE_EASE_IN_QUAD: return t * t;
    case WL_LANIMATE_EASE_OUT_QUAD: return 1.0 - inverse * inverse;
    case WL_LANIMATE_EASE_IN_OUT_QUAD:
        if (t < 0.5) return 2.0 * t * t;
        return 1.0 - ((-2.0 * t + 2.0) * (-2.0 * t + 2.0)) / 2.0;
    case WL_LANIMATE_EASE_IN_CUBIC: return t * t * t;
    case WL_LANIMATE_EASE_OUT_CUBIC: return 1.0 - inverse * inverse * inverse;
    case WL_LANIMATE_EASE_IN_OUT_CUBIC:
        if (t < 0.5) return 4.0 * t * t * t;
        return 1.0 - ((-2.0 * t + 2.0) * (-2.0 * t + 2.0) * (-2.0 * t + 2.0)) / 2.0;
    case WL_LANIMATE_EASE_IN_SINE:
        angle = t * WL_LANIMATE_PI / 2.0;
        return 1.0 - cos(angle);
    case WL_LANIMATE_EASE_OUT_SINE:
        angle = t * WL_LANIMATE_PI / 2.0;
        return sin(angle);
    case WL_LANIMATE_EASE_IN_OUT_SINE:
        angle = WL_LANIMATE_PI * t;
        return -(cos(angle) - 1.0) / 2.0;
    case WL_LANIMATE_EASE_IN_QUART: return t * t * t * t;
    case WL_LANIMATE_EASE_OUT_QUART: return 1.0 - inverse * inverse * inverse * inverse;
    case WL_LANIMATE_EASE_IN_OUT_QUART:
        if (t < 0.5) return 8.0 * t * t * t * t;
        return 1.0 - ((-2.0 * t + 2.0) * (-2.0 * t + 2.0) *
                      (-2.0 * t + 2.0) * (-2.0 * t + 2.0)) / 2.0;
    case WL_LANIMATE_LINEAR:
    default: return t;
    }
}

wl_Lanimate_point2 wl_Lanimate_point2_lerp(wl_Lanimate_point2 from_value,
                                            wl_Lanimate_point2 to_value,
                                            double progress)
{
    wl_Lanimate_point2 result;
    result.x = wl_Lanimate_lerp(from_value.x, to_value.x, progress);
    result.y = wl_Lanimate_lerp(from_value.y, to_value.y, progress);
    return result;
}

wl_Lanimate_rgba wl_Lanimate_rgba_lerp(wl_Lanimate_rgba from_value,
                                        wl_Lanimate_rgba to_value,
                                        double progress)
{
    wl_Lanimate_rgba result;
    result.r = wl_Lanimate_lerp(from_value.r, to_value.r, progress);
    result.g = wl_Lanimate_lerp(from_value.g, to_value.g, progress);
    result.b = wl_Lanimate_lerp(from_value.b, to_value.b, progress);
    result.a = wl_Lanimate_lerp(from_value.a, to_value.a, progress);
    return result;
}
