// BAccessible wrappers keep their native IDs between frames while the
// corresponding egui widget exists. Update and queries hold the window lock.
#include <private/interface/Accessible.h>
#include <memory>
#include <set>

struct AccessibleRow {
    uint64_t id, parent, states;
    uint32_t actions;
    float rect[4];
    const char *role, *name, *value, *description;
};

static accessible_role
NativeRole(const char* role)
{
    static const std::map<std::string, accessible_role> roles = {
        {"Application", B_ACCESSIBLE_ROLE_APPLICATION},
        {"Window", B_ACCESSIBLE_ROLE_WINDOW},
        {"Dialog", B_ACCESSIBLE_ROLE_DIALOG},
        {"AlertDialog", B_ACCESSIBLE_ROLE_ALERT},
        {"Alert", B_ACCESSIBLE_ROLE_ALERT},
        {"Pane", B_ACCESSIBLE_ROLE_GROUP},
        {"Group", B_ACCESSIBLE_ROLE_GROUP},
        {"GenericContainer", B_ACCESSIBLE_ROLE_GROUP},
        {"ScrollView", B_ACCESSIBLE_ROLE_SCROLL_PANE},
        {"Splitter", B_ACCESSIBLE_ROLE_SPLIT_PANE},
        {"Label", B_ACCESSIBLE_ROLE_LABEL},
        {"TextRun", B_ACCESSIBLE_ROLE_LABEL},
        {"Button", B_ACCESSIBLE_ROLE_BUTTON},
        {"DefaultButton", B_ACCESSIBLE_ROLE_BUTTON},
        {"Switch", B_ACCESSIBLE_ROLE_TOGGLE_BUTTON},
        {"CheckBox", B_ACCESSIBLE_ROLE_CHECK_BOX},
        {"RadioButton", B_ACCESSIBLE_ROLE_RADIO_BUTTON},
        {"TextInput", B_ACCESSIBLE_ROLE_TEXT_FIELD},
        {"PasswordInput", B_ACCESSIBLE_ROLE_TEXT_FIELD},
        {"SearchInput", B_ACCESSIBLE_ROLE_TEXT_FIELD},
        {"MultilineTextInput", B_ACCESSIBLE_ROLE_TEXT_AREA},
        {"NumberInput", B_ACCESSIBLE_ROLE_SPIN_BUTTON},
        {"SpinButton", B_ACCESSIBLE_ROLE_SPIN_BUTTON},
        {"Slider", B_ACCESSIBLE_ROLE_SLIDER},
        {"ScrollBar", B_ACCESSIBLE_ROLE_SCROLL_BAR},
        {"Meter", B_ACCESSIBLE_ROLE_PROGRESS_BAR},
        {"ProgressIndicator", B_ACCESSIBLE_ROLE_PROGRESS_BAR},
        {"ComboBox", B_ACCESSIBLE_ROLE_COMBO_BOX},
        {"EditableComboBox", B_ACCESSIBLE_ROLE_COMBO_BOX},
        {"ColorWell", B_ACCESSIBLE_ROLE_COLOR_PICKER},
        {"List", B_ACCESSIBLE_ROLE_LIST},
        {"ListBox", B_ACCESSIBLE_ROLE_LIST},
        {"ListBoxOption", B_ACCESSIBLE_ROLE_LIST_ITEM},
        {"ListItem", B_ACCESSIBLE_ROLE_LIST_ITEM},
        {"Tree", B_ACCESSIBLE_ROLE_TREE},
        {"TreeItem", B_ACCESSIBLE_ROLE_TREE_ITEM},
        {"Grid", B_ACCESSIBLE_ROLE_TABLE},
        {"Table", B_ACCESSIBLE_ROLE_TABLE},
        {"Row", B_ACCESSIBLE_ROLE_ROW},
        {"Cell", B_ACCESSIBLE_ROLE_CELL},
        {"GridCell", B_ACCESSIBLE_ROLE_CELL},
        {"ColumnHeader", B_ACCESSIBLE_ROLE_COLUMN_HEADER},
        {"MenuBar", B_ACCESSIBLE_ROLE_MENU_BAR},
        {"Menu", B_ACCESSIBLE_ROLE_MENU},
        {"MenuItem", B_ACCESSIBLE_ROLE_MENU_ITEM},
        {"MenuItemCheckBox", B_ACCESSIBLE_ROLE_MENU_ITEM},
        {"MenuItemRadio", B_ACCESSIBLE_ROLE_MENU_ITEM},
        {"MenuListPopup", B_ACCESSIBLE_ROLE_MENU},
        {"TabList", B_ACCESSIBLE_ROLE_TAB_LIST},
        {"Tab", B_ACCESSIBLE_ROLE_TAB},
        {"TabPanel", B_ACCESSIBLE_ROLE_TAB_PANEL},
        {"Toolbar", B_ACCESSIBLE_ROLE_TOOL_BAR},
        {"Status", B_ACCESSIBLE_ROLE_STATUS_BAR},
        {"Tooltip", B_ACCESSIBLE_ROLE_TOOL_TIP},
        {"Image", B_ACCESSIBLE_ROLE_IMAGE},
        {"Canvas", B_ACCESSIBLE_ROLE_CANVAS},
        {"Link", B_ACCESSIBLE_ROLE_LINK},
        {"Heading", B_ACCESSIBLE_ROLE_HEADING},
        {"Paragraph", B_ACCESSIBLE_ROLE_PARAGRAPH},
        {"Document", B_ACCESSIBLE_ROLE_DOCUMENT},
        {"GraphicsDocument", B_ACCESSIBLE_ROLE_DOCUMENT},
        {"Article", B_ACCESSIBLE_ROLE_ARTICLE},
        {"Form", B_ACCESSIBLE_ROLE_FORM},
    };
    auto it = roles.find(role);
    return it == roles.end() ? B_ACCESSIBLE_ROLE_GROUP : it->second;
}

