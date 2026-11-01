# SPDX-License-Identifier: AGPL-3.0-only
# MuConsole concept generator (visual reference for muconsole-theme).
#
#   scripts/fetch-fonts.sh
#   python3 docs/design/gen_concept.py /tmp/concept_desktop.svg hurmit
#   python3 docs/design/gen_concept.py /tmp/concept_desktop_dyslexic.svg dyslexic
#
# The output SVG embeds the HeavyData subset as base64, so it must never be
# committed (HeavyData is freeware, not OFL). Commit PNG renders only.
"""Concept v2: hardwood veneer + dark tungsten plates, pistol-grip control switches,
fake CRT terminals, dot-matrix STN LCD dynamic labels, acid-etched static labels.
Trainway (CI repo as railway) + Delivery integration-test tracker (CD)."""
import math, sys

W, H = 1512, 860
OUT = sys.argv[1] if len(sys.argv) > 1 else "concept_desktop.svg"
CRTFONT = sys.argv[2] if len(sys.argv) > 2 else "hurmit"
import base64, os
# Subset WOFF2 files written by scripts/fetch-fonts.sh (never committed).
FD = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "dist", "fonts")
def b64(n):
    return base64.b64encode(open(os.path.join(FD, n), "rb").read()).decode()
CRT_FAMILY = "MuCRT"
ETCH_FAMILY = "MuEtch"
o = []
a = o.append

# ---------------- 5x7 dot-matrix font ----------------
F = {
'A':[" ### ","#   #","#   #","#####","#   #","#   #","#   #"],
'B':["#### ","#   #","#   #","#### ","#   #","#   #","#### "],
'C':[" ### ","#   #","#    ","#    ","#    ","#   #"," ### "],
'D':["#### ","#   #","#   #","#   #","#   #","#   #","#### "],
'E':["#####","#    ","#    ","#### ","#    ","#    ","#####"],
'F':["#####","#    ","#    ","#### ","#    ","#    ","#    "],
'G':[" ### ","#   #","#    ","# ###","#   #","#   #"," ####"],
'H':["#   #","#   #","#   #","#####","#   #","#   #","#   #"],
'I':[" ### ","  #  ","  #  ","  #  ","  #  ","  #  "," ### "],
'J':["  ###","   # ","   # ","   # ","   # ","#  # "," ##  "],
'K':["#   #","#  # ","# #  ","##   ","# #  ","#  # ","#   #"],
'L':["#    ","#    ","#    ","#    ","#    ","#    ","#####"],
'M':["#   #","## ##","# # #","# # #","#   #","#   #","#   #"],
'N':["#   #","#   #","##  #","# # #","#  ##","#   #","#   #"],
'O':[" ### ","#   #","#   #","#   #","#   #","#   #"," ### "],
'P':["#### ","#   #","#   #","#### ","#    ","#    ","#    "],
'Q':[" ### ","#   #","#   #","#   #","# # #","#  # "," ## #"],
'R':["#### ","#   #","#   #","#### ","# #  ","#  # ","#   #"],
'S':[" ####","#    ","#    "," ### ","    #","    #","#### "],
'T':["#####","  #  ","  #  ","  #  ","  #  ","  #  ","  #  "],
'U':["#   #","#   #","#   #","#   #","#   #","#   #"," ### "],
'V':["#   #","#   #","#   #","#   #","#   #"," # # ","  #  "],
'W':["#   #","#   #","#   #","# # #","# # #","# # #"," # # "],
'X':["#   #","#   #"," # # ","  #  "," # # ","#   #","#   #"],
'Y':["#   #","#   #"," # # ","  #  ","  #  ","  #  ","  #  "],
'Z':["#####","    #","   # ","  #  "," #   ","#    ","#####"],
'0':[" ### ","#   #","#  ##","# # #","##  #","#   #"," ### "],
'1':["  #  "," ##  ","  #  ","  #  ","  #  ","  #  "," ### "],
'2':[" ### ","#   #","    #","   # ","  #  "," #   ","#####"],
'3':["#####","   # ","  #  ","   # ","    #","#   #"," ### "],
'4':["   # ","  ## "," # # ","#  # ","#####","   # ","   # "],
'5':["#####","#    ","#### ","    #","    #","#   #"," ### "],
'6':["  ## "," #   ","#    ","#### ","#   #","#   #"," ### "],
'7':["#####","    #","   # ","  #  "," #   "," #   "," #   "],
'8':[" ### ","#   #","#   #"," ### ","#   #","#   #"," ### "],
'9':[" ### ","#   #","#   #"," ####","    #","   # "," ##  "],
' ':["     "]*7,
'/':["    #","    #","   # ","  #  "," #   ","#    ","#    "],
'-':["     ","     ","     ","#####","     ","     ","     "],
'.':["     ","     ","     ","     ","     "," ##  "," ##  "],
':':["     "," ##  "," ##  ","     "," ##  "," ##  ","     "],
'#':[" # # "," # # ","#####"," # # ","#####"," # # "," # # "],
'>':[" #   ","  #  ","   # ","    #","   # ","  #  "," #   "],
'+':["     ","  #  ","  #  ","#####","  #  ","  #  ","     "],
'%':["##   ","##  #","   # ","  #  "," #   ","#  ##","   ##"],
'@':[" ### ","#   #","# ###","# # #","# ###","#    "," ### "],
'_':["     ","     ","     ","     ","     ","     ","#####"],
}

def gid(c):
    return "g%02x" % ord(c)

# ---------------- defs ----------------
a(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
  f'font-family="{CRT_FAMILY}, ui-monospace, monospace">')
a('<defs>')
crt_files = {"hurmit": ("HurmitNerdFontMono-Regular.woff2", "HurmitNerdFontMono-Bold.woff2"),
             "dyslexic": ("OpenDyslexicMNerdFontMono-Regular.woff2", "OpenDyslexicMNerdFontMono-Regular.woff2")}[CRTFONT]
a('<style>'
  f'@font-face{{font-family:{CRT_FAMILY};font-weight:400;src:url(data:font/woff2;base64,{b64(crt_files[0])}) format("woff2")}}'
  f'@font-face{{font-family:{CRT_FAMILY};font-weight:700;src:url(data:font/woff2;base64,{b64(crt_files[1])}) format("woff2")}}'
  f'@font-face{{font-family:{ETCH_FAMILY};src:url(data:font/woff2;base64,{b64("HeavyDataNerdFont-Regular.woff2")}) format("woff2")}}'
  '</style>')
