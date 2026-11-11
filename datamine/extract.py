# /// script
# requires-python = ">=3.11"
# dependencies = ["UnityPy==1.25.4", "TypeTreeGeneratorAPI==0.0.10", "dnfile==0.18.0"]
# ///
import csv
import io
import json
import os
import struct
import sys

import dnfile
import UnityPy
from UnityPy.classes import PPtr
from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator

LANGS = {"English": "en", "French": "fr", "Spanish": "es", "Portuguese": "pt", "Portuguese (Brazil)": "pt-BR", "Italian": "it", "German": "de"}
ID_FIELD = {
    "Spam.Data.CardData": "_id",
    "Spam.Data.ArtistData": "_id",
    "TcgEngine.AbilityData": "id",
    "TcgEngine.TalentData": "id",
    "TcgEngine.ModificatorStatusData": "_UUID",
    "Spam.Data.ElementData": "_id",
    "Spam.Data.RarityData": "_id",
    "Spam.Data.RankData": "_id",
    "Spam.Data.GameTypeData": "_id",
    "TcgEngine.CardTypeData": "_id",
}
CARD, TOX = "Spam.Data.CardData", "TcgEngine.ModificatorStatusData"
IGNORE = {
    "Spam.Data.AvatarData", "Spam.Data.BadgeData", "Spam.Data.BannerData", "Spam.Data.CardStyleData",
    "Spam.Data.CurrencyData", "Spam.Data.DeckSlotData", "Spam.Data.EffectFilterData", "Spam.Data.ElementalOrnamentData",
    "Spam.Data.EmoteData", "Spam.Data.FinisherData", "Spam.Data.InfoData", "Spam.Data.MoneyArticleData",
    "Spam.Data.MonthlySubscriptionData", "Spam.Data.OrbData", "Spam.Data.RewardData", "Spam.Data.SideArenaData",
    "Spam.Data.TrailData", "Spam.Data.TrailShapeSettings", "Spam.Data.TrailTipDeformSettings",
    "TcgEngine.ApiClient", "TcgEngine.AssetData", "TcgEngine.AudioTool", "TcgEngine.BootLoader", "TcgEngine.BuyRandomItem",
    "TcgEngine.ChangeCardElement", "TcgEngine.DataLoader", "TcgEngine.GamepadUiAssets", "TcgEngine.GameplayData",
    "TcgEngine.HasAmountSpamton", "TcgEngine.HasFullHP", "TcgEngine.HasPassiveRunning", "TcgEngine.IsHPMatch",
    "TcgEngine.IsNotCardElement", "TcgEngine.IsNotLastTarget", "TcgEngine.LeftDragOnlyInputModule", "TcgEngine.NetworkData",
    "TcgEngine.RefreshAllShop", "TcgEngine.StealLuck", "TcgEngine.StealSpamton", "TcgEngine.TcgNetwork", "TcgEngine.TcgTransport",
    "TcgEngine._EffectNULL",
}
IGNORE_PREFIXES = ("TcgEngine.Effect", "TcgEngine.Condition", "TcgEngine.Filter")
OUT = {
    "Spam.Data.CardData": "cards", "Spam.Data.ArtistData": "artists", "TcgEngine.AbilityData": "abilities",
    "TcgEngine.TalentData": "talents", "TcgEngine.ModificatorStatusData": "tox",
}
DROP = {
    "m_GameObject", "m_Enabled", "m_Script", "m_Name", "_id", "_hasCustomId", "_name",
    "_illustration", "_illustrationResourcePath", "_cardColor", "_avatarImageOffset", "_avatarImageSize", "_styles",
    "status_fx",
}
TEXT_KEYS = {
    "cards": lambda r: ("Card.Name." + r["sanitizedName"], "Card.Lore." + r["sanitizedName"] + ".Description"),
    "abilities": lambda r: ("Card.Ability." + r["id"], "Card.AbilityDesc." + r["id"]),
    "talents": lambda r: ("Talent.Name." + r["id"], "Talent.Description." + r["id"]),
    "tox": lambda r: ("Card.Passif." + r["title"].split("_")[0], "Card.PassifDesc." + r["title"].split("_")[0]),
}
GLOSSARY = {"keywords": ("Keyword.Title.", "Keyword.Desc."), "effects": ("Card.Effect.", "Card.EffectDesc.")}
NAMES = {
    "ranks": ("Spam.Data.RankData", lambda i: "Rank." + i.removeprefix("rank-"), "_threshold"),
    "elements": ("Spam.Data.ElementData", lambda i: "Element.Name." + i, "_order"),
    "rarities": ("Spam.Data.RarityData", lambda i: "Rarity.Name." + i.removeprefix("rarity-"), "_order"),
    "gameTypes": ("Spam.Data.GameTypeData", lambda i: "GameType.Name." + i, None),
}
GAME_MODES = {"Friendly": "Friends"}
ENUMS = {
    "trigger": ("TcgEngine.AbilityData", "trigger"),
    "target": ("TcgEngine.AbilityData", "target"),
    "displayType": ("TcgEngine.AbilityData", "displayType"),
    "effect": (TOX, "effect"),
}
CONSTANT = {0x04: "<b", 0x05: "<B", 0x06: "<h", 0x07: "<H", 0x08: "<i", 0x09: "<I", 0x0A: "<q", 0x0B: "<Q"}