class WidgetNode : public BAccessible {
public:
    WidgetNode(BView* host, int32 window, uint64 key)
        : view(host), windowID(window), key(key) {}
    accessible_role Role() const override { return role; }
    BString Name() const override { return name; }
    BString Description() const override { return description; }
    BString Value() const override { return value; }
    uint64 States() const override { return states; }
    uint32 Actions() const override { return actions; }
    BRect Frame() const override { return view->ConvertToScreen(rect.IsValid() ? rect : view->Bounds()); }
    BAccessible* Parent() const override { return parent; }
    int32 CountChildren() const override { return children.size(); }
    BAccessible* ChildAt(int32 i) const override {
        return i >= 0 && (size_t)i < children.size() ? children[i] : NULL;
    }
    status_t PerformAction(accessible_action action) override {
        if ((actions & action) == 0 || (states & B_ACCESSIBLE_STATE_DISABLED)) return B_NOT_ALLOWED;
        AccessibleAction(windowID, key, action);
        return B_OK;
    }
    BView* view;
    int32 windowID;
    uint64 key;
    accessible_role role = B_ACCESSIBLE_ROLE_UNKNOWN;
    BString name, value, description;
    uint64 states = 0;
    uint32 actions = 0;
    BRect rect;
    WidgetNode* parent = NULL;
    std::vector<WidgetNode*> children;
};

class AccessibleTree {
public:
    AccessibleTree(BView* view, int32 window) : fView(view), fWindow(window) {}
    ~AccessibleTree() { if (fRoot) BAccessible::Detach(fView); }

    void Update(const AccessibleRow* rows, size_t count) {
        if (count == 0) return;
        std::set<uint64> live;
        bool structureChanged = count != fNodes.size();
        WidgetNode* focus = NULL;
        for (size_t i = 0; i < count; i++) {
            const AccessibleRow& row = rows[i];
            live.insert(row.id);
            auto& slot = fNodes[row.id];
            if (!slot) {
                slot.reset(new WidgetNode(fView, fWindow, row.id));
                structureChanged = true;
            }
            WidgetNode& node = *slot;
            node.role = NativeRole(row.role);
            bool renamed = node.name != row.name;
            bool changed = node.value != row.value;
            uint64 statesChanged = node.states ^ row.states;
            node.name = row.name; node.value = row.value; node.description = row.description;
            node.actions = row.actions; node.states = row.states;
            node.rect = BRect(row.rect[0],row.rect[1],row.rect[2]-1,row.rect[3]-1);
            node.children.clear();
            if (i == 0) {
                node.parent = NULL;
                if (fRoot != &node) {
                    if (fRoot) BAccessible::Detach(fView);
                    BAccessible::Attach(fView, &node);
                    fRoot = &node;
                }
            } else {
                auto parent = fNodes.find(row.parent);
                if (parent == fNodes.end()) continue;
                structureChanged |= node.parent != parent->second.get();
                node.parent = parent->second.get();
                node.parent->children.push_back(&node);
            }
            if (node.states & B_ACCESSIBLE_STATE_FOCUSED) focus = &node;
            if (BAccessible::IsEnabled()) {
                if (renamed) node.NotifyEvent(B_ACCESSIBLE_EVENT_NAME_CHANGED);
                if (changed) node.NotifyEvent(B_ACCESSIBLE_EVENT_VALUE_CHANGED);
                if (statesChanged) {
                    BMessage details;
                    details.AddInt64("changed", statesChanged);
                    node.NotifyEvent(B_ACCESSIBLE_EVENT_STATE_CHANGED, &details);
                }
            }
        }
        for (auto it=fNodes.begin(); it!=fNodes.end();) {
            if (live.count(it->first) == 0) it=fNodes.erase(it); else ++it;
        }
        if (structureChanged) fRoot->NotifyEvent(B_ACCESSIBLE_EVENT_CHILDREN_CHANGED);
        if (focus && focus->key != fFocusKey) focus->NotifyEvent(B_ACCESSIBLE_EVENT_FOCUS_CHANGED);
        fFocusKey = focus ? focus->key : 0;
    }
private:
    BView* fView;
    int32 fWindow;
    WidgetNode* fRoot = NULL;
    uint64 fFocusKey = 0;
    std::map<uint64,std::unique_ptr<WidgetNode>> fNodes;
};