# wood veneer
a('''<filter id="woodgrain" x="0" y="0" width="100%" height="100%">
  <!-- fine, long grain: a large cabinet surface (~1 CSS px = 0.6 mm), not a small box -->
  <feTurbulence type="fractalNoise" baseFrequency="0.0011 0.46" numOctaves="4" seed="11" result="n"/>
  <feColorMatrix in="n" type="matrix" values="0 0 0 0 0.25  0 0 0 0 0.15  0 0 0 0 0.085  2.0 0 0 0 -0.88" result="grain"/>
  <!-- broad figure / tonal drift across the veneer leaf -->
  <feTurbulence type="fractalNoise" baseFrequency="0.0007 0.018" numOctaves="2" seed="4" result="f"/>
  <feColorMatrix in="f" type="matrix" values="0 0 0 0 0.20  0 0 0 0 0.12  0 0 0 0 0.07  1.2 0 0 0 -0.45" result="figure"/>
  <!-- open pores: tiny dark dashes -->
  <feTurbulence type="fractalNoise" baseFrequency="0.035 1.1" numOctaves="1" seed="9" result="p"/>
  <feColorMatrix in="p" type="matrix" values="0 0 0 0 0.02  0 0 0 0 0.01  0 0 0 0 0.005  3.0 0 0 0 -1.75" result="pores"/>
  <feMerge><feMergeNode in="figure"/><feMergeNode in="grain"/><feMergeNode in="pores"/></feMerge>
</filter>''')
# brushed tungsten
a('''<filter id="brushed" x="0" y="0" width="100%" height="100%">
  <!-- fine satin brushing: long hairlines (low x freq), very tight pitch (high y freq), low amplitude -->
  <feTurbulence type="fractalNoise" baseFrequency="0.004 2.4" numOctaves="3" seed="5" result="t"/>
  <feColorMatrix in="t" type="matrix" values="0 0 0 0 0.78  0 0 0 0 0.80  0 0 0 0 0.83  0.22 0 0 0 -0.085" result="hair"/>
  <!-- faint darker counter-lines for depth -->
  <feTurbulence type="fractalNoise" baseFrequency="0.006 1.6" numOctaves="2" seed="17" result="t2"/>
  <feColorMatrix in="t2" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0.20 0 0 0 -0.08" result="dark"/>
  <feMerge result="m"><feMergeNode in="hair"/><feMergeNode in="dark"/></feMerge>
  <feComposite in="m" in2="SourceGraphic" operator="in" result="sc"/>
  <feMerge><feMergeNode in="SourceGraphic"/><feMergeNode in="sc"/></feMerge>
</filter>
<linearGradient id="sheen" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".42" stop-color="#fff" stop-opacity=".045"/><stop offset=".5" stop-color="#fff" stop-opacity=".07"/><stop offset=".58" stop-color="#fff" stop-opacity=".045"/><stop offset="1" stop-color="#fff" stop-opacity="0"/></linearGradient>''')
a('''<linearGradient id="tung" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2b2d31"/><stop offset="1" stop-color="#17181b"/></linearGradient>
<linearGradient id="chrome" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#5d6168"/><stop offset=".35" stop-color="#f2f4f6"/><stop offset=".6" stop-color="#a9adb3"/><stop offset="1" stop-color="#4a4d52"/></linearGradient>
<linearGradient id="chromeV" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#5d6168"/><stop offset=".35" stop-color="#f2f4f6"/><stop offset=".6" stop-color="#a9adb3"/><stop offset="1" stop-color="#4a4d52"/></linearGradient>
<linearGradient id="steel" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#4b4f55"/><stop offset=".4" stop-color="#c3c7cc"/><stop offset=".62" stop-color="#8a8e94"/><stop offset="1" stop-color="#3a3d42"/></linearGradient>
<linearGradient id="park" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a3c40"/><stop offset=".5" stop-color="#26282b"/><stop offset="1" stop-color="#161719"/></linearGradient>
<radialGradient id="rubber" cx=".35" cy=".3" r=".8"><stop offset="0" stop-color="#3a3a3a"/><stop offset=".6" stop-color="#1a1a1a"/><stop offset="1" stop-color="#0a0a0a"/></radialGradient>
<radialGradient id="bakelite" cx=".35" cy=".3" r=".85"><stop offset="0" stop-color="#4a4a4d"/><stop offset=".35" stop-color="#1c1c1e"/><stop offset="1" stop-color="#050506"/></radialGradient>
<linearGradient id="esc" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1d1e20"/><stop offset="1" stop-color="#0c0d0e"/></linearGradient>
<radialGradient id="lensR" cx=".4" cy=".35" r=".7"><stop offset="0" stop-color="#ffd0c8"/><stop offset=".35" stop-color="#ff3b30"/><stop offset="1" stop-color="#7a0d08"/></radialGradient>
<radialGradient id="lensG" cx=".4" cy=".35" r=".7"><stop offset="0" stop-color="#d8ffe0"/><stop offset=".35" stop-color="#2fe060"/><stop offset="1" stop-color="#0b5a22"/></radialGradient>
<radialGradient id="lensA" cx=".4" cy=".35" r=".7"><stop offset="0" stop-color="#fff0c8"/><stop offset=".35" stop-color="#ffb43a"/><stop offset="1" stop-color="#7a4a08"/></radialGradient>
<radialGradient id="lensOff" cx=".4" cy=".35" r=".7"><stop offset="0" stop-color="#5a4a48"/><stop offset=".5" stop-color="#2a1e1d"/><stop offset="1" stop-color="#100b0b"/></radialGradient>
<radialGradient id="ball" cx=".35" cy=".35" r=".7"><stop offset="0" stop-color="#ffffff"/><stop offset=".4" stop-color="#c9ccd0"/><stop offset="1" stop-color="#55585d"/></radialGradient>
<linearGradient id="nut" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#8d9197"/><stop offset=".5" stop-color="#3e4146"/><stop offset="1" stop-color="#202226"/></linearGradient>
<linearGradient id="guard" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#6e0a10"/><stop offset=".45" stop-color="#d4202b"/><stop offset="1" stop-color="#5a080d"/></linearGradient>
<linearGradient id="lcdbg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#b6cc4a"/><stop offset="1" stop-color="#8ea42a"/></linearGradient>
<radialGradient id="crtG" cx=".5" cy=".5" r=".75"><stop offset="0" stop-color="#0b1a0f"/><stop offset="1" stop-color="#020604"/></radialGradient>
<radialGradient id="crtA" cx=".5" cy=".5" r=".75"><stop offset="0" stop-color="#1c1306"/><stop offset="1" stop-color="#060402"/></radialGradient>
<radialGradient id="vignette" cx=".5" cy=".5" r=".72"><stop offset=".6" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".75"/></radialGradient>
<linearGradient id="glass" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".09"/><stop offset=".35" stop-color="#fff" stop-opacity="0"/></linearGradient>
<pattern id="scan" width="3" height="3" patternUnits="userSpaceOnUse"><rect width="3" height="1.2" fill="#000" opacity=".45"/></pattern>
<filter id="phos" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="1.6" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
<filter id="lamp" x="-60%" y="-60%" width="220%" height="220%"><feGaussianBlur stdDeviation="3" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
''')
# glyph symbols + ghost
for c, rows in F.items():
    rects = ''.join(f'<rect x="{x}" y="{y}" width=".82" height=".82"/>'
                    for y, r in enumerate(rows) for x, ch in enumerate(r) if ch == '#')
    a(f'<symbol id="{gid(c)}" viewBox="0 0 5 7" overflow="visible">{rects}</symbol>')
