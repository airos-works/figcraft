// Native widget snapshots for the design canvas and saved-design review.
#include "preview.h"
#include <Application.h>
#include <Bitmap.h>
#include <Box.h>
#include <Button.h>
#include <ChannelSlider.h>
#include <CheckBox.h>
#include <ColorControl.h>
#include <ColumnTypes.h>
#include <StripeView.h>
#include <IconView.h>
#include <ListView.h>
#include <MenuBar.h>
#include <MenuField.h>
#include <MenuItem.h>
#include <OptionPopUp.h>
#include <OutlineListView.h>
#include <PictureButton.h>
#include <PopUpMenu.h>
#include <RadioButton.h>
#include <ScrollBar.h>
#include <ScrollView.h>
#include <SeparatorView.h>
#include <Slider.h>
#include <StatusBar.h>
#include <StringItem.h>
#include <StringView.h>
#include <TabView.h>
#include <TextControl.h>
#include <TextView.h>
#include <Window.h>
#include <AboutMenuItem.h>
#include <BarberPole.h>
#include <BitmapButton.h>
#include <CalendarView.h>
#include <ColorItem.h>
#include <ColorListView.h>
#include <ColorMenuItem.h>
#include <ColorPreview.h>
#include <DateTimeEdit.h>
#include <DecimalSpinner.h>
#include <IconButton.h>
#include <Spinner.h>
#include <ToolBar.h>
#include <GroupView.h>
#include <SplitView.h>
#include <Dragger.h>
#include <ToolTip.h>
#include <cmath>
#include <ctime>

#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

namespace {
// A free-standing BMenu normally resizes its popup window. In a bitmap the owner
// is an offscreen BWindow, so use the fixed-size constructor intended for embedding.
class EmbeddedMenu : public BMenu {
public:
    explicit EmbeddedMenu(BRect frame)
        : BMenu(frame, "menu", B_FOLLOW_NONE, B_WILL_DRAW, B_ITEMS_IN_COLUMN, false) {}
};

class ColumnCell : public BView {
public:
    ColumnCell(BRect frame, BColumn* column, BField* field)
        : BView(frame, "Column cell", B_FOLLOW_NONE, B_WILL_DRAW), fColumn(column), fField(field) {
        SetViewUIColor(B_LIST_BACKGROUND_COLOR);
        SetHighUIColor(B_LIST_ITEM_TEXT_COLOR);
    }
    void Draw(BRect) override { fColumn->DrawField(fField.get(), Bounds(), this); }
private:
    std::unique_ptr<BColumn> fColumn;
    std::unique_ptr<BField> fField;
};

class ToolTipSample : public BView {
public:
    ToolTipSample(BRect r, const char* label)
        : BView(r, "Tooltip", B_FOLLOW_NONE, B_WILL_DRAW), fTip(label) {
        SetViewUIColor(B_TOOL_TIP_BACKGROUND_COLOR);
        AddChild(fTip.View());
    }
    ~ToolTipSample() { RemoveChild(fTip.View()); }
private:
    BTextToolTip fTip;
};

struct Sample {
    // BListView borrows its items. Keep them alive until the bitmap's view tree is gone.
    std::vector<std::unique_ptr<BStringItem>> items;

    std::vector<std::unique_ptr<BBitmap>> icons;

    BBitmap* Icon() {
        auto icon = std::make_unique<BBitmap>(BRect(0, 0, 15, 15), B_RGBA32);
        if (icon->InitCheck() == B_OK) {
            std::memset(icon->Bits(), 0, icon->BitsLength());
            auto* bits = static_cast<uint32*>(icon->Bits());
            for (int i = 2; i < 14; ++i) bits[i * (icon->BytesPerRow() / 4) + 15 - i] = 0xff222222;
        }
        icons.push_back(std::move(icon));
        return icons.back().get();
    }

