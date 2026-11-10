# TGA images and cockpits

`TARG` resources and the files in `3DObjects/` are standard Truevision TGA. **verified**

- Type 2 (uncompressed true-colour), 16 bits per pixel (**RGB555**, with an attribute bit),
  18-byte header, no image ID. Descriptor bit 5 clear: rows bottom-up.
- Some wrapper screens are type 10 (RLE true-colour).

## Cockpits

`CockpitA.dll`, `CockpitB.dll` and `CockpitC.dll` each hold a 640×480 cockpit frame (resource
IDs 12000, 12200, 12400) and many smaller parts: gauge needles, MFD and radar elements, damage
states. One DLL per player hull (derived: the game has exactly three hulls).

The frame uses **cyan, RGB555 `0x03DF`**, as its colour key: those pixels are the window onto
space. The engine converts the key to transparent alpha at load.

The frame shows (from the art): a message strip across the top, two round scopes (radar), a
centre MFD, four small gauges above it, and two lower dials. What each scope shows (fore/aft or
range modes) is **derived** and refined in Stage 1–2.

## Other images

`3DObjects/` holds reticles (five variants), lens-flare elements, sun and star sprites.
`SpritesDLL.dll` holds ~1,700 small sprites; `WrappersDLL.dll` the full-screen menus.