a('<symbol id="ghost" viewBox="0 0 5 7">' + ''.join(
    f'<rect x="{x}" y="{y}" width=".82" height=".82"/>' for y in range(7) for x in range(5)) + '</symbol>')
a('</defs>')

# ---------------- primitives ----------------
def wood():
    a(f'<rect width="{W}" height="{H}" fill="#120b07"/>')
    a(f'<rect width="{W}" height="{H}" filter="url(#woodgrain)"/>')
    # book-matched veneer leaves (~230 mm each at cabinet scale): alternate tone + hairline seams
    leaf = 384
    for k, x0 in enumerate(range(0, W, leaf)):
        if k % 2:
            a(f'<rect x="{x0}" y="0" width="{leaf}" height="{H}" fill="#000" opacity=".10"/>')
        if x0:
            a(f'<line x1="{x0}" y1="0" x2="{x0}" y2="{H}" stroke="#070403" stroke-width="1"/>'
              f'<line x1="{x0+1}" y1="0" x2="{x0+1}" y2="{H}" stroke="#3a2618" stroke-width=".5" opacity=".6"/>')
    a(f'<rect width="{W}" height="{H}" fill="url(#vignette)" opacity=".5"/>')

def hole(cx, cy, r=3.4):
    a(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="#050505"/>'
      f'<path d="M{cx-r} {cy} A{r} {r} 0 0 0 {cx+r} {cy}" fill="none" stroke="#5c6067" stroke-width=".8"/>')

def plate(x, y, w, h, holes=True):
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="url(#tung)" filter="url(#brushed)"/>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="url(#sheen)"/>')
    a(f'<path d="M{x+0.5} {y+h-4} V{y+4} Q{x+0.5} {y+0.5} {x+4} {y+0.5} H{x+w-4}" fill="none" stroke="#5a5e65" stroke-width="1"/>')
    a(f'<path d="M{x+w-0.5} {y+4} V{y+h-4} Q{x+w-0.5} {y+h-0.5} {x+w-4} {y+h-0.5} H{x+4}" fill="none" stroke="#0c0d0f" stroke-width="1"/>')
    if holes:
        for hx, hy in ((x+9, y+9), (x+w-9, y+9), (x+9, y+h-9), (x+w-9, y+h-9)):
            hole(hx, hy)

def etched(x, y, txt, size=11, anchor="start", spacing=1, emboss=False):
    size = max(11, size * 1.05)
    base = (f'font-family="{ETCH_FAMILY}, sans-serif" '
            f'font-size="{size:.1f}" letter-spacing="{spacing}" text-anchor="{anchor}"')
    if emboss:   # raised: light lip top-left, shadow bottom-right
        a(f'<text x="{x+1.3}" y="{y+1.5}" {base} fill="#050506">{txt}</text>')
        a(f'<text x="{x-0.7}" y="{y-0.7}" {base} fill="#b4b9c1">{txt}</text>')
        a(f'<text x="{x}" y="{y}" {base} fill="#6a6f77">{txt}</text>')
    else:        # acid-etched recess: shadow top-left, lip bottom-right, frosted face
        a(f'<text x="{x-0.7}" y="{y-0.7}" {base} fill="#0b0c0e">{txt}</text>')
        a(f'<text x="{x+0.6}" y="{y+0.6}" {base} fill="#545961">{txt}</text>')
        a(f'<text x="{x}" y="{y}" {base} fill="#9499a1">{txt}</text>')

def lcd(x, y, txt, cols=None, p=2.0, pad=4):
    cols = cols or len(txt)
    cw = 6 * p
    w = cols * cw - p + 2 * pad
    h = 7 * p + 2 * pad
    a(f'<rect x="{x-2}" y="{y-2}" width="{w+4}" height="{h+4}" rx="2" fill="#08090a"/>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="1.5" fill="url(#lcdbg)"/>')
    g = [f'<g fill="#1c2608">']
    for i in range(cols):
        gx = x + pad + i * cw
        g.append(f'<use href="#ghost" x="{gx}" y="{y+pad}" width="{5*p}" height="{7*p}" opacity=".07"/>')
        ch = txt[i].upper() if i < len(txt) else ' '
        if ch in F and ch != ' ':
            g.append(f'<use href="#{gid(ch)}" x="{gx}" y="{y+pad}" width="{5*p}" height="{7*p}" opacity=".88"/>')
    g.append('</g>')
    a(''.join(g))
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h/2}" rx="1.5" fill="#fff" opacity=".07"/>')
    return w, h

# ---- oblique projection for raised hardware (panel = x,y ; height z out of the panel) ----
KX, KY = 0.30, -0.30          # where height projects on screen (viewer slightly below-left)
SX, SY = 0.45, 0.55           # where a raised point casts its shadow (light from top-left)

def proj(x, y, z):
    return x + KX * z, y + KY * z

def screw(x, y, r=2.6, rot=30):
    a(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r+0.6}" fill="#0b0b0c"/>'
      f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r}" fill="url(#steel)" stroke="#111" stroke-width=".5"/>'
      f'<g transform="rotate({rot} {x:.1f} {y:.1f})" stroke="#1a1b1d" stroke-width="1" stroke-linecap="round">'
      f'<line x1="{x-r*0.65:.1f}" y1="{y:.1f}" x2="{x+r*0.65:.1f}" y2="{y:.1f}"/><line x1="{x:.1f}" y1="{y-r*0.65:.1f}" x2="{x:.1f}" y2="{y+r*0.65:.1f}"/></g>')