    BView* Make(const std::string& kind, BRect r, const char* label, const char* value,
        bool checked, BView* surface)
    {
        const int percent = std::clamp(std::strtol(value, nullptr, 10), 0L, 100L);
        rgb_color color = ui_color(B_CONTROL_HIGHLIGHT_COLOR);
        unsigned red, green, blue;
        if (std::strlen(value) == 7 && sscanf(value, "#%2x%2x%2x", &red, &green, &blue) == 3)
            color = {static_cast<uint8>(red), static_cast<uint8>(green), static_cast<uint8>(blue), 255};
        if (kind == "BTextToolTip") return new ToolTipSample(r, label);
        if (kind == "BDragger") return new BDragger(r, surface);
        if (kind == "BButton") return new BButton(r, "button", label, nullptr);
        if (kind == "BCheckBox" || kind == "BRadioButton") {
            BControl* control = kind == "BCheckBox"
                ? static_cast<BControl*>(new BCheckBox(r, "check", label, nullptr))
                : static_cast<BControl*>(new BRadioButton(r, "radio", label, nullptr));
            control->SetValue(checked ? B_CONTROL_ON : B_CONTROL_OFF);
            return control;
        }
        if (kind == "BSlider") {
            auto* slider = new BSlider(r, "slider", label, nullptr, 0, 100, B_HORIZONTAL);
            slider->SetValue(percent);
            return slider;
        }
        if (kind == "BChannelSlider") {
            auto* slider = new BChannelSlider(r, "channels", label, nullptr, B_HORIZONTAL, 2);
            slider->SetLimits(0, 100);
            slider->SetAllValue(percent);
            return slider;
        }
        if (kind == "BStatusBar") {
            auto* bar = new BStatusBar(r, "progress", label);
            bar->SetTo(percent);
            return bar;
        }
        if (kind == "BTextControl" || kind == "BTextControl / Password") {
            auto* field = new BTextControl(r, "field", label, value, nullptr);
            field->SetDivider(r.Width() / 2);
            field->TextView()->HideTyping(kind == "BTextControl / Password");
            return field;
        }
        if (kind == "BTextView") {
            BRect inset(r);
            inset.InsetBy(3, 3);
            auto* text = new BTextView(r, "text", inset, B_FOLLOW_NONE, B_WILL_DRAW);
            text->SetText(label);
            return text;
        }
        if (kind == "BStringView") return new BStringView(r, "label", label);
        if (kind == "BOptionPopUp") {
            auto* options = new BOptionPopUp(r, "options", "", nullptr);
            options->AddOption(label, 1);
            options->SetValue(1);
            return options;
        }
        if (kind == "BMenuField") {
            auto* menu = new BPopUpMenu(value);
            auto* item = new BMenuItem(value, nullptr);
            menu->AddItem(item);
            item->SetMarked(true);
            return new BMenuField(r, "menu field", label, menu);
        }
        if (kind == "BColorControl") {
            auto* picker = new BColorControl(B_ORIGIN, B_CELLS_32x8, 4, "color");
            picker->SetValue(color);
            return picker;
        }
        if (kind == "BPictureButton") {
            // A picture button paints the picture supplied by the app.
            auto drawPicture = [&](bool pressed) {
                surface->BeginPicture(new BPicture());
                surface->SetHighColor(ui_color(pressed ? B_CONTROL_HIGHLIGHT_COLOR : B_CONTROL_TEXT_COLOR));
                surface->StrokeLine(BPoint(6, 18), BPoint(19, 5));
                surface->StrokeLine(BPoint(7, 19), BPoint(20, 6));
                return surface->EndPicture();
            };
            std::unique_ptr<BPicture> off(drawPicture(false)), on(drawPicture(true));
            return new BPictureButton(r, "picture", off.get(), on.get(), nullptr);
        }
        if (kind == "BMenuBar") {
            auto* bar = new BMenuBar(r, "menu bar");
            for (auto title : {label, "Edit", "View"}) bar->AddItem(new BMenu(title));
            return bar;
        }
        if (kind == "BMenu" || kind == "BPopUpMenu" || kind == "BMenuItem" || kind == "BSeparatorItem") {
            auto* menu = new EmbeddedMenu(r);
            if (kind == "BSeparatorItem") menu->AddSeparatorItem();
            else {
                menu->AddItem(new BMenuItem(label, nullptr));
                if (kind != "BMenuItem") {
                    menu->AddItem(new BMenuItem("Save", nullptr));
                    menu->AddItem(new BMenuItem("Close", nullptr));
                }
            }
            return menu;
        }
        if (kind == "BListView" || kind == "BOutlineListView" || kind == "BScrollView" || kind == "BStringItem") {
            BListView* list = kind == "BOutlineListView" ? new BOutlineListView(r, "outline") : new BListView(r, "list");
            for (auto text : {label, "Item", "Details"}) {
                auto* item = new BStringItem(text, kind == "BOutlineListView" && list->CountItems() != 0 ? 1 : 0);
                items.emplace_back(item);
                list->AddItem(item);
                if (kind == "BStringItem") break;
            }
            if (kind != "BStringItem") list->Select(0);
            if (kind == "BScrollView") return new BScrollView("scroll", list, B_FOLLOW_NONE, 0, false, true);
            return list;
        }
        if (kind == "BScrollBar") {
            auto* bar = new BScrollBar(r, "scrollbar", nullptr, 0, 100, B_HORIZONTAL);
            bar->SetProportion(.3f);
            bar->SetValue(percent);
            return bar;
        }
        if (kind == "BTabView" || kind == "BTab") {
            auto* tabs = new BTabView(r, "tabs");
            tabs->AddTab(new BView(r, label, B_FOLLOW_NONE, B_WILL_DRAW));
            if (kind == "BTabView") tabs->AddTab(new BView(r, "Advanced", B_FOLLOW_NONE, B_WILL_DRAW));
            return tabs;
        }
        if (kind == "BSeparatorView") return new BSeparatorView(B_HORIZONTAL);
        if (kind == "BBox") {
            auto* box = new BBox(r);
            box->SetLabel(label);
            return box;
        }
        if (kind == "BSpinner") {
            auto* spin = new BSpinner(r, "spinner", "", nullptr);
            spin->SetValue(std::clamp(std::strtol(value, nullptr, 10), -2147483647L, 2147483647L));
            return spin;
        }
        if (kind == "BDecimalSpinner") {
            auto* spin = new BDecimalSpinner(r, "spinner", "", nullptr);
            const double v = std::strtod(value, nullptr);
            spin->SetValue(std::isfinite(v) ? v : 0.0);
            return spin;
        }
        if (kind == "BCalendarView") {
            auto* calendar = new BPrivate::BCalendarView(r, "calendar");
            int year, month, day;
            if (sscanf(value, "%4d-%2d-%2d", &year, &month, &day) == 3)
                calendar->SetDate(year, month, day);
            return calendar;
        }
        if (kind == "TimeEdit") {
            auto* time = new BPrivate::TimeEdit("time", 3, nullptr);
            int hour, minute, second = 0;
            if (sscanf(value, "%2d:%2d:%2d", &hour, &minute, &second) >= 2 && hour >= 0 && hour < 24 && minute >= 0 && minute < 60 && second >= 0 && second < 60)
                time->SetTime(hour, minute, second);
            return time;
        }
        if (kind == "DateEdit") {
            auto* date = new BPrivate::DateEdit("date", 3, nullptr);
            int year, month, day;
            if (sscanf(value, "%4d-%2d-%2d", &year, &month, &day) == 3 && BDate(year, month, day).IsValid())
                date->SetDate(year, month, day);
            return date;
        }
        if (kind == "BColorPreview") {
            auto* preview = new BPrivate::BColorPreview("color", label, nullptr);
            preview->SetColor(color);
            return preview;
        }
        if (kind == "BColorItem" || kind == "BColorListView") {
            auto* list = new BPrivate::BColorListView("colors");
            for (auto text : {label, "Panel", "Document"}) {
                auto* item = new BPrivate::BColorItem(text, list->CountItems() == 0 ? color : ui_color(B_PANEL_BACKGROUND_COLOR));
                items.emplace_back(item);
                list->AddItem(item);
                if (kind == "BColorItem") break;
            }
            return list;
        }
        if (kind == "BAboutMenuItem" || kind == "BColorMenuItem") {
            auto* menu = new EmbeddedMenu(r);
            if (kind == "BColorMenuItem") menu->AddItem(new BColorMenuItem(label, nullptr, color));
            else { auto* item = new BAboutMenuItem(); item->SetLabel(label); menu->AddItem(item); }
            return menu;
        }
        if (kind == "BToolBar") {
            auto* bar = new BPrivate::BToolBar(r);
            uint32 id = 1;
            for (auto text : {"+", "−", "↶", "↷"}) bar->AddAction(id++, nullptr, nullptr, nullptr, text);
            return bar;
        }
        if (kind == "BIconButton" || kind == "BBitmapButton") {
            // An original pencil glyph supplied to the real icon control.
            auto* bitmap = Icon();
            if (kind == "BBitmapButton")
                return new BPrivate::BBitmapButton(static_cast<uint8*>(bitmap->Bits()), 16, 16, B_RGBA32, nullptr);
            auto* button = new BPrivate::BIconButton("icon");
            button->SetIcon(bitmap);
            return button;
        }
        if (kind == "IconView") {
            auto* icon = new IconView();
            icon->SetIcon(Icon());
            return icon;
        }
        if (kind == "BStripeView") return new BPrivate::BStripeView(*Icon());
        if (kind == "BColumnListView" || kind == "BColumn" || kind.find("Column") != std::string::npos
            || kind == "BRow" || kind == "BStringField" || kind == "BDateField" || kind == "BSizeField"
            || kind == "BIntegerField" || kind == "BBitmapField") {
            const float width = r.Width();
            auto stringColumn = [&] { return new BStringColumn(label, width, 5, 4096, B_TRUNCATE_END); };
            std::unique_ptr<BColumn> col;
            std::unique_ptr<BField> field;
            if (kind == "GraphColumn" || kind == "BIntegerColumn" || kind == "BIntegerField") {
                col.reset(kind == "GraphColumn" ? new GraphColumn(label, width, 5, 4096) : new BIntegerColumn(label, width, 5, 4096));
                field.reset(new BIntegerField(kind == "GraphColumn" ? percent : static_cast<int32>(std::clamp(std::strtol(value, nullptr, 10), -2147483647L, 2147483647L))));
            } else if (kind == "BSizeColumn" || kind == "BSizeField") {
                col.reset(new BSizeColumn(label, width, 5, 4096)); field.reset(new BSizeField(std::strtoll(value, nullptr, 10)));
            } else if (kind == "BDateColumn" || kind == "BDateField") {
                std::tm date = {};
                int year = 2026, month = 11, day = 22;
                if (sscanf(value, "%4d-%2d-%2d", &year, &month, &day) != 3 || !BDate(year, month, day).IsValid()) {
                    year = 2026; month = 11; day = 22;
                }
                date.tm_year = year-1900; date.tm_mon = month-1; date.tm_mday = day; date.tm_hour = 12; date.tm_isdst = -1;
                time_t example = std::mktime(&date);
                col.reset(new BDateColumn(label, width, 5, 4096)); field.reset(new BDateField(&example));
            } else if (kind == "BBitmapColumn" || kind == "BBitmapField") {
                col.reset(new BBitmapColumn(label, width, 5, 4096)); field.reset(new BBitmapField(Icon()));
            } else {
                col.reset(stringColumn()); field.reset(new BStringField(label));
            }
            if (kind == "BRow" || kind == "GraphColumn" || kind.find("Field") != std::string::npos)
                return new ColumnCell(r, col.release(), field.release());
            auto* list = new BColumnListView(r, "Columns", B_FOLLOW_NONE, B_WILL_DRAW, B_PLAIN_BORDER, false);
            list->AddColumn(col.release(), 0);
            if (kind == "BColumnListView") {
                list->AddColumn(new BSizeColumn("Size", width / 3, 5, 4096), 1);
                list->ColumnAt(0)->SetWidth(width * .60f);
                for (auto text : {"Document", "Details"}) {
                    auto* row = new BRow;
                    row->SetField(new BStringField(text), 0);
                    row->SetField(new BSizeField(1258291), 1);
                    list->AddRow(row);
                }
            }
            return list;
        }
        if (kind == "BarberPole") return new BarberPole("activity");
        if (kind == "BSplitView") {
            auto* split = new BSplitView(B_HORIZONTAL);
            split->AddChild(new BStringView("sidebar", label));
            split->AddChild(new BGroupView(B_VERTICAL));
            return split;
        }
        return nullptr;
    }
};

// The offscreen BWindow supplies clipping and the native coordinate system for each child.
void DrawTree(BView* view)
{
    if (view->IsHidden(view)) return;
    const BRect bounds = view->Bounds();
    const rgb_color background = view->ViewColor();
    if (background != B_TRANSPARENT_COLOR) {
        view->PushState();
        view->SetHighColor(background);
        view->FillRect(bounds);
        view->PopState();
    }
    if (view->Flags() & B_WILL_DRAW) view->Draw(bounds);
    for (BView* child = view->ChildAt(0); child; child = child->NextSibling()) DrawTree(child);
    if (view->Flags() & B_DRAW_ON_CHILDREN) view->DrawAfterChildren(bounds);
}
}

