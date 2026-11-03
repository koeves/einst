"""Compare v1 and Geist2 fingerprints per effect class (and the DSP helpers it reaches)."""
import json, re, sys

v1 = json.load(open('v1.json'))
g2 = json.load(open('g2.json'))
classes = sys.argv[1:]
NOISE = {'nan', '-0.0', '0.0', '1.0', '-1.0', '0.5', 'inf', '-inf'}
SKIP = re.compile(r'InitializeClass|UninitializeClass|TheClass|::New\(\)|Copy\(\) const|~|GetDescriptorString|SerializeAsXML|operator=|ms_theClass|GetTypeName|GetTypeDescription|ConvertToText|ConvertFromText')


def cls_of(name):
    m = re.match(r'(?:VDSP::)?(\w+)', name)
    return m.group(1) if m else name


def reach(db, roots):
    """Functions of the root classes plus DSP helpers they call, transitively."""
    seen, todo = set(), [f for f in db if any(f.startswith(f'VDSP::{c}::') or f.startswith(f'{c}::') for c in roots)]
    while todo:
        f = todo.pop()
        if f in seen or f not in db:
            continue
        seen.add(f)
        for c in db[f]['calls']:
            if c in db and c not in seen and re.match(r'(VDSP::|TDImpl|Breverb|BRV)', c) and not re.search(r'VDSPNode::|VDSPPort|VDSPVoice::|VFoundation', c):
                todo.append(c)
    return {f for f in seen if not SKIP.search(f)}


def strip_cls(f, c):
    return f.replace(f'VDSP::{c}', '@').replace(c, '@')


for c in classes:
    a, b = reach(v1, [c]), reach(g2, [c])
    print(f'### {c}: v1 {len(a)} functions, Geist2 {len(b)}')
    ka = {strip_cls(f, c): f for f in a}
    kb = {strip_cls(f, c): f for f in b}
    only_a = sorted(set(ka) - set(kb))
    only_b = sorted(set(kb) - set(ka))
    if only_a:
        print('  v1 only:', only_a)
    if only_b:
        print('  Geist2 only:', only_b)
    for k in sorted(set(ka) & set(kb)):
        x, y = v1[ka[k]], g2[kb[k]]
        ca, cb = set(x['consts']) - NOISE, set(y['consts']) - NOISE
        sa, sb = set(x['strs']), set(y['strs'])
        la = {cl for cl in x['calls'] if cl != 'stub'}
        lb = {cl for cl in y['calls'] if cl != 'stub'}
        notes = []
        if ca != cb:
            notes.append(f'consts v1-only {sorted(ca - cb)} g2-only {sorted(cb - ca)}')
        if sa != sb:
            notes.append(f'strs v1-only {sorted(sa - sb)} g2-only {sorted(sb - sa)}')
        da = {re.sub(r'\(.*', '', cl) for cl in la} - {re.sub(r'\(.*', '', cl) for cl in lb}
        db_ = {re.sub(r'\(.*', '', cl) for cl in lb} - {re.sub(r'\(.*', '', cl) for cl in la}
        if da or db_:
            notes.append(f'calls v1-only {sorted(da)} g2-only {sorted(db_)}')
        if notes:
            print(f'  {k} [v1 {x["n"]} insns, g2 {y["n"]}]')
            for n in notes:
                print('     ', n[:600])
