"""SPU ADPCM (VAG) block decoder used to analyse the game's music / sfx data."""
import struct
COEF = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)]

def decode_blocks(data, s1=0, s2=0):
    out = []
    for i in range(0, len(data) - 15, 16):
        shift = data[i] & 15
        filt = (data[i] >> 4) & 7
        flag = data[i + 1]
        c0, c1 = COEF[min(filt, 4)]
        for b in data[i + 2:i + 16]:
            for nib in (b & 15, b >> 4):
                if nib >= 8:
                    nib -= 16
                s = (nib << 12) >> shift
                s += (s1 * c0 + s2 * c1 + 32) >> 6
                s = max(-32768, min(32767, s))
                out.append(s)
                s2, s1 = s1, s
    return out