extern "C" bool fc_draw_widget(const char* kind, const char* label, const char* value,
    bool enabled, bool checked, uint32 w, uint32 h, uint8* pixels)
{
    // The GUI already owns a BApplication. CLI exports also need an app_server connection.
    static std::unique_ptr<BApplication> cli;
    if (!be_app) {
        status_t status;
        cli.reset(new BApplication("application/x-vnd.airos-FigCraft-render", &status));
        if (status != B_OK) { cli.reset(); return false; }
    }
    Sample sample;
    BRect frame(0, 0, w - 1, h - 1);
    BBitmap bitmap(frame, B_BITMAP_ACCEPTS_VIEWS, B_RGB32);
    if (bitmap.InitCheck() != B_OK || !bitmap.Lock()) return false;
    BView* surface = new BView(frame, "panel", B_FOLLOW_NONE, B_WILL_DRAW);
    bitmap.AddChild(surface);
    surface->SetViewUIColor(B_PANEL_BACKGROUND_COLOR);
    BView* widget = sample.Make(kind, frame, label, value, checked, surface);
    if (!widget) { bitmap.Unlock(); return false; }
    surface->AddChild(widget);
    widget->MoveTo(B_ORIGIN);
    widget->ResizeTo(frame.Width(), frame.Height());
    if (auto* control = dynamic_cast<BControl*>(widget)) control->SetEnabled(enabled);
    if (auto* field = dynamic_cast<BMenuField*>(widget)) field->SetEnabled(enabled);
    surface->Layout(true);
    DrawTree(surface);
    surface->Sync();
    for (uint32 y = 0; y < h; ++y) {
        const uint8* src = static_cast<const uint8*>(bitmap.Bits()) + y * bitmap.BytesPerRow();
        for (uint32 x = 0; x < w; ++x, src += 4, pixels += 4) {
            pixels[0] = src[2]; pixels[1] = src[1]; pixels[2] = src[0]; pixels[3] = 255;
        }
    }
    bitmap.Unlock();
    return true;
}

