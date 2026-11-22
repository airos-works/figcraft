// Operator review uses its own Be window, outside the editor's input channel.
#include <Application.h>
#include <LayoutBuilder.h>
#include <StringView.h>
#include <Window.h>
#include <private/interface/Accessible.h>
#include <private/interface/AccessibilityProtocol.h>
#include <Bitmap.h>
#include <Button.h>
#include <GroupLayout.h>
#include <Screen.h>
#include <View.h>
#include <algorithm>
#include <atomic>
#include <memory>
#include <OS.h>

class PreviewInfo : public BAccessible {
public:
    PreviewInfo(BView* view, const char* title) : fView(view), fTitle(title) {}
    accessible_role Role() const override { return B_ACCESSIBLE_ROLE_IMAGE; }
    BString Name() const override { return fTitle; }
    BRect Frame() const override { return fView->ConvertToScreen(fView->Bounds()); }
    void AddProperties(BMessage& node) const override
    {
        node.AddString(B_ACCESSIBLE_FIELD_DRAWN_TEXT, fTitle);
    }
private:
    BView* fView;
    BString fTitle;
};

class Preview : public BView {
public:
    Preview(BBitmap* image, BSize size, const char* title)
        : BView("Saved design", B_WILL_DRAW), fImage(image), fInfo(this, title)
    {
        BAccessible::Attach(this, &fInfo);
        SetExplicitMinSize(size);
        SetExplicitPreferredSize(size);
        SetViewUIColor(B_PANEL_BACKGROUND_COLOR);
    }
    ~Preview() { BAccessible::Detach(this); delete fImage; }

    void Draw(BRect)
    {
        BRect area = Bounds();
        BRect src = fImage->Bounds();
        float scale = std::min((area.Width()+1)/(src.Width()+1), (area.Height()+1)/(src.Height()+1));
        BRect dst(0, 0, (src.Width()+1)*scale-1, (src.Height()+1)*scale-1);
        dst.OffsetTo((area.Width()-dst.Width())/2, (area.Height()-dst.Height())/2);
        DrawBitmap(fImage, src, dst, (src.Width() > 0 && src.Height() > 0) ? B_FILTER_BITMAP_BILINEAR : 0);
    }
private:
    BBitmap* fImage;
    PreviewInfo fInfo;
};

struct ReviewReply {
    sem_id ready = create_sem(0, "FigCraft review decision");
    std::atomic<int32> decision{-1};
    ~ReviewReply() { if (ready >= B_OK) delete_sem(ready); }
};

class ReviewWindow : public BWindow {
public:
    ReviewWindow(const char* title, BBitmap* bitmap, bool back, bool last, std::shared_ptr<ReviewReply> reply)
        : BWindow(BRect(40, 40, 300, 200), "Operator design review", B_TITLED_WINDOW_LOOK,
            B_MODAL_APP_WINDOW_FEEL, B_NOT_ZOOMABLE | B_NOT_RESIZABLE | B_AUTO_UPDATE_SIZE_LIMITS),
          fReply(std::move(reply))
    {
        const BRect screen = BScreen(this).Frame();
        BSize size(std::min(960.f, screen.Width() * .76f), std::min(660.f, screen.Height() * .55f));
        BButton* reject = new BButton("Request changes", new BMessage('FCRj'));
        BGroupView* buttons = new BGroupView(B_HORIZONTAL);
        BLayoutBuilder::Group<>(buttons).AddGlue().Add(reject);
        if (back)
            buttons->AddChild(new BButton("Previous", new BMessage('FCBk')));
        buttons->AddChild(new BButton(last ? "Approve design" : "Next page", new BMessage('FCGo')));
        BLayoutBuilder::Group<>(this, B_VERTICAL)
            .SetInsets(B_USE_WINDOW_INSETS)
            .Add(new BStringView("Page", title))
            .Add(new Preview(bitmap, size, title))
            .Add(buttons);
        SetDefaultButton(reject);
        AddShortcut(B_ESCAPE, 0, new BMessage('FCRj'));
        ResizeToPreferred();
        CenterOnScreen();
    }

    ~ReviewWindow() { release_sem(fReply->ready); }

    void DispatchMessage(BMessage* message, BHandler* target) override
    {
        if (!message->GetBool("be:agent", false))
            BWindow::DispatchMessage(message, target);
    }

    void MessageReceived(BMessage* message) override
    {
        switch (message->what) {
            case 'FCRj': fReply->decision.store(0); break;
            case 'FCBk': fReply->decision.store(1); break;
            case 'FCGo': fReply->decision.store(2); break;
            default: BWindow::MessageReceived(message); return;
        }
        PostMessage(B_QUIT_REQUESTED);
    }
private:
    std::shared_ptr<ReviewReply> fReply;
};

extern "C" int32_t
fc_review_page(const char* title, const uint8* rgba, int32 width, int32 height,
    int32 back, int32 last)
{
    BBitmap* bitmap = new BBitmap(BRect(0,0,width-1,height-1), B_RGB32);
    if (bitmap->InitCheck() != B_OK) { delete bitmap; return -1; }
    for (int32 y=0; y<height; y++) {
        uint8* row = (uint8*)bitmap->Bits() + y*bitmap->BytesPerRow();
        for (int32 x=0; x<width; x++, rgba+=4, row+=4) {
            // Premultiplied RGBA on white for transparent pages.
            row[0] = rgba[2]+(255-rgba[3]); row[1] = rgba[1]+(255-rgba[3]);
            row[2] = rgba[0]+(255-rgba[3]); row[3] = 255;
        }
    }
    auto reply = std::make_shared<ReviewReply>();
    if (reply->ready < B_OK) { delete bitmap; return -1; }
    ReviewWindow* window = new ReviewWindow(title, bitmap, back, last, reply);
    window->Show();
    status_t status;
    do { status = acquire_sem(reply->ready); } while (status == B_INTERRUPTED);
    return status == B_OK ? reply->decision.load() : -1;
}