def ind_lamp(x, y, color, lit, r=6):
    """Control-room indicating lamp: knurled steel bezel + domed coloured lens."""
    a(f'<circle cx="{x+1.2}" cy="{y+1.6}" r="{r+2.6}" fill="#000" opacity=".5"/>')
    a(f'<circle cx="{x}" cy="{y}" r="{r+2.6}" fill="url(#steel)" stroke="#111" stroke-width=".6"/>')
    a('<g stroke="#2a2c30" stroke-width=".6">' + ''.join(
        f'<line x1="{x+(r+2.5)*math.cos(math.radians(k*15)):.1f}" y1="{y+(r+2.5)*math.sin(math.radians(k*15)):.1f}" x2="{x+(r+1.2)*math.cos(math.radians(k*15)):.1f}" y2="{y+(r+1.2)*math.sin(math.radians(k*15)):.1f}"/>' for k in range(24)) + '</g>')
    lens = {"red": "lensR", "green": "lensG", "amber": "lensA"}[color] if lit else "lensOff"
    tint = {"red": "#5a1410", "green": "#123d1d", "amber": "#4a3008"}[color]
    a(f'<circle cx="{x}" cy="{y}" r="{r}" fill="url(#{lens})" {"filter=\"url(#lamp)\"" if lit else ""}/>')
    if not lit:
        a(f'<circle cx="{x}" cy="{y}" r="{r}" fill="{tint}" opacity=".55"/>')
    a(f'<ellipse cx="{x-r*0.3:.1f}" cy="{y-r*0.38:.1f}" rx="{r*0.42:.1f}" ry="{r*0.22:.1f}" fill="#fff" opacity="{.55 if lit else .18}"/>')

def pistol_switch(cx, cy, positions, cur, size=64, spring=False, pull=False, name=None):
    """Nuclear control-room pistol-grip rotary control switch (GE SBM / W-2 style):
    square black engraved escutcheon, position ticks + legends, black bakelite pistol grip
    with a white pointer inlay. spring=True engraves return-to-centre arrows; pull=True adds
    the red PULL-TO-TURN collar for state-changing actions."""
    k = size / 64.0
    h = size / 2
    a(f'<rect x="{cx-h+2:.1f}" y="{cy-h+3:.1f}" width="{size}" height="{size}" rx="{4*k:.1f}" fill="#000" opacity=".5"/>')
    a(f'<rect x="{cx-h:.1f}" y="{cy-h:.1f}" width="{size}" height="{size}" rx="{4*k:.1f}" fill="url(#esc)" stroke="#45484d" stroke-width=".8"/>')
    a(f'<path d="M{cx-h+1:.1f} {cy+h-4*k:.1f} V{cy-h+4*k:.1f} Q{cx-h+1:.1f} {cy-h+1:.1f} {cx-h+4*k:.1f} {cy-h+1:.1f} H{cx+h-4*k:.1f}" fill="none" stroke="#5c6066" stroke-width=".6" opacity=".7"/>')
    screw(cx - h + 6 * k, cy - h + 6 * k, r=2.0 * k, rot=20); screw(cx + h - 6 * k, cy + h - 6 * k, r=2.0 * k, rot=-30)
    ef = 'font-family="MuEtch, sans-serif" fill="#e6e3d8" text-anchor="middle"'
    for i, (ang, leg) in enumerate(positions):
        r1, r2, rl = 22.5 * k, 25.5 * k, 29.5 * k
        rad = math.radians(ang - 90)
        a(f'<line x1="{cx+r1*math.cos(rad):.1f}" y1="{cy+r1*math.sin(rad):.1f}" x2="{cx+r2*math.cos(rad):.1f}" y2="{cy+r2*math.sin(rad):.1f}" stroke="#e6e3d8" stroke-width="{1.4*k:.1f}"/>')
        a(f'<text x="{cx+rl*math.cos(rad):.1f}" y="{cy+rl*math.sin(rad)+2.6*k:.1f}" {ef} font-size="{7.2*k:.1f}">{leg}</text>')
    if spring:
        for sgn in (-1, 1):
            a0, a1 = math.radians(sgn * 40 - 90), math.radians(sgn * 12 - 90)
            rr = 15 * k
            a(f'<path d="M{cx+rr*math.cos(a0):.1f} {cy+rr*math.sin(a0):.1f} A{rr:.1f} {rr:.1f} 0 0 {1 if sgn<0 else 0} {cx+rr*math.cos(a1):.1f} {cy+rr*math.sin(a1):.1f}" fill="none" stroke="#e6e3d8" stroke-width="{.9*k:.1f}" opacity=".7"/>')
            ax, ay = cx + rr * math.cos(a1), cy + rr * math.sin(a1)
            a(f'<circle cx="{ax:.1f}" cy="{ay:.1f}" r="{1.2*k:.1f}" fill="#e6e3d8" opacity=".7"/>')
    if name:
        a(f'<text x="{cx}" y="{cy+h-5*k:.1f}" {ef} font-size="{6.4*k:.1f}" opacity=".9">{name}</text>')
    if pull:
        a(f'<circle cx="{cx}" cy="{cy}" r="{11.5*k:.1f}" fill="#b3121c" stroke="#3a0407" stroke-width=".8"/>')
    # pistol grip (local frame points up), rotated to the current position
    ang = positions[cur][0]
    d = (f"M{-4.6*k:.1f} {-2*k:.1f} L{-2.3*k:.1f} {-18.5*k:.1f} Q0 {-22.5*k:.1f} {2.3*k:.1f} {-18.5*k:.1f} L{4.6*k:.1f} {-2*k:.1f} "
         f"C{7.4*k:.1f} {4*k:.1f} {8*k:.1f} {10*k:.1f} {6.8*k:.1f} {13.5*k:.1f} C{5*k:.1f} {18.5*k:.1f} {-5*k:.1f} {18.5*k:.1f} {-6.8*k:.1f} {13.5*k:.1f} "
         f"C{-8*k:.1f} {10*k:.1f} {-7.4*k:.1f} {4*k:.1f} {-4.6*k:.1f} {-2*k:.1f} Z")
    a(f'<g transform="translate({cx+2.8*k:.1f} {cy+3.8*k:.1f}) rotate({ang})"><path d="{d}" fill="#000" opacity=".55"/></g>')
    a(f'<g transform="translate({cx} {cy}) rotate({ang})">'
      f'<path d="{d}" fill="url(#bakelite)" stroke="#000" stroke-width=".8"/>'
      f'<path d="M{-3.6*k:.1f} {-4*k:.1f} L{-1.9*k:.1f} {-17*k:.1f}" stroke="#fff" stroke-width="{1*k:.1f}" opacity=".22" stroke-linecap="round"/>'
      f'<path d="M{-5.6*k:.1f} {6*k:.1f} C{-6.2*k:.1f} {10*k:.1f} {-5*k:.1f} {14*k:.1f} {-3*k:.1f} {15.5*k:.1f}" fill="none" stroke="#fff" stroke-width="{.9*k:.1f}" opacity=".18"/>'
      f'<line x1="0" y1="{-6*k:.1f}" x2="0" y2="{-18.5*k:.1f}" stroke="#f2efe6" stroke-width="{1.6*k:.1f}" stroke-linecap="round"/>'
      f'<circle cx="0" cy="0" r="{3.4*k:.1f}" fill="url(#steel)" stroke="#111" stroke-width=".5"/>'
      f'<line x1="{-2*k:.1f}" y1="0" x2="{2*k:.1f}" y2="0" stroke="#222" stroke-width=".8"/>'
      '</g>')