#include <Messenger.h>
#include <Screen.h>

namespace {
class DesignPage : public BView {
public:
    DesignPage(BRect r, const uint8* rgba)
        : BView(r, "Design page", B_FOLLOW_NONE, B_WILL_DRAW), fBackground(r, B_RGB32)
    {
        SetViewUIColor(B_PANEL_BACKGROUND_COLOR);
        if (fBackground.InitCheck() != B_OK) return;
        for (int32 y = 0; y <= r.IntegerHeight(); ++y) {
            auto* dst = static_cast<uint8*>(fBackground.Bits()) + y * fBackground.BytesPerRow();
            for (int32 x = 0; x <= r.IntegerWidth(); ++x, rgba += 4, dst += 4) {
                dst[0] = rgba[2]; dst[1] = rgba[1]; dst[2] = rgba[0]; dst[3] = 255;
            }
        }
    }
    ~DesignPage() {
        // Views may reference sample-owned list items until their destructors return.
        while (BView* view = ChildAt(0)) { RemoveChild(view); delete view; }
    }
    bool Ready() const { return fBackground.InitCheck() == B_OK; }
    void Draw(BRect rect) override { DrawBitmap(&fBackground, rect, rect); }

    bool Place(const PreviewWidget& item) {
        BRect r(0, 0, item.width - 1, item.height - 1);
        // Give every control a native panel parent at the design rectangle. It clips
        // children and supplies the system background just like an ordinary settings view.
        auto* panel = new BView(r.OffsetToCopy(item.x, item.y), "Control", B_FOLLOW_NONE, B_WILL_DRAW);
        panel->SetViewUIColor(B_PANEL_BACKGROUND_COLOR);
        AddChild(panel);
        auto* widget = fSample.Make(item.kind, r, item.label, item.value, item.checked, panel);
        if (!widget) { RemoveChild(panel); delete panel; return false; }
        panel->AddChild(widget);
        widget->MoveTo(B_ORIGIN);
        widget->ResizeTo(r.Width(), r.Height());
        if (auto* control = dynamic_cast<BControl*>(widget)) control->SetEnabled(item.enabled);
        if (auto* field = dynamic_cast<BMenuField*>(widget)) field->SetEnabled(item.enabled);
        widget->Layout(true);
        return true;
    }
private:
    BBitmap fBackground;
    Sample fSample;
};
}

