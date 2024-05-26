from fontTools.ttLib import TTFont

font = TTFont('src/fonts/Retro Gaming.ttf')
cmap = font['cmap']
t = cmap.getBestCmap()
s = font.getGlyphSet()

def width(c):
    if ord(c) in t and t[ord(c)] in s:
        return s[t[ord(c)]].width
    else:
        return s['.notdef'].width

print(width('A'))