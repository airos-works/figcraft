#!/bin/sh
app=$(findpaths -p "$0" B_FIND_PATH_APPS_DIRECTORY FigCraft)
[ -f "$app" ] && mimeset -a -f "$app"
setmime -set application/x-figcraft -short "FigCraft design" -extension figcraft
setmime -dump application/x-figcraft | grep -q -- -preferredAppSig ||
    setmime -set application/x-figcraft -preferredAppSig application/x-vnd.airos-FigCraft
