import json, re, sys
exec(open('compare.py').read().split("for c in classes:")[0])
FAST = {'0.07944023609161377', '0.2244943380355835', '0.6960656642913818'}
def norm(cs):
    out = set()
    for c in cs:
        try: v = abs(float(c))
        except ValueError: continue
        if c in FAST or v in (0.0, 1.0, 0.5) or v != v or v == float('inf'): continue
        out.add(float('%.6g' % v))
    return out
for c in classes:
    a, b = reach(v1, [c]), reach(g2, [c])
    ka = {strip_cls(f, c): f for f in a}; kb = {strip_cls(f, c): f for f in b}
    fast = any(FAST & set(v1[f]['consts']) for f in a)
    ga = set().union(*[norm(v1[f]['consts']) for f in a if 'InitializePorts' not in f]) if a else set()
    gb = set().union(*[norm(g2[f]['consts']) for f in b if 'InitializePorts' not in f]) if b else set()
    pa = set().union(*[set(v1[f]['strs']) for f in a]) if a else set()
    pb = set().union(*[set(g2[f]['strs']) for f in b]) if b else set()
    ia = set().union(*[norm(v1[f]['consts']) for f in a if 'InitializePorts' in f]) if a else set()
    ib = set().union(*[norm(g2[f]['consts']) for f in b if 'InitializePorts' in f]) if b else set()
    print(f"{c}: fastexp2={'v1' if fast else '-'} | dsp consts v1-only {sorted(ga-gb)} g2-only {sorted(gb-ga)} | port consts v1-only {sorted(ia-ib)} g2-only {sorted(ib-ia)} | strs v1-only {sorted(pa-pb)[:8]} g2-only {sorted(pb-pa)[:8]}")