extern "C" int32 fc_present_design(const char* title, uint32 w, uint32 h, const uint8* rgba,
    const PreviewWidget* widgets, size_t count)
{
    static BMessenger previous;
    BWindow* window = nullptr;
    if (previous.LockTarget()) {
        BLooper* looper;
        previous.Target(&looper);
        window = static_cast<BWindow*>(looper);
    }
    const bool fresh = window == nullptr;
    if (fresh) {
        window = new BWindow(BRect(64, 96, 383, 335), title, B_TITLED_WINDOW,
            B_ASYNCHRONOUS_CONTROLS | B_NOT_ZOOMABLE);
    }
    auto* page = new DesignPage(BRect(0, 0, w-1, h-1), rgba);
    if (!page->Ready()) {
        delete page;
        if (fresh) window->Quit(); else window->Unlock();
        return -1;
    }
    while (BView* old = window->ChildAt(0)) { window->RemoveChild(old); delete old; }
    BRect screen = BScreen(window).Frame();
    const float width = std::min<float>(w, screen.Width()*.8f);
    const float height = std::min<float>(h, screen.Height()*.75f);
    auto* scroll = new BScrollView("Page", page, B_FOLLOW_ALL, 0, w > width, h > height, B_NO_BORDER);
    window->SetTitle(title);
    window->AddChild(scroll);
    window->ResizeTo(width-1 + (h > height ? B_V_SCROLL_BAR_WIDTH : 0),
        height-1 + (w > width ? B_H_SCROLL_BAR_HEIGHT : 0));
    scroll->MoveTo(B_ORIGIN);
    scroll->ResizeTo(window->Bounds().Width(), window->Bounds().Height());
    if (auto* bar = scroll->ScrollBar(B_HORIZONTAL)) { bar->SetRange(0, w-width); bar->SetProportion(width/w); }
    if (auto* bar = scroll->ScrollBar(B_VERTICAL)) { bar->SetRange(0, h-height); bar->SetProportion(height/h); }
    int32 placed = 0;
    for (size_t i=0; i<count; ++i) if (page->Place(widgets[i])) ++placed;
    if (fresh) { previous = BMessenger(window); window->CenterOnScreen(); window->Show(); }
    else { window->Activate(); window->Unlock(); }
    return placed;
}
