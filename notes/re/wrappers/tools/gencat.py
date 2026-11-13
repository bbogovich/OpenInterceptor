"""Generate catalogue.md: every WrappersDLL image id grouped by screen and role."""
import sys; sys.path.insert(0,'.')
SIZES={int(l.split()[0]):(int(l.split()[1]),int(l.split()[2])) for l in open('sizes.txt')}
SPR={}
for l in open('spr_wrappers.txt'):
    p=l.split(); SPR[int(p[2])]=int(p[0])
V,D='verified','derived'
# (lo,hi,group,role,note,confidence)
R=[
(5000,5000,'Shared backdrops','background','metal tile; screen 12 (multiplayer home, MultiHome.cpp) bg and tiled behind many dialogs/screens (loaded by 0x41b486 and ~25 other functions)',V),
(5005,5005,'Shared backdrops','background','blue grid 640x480; sprite-table index 429 (group 0, flag byte=1)',V),
(5006,5006,'Shared backdrops','panel','524x275 dark blue panel; sprite index 430 (group 0, flag=1); role derived: dialog/report backdrop',D),
(5030,5030,'CSD (screen 17)','background','CSD star map; loaded in CSD::Init 0x438101',V),
(5050,5050,'CSD (screen 17)','frame','round-map bezel/mask (green = see-through), sprite 891 (group 1); its rect is offset by (52,58) -> drawn at (52,58)',V),
(5051,5051,'CSD (screen 17)','frame','minimap mask (red = see-through), sprite 892 (group 1); drawn over minimap hole (502,43,596,137)',D),
(5052,5052,'CSD (screen 17)','panel','slide-out UFOpedia panel, 9 icons + close arrow (red = see-through); member +0x5cc0; shown when panel mode 2 (button 503)',V),
(5053,5054,'CSD (screen 17)','lamp','25x25 red lamp, 2 frames (member array +0x5d5c / +0x5d9c)',V),
(5055,5055,'CSD (screen 17)','lamp','23x23 black lamp-off frame (not referenced directly)',D),
(5056,5056,'CSD (screen 17)','background','550x550 starfield behind the round map',D),
(5057,5057,'CSD (screen 17)','panel','slide-out status-report panel, 6 icons + close arrow (white = see-through); member +0x5d00; panel mode 1 (button 500)',V),
(5060,5064,'CSD (screen 17)','animation','green lamp, 5 frames; Animation object +0x54fc placed at (618,453)',V),
(5065,5069,'CSD (screen 17)','animation','red lamp, 5 frames (alternate set prepared in same function 0x44068d)',V),
(5070,5073,'Shared UI kit','button','92x28 pill with arrow (cyan = see-through), sprite indices 367-370; up/down pairs',D),
(5075,5076,'CSD (screen 17)','icon','15x15 corner brackets green/red (map selection marker)',D),
(5077,5078,'CSD (screen 17)','icon','25x23 corner brackets (map target marker)',D),
(5101,5101,'Research (screen 9)','background','research screen; loaded 0x468ff0',V),
(5103,5106,'UFOpedia (screen 23)','button','28x28 round arrow buttons, sprite 396-399. Button 102 (616,64)=5103 up/5104 down; button 103 (336,64)=5105/5106',V),
(5107,5107,'UFOpedia (screen 23)','background','Databank/UFOpedia screen; loaded 0x530b6b',V),
(5108,5111,'UFOpedia (screen 23)','button','27x26 red arrow buttons, sprite 400-403. Button 108 (614,123)=5108/5109; 109 (614,160)=5110/5111',V),
(5112,5122,'UFOpedia (screen 23)','animation','265x265 iris/aperture transition, 11 frames (loop base 5112 in 0x530b6b); drawn in picture box (32,12,297,277)',V),
(5199,5199,'UFOpedia (screen 23)','icon','265x265 default/blank article picture',D),
(5200,5356,'UFOpedia (screen 23)','icon','264x264 article pictures (ships, weapons, modules, aliens); id table in 0x52eb7b; ids 5259-5262 absent',V),
(5400,5403,'Shared UI kit','button','27x26 red arrow buttons, sprite indices 363-366 (5402/5403 at 363/364)',D),
(6100,6100,'Equip Ships (screens 7/8)','background','hangar: ring of 9 pilot/ship sockets, stat panel, docked-ship list; loaded 0x535566',V),
(6105,6105,'Bases (screen 6)','background','bases: 3D view, info pane, rotate dial; loaded 0x47c74d',V),
(6110,6110,'Equipment & Personnel (screen 18)','background','purchase/sell/transfer/manufacture screen; loaded 0x563a7f',V),
(6300,6312,'Shared UI kit','animation','50x50 lever-switch frames (6303 absent; 6308 is 51x54); used with "add" offsets in MiscWrappers dialogs',D),
(6320,6329,'Shared UI kit','icon','80x20 triple-arrow strips, sprite indices 35-44 (6329 is index 39)',V),
(6330,6330,'Base Stats (screen 22)','background','Stat-O-Matic base statistics; loaded 0x50179b',V),
(6500,6507,'Dialog frame kit "pipe"','frame','pipe frame: straights and corners, sprite 234-241',V),
(6520,6525,'Dialog frame kit "pipe"','button','green capsule pieces (left/mid/right x normal/pressed), sprite 258-263; MsgBox buttons use sprite 258',V),
(6530,6537,'Dialog frame kit "pipe"','frame','pipe frame, larger pieces, sprite 242-249',V),
(6542,6542,'Flight loading','background','"downloading real-time" loading screen before flight; loaded 0x45c4fd',V),
(6550,6557,'Dialog frame kit "pipe"','frame','pipe frame, tall pieces, sprite 250-257',V),
(6560,6575,'Dialog frame kit style 1 (radio bubble)','frame','bars, header strips and 82x73 corner/bubble pieces, sprite 163-178; MsgBox frame painter 0x518604 draws 171+ for style 1. Sprite ids verified; kit grouping derived',D),
(6580,6589,'Dialog frame kit style 2 (teal)','frame','teal frame pieces, sprite 179-188; painter uses 181-189 for style 2. Grouping derived',D),
(6590,6599,'Dialog frame kit style 3 (red/alien)','frame','red frame pieces incl. 227x99 header with label plate, sprite 189-198; painter uses 198 for style 3. Grouping derived',D),
(6600,6600,'Dialogs','dialog','579x409 message panel with portrait socket and two green text panes (advisor/news style)',D),
(6601,6605,'Shared UI kit','lamp','small LEDs 11-14 px, sprite 387-391',V),
(6606,6609,'Shared UI kit','button','35x35 red round buttons, sprite 394/395/392/393 (pairs)',V),
(6800,6800,'Briefing (screen 25)','background','mission briefing (hangar screen + green text pane); loaded 0x522398',V),
(6801,6817,'Briefing (screen 25)','animation','50x50 red alarm button, 17 frames (loop base 6801)',V),
(6818,6851,'Briefing (screen 25)','gauge','24x276 vertical gauge, two 17-frame sets (loop bases 6818 and 6835)',V),
(6852,6852,'Briefing (screen 25)','frame','516x275 grid overlay for the briefing map (member +0x668c)',V),
(6853,6891,'Briefing (screen 25)','icon','green holographic object icons for the briefing grid (loop base 6853)',V),
(6900,6900,'Debriefing (screen 26)','background','mission debriefing; loaded 0x53ee96',V),
(6901,6910,'Debriefing (screen 26)','animation','65x125 hatch/stamp animation, 10 frames (loop base 6901, also loaded by Briefing)',V),
(6911,6922,'Shared UI kit','button','24-25 px green triangle arrows, sprite 404-415 (up/down x normal/pressed); list scroll on options, purchase, debriefing',V),
(6923,6924,'Briefing (screen 25)','icon','10x12 marker pair (members +0x924/+0x964; rects offset to (522,400)/(522,415))',V),
(7010,7010,'Combat-sim setup (screen 4)','background','TrainingSetup.cpp (title "SIM", menu cmd 282); loaded 0x544295',V),
(7020,7020,'Game configuration (screen 3)','background','MainConfig.cpp options/keyboard configuration; loaded 0x52a555',V),
(7021,7024,'Game configuration (screen 3)','icon','slider thumb / small markers (7022 absent)',V),
(7030,7031,'Shared UI kit','gauge','200x32 bar, 2 states (loaded by MainConfig)',V),
(7032,7033,'Shared UI kit','gauge','331x32 bar, 2 states; loaded by Bases, BaseStats, EquipShip, EquipPerson, MainConfig',V),
(7035,7038,'Game configuration (screen 3)','button','44x32 input-device buttons, 2 pairs (normal/pressed)',V),
(7039,7041,'Game configuration (screen 3)','button','40x40 side buttons accept / cancel / keyboard',V),
(7042,7044,'Game configuration (screen 3)','icon','76x76 logo preview, 3 brightness steps',V),
(7045,7054,'Game configuration (screen 3)','animation','91x91 rotary knob, 10 positions',V),
(7055,7065,'Game configuration (screen 3)','gauge','80x80 volume meter, 11 positions',V),
(7066,7067,'Game configuration (screen 3)','icon','16x13 red arrows',V),
(7068,7075,'Game configuration (screen 3)','animation','76x79 music toggle switch, 8 frames',V),
(7500,7504,'Title','background','title splash images; 7500+n chosen by index (0x4b6ab9, 0x4fa982, 0x57f0b5)',V),
(7600,7600,'Title (screen 2)','background','main menu: alien behind glass; loaded 0x50efd7',V),
(7601,7615,'Title (screen 2)','animation','134x479 left door, 15 frames (loop base 7601)',V),
(7616,7630,'Title (screen 2)','animation','136x479 right door, 15 frames (loop base 7616)',V),
(7631,7631,'Title (screen 2)','list highlight','108x29 menu-button highlight (member +0x106c)',V),
(7632,7645,'Title (screen 2)','animation','220x238 porthole, 14 frames (loop base 7632); hot rect (245,206,398,359)',V),
(7646,7651,'Title (screen 2)','lamp','40x26 lamp animation, 6 frames (loop base 7646)',V),
(7652,7657,'Title (screen 2)','lamp','35x30 lamp animation, 6 frames (loop base 7652)',V),
(7700,7700,'Status reports (screens 32-37)','background','report frame; loaded 0x48fef4 (StatusReports base class)',V),
(7800,7800,'Multiplayer players (screen 29)','background','NetPlayers.cpp; loaded 0x50868a',V),
(7900,7900,'Multiplayer team select (screen 31)','background','TeamSel.cpp four-panel; loaded 0x5a78f5',V),
(8020,8022,'CSD (screen 17)','icon','15x15 map markers, 3 (loop base 8020 into member array +0x6240)',V),
(8100,8100,'Flight loading splash','background','spiral alien (yellow); chosen 50/50 with 8116 in a loader thread (0x45d0a3)',V),
(8101,8115,'Flight loading splash','animation','177x99 eye animation for 8100, 15 frames (loop base 8101)',V),
(8116,8116,'Flight loading splash','background','spiral alien (green)',V),
(8117,8131,'Flight loading splash','animation','203x91 eye animation for 8116, 15 frames (loop base 8117)',V),
(8132,8132,'Credits (screen 10)','background','320x240 team photo; loaded 0x4b4b3d (Floaters.cpp)',V),
]
groups={}
unassigned=[]
for i in sorted(SIZES):
    for lo,hi,g,role,note,c in R:
        if lo<=i<=hi:
            groups.setdefault(g,[]).append((i,role,note,c,lo,hi)); break
    else: unassigned.append(i)
