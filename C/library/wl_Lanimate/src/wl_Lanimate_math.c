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

static double wl_Lanimate_fast_sin(double value)
{
    double square;
    square = value * value;
    return value * (1.0 + square * (-1.0 / 6.0 + square * (
        1.0 / 120.0 + square * (-1.0 / 5040.0 + square * (
        1.0 / 362880.0 + square * (-1.0 / 39916800.0 + square * (
        1.0 / 6227020800.0 - square / 1307674368000.0)))))));
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
        angle = wl_Lanimate_fast_sin(t * WL_LANIMATE_PI / 4.0);
        return 2.0 * angle * angle;
    case WL_LANIMATE_EASE_OUT_SINE:
        return wl_Lanimate_fast_sin(t * WL_LANIMATE_PI / 2.0);
    case WL_LANIMATE_EASE_IN_OUT_SINE:
        angle = wl_Lanimate_fast_sin(t * WL_LANIMATE_PI / 2.0);
        return angle * angle;
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

wl_Lanimate_vector3 wl_Lanimate_vector3_lerp(wl_Lanimate_vector3 from_value,
    wl_Lanimate_vector3 to_value, double progress)
{
    wl_Lanimate_vector3 result;
    result.x = wl_Lanimate_lerp(from_value.x, to_value.x, progress);
    result.y = wl_Lanimate_lerp(from_value.y, to_value.y, progress);
    result.z = wl_Lanimate_lerp(from_value.z, to_value.z, progress);
    return result;
}

wl_Lanimate_vector4 wl_Lanimate_vector4_lerp(wl_Lanimate_vector4 from_value,
    wl_Lanimate_vector4 to_value, double progress)
{
    wl_Lanimate_vector4 result;
    result.x = wl_Lanimate_lerp(from_value.x, to_value.x, progress);
    result.y = wl_Lanimate_lerp(from_value.y, to_value.y, progress);
    result.z = wl_Lanimate_lerp(from_value.z, to_value.z, progress);
    result.w = wl_Lanimate_lerp(from_value.w, to_value.w, progress);
    return result;
}

wl_Lanimate_rect wl_Lanimate_rect_lerp(wl_Lanimate_rect from_value,
    wl_Lanimate_rect to_value, double progress)
{
    wl_Lanimate_rect result;
    result.x = wl_Lanimate_lerp(from_value.x, to_value.x, progress);
    result.y = wl_Lanimate_lerp(from_value.y, to_value.y, progress);
    result.width = wl_Lanimate_lerp(from_value.width, to_value.width, progress);
    result.height = wl_Lanimate_lerp(from_value.height, to_value.height, progress);
    return result;
}

wl_Lanimate_matrix3x2 wl_Lanimate_matrix3x2_identity(void)
{
    wl_Lanimate_matrix3x2 result;
    result.m11 = 1.0; result.m12 = 0.0;
    result.m21 = 0.0; result.m22 = 1.0;
    result.dx = 0.0; result.dy = 0.0;
    return result;
}

wl_Lanimate_matrix3x2 wl_Lanimate_matrix3x2_compose(
    wl_Lanimate_matrix3x2 self, wl_Lanimate_matrix3x2 rhs)
{
    wl_Lanimate_matrix3x2 result;
    result.m11 = self.m11 * rhs.m11 + self.m21 * rhs.m12;
    result.m12 = self.m12 * rhs.m11 + self.m22 * rhs.m12;
    result.m21 = self.m11 * rhs.m21 + self.m21 * rhs.m22;
    result.m22 = self.m12 * rhs.m21 + self.m22 * rhs.m22;
    result.dx = self.m11 * rhs.dx + self.m21 * rhs.dy + self.dx;
    result.dy = self.m12 * rhs.dx + self.m22 * rhs.dy + self.dy;
    return result;
}

wl_Lanimate_point2 wl_Lanimate_matrix3x2_transform_point(
    wl_Lanimate_matrix3x2 matrix, wl_Lanimate_point2 point)
{
    wl_Lanimate_point2 result;
    result.x = matrix.m11 * point.x + matrix.m21 * point.y + matrix.dx;
    result.y = matrix.m12 * point.x + matrix.m22 * point.y + matrix.dy;
    return result;
}

wl_Lanimate_opacity wl_Lanimate_opacity_make(double value)
{
    wl_Lanimate_opacity result;
    result.value = wl_Lanimate_clamp01(value);
    return result;
}

wl_Lanimate_transform2d wl_Lanimate_transform2d_lerp(
    wl_Lanimate_transform2d from_value, wl_Lanimate_transform2d to_value,
    double progress)
{
    wl_Lanimate_transform2d result;
    double delta;
    double t;
    t = wl_Lanimate_clamp01(progress);
    result.translation = wl_Lanimate_point2_lerp(from_value.translation, to_value.translation, t);
    result.scale = wl_Lanimate_point2_lerp(from_value.scale, to_value.scale, t);
    delta = fmod(to_value.rotation_radians - from_value.rotation_radians, 2.0 * WL_LANIMATE_PI);
    if (delta > WL_LANIMATE_PI) delta -= 2.0 * WL_LANIMATE_PI;
    if (delta < -WL_LANIMATE_PI) delta += 2.0 * WL_LANIMATE_PI;
    result.rotation_radians = from_value.rotation_radians + delta * t;
    return result;
}
