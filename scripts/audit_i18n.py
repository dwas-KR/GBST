# SPDX-License-Identifier: GPL-3.0-only
from pathlib import Path
import json, re, sys
ROOT=Path(__file__).resolve().parents[1]
LOCALES=ROOT/"crates/gbst-core/lang"
GUI=ROOT/"crates/gbst-gui/src"
LANGUAGES=("ko","en","ru","ja","zh-TW","vi","el","hi","ka","nl","ar","es")
data={key: json.loads((LOCALES/(key+".json")).read_text(encoding="utf-8")) for key in LANGUAGES}
base=data["en"]
print("GBST localization source audit")
print("Languages:", ", ".join(LANGUAGES))
print("Keys per catalog:", {lang:len(items) for lang,items in data.items()})

def placeholders(s):
    return set(re.findall(r"\{([a-zA-Z0-9_]+)\}",s))

errors=[]
for lang,items in data.items():
    if set(items)!=set(base):
        errors.append(f"{lang}: missing={sorted(set(base)-set(items))}, extra={sorted(set(items)-set(base))}")
    for k in base.keys() & items.keys():
        if placeholders(items[k]) != placeholders(base[k]):
            errors.append(f"{lang}: placeholder mismatch {k}")
        if lang!="ko" and re.search(r"[\uac00-\ud7a3]",items[k]):
            errors.append(f"{lang}: unexpected Hangul in catalog {k}")
used=set()
for p in GUI.glob("*.rs"):
    src=p.read_text(encoding="utf-8")
    for key in re.findall(r'\buix\s*\(\s*(?:self\.)?language\s*,\s*"([^"]+)"',src,re.S):
        used.add(key)
    for key in re.findall(r'\blog_label(?:_with_args)?\s*\(\s*(?:self\.)?language\s*,\s*"([^"]+)"',src,re.S):
        used.add(key)
for key in sorted(used):
    if key not in base:
        errors.append("GUI references missing key "+key)
visual=(GUI/"ltbox_visual.rs").read_text(encoding="utf-8")
for snippet in (
    'localized_device_details(self.language,d)',
    'ui_text(self.language, &self.dashboard_info.model_name)',
    'ui_text(self.language,&d.model_name)',
    'ui_text(self.language,&d.manufacturer)',
    'ui_text(self.language,&d.android_version)',
):
    if snippet not in visual:
        errors.append("unlocalized dashboard path or changed code: "+snippet)
for old in ('(ui_text(self.language,"모델"),d.model_name.clone())',
            '(ui_text(self.language,"Android 버전"),d.android_version.clone())'):
    if old in visual:
        errors.append("old unlocalized dashboard path: "+old)
print("GUI semantic keys found:",len(used))
print("Catalog locales validated:",len(data))
print("Issues:",len(errors))
for item in errors:
    print("ERROR:",item)
sys.exit(1 if errors else 0)
