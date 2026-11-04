# /// script
# requires-python = ">=3.11"
# dependencies = ["UnityPy==1.25.4", "TypeTreeGeneratorAPI==0.0.10", "Pillow==12.0.0"]
# ///
import os
import shutil
import sys

import UnityPy
from PIL import Image
from UnityPy.classes import PPtr
from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator

ELEMENTS = ("divine", "earth", "fire", "light", "nature", "shadow", "water")
RARITIES = ("common", "rare", "epic", "legendary", "divine")
UI = {
    "card_mask": (716, 1024),
    **{f"card_front_{e}": (716, 1024) for e in ELEMENTS},
    **{f"element_{e}": (180, 180) for e in ELEMENTS},
    **{f"card_rarity_{r}": (216, 108) for r in RARITIES},
    "card_ability_active": (680, 108),
    "card_cooldown": (68, 68),
    "card_hp_bar_mask": (656, 56),
    "gold_icon": (116, 116),
    **{f"card_ability_passive_line_{r}_{side}": (164, 82) for r in RARITIES[:4] for side in ("left", "right")},
}
CARD = (1432, 2048)
ILLUSTRATION = (716, 1024)
THUMB = (179, 256)
AVATAR = (256, 256)


def full_image(sprite):
    img = sprite.image
    width, height = round(sprite.m_Rect.width), round(sprite.m_Rect.height)
    if img.size == (width, height):
        return img
    offset = sprite.m_RD.textureRectOffset
    out = Image.new("RGBA", (width, height))
    out.paste(img, (round(offset.x), height - round(offset.y) - img.height))
    return out


def main(data_dir, out_dir):
    env = UnityPy.load(data_dir)
    container, sprites, cards = {}, {}, []
    for obj in env.objects:
        kind = obj.type.name
        if kind == "PlayerSettings":
            generator = TypeTreeGenerator(obj.assets_file.unity_version)
            generator.load_local_dll_folder(os.path.join(data_dir, "Managed"))
            env.typetree_generator = generator
        elif kind == "ResourceManager":
            for path, pointer in obj.read_typetree()["m_Container"]:
                container.setdefault(path, []).append((pointer, obj.assets_file))
        elif kind == "Sprite":
            sprites.setdefault(obj.peek_name(), []).append(obj)
        elif kind == "MonoBehaviour":
            pointer = obj.parse_monobehaviour_head().m_Script
            script = pointer.deref_parse_as_object() if pointer else None
            if script and f"{script.m_Namespace}.{script.m_ClassName}" == "Spam.Data.CardData":
                cards.append(obj)

    def resource(path):
        found = [
            PPtr(m_FileID=p["m_FileID"], m_PathID=p["m_PathID"]).deref(f) for p, f in container.get(path.lower(), [])
        ]
        found = [o for o in found if o.type.name == "Sprite"]
        if len(found) != 1:
            sys.exit(f"{path}: {len(found)} sprites")
        return found[0].read()

    for folder in ("card", "thumb", "avatar", "ui"):
        shutil.rmtree(os.path.join(out_dir, folder), ignore_errors=True)
    files = []

    def save(img, name, quality=85):
        path = os.path.join(out_dir, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        img.save(path, "WEBP", quality=quality)
        files.append(path)

    for name, size in UI.items():
        found = sprites.get(name, [])
        if len(found) != 1:
            sys.exit(f"sprite {name}: {len(found)} objects")
        save(full_image(found[0].read()).resize(size, Image.LANCZOS), f"ui/{name}.webp")

    released = 0
    for obj in cards:
        tree = obj.read_typetree()
        if not tree["_isReleased"]:
            continue
        released += 1
        art = full_image(resource(tree["_illustrationResourcePath"])).convert("RGB")
        if art.size != CARD:
            sys.exit(f"{tree['_illustrationResourcePath']}: illustration is {art.width}x{art.height}, expected 1432x2048")
        save(art.resize(ILLUSTRATION, Image.LANCZOS), f"card/{tree['_id']}.webp")
        save(art.resize(THUMB, Image.LANCZOS), f"thumb/{tree['_id']}.webp")
        x, y = tree["_avatarImageOffset"]["x"], tree["_avatarImageOffset"]["y"]
        w, h = tree["_avatarImageSize"]["x"], tree["_avatarImageSize"]["y"]
        save(art.crop((x, y, x + w, y + h)).resize(AVATAR, Image.LANCZOS), f"avatar/{tree['_id']}.webp")

    total = sum(os.path.getsize(f) for f in files)
    print(f"ui={len(UI)} card={released} thumb={released} avatar={released} files={len(files)} size={total / 1e6:.1f} MB")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: uv run datamine/art.py <SPAM_Data> <out dir>")
    main(sys.argv[1], sys.argv[2])
