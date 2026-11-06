"""Minimal CLAP / VST3 load check (no external validator installed).

Usage: python3 tools/plugincheck.py target/bundled/Einst.clap/Contents/MacOS/Einst target/bundled/Einst.vst3/Contents/MacOS/Einst
Loads the CLAP entry, reads the descriptor, creates, inits and activates the plugin; reads
the VST3 factory's first class.
"""
import ctypes as C, sys

clap_path, vst3_path = sys.argv[1], sys.argv[2]

class Ver(C.Structure):
    _fields_ = [("major", C.c_uint32), ("minor", C.c_uint32), ("rev", C.c_uint32)]

class Entry(C.Structure):
    _fields_ = [("ver", Ver), ("init", C.CFUNCTYPE(C.c_bool, C.c_char_p)), ("deinit", C.CFUNCTYPE(None)), ("get_factory", C.CFUNCTYPE(C.c_void_p, C.c_char_p))]

class Desc(C.Structure):
    _fields_ = [("ver", Ver), ("id", C.c_char_p), ("name", C.c_char_p), ("vendor", C.c_char_p), ("url", C.c_char_p), ("manual_url", C.c_char_p),
                ("support_url", C.c_char_p), ("version", C.c_char_p), ("description", C.c_char_p), ("features", C.POINTER(C.c_char_p))]

class Factory(C.Structure):
    pass

class Host(C.Structure):
    _fields_ = [("ver", Ver), ("host_data", C.c_void_p), ("name", C.c_char_p), ("vendor", C.c_char_p), ("url", C.c_char_p), ("version", C.c_char_p),
                ("get_extension", C.CFUNCTYPE(C.c_void_p, C.c_void_p, C.c_char_p)), ("request_restart", C.CFUNCTYPE(None, C.c_void_p)),
                ("request_process", C.CFUNCTYPE(None, C.c_void_p)), ("request_callback", C.CFUNCTYPE(None, C.c_void_p))]

class Plugin(C.Structure):
    _fields_ = [("desc", C.POINTER(Desc)), ("plugin_data", C.c_void_p), ("init", C.CFUNCTYPE(C.c_bool, C.c_void_p)), ("destroy", C.CFUNCTYPE(None, C.c_void_p)),
                ("activate", C.CFUNCTYPE(C.c_bool, C.c_void_p, C.c_double, C.c_uint32, C.c_uint32)), ("deactivate", C.CFUNCTYPE(None, C.c_void_p))]

Factory._fields_ = [("count", C.CFUNCTYPE(C.c_uint32, C.POINTER(Factory))), ("desc", C.CFUNCTYPE(C.POINTER(Desc), C.POINTER(Factory), C.c_uint32)),
                    ("create", C.CFUNCTYPE(C.POINTER(Plugin), C.POINTER(Factory), C.POINTER(Host), C.c_char_p))]

lib = C.CDLL(clap_path)
entry = Entry.in_dll(lib, "clap_entry")
print("clap version", entry.ver.major, entry.ver.minor, entry.ver.rev)
assert entry.init(clap_path.encode())
f = C.cast(entry.get_factory(b"clap.plugin-factory"), C.POINTER(Factory))
n = f.contents.count(f)
d = f.contents.desc(f, 0).contents
feats = []
i = 0
while d.features[i]:
    feats.append(d.features[i].decode()); i += 1
print("plugins", n, "id", d.id, "name", d.name, "vendor", d.vendor, "version", d.version, "features", feats)
no_ext = C.CFUNCTYPE(C.c_void_p, C.c_void_p, C.c_char_p)(lambda h, s: None)
noop = C.CFUNCTYPE(None, C.c_void_p)(lambda h: None)
host = Host(Ver(1, 2, 0), None, b"check", b"einst", b"", b"0", no_ext, noop, noop, noop)
p = f.contents.create(f, C.byref(host), d.id)
assert p, "create failed"
pl = p.contents
print("plugin init", pl.init(p), "activate 48k/512", pl.activate(p, 48000.0, 1, 512))
pl.deactivate(p)
pl.destroy(p)
entry.deinit()

v = C.CDLL(vst3_path)
v.GetPluginFactory.restype = C.c_void_p
fac = v.GetPluginFactory()
# IPluginFactory vtable: queryInterface, addRef, release, getFactoryInfo, countClasses, getClassInfo, createInstance
vt = C.cast(C.cast(fac, C.POINTER(C.c_void_p))[0], C.POINTER(C.c_void_p))
count = C.CFUNCTYPE(C.c_int32, C.c_void_p)(vt[4])(fac)

class PClassInfo(C.Structure):
    _fields_ = [("cid", C.c_uint8 * 16), ("cardinality", C.c_int32), ("category", C.c_char * 32), ("name", C.c_char * 64)]

info = PClassInfo()
C.CFUNCTYPE(C.c_int32, C.c_void_p, C.c_int32, C.POINTER(PClassInfo))(vt[5])(fac, 0, C.byref(info))
print("vst3 classes", count, "first", bytes(info.cid), info.category, info.name)