def camel(name):
    if name.startswith("<") and name.endswith(">k__BackingField"):
        name = name[1:-16]
    head, *rest = name.strip("_").split("_")
    head = head.lower() if head.isupper() else head[0].lower() + head[1:]
    return head + "".join(p[:1].upper() + p[1:] for p in rest)


def f32(x):
    for p in range(1, 10):
        y = float(f"{x:.{p}g}")
        if struct.unpack("f", struct.pack("f", y))[0] == x:
            return y
    return x


def enums(path):
    tables = dnfile.dnPE(path).net.mdtables
    types = {f"{t.TypeNamespace}.{t.TypeName}": t for t in tables.TypeDef.rows}
    constants = {
        c.Parent.row_index: struct.unpack(CONSTANT[c.Type], c.Value.value)[0]
        for c in tables.Constant.rows
        if c.Parent.table and c.Parent.table.name == "Field" and c.Type in CONSTANT
    }

    def values(row):
        base = row.Extends
        if not base.table or base.table.name != "TypeRef" or f"{base.row.TypeNamespace}.{base.row.TypeName}" != "System.Enum":
            sys.exit(f"{row.TypeNamespace}.{row.TypeName} is not an enum")
        return {constants[f.row_index]: str(f.row.Name) for f in row.FieldList if f.row_index in constants}

    def field_type(cls, name):
        row = types[cls]
        while True:
            for f in row.FieldList:
                if str(f.row.Name) == name:
                    sig = f.row.Signature.value
                    if sig[:2] != b"\x06\x11":
                        sys.exit(f"{cls}.{name} is not a value type")
                    b = sig[2:]
                    if b[0] < 0x80:
                        coded = b[0]
                    elif b[0] < 0xC0:
                        coded = (b[0] & 0x3F) << 8 | b[1]
                    else:
                        coded = int.from_bytes(b[:4]) & 0x1FFFFFFF
                    if coded & 3:
                        sys.exit(f"{cls}.{name} has a type of another assembly")
                    return tables.TypeDef.rows[(coded >> 2) - 1]
            base = row.Extends
            if not base.table or base.table.name != "TypeDef":
                sys.exit(f"{cls}.{name} not found")
            row = base.row

    modes = {name: value for value, name in values(types["TcgEngine.GameMode"]).items()}
    return {key: values(field_type(cls, name)) for key, (cls, name) in ENUMS.items()}, modes


