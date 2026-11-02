import sys, re, subprocess, os, urllib.request
SDK="https://raw.githubusercontent.com/phracker/MacOSX-SDKs/master/MacOSX11.3.sdk"
cache=sys.argv[1]; bins=sys.argv[2:]
os.makedirs(cache,exist_ok=True)
def tbd(install):
    p=install
    if p.endswith('.dylib'): p=p[:-6]
    fn=os.path.join(cache,p.strip('/').replace('/','_')+'.tbd')
    if not os.path.exists(fn):
        try: data=urllib.request.urlopen(SDK+p+'.tbd').read().decode()
        except Exception as e: data=''
        open(fn,'w').write(data)
    return open(fn).read()
syms={}
def load(install,seen):
    if install in seen: return set()
    seen.add(install); t=tbd(install)
    if not t: print("  !! no tbd for",install); return set()
    out=set()
    # parse documents; only blocks whose targets include x86_64
    for doc in t.split('--- !tapi-tbd')[1:]:
        for blk in re.split(r'\n\s*- targets:\s*', doc):
            tl=blk.split(']')[0]
            if 'x86_64-macos' not in tl and 'x86_64' not in tl: continue
            for key in ['symbols','objc-classes','weak-symbols','objc-eh-types','objc-ivars']:
                for m in re.finditer(key+r':\s*\[(.*?)\]',blk,re.S):
                    for s in re.split(r'[,\s]+',m.group(1)):
                        s=s.strip("'\"")
                        if not s: continue
                        if key=='objc-classes': out|={'_OBJC_CLASS_$_'+s,'_OBJC_METACLASS_$_'+s}
                        elif key=='objc-eh-types': out.add('_OBJC_EHTYPE_$_'+s)
                        elif key=='objc-ivars': out.add('_OBJC_IVAR_$_'+s)
                        else: out.add(s)
            for m in re.finditer(r'libraries:\s*\[(.*?)\]',blk,re.S):
                for l in re.split(r'[,\s]+',m.group(1)):
                    l=l.strip("'\"")
                    if l.startswith('/'): out|=load(l,seen)
    return out
for b in bins:
    print("==",b)
    libs=[l.split(' (')[0].strip() for l in subprocess.run(['llvm-objdump','--macho','--arch=x86_64','--dylibs-used',b],capture_output=True,text=True).stdout.splitlines()[1:]]
    nm=subprocess.run(['llvm-nm','--arch=x86_64','-u','-m',b],capture_output=True,text=True).stdout.splitlines()
    for lib in libs:
        if lib not in syms: syms[lib]=load(lib,set())
    allsyms=set().union(*syms.values())
    missing=[]
    for line in nm:
        m=re.search(r'external (\S+) \(from (\S+)\)',line)
        if not m: continue
        s=m.group(1)
        if s not in allsyms: missing.append((s,m.group(2)))
    print("  imports:",len(nm),"missing in 11.3 x86_64:",len(missing))
    for s,l in missing: print("   ",s,l)