out=['# WrappersDLL image catalogue','',
'Every TARG resource in `Game/Resources/WrappersDLL.dll` (581 ids), grouped by the screen that loads it and by role.',
'Sizes are read from the extracted PNGs. "sprite" = index into the exe sprite table at VA 0x69bc18 (20-byte records',
'`{index, resource_id, hot_x, hot_y, flag, flag}`); -2 hot spot = centred. Confidence: **verified** = id/position read from',
'the exe code or tables; **derived** = inferred from the art or context. Facts only; no image content is reproduced.','']
order=['CSD (screen 17)','Research (screen 9)','UFOpedia (screen 23)','Bases (screen 6)','Equip Ships (screens 7/8)','Equipment & Personnel (screen 18)','Base Stats (screen 22)','Briefing (screen 25)','Debriefing (screen 26)','Game configuration (screen 3)','Combat-sim setup (screen 4)','Title','Title (screen 2)','Status reports (screens 32-37)','Multiplayer players (screen 29)','Multiplayer team select (screen 31)','Credits (screen 10)','Flight loading','Flight loading splash','Shared backdrops','Shared UI kit','Dialogs','Dialog frame kit "pipe"','Dialog frame kit style 1 (radio bubble)','Dialog frame kit style 2 (teal)','Dialog frame kit style 3 (red/alien)']
for g in order:
    items=groups.get(g,[])
    if not items: continue
    out.append('## '+g); out.append('')
    out.append('| Ids | Size | Role | Notes | Conf. |'); out.append('|---|---|---|---|---|')
    seen=set()
    for i,role,note,c,lo,hi in items:
        if (lo,hi) in seen: continue
        seen.add((lo,hi))
        ids=[x for x in sorted(SIZES) if lo<=x<=hi]
        szs=sorted(set('%dx%d'%SIZES[x] for x in ids))
        rng=str(lo) if lo==hi else '%d-%d (%d)'%(lo,hi,len(ids))
        sp=[SPR[x] for x in ids if x in SPR]
        spn=(' sprite %d-%d.'%(min(sp),max(sp))) if sp and 'sprite' not in note else ''
        out.append('| %s | %s | %s | %s%s | %s |'%(rng,', '.join(szs) if len(szs)<4 else szs[0]+' ... ',role,note,spn,c))
    out.append('')
assert not unassigned, unassigned
out.append('Total ids catalogued: %d'%sum(len([x for x in SIZES]) for _ in [0]))
open('catalogue.md','w').write('\n'.join(out)+'\n')
print(len(SIZES),'ids; groups',len(groups))