def main(data_dir, out_dir):
    env = UnityPy.load(data_dir)
    version = None
    texts = {}
    data = {}
    unknown = set()
    for obj in env.objects:
        kind = obj.type.name
        if kind == "PlayerSettings":
            version = obj.read_typetree(check_read=False)["bundleVersion"]
            generator = TypeTreeGenerator(obj.assets_file.unity_version)
            generator.load_local_dll_folder(os.path.join(data_dir, "Managed"))
            env.typetree_generator = generator
        elif kind == "TextAsset":
            asset = obj.read()
            text = asset.m_Script.lstrip("\ufeff")
            if text.startswith("Key,"):
                rows = csv.reader(io.StringIO(text, newline=""))
                header = next(rows)
                for column in header[1:]:
                    if column not in LANGS:
                        sys.exit(f"unknown language column: {column}")
                for row in rows:
                    if not row:
                        continue
                    if len(row) != len(header):
                        sys.exit(f"{asset.m_Name}: row {row[0]!r} has {len(row)} cells, expected {len(header)}")
                    if row[0] in texts:
                        sys.exit(f"duplicate text key: {row[0]}")
                    texts[row[0]] = {LANGS[h]: cell.replace("\r\n", "\n") for h, cell in zip(header[1:], row[1:])}
        elif kind == "MonoBehaviour":
            pointer = obj.parse_monobehaviour_head().m_Script
            if not pointer:
                continue
            script = pointer.deref_parse_as_object()
            full = f"{script.m_Namespace}.{script.m_ClassName}"
            if full in ID_FIELD:
                data[(obj.assets_file.name, obj.path_id)] = (full, obj)
            elif script.m_Namespace in ("Spam.Data", "TcgEngine") and full not in IGNORE and not full.startswith(IGNORE_PREFIXES):
                unknown.add(full)
    if version is None:
        sys.exit("PlayerSettings not found")
    if unknown:
        sys.exit("unknown data classes: " + ", ".join(sorted(unknown)))
    empty = set(ID_FIELD) - {full for full, _ in data.values()}
    if empty:
        sys.exit("no objects for: " + ", ".join(sorted(empty)))

    trees = {key: (full, obj.read_typetree()) for key, (full, obj) in data.items()}
    ids = {key: tree[ID_FIELD[full]] for key, (full, tree) in trees.items()}

    def is_pointer(value):
        return isinstance(value, dict) and value.keys() == {"m_FileID", "m_PathID"}

    def deref(pointer, f):
        return PPtr(m_FileID=pointer["m_FileID"], m_PathID=pointer["m_PathID"]).deref(f) if pointer["m_PathID"] else None

    def refs(value, f):
        if is_pointer(value):
            target = deref(value, f)
            if target:
                yield target
        elif isinstance(value, dict):
            for k, v in value.items():
                if k not in DROP:
                    yield from refs(v, f)
        elif isinstance(value, list):
            for v in value:
                yield from refs(v, f)

    objects = {key: obj for key, (_, obj) in data.items()}
    kept = {key for key, (full, tree) in trees.items() if (full == CARD and tree["_isReleased"]) or full == TOX}
    todo = list(kept)
    while todo:
        key = todo.pop()
        obj = objects[key]
        tree = trees[key][1] if key in trees else obj.read_typetree()
        for target in refs(tree, obj.assets_file):
            target_key = (target.assets_file.name, target.path_id)
            if target_key in kept or target.type.name != "MonoBehaviour" or trees.get(target_key, ("",))[0] == CARD:
                continue
            kept.add(target_key)
            objects[target_key] = target
            todo.append(target_key)

    def conv(value, f):
        if isinstance(value, dict):
            if is_pointer(value):
                target = deref(value, f)
                if not target:
                    return None
                key = (target.assets_file.name, target.path_id)
                if key in ids:
                    return ids[key]
                return target.parse_monobehaviour_head().m_Name if target.type.name == "MonoBehaviour" else target.peek_name()
            return {camel(k): conv(v, f) for k, v in value.items() if k not in DROP}
        if isinstance(value, list):
            return [conv(v, f) for v in value]
        return f32(value) if isinstance(value, float) else value

    def localize(key):
        cells = texts.get(key)
        if not cells or not any(cells.values()):
            return None
        missing = set(LANGS.values()) - cells.keys()
        if missing:
            sys.exit(f"{key}: no column for {', '.join(sorted(missing))}")
        return cells

    catalog = {"appVersion": version, "languages": sorted(LANGS.values()), **{name: [] for name in OUT.values()}}
    skipped = []
    for name, (title, desc) in GLOSSARY.items():
        keys = {k[len(p):] for p in (title, desc) for k in texts if k.startswith(p)}
        catalog[name] = []
        for key in sorted(keys):
            record = {"id": key, "name": localize(title + key), "description": localize(desc + key)}
            if not record["name"] or not record["description"]:
                skipped.append(f"{name}:{key}")
                continue
            if not all(record[field][lang] for field in ("name", "description") for lang in ("en", "fr")):
                sys.exit(f"{name} {key}: no en or fr text")
            catalog[name].append(record)
    removed = {name: 0 for name in OUT.values()}
    for key, (full, tree) in trees.items():
        if full not in OUT:
            continue
        if key not in kept:
            removed[OUT[full]] += 1
            continue
        record = conv({k: v for k, v in tree.items() if k != ID_FIELD[full]}, data[key][1].assets_file)
        if full != TOX:
            record.pop("modificatorImage", None)
        record["id"] = tree[ID_FIELD[full]]
        assert "name" not in record and "description" not in record
        if OUT[full] == "artists":
            record["name"] = tree["_name"]
        else:
            name_key, desc_key = TEXT_KEYS[OUT[full]](record)
            record["name"], record["description"] = localize(name_key), localize(desc_key)
        catalog[OUT[full]].append(record)

    abilities = {a["id"]: a for a in catalog["abilities"]}
    for card in catalog["cards"]:
        card["activeAbilities"] = [{"id": a, "cooldown": abilities[a]["cooldown"]} for a in card["activeAbilities"]]
    for name in OUT.values():
        catalog[name].sort(key=lambda r: r["id"])
        assert len({r["id"] for r in catalog[name]}) == len(catalog[name]), name

    def named(record):
        return record["name"] and record["name"]["en"] and record["name"]["fr"]

    for card in catalog["cards"]:
        if not named(card):
            sys.exit(f"released card without en and fr name: {card['id']}")
    for talent in catalog["talents"]:
        if "Talent.Name." + talent["id"] in texts and not named(talent):
            sys.exit(f"talent without en and fr name: {talent['id']}")

    catalog["enums"], modes = enums(os.path.join(data_dir, "Managed", "Assembly-CSharp.dll"))
    for name, (full, text_key, field) in NAMES.items():
        catalog[name] = []
        for kind, tree in trees.values():
            if kind != full:
                continue
            record = {"id": tree["_id"], "name": localize(text_key(tree["_id"]))}
            mode = GAME_MODES.get(record["id"], record["id"])
            if field:
                record[camel(field)] = tree[field]
            elif mode in modes:
                record["value"] = modes[mode]
            else:
                sys.exit(f"game type without GameMode value: {record['id']}")
            if not named(record):
                sys.exit(f"{name} {record['id']}: no en or fr name")
            catalog[name].append(record)
        catalog[name].sort(key=lambda r: r[camel(field) if field else "value"])

    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, f"{version}.json")
    with open(path, "w", encoding="utf-8", newline="\n") as out:
        out.write(json.dumps(catalog, ensure_ascii=False, sort_keys=True, indent=2) + "\n")
    print(path, *(f"{name}={len(catalog[name])}" for name in OUT.values()))
    print("removed", *(f"{name}={count}" for name, count in removed.items()))
    print("null names", *(f"{name}={sum(1 for r in catalog[name] if not r['name'])}" for name in OUT.values()))
    print(*(f"{name}={len(catalog[name])}" for name in GLOSSARY), "skipped", *skipped)
    print(*(f"{name}={len(catalog[name])}" for name in NAMES), "enums", *(f"{k}={len(v)}" for k, v in catalog["enums"].items()))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: uv run datamine/extract.py <SPAM_Data> <out dir>")
    main(sys.argv[1], sys.argv[2])