def annunciator(x, y, w, h, legend, color=None):
    a(f'<rect x="{x-2}" y="{y-2}" width="{w+4}" height="{h+4}" rx="3" fill="#0b0c0e"/>')
    if color:
        a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="2" fill="{color}" filter="url(#lamp)"/>')
        a(f'<text x="{x+w/2}" y="{y+h/2+4}" text-anchor="middle" font-family="MuEtch, sans-serif" font-size="11" fill="#101010">{legend}</text>')
    else:
        a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="2" fill="#1d1f22"/>')
        a(f'<text x="{x+w/2}" y="{y+h/2+4}" text-anchor="middle" font-family="MuEtch, sans-serif" font-size="11" fill="#4d5157">{legend}</text>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h/2}" rx="2" fill="#fff" opacity=".08"/>')

def crt_open(x, y, w, h, kind="G"):
    a(f'<rect x="{x-6}" y="{y-6}" width="{w+12}" height="{h+12}" rx="20" fill="#0a0b0c" stroke="#3b3e43" stroke-width="1"/>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16" fill="url(#crt{kind})"/>')
    a(f'<g filter="url(#phos)">')

def crt_close(x, y, w, h):
    a('</g>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16" fill="url(#scan)"/>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16" fill="url(#vignette)"/>')
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="16" fill="url(#glass)"/>')

G, GD, A, AD = "#4dff7a", "#1f6b34", "#ffb43a", "#6b4a12"

CRT_SCALE = 0.618 / 0.732 if CRTFONT == "dyslexic" else 1.0   # keep the same column grid

def t(x, y, s, c=G, size=16, anchor="start", extra=""):
    size = round(size * CRT_SCALE, 1)
    a(f'<text x="{x}" y="{y}" fill="{c}" font-size="{size}" text-anchor="{anchor}" {extra}>{s}</text>')

# ================= SCENE =================
wood()

# ---- Area rail ----
plate(6, 6, 80, 848)
lcd(20, 18, "0516", cols=4, p=1.6, pad=3)
etched(46, 62, "AREA", size=11, anchor="middle", emboss=True)
area_names = ["OVR", "CI", "CD", "FLT"]
pistol_switch(46, 112, [(-60, "OVR"), (-20, "CI"), (20, "CD"), (60, "FLT")], 1, size=66)
for i, (lbl, badge) in enumerate(zip(["OVERVIEW", "CI", "CD", "FLEET"], ["", "3", "1", ""])):
    yy = 176 + i * 30
    ind_lamp(22, yy, "green", i == 1, r=5)
    etched(34, yy + 4, lbl, size=8 if len(lbl) > 5 else 9, spacing=0)
    if badge:
        lcd(62, yy - 8, badge, cols=1, p=1.4, pad=3)
for i in range(6):  # punched vent slots
    a(f'<rect x="24" y="{330+i*12}" width="44" height="5" rx="2.5" fill="#050505"/>')
etched(46, 708, "CRT FONT", size=9, anchor="middle")
pistol_switch(46, 760, [(-45, "HRMT"), (45, "DYSLX")], 0 if CRTFONT == "hurmit" else 1, size=66)
# ---- Top bar ----
plate(94, 6, 1412, 48)
segs = [("AREA", "CI"), ("REPO", "LMA1980/MODEL-RUNNER"), ("SUBMODULE", "K3-ENGINE"), ("BRANCH", "FEAT/RVV-GEMM"), ("JOB", "#1234")]
x = 112
for i, (lab, val) in enumerate(segs):
    etched(x, 20, lab, size=9, spacing=2)
    w, h = lcd(x, 24, val, p=1.6, pad=3)
    x += max(w, 40) + 14
    if i < len(segs) - 1:  # punched chevron
        a(f'<path d="M{x-4} 30 L{x+4} 36 L{x-4} 42 Z" fill="#050505"/>')
        x += 14
annunciator(1324, 16, 50, 28, "CI OK", "#3ddc6a")
annunciator(1382, 16, 50, 28, "CD STG", "#ffb43a")
annunciator(1440, 16, 50, 28, "FAULT")

# ---- Carousel ----
plate(1102, 64, 92, 192)
etched(1148, 90, "CAROUSEL", size=10, anchor="middle", emboss=True)
pistol_switch(1148, 150, [(-45, "PREV"), (0, "N"), (45, "NEXT")], 1, size=70, spring=True, name="SPRING RTN")
etched(1148, 214, "CARD 2 OF 8", size=9, anchor="middle")
cards = [
    ("MODEL-RUNNER", 0.81, "83.0", "+1.2", "B", "STAGING", "4/7", [3,4,4,5,6,6,7]),
    ("K3-ENGINE", 0.86, "87.4", "+2.1", "B", "PROMOTED", "7/7", [6,6,5,5,6,7,7]),
    ("K3-CORE", 0.93, "91.2", "+0.8", "B", "ROLLBACK", "5/7", [4,5,5,6,6,7,7]),
]
for i, (name, q, cov, bench, slot, cd, it, spark) in enumerate(cards):
    cx, cy, cw, ch = 94 + i * 336, 64, 324, 192
    plate(cx, cy, cw, ch)
    lcd(cx + 18, cy + 8, name, cols=15, p=1.6, pad=3)
    if i == 1:
        a(f'<circle cx="{cx+cw-90}" cy="{cy+17}" r="3.4" fill="#3ddc6a" filter="url(#lamp)"/>')
    etched(cx + cw - 20, cy + 22, "CI / CD", size=9, anchor="end")
    # CI screen
    sx, sy, sw, sh = cx + 16, cy + 42, 140, 132
    crt_open(sx, sy, sw, sh, "G")
    t(sx + 10, sy + 20, "QUALITY", GD, 14)
    segsN = 10
    for k in range(segsN):
        on = k < round(q * segsN)
        a(f'<rect x="{sx+10+k*9}" y="{sy+26}" width="7" height="12" fill="{G if on else GD}" opacity="{1 if on else .35}"/>')
    t(sx + sw - 10, sy + 37, f"{int(q*100)}", G, 18, "end")
    t(sx + 10, sy + 60, "COV", GD, 14); t(sx + sw - 10, sy + 60, f"{cov}%", G, 18, "end")
    t(sx + 10, sy + 82, "BENCH", GD, 14); t(sx + sw - 10, sy + 82, f"{bench}%", G, 18, "end")
    pts = ' '.join(f'{sx+12+k*19},{sy+120-v*3.5}' for k, v in enumerate(spark))
    a(f'<polyline points="{pts}" fill="none" stroke="{G}" stroke-width="1.5"/>')
    crt_close(sx, sy, sw, sh)
    # CD screen
    sx2 = cx + cw - 16 - 140
    crt_open(sx2, sy, sw, sh, "A")
    t(sx2 + 10, sy + 20, "SLOT", AD, 14); t(sx2 + sw - 10, sy + 20, slot, A, 20, "end")
    ty = sy + 52
    a(f'<line x1="{sx2+10}" y1="{ty}" x2="{sx2+50}" y2="{ty}" stroke="{A}" stroke-width="2"/>')
    for lbl, dy in (("A", -14), ("B", 14)):
        on = lbl == slot
        a(f'<path d="M{sx2+50} {ty} L{sx2+75} {ty+dy} L{sx2+108} {ty+dy}" fill="none" stroke="{A if on else AD}" stroke-width="{2 if on else 1.2}"/>')
        t(sx2 + 118, ty + dy + 5, lbl, A if on else AD, 14)
    t(sx2 + 10, sy + 96, "IT", AD, 14); t(sx2 + sw - 10, sy + 96, it, A, 18, "end")
    inv = cd in ("ROLLBACK", "TAINTED")
    if inv:
        a(f'<rect x="{sx2+8}" y="{sy+104}" width="{sw-16}" height="18" fill="{A}"/>')
    t(sx2 + sw / 2, sy + 118, cd, "#1a1004" if inv else A, 17, "middle")
    crt_close(sx2, sy, sw, sh)
