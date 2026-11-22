//! Native appearances understood by the air/OS design template and renderer.

pub fn has_native_widget(class: &str) -> bool {
    matches!(
        class,
        "BTextToolTip"
            | "BDragger"
            | "BButton"
            | "BPictureButton"
            | "BCheckBox"
            | "BRadioButton"
            | "BSlider"
            | "BChannelSlider"
            | "BStatusBar"
            | "BColorControl"
            | "BTextControl"
            | "BTextView"
            | "BStringView"
            | "BOptionPopUp"
            | "BMenuBar"
            | "BMenu"
            | "BPopUpMenu"
            | "BMenuItem"
            | "BSeparatorItem"
            | "BMenuField"
            | "BListView"
            | "BOutlineListView"
            | "BStringItem"
            | "BScrollView"
            | "BScrollBar"
            | "BTabView"
            | "BTab"
            | "BSeparatorView"
            | "BSpinner"
            | "BDecimalSpinner"
            | "BCalendarView"
            | "TimeEdit"
            | "DateEdit"
            | "BColorPreview"
            | "BColorItem"
            | "BColorListView"
            | "BColorMenuItem"
            | "BAboutMenuItem"
            | "BToolBar"
            | "BIconButton"
            | "BBitmapButton"
            | "BarberPole"
            | "BSplitView"
            | "BStripeView"
            | "IconView"
            | "BColumnListView"
            | "BColumn"
            | "BRow"
            | "BTitledColumn"
            | "GraphColumn"
            | "BStringColumn"
            | "BDateColumn"
            | "BSizeColumn"
            | "BIntegerColumn"
            | "BBitmapColumn"
            | "BStringField"
            | "BDateField"
            | "BSizeField"
            | "BIntegerField"
            | "BBitmapField"
            | "BBox"
    )
}

/// An instance's child carries the resolved text. The binding itself stays on its
/// source layer in the main component, including after a save and reopen.
pub fn widget_label<'a>(doc: &'a crate::Document, frame: &crate::Node) -> Option<&'a str> {
    frame.children.iter().filter_map(|id| doc.node(*id)).find_map(|child| {
        let source = child.source.and_then(|id| doc.node(id)).unwrap_or(child);
        source.prop_refs.get("characters")?;
        Some(child.kind.text()?.characters.as_str())
    })
}
