"""Compiler-independent fingerprints of every function in a Mach-O (x86_64); used to compare v1's effect classes with Geist2's (docs/re/effects-v1.md).

Per function (demangled name): the float/double constants it loads (RIP-relative
loads, by value), the C strings it references, the named functions it calls, the
immediate integers it uses, and its size. Writes JSON {name: {...}}.

Usage: fingerprint.py <binary> <out.json> [name-regex]
"""
import bisect, json, re, struct, subprocess, sys

binary, out_path = sys.argv[1], sys.argv[2]
pat = re.compile(sys.argv[3]) if len(sys.argv) > 3 else None
data = open(binary, 'rb').read()

# Sections (vmaddr -> file offset), for reading constants.
sects = []
cur = {}
for l in subprocess.run(['otool', '-l', binary], capture_output=True, text=True).stdout.splitlines():
    t = l.split()
    if len(t) >= 2 and t[0] in ('sectname', 'addr', 'size', 'offset'):
        cur[t[0]] = t[1]
    if len(t) >= 2 and t[0] == 'offset' and 'addr' in cur:
        sects.append((int(cur['addr'], 16), int(cur['size'], 16), int(cur['offset']), cur.get('sectname', '')))


def off(a):
    for s, z, o, n in sects:
        if s <= a < s + z:
            return o + a - s, n
    return None, None


syms = []
for l in subprocess.run(['nm', '-n', binary], capture_output=True, text=True).stdout.splitlines():
    p = l.split()
    if len(p) == 3 and p[1] in 'tT':
        syms.append((int(p[0], 16), p[2]))
names = [s for _, s in syms]
dem = dict(zip(names, subprocess.run(['c++filt'], input='\n'.join(names), capture_output=True, text=True).stdout.splitlines()))
addrs = [a for a, _ in syms]
by_addr = {a: dem[s] for a, s in syms}

result = {}
dem_full = {s: dem[s] for s in names}
want = [(a, dem[s]) for a, s in syms if not pat or pat.search(dem[s])]
want_addrs = {a for a, _ in want}
asm = subprocess.run(['objdump', '-d', '--no-show-raw-insn', binary], capture_output=True, text=True).stdout
cur = None
for line in asm.splitlines():
    m = re.match(r'\s*([0-9a-f]+):\s+(\S+)\s*(.*)', line)
    if not m:
        continue
    a = int(m.group(1), 16)
    k = bisect.bisect_right(addrs, a) - 1
    if k < 0 or addrs[k] not in want_addrs:
        continue
    name = by_addr[addrs[k]]
    r = result.setdefault(name, {'consts': [], 'strs': [], 'calls': [], 'imms': [], 'n': 0})
    r['n'] += 1
    mn, ops = m.group(2), m.group(3)
    c = re.search(r'## 0x([0-9a-f]+)', ops)
    if c:
        x = int(c.group(1), 16)
        o, sect = off(x)
        if o is None or mn.startswith('lea') and sect != '__cstring':
            pass
        elif sect == '__cstring':
            e = data.find(b'\0', o)
            r['strs'].append(data[o:e].decode('latin1'))
        elif mn.endswith('sd') or mn.endswith('pd'):
            r['consts'].append(repr(struct.unpack('<d', data[o:o + 8])[0]))
        elif mn.endswith('ss') or mn.endswith('ps'):
            r['consts'].append(repr(struct.unpack('<f', data[o:o + 4])[0]))
    if mn.startswith('call'):
        t = re.search(r'<([^>+]+)', ops)
        if t:
            tn = t.group(1)
            r['calls'].append('stub' if 'dyld_stub' in tn else dem_full.get(tn, tn))
    for im in re.findall(r'\$(0x[0-9a-f]+|-?\d+)', ops):
        v = int(im, 0)
        if abs(v) > 1 and v != 0xffffffff:
            r['imms'].append(v)
for r in result.values():
    for k in ('consts', 'strs', 'calls', 'imms'):
        r[k].sort()
json.dump(result, open(out_path, 'w'))
print(len(result))