for i in range(8):
    on = i == 1
    a(f'<circle cx="{530+i*18}" cy="{268}" r="3.5" fill="{"#3ddc6a" if on else "#222"}" {"filter=\"url(#lamp)\"" if on else ""} stroke="#0a0a0a"/>')

# ---- Rugged concentric rotary: OUTER = repository, INNER = submodule ----
def rotary(x, y, w, h, repos, ri, subs, si):
    plate(x, y, w, h)
    etched(x + 20, y + 26, "REPOSITORY", size=12, spacing=1, emboss=True)
    cx, cy = x + 84, y + 106
    # parkerized flange with 4 screws
    a(f'<circle cx="{cx+2}" cy="{cy+3}" r="60" fill="#000" opacity=".5"/>')
    a(f'<circle cx="{cx}" cy="{cy}" r="60" fill="url(#park)" stroke="#55585e" stroke-width=".6"/>')
    for k in range(4):
        ang = math.radians(45 + 90 * k)
        screw(cx + 54 * math.cos(ang), cy + 54 * math.sin(ang), r=2.2, rot=15 * k)
    # position LED arc (270 deg): one lamp per repository slot, lit = selected
    n = len(repos)
    for k in range(n):
        ang = math.radians(135 + 270 * k / max(n - 1, 1))
        lx, ly = cx + 52 * math.cos(ang), cy + 52 * math.sin(ang)
        on = k == ri
        a(f'<circle cx="{lx:.1f}" cy="{ly:.1f}" r="2.4" fill="{"#3ddc6a" if on else "#1b1d1f"}" stroke="#000" stroke-width=".5" {"filter=\"url(#lamp)\"" if on else ""}/>')
    # outer knob: skirted, fluted rubber grip
    pa = math.radians(135 + 270 * ri / max(n - 1, 1))
    a(f'<circle cx="{cx+3}" cy="{cy+4}" r="46" fill="#000" opacity=".55"/>')
    a(f'<circle cx="{cx}" cy="{cy}" r="46" fill="url(#rubber)" stroke="#000" stroke-width="1"/>')
    for k in range(30):
        ang = math.radians(k * 360 / 30)
        a(f'<circle cx="{cx+45*math.cos(ang):.1f}" cy="{cy+45*math.sin(ang):.1f}" r="3.4" fill="#070707"/>')
    a(f'<circle cx="{cx}" cy="{cy}" r="36" fill="url(#park)" stroke="#0a0a0a" stroke-width="1"/>')
    for r_ in (33, 29, 25):
        a(f'<circle cx="{cx}" cy="{cy}" r="{r_}" fill="none" stroke="#fff" stroke-opacity=".04"/>')
    # skirt pointer (raised wedge) + engraved index line
    a(f'<line x1="{cx+24*math.cos(pa):.1f}" y1="{cy+24*math.sin(pa):.1f}" x2="{cx+47*math.cos(pa):.1f}" y2="{cy+47*math.sin(pa):.1f}" stroke="#e8e6dc" stroke-width="3.2" stroke-linecap="round"/>')
    # inner knob: knurled steel (submodule), disabled look when no submodules
    has_sub = len(subs) > 0
    a(f'<circle cx="{cx+2}" cy="{cy+3}" r="17" fill="#000" opacity=".5"/>')
    a(f'<circle cx="{cx}" cy="{cy}" r="17" fill="url(#steel)" stroke="#111" stroke-width=".8"/>')
    a('<g stroke="#2a2c30" stroke-width=".9">' + ''.join(
        f'<line x1="{cx+16.5*math.cos(math.radians(k*10)):.1f}" y1="{cy+16.5*math.sin(math.radians(k*10)):.1f}" x2="{cx+13*math.cos(math.radians(k*10)):.1f}" y2="{cy+13*math.sin(math.radians(k*10)):.1f}"/>' for k in range(36)) + '</g>')
    a(f'<circle cx="{cx}" cy="{cy}" r="11" fill="#2c2e32" stroke="#111" stroke-width=".6"/>')
    if has_sub:
        sa = math.radians(-90 + 360 * si / len(subs))
        a(f'<line x1="{cx}" y1="{cy}" x2="{cx+10*math.cos(sa):.1f}" y2="{cy+10*math.sin(sa):.1f}" stroke="#ff3b3b" stroke-width="2.4" stroke-linecap="round"/>')
    # LCD drum: previous / SELECTED / next
    lx = x + 168
    etched(lx, y + h - 14, "OUTER REPO  INNER SUB", size=8, spacing=.5)
    etched(lx, y + 26, "REPO", size=9)
    for k, dy_ in ((-1, 34), (0, 56), (1, 78)):
        idx = (ri + k) % n
        txt = (">" if k == 0 else " ") + repos[idx]
        lcd(lx, y + dy_, txt, cols=14, p=1.3, pad=3)
    # submodule readout + one lamp per submodule
    etched(lx, y + 112, f"SUBMODULE {si+1} OF {len(subs)}" if has_sub else "SUBMODULE", size=9)
    lcd(lx, y + 118, (">" + subs[si]) if has_sub else " -- NONE --", cols=14, p=1.3, pad=3)
    for k in range(len(subs)):
        on = k == si
        a(f'<rect x="{lx+3+k*16}" y="{y+146}" width="11" height="6" rx="1" fill="{"#ffb43a" if on else "#1b1d1f"}" stroke="#000" stroke-width=".5" {"filter=\"url(#lamp)\"" if on else ""}/>')
    if has_sub:
        pass

