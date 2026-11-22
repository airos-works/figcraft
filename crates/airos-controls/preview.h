#pragma once

#include <SupportDefs.h>

struct PreviewWidget {
    const char* kind;
    const char* label;
    const char* value;
    int32 x, y;
    uint32 width, height;
    bool enabled, checked;
};

extern "C" int32 fc_present_design(const char* title, uint32 width, uint32 height,
    const uint8* rgba, const PreviewWidget* widgets, size_t count);
