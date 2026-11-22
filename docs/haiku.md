# Designing an air/OS app with FigCraft

Open **Developer → FigCraft** in the Deskbar. In the template chooser, select **air/OS application** to start a design with a sample settings window and reusable components. The first page holds the design. The other pages contain the component library. Place instances from Assets, set their exposed properties and edit the main component to update its instances.

The library covers Interface Kit controls, menus, lists and columns, navigation, containers and layouts, windows, file and print panels, shared controls, and views from other kits. Each main component stores its Be class and SDK header in the node's `airOS.class` and `airOS.header` plugin data. On air/OS, the canvas, PNG renderer and saved-design review draw supported controls with the installed Interface Kit. The portable drawing is stored in the component for use on other platforms. Window decorations, stock dialog layouts and abstract kit classes are editable design representations. The catalog returned by `airOS.listElements` identifies each entry as `nativeOnAirOS` or `designLayers`. Headers under `private/` identify shared controls with a private API. The native editor uses Be windows, file panels, clipboard, Tracker, installed fonts, locale and accessibility. It can present the same editor with Vulkan or with its software renderer.

## Connect an MCP client

Start the editor for a project or attach a project to the editor already open from the Deskbar. Choose an existing project folder that the agent can read and write.

```sh
project=/boot/home/Projects/Example
mkdir -p "$project"
/boot/system/apps/FigCraft --control 4979 \
    --automation-read-root "$project" --automation-write-root "$project" &
```

Configure the MCP client to run `figcraft-cli` with arguments `mcp --attach 4979`. The editor listens on loopback. A running control session keeps its assigned roots. Close that editor to switch to another project. For design generation without a window, run `figcraft-cli mcp --automation-read-root "$project" --automation-write-root "$project"`.

The air/OS system MCP can inspect the editor through `ui_windows`, `ui_tree` and `ui_act`. Its `run_command` tool can also start `figcraft-cli mcp --attach 4979` with newline-delimited MCP messages in `stdin`. An MCP client with a persistent stdio connection can use the FigCraft tools directly.

## Iterate on an app idea

Tell the connected AI which task the app should help someone do. For example, “Design a window for renaming a group of files. Let me choose a naming pattern and preview the results.” The AI can build the design on the **Design** page of the air/OS template using the component library.

Use `component.list` to find the main component, `instance.create` to place it, and `instance.setProperties` to edit the instance. The component exposes the properties it supports. For example, a check box accepts `{"Label":"Include subfolders","Checked":true,"Enabled":true}`. A text field accepts `Label` and `Value`. Slider and progress values are text numbers from 0 to 100. Use a fresh instance of the same main for each control in the design.

Call **preview_air_os_design** after each round of edits. It opens or refreshes the active page at actual size on the operator’s desktop, including unsaved edits. The ordinary controls are native views, so the system paints their text, borders and selection indicators at the display’s current scale. You can try a control in this window. Apply changes to the design through the editor or MCP, then refresh the preview. The tool reports how many controls are native views and how many are composited into the page to preserve effects, transforms, clipping or overlapping artwork. Older air/OS templates also work in this preview.

Use **request_design_review** once the design is ready to sign off. The review and its decision apply to the saved file digest.

## Design and operator review

1. Call `run_command` with `{"command":"airOS.listElements","params":{}}` to discover the available elements and SDK headers.
2. Start the design with `run_command` and `{"command":"file.newFromTemplate","params":{"template":"air-os","name":"Example"}}`. Use `component.list` to find main component IDs, then `instance.create` with the desired component, parent and position. Design all screens and use FigCraft's document tools to inspect or render them.
3. Call `save_file` with an absolute project path, for example `/boot/home/Projects/Example/Example.figcraft`. With that file frontmost in the attached editor, call `request_design_review` with `{}`. It returns `pending` and the SHA-256 of the saved bytes.
4. The operator reviews each rendered page in a native window and chooses **Approve design** on the last page or **Request changes**. Poll `design_review_status` with the returned `sha256`. Continue implementation only when it returns `approved`. After changes, save and request review again.
5. Run the following from the build or agent workflow immediately before implementation. It exits successfully and prints the digest only for the approved file while its review session is active. A pending, rejected, stale or unknown review, an unreachable editor, and changed file contents all stop the command.

```sh
figcraft-cli review-check --attach 4979 "$project/Example.figcraft" || exit 1
```

The approval belongs to that saved revision in the running editor. Reopening FigCraft starts a new review session. The gate is available to build scripts and agent workflows that invoke it.

## Building the port

Use the air/OS cross SDK, the Rust toolchain and the craft-fonts revision pinned with FigCraft in `airos-ci/apps/developer.lock.json`. The image recipe builds `figcraft-haiku` and `figcraft-cli`, attaches the icon and application signature, then packages both executables. The x86_64 image uses the x86_64 HPKG. The generic ARM64, Raspberry Pi and Cubie images use the ARM64 HPKG.

The native pixel comparison opens two short-lived windows on a clear workspace. Build and run it from the repository root on air/OS to compare the preview with independently constructed Interface Kit controls at the active display scale.

```sh
headers="$(finddir B_SYSTEM_DEVELOP_DIRECTORY)/headers/private"
g++ -std=c++17 -I"$headers/interface" -I"$headers/shared" crates/airos-controls/controls.cpp \
    crates/airos-controls/tests/preview_pixels.cpp -lcolumnlistview -lshared -llocalestub \
    -lbe -ltranslation -ltracker -o /tmp/figcraft-preview-pixels
/tmp/figcraft-preview-pixels
```