rotary(1204, 64, 302, 192,
       ["ACTIONRUNNER-RS", "ACTIONRUNNER-WEB", "GITHUB.IO", "MODEL-RUNNER", "MUCONDE-AGENT", "MUCONDE-CTRL", "MUISOLATION", "MUOBSERVE", "MUREGIST"],
       3,
       ["K3-MANAGER", "K3-ENGINE", "K3-CORE", "K3-KV", "K3-ARCH-SYNTH", "K3-KERNELS-RVV", "K3-TOKENIZER"], 1)

# ---- Trainway panel (CI) ----
px, py, pw, ph = 94, 282, 898, 572
plate(px, py, pw, ph)
etched(px + 22, py + 28, "TRAINWAY", size=17, spacing=2, emboss=True)
lcd(px + 130, py + 12, "K3-ENGINE @ MODEL-RUNNER", p=1.6, pad=3)
etched(px + 396, py + 26, "TRAINS", size=9); lcd(px + 442, py + 12, "3", p=1.6, pad=3)
etched(px + 472, py + 26, "SIDINGS", size=9); lcd(px + 522, py + 12, "3", p=1.6, pad=3)
annunciator(px + 560, py + 12, 60, 22, "MAIN OK", "#3ddc6a")
annunciator(px + 628, py + 12, 60, 22, "STOP")
ind_lamp(px + 712, py + 20, "green", True, r=5); etched(px + 722, py + 24, "RDY", size=8, spacing=.5)
ind_lamp(px + 712, py + 44, "red", False, r=5); etched(px + 722, py + 48, "RUN", size=8, spacing=.5)
pistol_switch(px + 778, py + 33, [(-40, "NORM"), (40, "RST")], 0, size=50, spring=True, pull=True, name="PULL")
pistol_switch(px + 850, py + 33, [(-40, "HOLD"), (40, "LIVE")], 1, size=50, name="FEED")

sx, sy, sw, sh = px + 22, py + 64, pw - 44, ph - 88
crt_open(sx, sy, sw, sh, "G")
stations = ["PUSH", "FMT", "CLIPPY", "AUDIT", "COV", "BENCH", "RELEASE", "MERGE"]
X = [sx + 80 + k * 106 for k in range(len(stations))]
for k, s in enumerate(stations):
    t(X[k], sy + 32, s, GD if k else G, 16, "middle")
    a(f'<line x1="{X[k]}" y1="{sy+42}" x2="{X[k]}" y2="{sy+sh-60}" stroke="{GD}" stroke-dasharray="2 6" opacity=".6"/>')

def rail(path, lit=True, dashed=False, building=False):
    c = G if lit else GD
    extra = ' stroke-dasharray="10 6"' if dashed else ''
    a(f'<path d="{path}" fill="none" stroke="{c}" stroke-width="{2.4 if lit else 1.6}"{extra}>' +
      ('<animate attributeName="stroke-dashoffset" from="32" to="0" dur="0.8s" repeatCount="indefinite"/>' if building else '') + '</path>')

def sleepers(x0, x1, y, lit=True):
    for xx in range(int(x0), int(x1), 12):
        a(f'<line x1="{xx}" y1="{y-4}" x2="{xx}" y2="{y+4}" stroke="{G if lit else GD}" stroke-width="1" opacity="{.55 if lit else .4}"/>')

def signal(x, y, kind):
    if kind == "pass":
        a(f'<rect x="{x-4}" y="{y-4}" width="8" height="8" fill="{G}"/>')
    elif kind == "run":
        a(f'<rect x="{x-5}" y="{y-5}" width="10" height="10" fill="none" stroke="{G}" stroke-width="2"><animate attributeName="opacity" values="1;.2;1" dur="1s" repeatCount="indefinite"/></rect>')
    elif kind == "fail":
        a(f'<path d="M{x-5} {y-5} L{x+5} {y+5} M{x+5} {y-5} L{x-5} {y+5}" stroke="{G}" stroke-width="2.4"/>')
    else:
        a(f'<rect x="{x-3.5}" y="{y-3.5}" width="7" height="7" fill="none" stroke="{GD}" stroke-width="1.2"/>')

def train(x, y, label, node, state="run"):
    w = 66
    if state == "fail":
        a(f'<rect x="{x-w/2}" y="{y-9}" width="{w}" height="18" rx="2" fill="{G}"/>')
        t(x, y + 5, label, "#04140a", 16, "middle")
    else:
        a(f'<path d="M{x-w/2} {y-9} H{x+w/2-8} L{x+w/2} {y} L{x+w/2-8} {y+9} H{x-w/2} Z" fill="#04140a" stroke="{G}" stroke-width="2">' +
          ('<animate attributeName="stroke-opacity" values="1;.3;1" dur="1s" repeatCount="indefinite"/>' if state == "run" else '') + '</path>')
        t(x - 3, y + 5, label, G, 16, "middle")
    t(x, y + 26, ("\uf233 " + node) if node != "QUEUED" else node, GD, 13, "middle")

yM, y1, y2, y3 = sy + 225, sy + 115, sy + 325, sy + 400
# main line
rail(f"M{sx+24} {yM} H{sx+sw-24}")
sleepers(sx + 24, sx + sw - 24, yM)
t(sx + 12, yM - 14, "\ue0a0 main", G, 15)
for k in range(len(stations)):
    signal(X[k], yM - 16 if k else yM - 16, "pass")
train(X[6], yM, "#1231", "NODE-B", "done")
# siding 1: feat/oci-push (running at COV), PR pending merge
rail(f"M{X[0]+14} {yM} C{X[0]+50} {yM} {X[0]+40} {y1} {X[0]+80} {y1} H{X[4]}")
rail(f"M{X[4]} {y1} H{X[6]+30}", lit=False)
rail(f"M{X[6]+30} {y1} C{X[6]+70} {y1} {X[7]-40} {yM} {X[7]-6} {yM}", lit=False, dashed=True)
sleepers(X[0] + 80, X[4], y1)
t(sx + 12, y1 + 5, "\ue0a0 feat/rvv-gemm", G, 14)
for k, s in enumerate(["pass", "pass", "pass", "run", "idle", "idle"], start=1):
    signal(X[k], y1 - 16, s)
