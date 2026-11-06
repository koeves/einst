# For each function matching a pattern, list C-string literals referenced (in order) via RIP-relative LEA,
# plus direct calls (demangled). Static analysis only.
import sys, re, warnings, lief, capstone, subprocess
warnings.filterwarnings("ignore")
binpath, pat = sys.argv[1], sys.argv[2]
fat = lief.MachO.parse(binpath)
b = [x for x in fat if x.header.cpu_type == lief.MachO.Header.CPU_TYPE.X86_64][0]
text = b.get_section("__text")
code = bytes(text.content); base = text.virtual_address
secs = [(s.virtual_address, s.virtual_address+s.size, s) for s in b.sections]
def read_cstr(addr):
    for lo,hi,s in secs:
        if lo<=addr<hi and s.name in ("__cstring","__const","__data","__cfstring"):
            data=bytes(s.content); off=addr-lo
            if s.name=="__cfstring":
                # CFString struct: isa, flags, ptr, len
                import struct
                ptr=struct.unpack_from("<Q",data,off+16)[0]; return "@"+read_cstr(ptr) if ptr else None
            end=data.find(b"\0",off)
            raw=data[off:end]
            if 1<=len(raw)<200 and all(32<=c<127 for c in raw): return raw.decode()
            return None
    return None
syms = sorted([(s.value, s.name) for s in b.symbols if s.value and s.type is not None], key=lambda x:x[0])
addr2name = {a:n for a,n in syms}
starts=[a for a,_ in syms if base<=a<base+len(code)]
starts=sorted(set(starts))
import bisect
def demangle(n):
    try: return subprocess.run(["llvm-cxxfilt",n[1:] if n.startswith("__Z") else n],capture_output=True,text=True).stdout.strip()
    except: return n
md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_64)
rx=re.compile(pat)
targets=[(a,n) for a,n in syms if base<=a<base+len(code) and rx.search(n)]
stubs={}
for a,n in targets:
    i=bisect.bisect_right(starts,a); end=starts[i] if i<len(starts) else base+len(code)
    print("\n###", demangle(n))
    out=[]
    for ins in md.disasm(code[a-base:end-base], a):
        if ins.mnemonic=="lea" and "rip" in ins.op_str:
            m=re.search(r"\[rip \+ (0x[0-9a-f]+)\]",ins.op_str)
            if m:
                t=ins.address+ins.size+int(m.group(1),16); s=read_cstr(t)
                if s: out.append('"'+s+'"')
        elif ins.mnemonic=="call":
            try:
                t=int(ins.op_str,16); nm=addr2name.get(t)
                if nm and ("VXMLFile" in nm or "Serialize" in nm or "XML" in nm): d=demangle(nm); out.append("call "+d.replace("VFoundation::",""))
            except ValueError: pass
    print("  "+"\n  ".join(out))
