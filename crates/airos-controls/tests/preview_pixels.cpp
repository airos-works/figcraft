// Run on an unobstructed air/OS workspace. Compare the actual-size preview with a
// second, independently assembled Interface Kit window at exactly the same screen
// coordinates. This preserves the device-pixel phase on fractional-scale displays.
#include "../preview.h"
#include <Application.h>
#include <Bitmap.h>
#include <Button.h>
#include <CheckBox.h>
#include <RadioButton.h>
#include <Screen.h>
#include <TextControl.h>
#include <Window.h>
#include <cstdio>
#include <cstring>
#include <vector>

static bool
ReadPixels(BRect frame, std::vector<uint32>& result)
{
    int width = frame.IntegerWidth() + 1;
    int height = frame.IntegerHeight() + 1;
    BBitmap bitmap(BRect(0, 0, width - 1, height - 1), B_RGB32);
    if (BScreen().ReadBitmap(&bitmap, false, &frame) != B_OK)
        return false;
    result.resize(width * height);
    for (int y = 0; y < height; ++y) {
        const auto* row = static_cast<const uint8*>(bitmap.Bits()) + y * bitmap.BytesPerRow();
        std::memcpy(result.data() + width * y, row, width * sizeof(uint32));
    }
    return true;
}

static BRect
Frame(const PreviewWidget& item)
{
    return BRect(item.x, item.y, item.x + item.width - 1, item.y + item.height - 1);
}

int
main()
{
    BApplication app("application/x-vnd.airos-FigCraft-preview-test");
    PreviewWidget items[] = {
        {"BButton", "Apply", "", 16, 12, 170, 30, true, false},
        {"BCheckBox", "Show labels", "", 16, 48, 170, 28, true, true},
        {"BRadioButton", "Automatic", "", 16, 84, 170, 28, true, true},
        {"BTextControl", "Name", "Example", 16, 120, 170, 25, true, false},
    };
    const char* title = "FigCraft preview pixel comparison";
    std::vector<uint8> background(220 * 180 * 4, 255);
    if (fc_present_design(title, 220, 180, background.data(), items, 4) != 4)
        return 2;
    BWindow* preview = nullptr;
    for (int i = 0; i < app.CountWindows(); ++i) {
        auto* window = app.WindowAt(i);
        if (window && std::strcmp(window->Title(), title) == 0)
            preview = window;
    }
    if (!preview || !preview->Lock())
        return 3;
    preview->MoveTo(400, 300);
    preview->UpdateIfNeeded();
    preview->Unlock();
    snooze(120000);
    preview->Lock();
    auto* page = preview->FindView("Design page");
    if (!page)
        return 4;
    BPoint origin = page->ConvertToScreen(B_ORIGIN);
    BRect frames[4];
    std::vector<uint32> expected[4];
    for (int i = 0; i < 4; ++i) {
        frames[i] = page->ConvertToScreen(Frame(items[i]));
        if (!ReadPixels(frames[i], expected[i]))
            return 5;
    }
    preview->Hide();
    preview->Unlock();

    auto* reference = new BWindow(BRect(0, 0, 219, 179), "Native reference", B_TITLED_WINDOW,
        B_ASYNCHRONOUS_CONTROLS);
    auto* panel = new BView(reference->Bounds(), "panel", B_FOLLOW_NONE, B_WILL_DRAW);
    panel->SetViewUIColor(B_PANEL_BACKGROUND_COLOR);
    reference->AddChild(panel);
    auto* field = new BTextControl(Frame(items[3]), "field", "Name", "Example", nullptr);
    field->SetDivider(Frame(items[3]).Width() / 2);
    BControl* controls[] = {
        new BButton(Frame(items[0]), "button", "Apply", nullptr),
        new BCheckBox(Frame(items[1]), "check", "Show labels", nullptr),
        new BRadioButton(Frame(items[2]), "radio", "Automatic", nullptr),
        field,
    };
    for (int i = 0; i < 4; ++i) {
        if (items[i].checked)
            controls[i]->SetValue(B_CONTROL_ON);
        panel->AddChild(controls[i]);
        controls[i]->ResizeTo(Frame(items[i]).Width(), Frame(items[i]).Height());
    }
    reference->MoveTo(origin);
    reference->Show();
    snooze(120000);
    reference->Lock();
    reference->UpdateIfNeeded();
    panel->Sync();
    size_t total = 0;
    for (int i = 0; i < 4; ++i) {
        std::vector<uint32> actual;
        if (!ReadPixels(frames[i], actual))
            return 6;
        size_t differences = 0;
        for (size_t j = 0; j < actual.size(); ++j) {
            if ((actual[j] & 0xffffff) != (expected[i][j] & 0xffffff))
                ++differences;
        }
        std::printf("%s: %zu / %zu pixels differ\n", items[i].kind, differences, actual.size());
        total += differences;
    }
    reference->Quit();
    preview->Lock();
    preview->Quit();
    return total != 0;
}