train(X[4], y1, "#1234", "NODE-A", "run")
# siding 2: feat/gc-sweep failed at CLIPPY
rail(f"M{X[0]+30} {yM} C{X[0]+60} {yM} {X[0]+50} {y2} {X[0]+90} {y2} H{X[2]}")
rail(f"M{X[2]} {y2} H{X[6]+30}", lit=False)
sleepers(X[0] + 90, X[2], y2)
t(sx + 12, y2 + 5, "\ue0a0 feat/kv-paging", G, 14)
for k, s in enumerate(["pass", "fail", "idle", "idle", "idle", "idle"], start=1):
    signal(X[k], y2 - 16, s)
train(X[2], y2, "#1233", "WORKER-RV64", "fail")
# siding 3: newly created branch, track under construction
rail(f"M{X[0]+40} {yM} C{X[0]+70} {yM} {X[0]+60} {y3} {X[0]+100} {y3} H{X[1]+40}", lit=False, dashed=True, building=True)
t(X[1] + 52, y3 + 5, "\ue0a0 feat/q4-tiles \u00b7 new siding under construction", GD, 14)
train(X[0], y3 + 1, "#1235", "QUEUED", "idle")
# legend
ly = sy + sh - 26
signal(sx + 30, ly - 5, "pass"); t(sx + 42, ly, "PASSED", GD, 15)
signal(sx + 120, ly - 5, "run"); t(sx + 132, ly, "RUNNING", GD, 15)
signal(sx + 218, ly - 5, "fail"); t(sx + 230, ly, "FAILED", GD, 15)
signal(sx + 306, ly - 5, "idle"); t(sx + 318, ly, "PENDING", GD, 15)
a(f'<line x1="{sx+400}" y1="{ly-5}" x2="{sx+430}" y2="{ly-5}" stroke="{GD}" stroke-width="1.6" stroke-dasharray="6 4"/>'); t(sx + 438, ly, "MERGE PENDING / NEW TRACK", GD, 15)
t(sx + sw - 24, ly, "FABRIC REV 482", GD, 15, "end")
crt_close(sx, sy, sw, sh)

# ---- Delivery panel (CD) ----
dx, dy, dw, dh = 1000, 282, 506, 572
plate(dx, dy, dw, dh)
etched(dx + 22, dy + 28, "DELIVERY", size=17, spacing=2, emboss=True)
lcd(dx + 128, dy + 12, "MODEL-RUNNER 7F3A B", p=1.6, pad=3)
annunciator(dx + 22, dy + 40, 66, 18, "PROMOTED")
annunciator(dx + 94, dy + 40, 66, 18, "STAGING", "#ffb43a")
annunciator(dx + 166, dy + 40, 66, 18, "ROLLBACK")
annunciator(dx + 238, dy + 40, 66, 18, "TAINT")
etched(dx + 330, dy + 54, "LEASE", size=9); lcd(dx + 368, dy + 40, "212S", p=1.6, pad=3)
sx, sy, sw, sh = dx + 22, dy + 74, dw - 44, dh - 98
crt_open(sx, sy, sw, sh, "A")
t(sx + 16, sy + 26, "\uf0e7 GATE  \uf233 node-a  slot B", A, 16)
t(sx + sw - 16, sy + 26, "4/7", A, 20, "end")
for k in range(7):
    on = k < 4
    a(f'<rect x="{sx+16+k*34}" y="{sy+36}" width="30" height="8" fill="{A if on else AD}" opacity="{1 if on else .5}"/>')
tests = [
    ("it_api_health", True, "pass", "0.3S"),
    ("it_tokenizer_roundtrip", True, "pass", "0.9S"),
    ("it_memlock_ceiling", False, "pass", "0.4S"),
    ("it_kv_cache_admission", False, "pass", "2.1S"),
    ("it_infer_smoke", True, "run", "--"),
    ("it_rvv_gemm_parity", True, "idle", ""),
    ("it_ime_matrix_probe", False, "idle", ""),
]
tx = sx + 34
a(f'<line x1="{tx}" y1="{sy+62}" x2="{tx}" y2="{sy+62+len(tests)*36}" stroke="{A}" stroke-width="2.4"/>')
for k, (name, crit, st, dur) in enumerate(tests):
    yy = sy + 78 + k * 36
    if st == "pass":
        a(f'<circle cx="{tx}" cy="{yy}" r="6" fill="{A}"/>')
    elif st == "run":
        a(f'<circle cx="{tx}" cy="{yy}" r="7" fill="#060402" stroke="{A}" stroke-width="2"><animate attributeName="opacity" values="1;.2;1" dur="1s" repeatCount="indefinite"/></circle>')
    else:
        a(f'<circle cx="{tx}" cy="{yy}" r="5" fill="#060402" stroke="{AD}" stroke-width="1.4"/>')
    c = A if st != "idle" else AD
    t(tx + 20, yy + 5, name, c, 15)
    if crit:
        t(sx + sw - 92, yy + 5, "[C]", c, 16)
    t(sx + sw - 16, yy + 5, dur if st != "run" else "RUN", c, 16, "end")
gy = sy + 62 + len(tests) * 36 + 22
a(f'<path d="M{tx} {gy-22} V{gy}" stroke="{A}" stroke-width="2.4"/>')
a(f'<path d="M{tx} {gy} C{tx} {gy+18} {tx+30} {gy+18} {tx+60} {gy+18} H{tx+150}" fill="none" stroke="{AD}" stroke-width="1.8" stroke-dasharray="6 4"/>')
a(f'<path d="M{tx} {gy} V{gy+44} H{tx+150}" fill="none" stroke="{AD}" stroke-width="1.8" stroke-dasharray="6 4"/>')
t(tx + 160, gy + 23, "\uf00c PROMOTE needs 4/4 [C]", AD, 14)
t(tx + 160, gy + 49, "\uf071 ROLLBACK on [C] fail", AD, 14)
hy = sy + sh - 22
t(sx + 16, hy, "HIST", AD, 15)
for k, (h_, ok) in enumerate((("6C21 7/7", True), ("5B10 5/7 RB", False), ("4A0F 7/7", True))):
    hx = sx + 70 + k * 120
    if ok:
        t(hx, hy, h_, A, 15)
    else:
        a(f'<rect x="{hx-3}" y="{hy-13}" width="{len(h_)*15*0.62*CRT_SCALE+8:.0f}" height="17" fill="{A}"/>'); t(hx, hy, h_, "#1a1004", 15)
crt_close(sx, sy, sw, sh)

a('</svg>')
open(OUT, "w").write('\n'.join(o))
print("wrote", OUT, sum(len(s) for s in o) // 1024, "KB")
