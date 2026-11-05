// Editor test host (macOS): loads CLAP / VST3 bundles into one process and
// opens their editors, as a DAW does with two instruments. Built and run by
// tools/editorhost.sh; usage: editorhost <bundle> <bundle> ...
// Sequence: for each bundle in turn, create -> open editor -> pump -> close ->
// destroy (the "replace instrument" flow), then all at once (concurrent).
// Runs in background mode: a prohibited (never active) app whose windows are
// off screen, ordered to the back and never key; DREIST_BACKGROUND_GUI=1 is
// set for the plugins too. Exits non-zero (or aborts) when an editor fails.
#import <Cocoa/Cocoa.h>
#include <dlfcn.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

// ---- CLAP (minimal) ----
typedef struct { uint32_t major, minor, revision; } clap_version;
typedef struct clap_host clap_host;
struct clap_host { clap_version v; void *host_data; const char *name, *vendor, *url, *version;
  const void *(*get_extension)(const clap_host *, const char *); void (*request_restart)(const clap_host *);
  void (*request_process)(const clap_host *); void (*request_callback)(const clap_host *); };
typedef struct { clap_version v; const char *id, *name, *vendor, *url, *manual_url, *support_url, *version, *description; const char *const *features; } clap_desc;
typedef struct clap_plugin clap_plugin;
struct clap_plugin { const clap_desc *desc; void *data; bool (*init)(const clap_plugin *); void (*destroy)(const clap_plugin *);
  void *activate, *deactivate, *start, *stop, *reset, *process;
  const void *(*get_extension)(const clap_plugin *, const char *); void (*on_main_thread)(const clap_plugin *); };
typedef struct clap_factory clap_factory;
struct clap_factory { uint32_t (*count)(const clap_factory *); const clap_desc *(*desc)(const clap_factory *, uint32_t);
  const clap_plugin *(*create)(const clap_factory *, const clap_host *, const char *); };
typedef struct { clap_version v; bool (*init)(const char *); void (*deinit)(void); const void *(*get_factory)(const char *); } clap_entry_t;
typedef struct { const char *api; void *cocoa; } clap_window;
typedef struct { bool (*is_api_supported)(const clap_plugin *, const char *, bool); void *pref; bool (*create)(const clap_plugin *, const char *, bool);
  void (*destroy)(const clap_plugin *); bool (*set_scale)(const clap_plugin *, double); bool (*get_size)(const clap_plugin *, uint32_t *, uint32_t *);
  void *can_resize, *hints, *adjust, *set_size; bool (*set_parent)(const clap_plugin *, const clap_window *); void *transient, *title;
  bool (*show)(const clap_plugin *); bool (*hide)(const clap_plugin *); } clap_gui;

static const void *h_ext(const clap_host *h, const char *id) { return NULL; }
static void h_noop(const clap_host *h) {}
static clap_host HOST = { {1, 2, 0}, NULL, "harness", "harness", "", "1", h_ext, h_noop, h_noop, h_noop };

// ---- VST3 (minimal vtables) ----
typedef int32_t tresult;
typedef struct { char cid[16]; int32_t card; char category[32]; char name[64]; } PClassInfo;
typedef struct { void *qi, *addref; uint32_t (*release)(void *); void *info; int32_t (*count)(void *); tresult (*classinfo)(void *, int32_t, PClassInfo *);
  tresult (*create)(void *, const char *, const char *, void **); } IPluginFactoryV;
typedef struct { void *qi, *addref; uint32_t (*release)(void *); tresult (*initialize)(void *, void *); tresult (*terminate)(void *);
  void *s5, *s6, *s7, *s8, *s9, *s10, *s11, *s12, *s13, *s14, *s15, *s16; void *(*create_view)(void *, const char *); } IEditControllerV;
typedef struct { int32_t l, t, r, b; } ViewRect;
typedef struct { void *qi, *addref; uint32_t (*release)(void *); tresult (*supported)(void *, const char *); tresult (*attached)(void *, void *, const char *);
  tresult (*removed)(void *); void *wheel, *kd, *ku; tresult (*get_size)(void *, ViewRect *); } IPlugViewV;
static const char IID_IEditController[16] = {0xDC,0xD7,0xBB,0xE3,0x77,0x42,0x44,0x8D,0xA8,0x74,0xAA,0xCC,0x97,0x9C,0x75,0x9E};
#define VT(o, T) ((T *)*(void **)(o))

static bool failed = false;

typedef struct {
  const char *path; bool vst3; NSWindow *win;
  const clap_plugin *cp; const clap_gui *gui;   // CLAP
  void *ctrl, *view;                             // VST3
} Inst;

static void pump(double s) {
  NSDate *until = [NSDate dateWithTimeIntervalSinceNow:s];
  while ([until timeIntervalSinceNow] > 0) {
    NSEvent *e = [NSApp nextEventMatchingMask:NSEventMaskAny untilDate:[NSDate dateWithTimeIntervalSinceNow:0.02] inMode:NSDefaultRunLoopMode dequeue:YES];
    if (e) [NSApp sendEvent:e];
  }
}

static void *load(const char *bundle, bool vst3) {
  NSString *b = [NSString stringWithUTF8String:bundle];
  NSString *exe = [[b stringByAppendingPathComponent:@"Contents/MacOS"] stringByAppendingPathComponent:[[b lastPathComponent] stringByDeletingPathExtension]];
  void *h = dlopen(exe.UTF8String, RTLD_NOW | RTLD_LOCAL);
  if (!h) { fprintf(stderr, "dlopen %s: %s\n", exe.UTF8String, dlerror()); exit(2); }
  if (vst3) {
    bool (*be)(CFBundleRef) = dlsym(h, "bundleEntry");
    CFBundleRef ref = CFBundleCreate(NULL, (__bridge CFURLRef)[NSURL fileURLWithPath:b]);
    if (be) be(ref);
  }
  return h;
}

static NSWindow *window(uint32_t w, uint32_t h, const char *title) {
  // Borderless, offscreen, behind everything, never key: the harness is a
  // prohibited (never-activating) app.
  NSWindow *win = [[NSWindow alloc] initWithContentRect:NSMakeRect(-20000, -20000, w, h) styleMask:NSWindowStyleMaskBorderless backing:NSBackingStoreBuffered defer:NO];
  win.releasedWhenClosed = NO;
  [win orderBack:nil];
  return win;
}

static void open_inst(Inst *in) {
  printf("open  %s\n", in->path); fflush(stdout);
  void *h = load(in->path, in->vst3);
  if (!in->vst3) {
    const clap_entry_t *e = dlsym(h, "clap_entry");
    e->init(in->path);
    const clap_factory *f = e->get_factory("clap.plugin-factory");
    const clap_desc *d = f->desc(f, 0);
    in->cp = f->create(f, &HOST, d->id);
    if (!in->cp || !in->cp->init(in->cp)) { fprintf(stderr, "create failed\n"); exit(3); }
    in->gui = in->cp->get_extension(in->cp, "clap.gui");
    if (!in->gui->create(in->cp, "cocoa", false)) { printf("  gui.create returned false\n"); in->gui = NULL; failed = true; return; }
    uint32_t w = 800, hh = 600; in->gui->get_size(in->cp, &w, &hh);
    in->win = window(w, hh, d->name);
    clap_window cw = { "cocoa", (__bridge void *)in->win.contentView };
    bool ok = in->gui->set_parent(in->cp, &cw);
    printf("  set_parent -> %d\n", ok); fflush(stdout);
    if (!ok) failed = true;
    in->gui->show(in->cp);
  } else {
    void *(*gpf)(void) = dlsym(h, "GetPluginFactory");
    void *f = gpf();
    PClassInfo ci; int n = VT(f, IPluginFactoryV)->count(f);
    for (int i = 0; i < n; i++) { VT(f, IPluginFactoryV)->classinfo(f, i, &ci); if (!strcmp(ci.category, "Audio Module Class")) break; }
    tresult r = VT(f, IPluginFactoryV)->create(f, ci.cid, IID_IEditController, &in->ctrl);
    if (r != 0 || !in->ctrl) { fprintf(stderr, "createInstance failed %d\n", r); exit(3); }
    VT(in->ctrl, IEditControllerV)->initialize(in->ctrl, NULL);
    in->view = VT(in->ctrl, IEditControllerV)->create_view(in->ctrl, "editor");
    ViewRect vr = {0, 0, 800, 600}; VT(in->view, IPlugViewV)->get_size(in->view, &vr);
    in->win = window(vr.r - vr.l, vr.b - vr.t, ci.name);
    tresult a = VT(in->view, IPlugViewV)->attached(in->view, (__bridge void *)in->win.contentView, "NSView");
    printf("  attached -> %d\n", a); fflush(stdout);
    if (a != 0) failed = true;
  }
  pump(0.7);
}

static void close_inst(Inst *in) {
  printf("close %s\n", in->path); fflush(stdout);
  if (!in->vst3) {
    if (in->gui) { in->gui->hide(in->cp); in->gui->destroy(in->cp); }
    in->cp->destroy(in->cp);
  } else {
    VT(in->view, IPlugViewV)->removed(in->view);
    VT(in->view, IPlugViewV)->release(in->view);
    VT(in->ctrl, IEditControllerV)->terminate(in->ctrl);
    VT(in->ctrl, IEditControllerV)->release(in->ctrl);
  }
  [in->win close]; in->win = nil;
  pump(0.3);
}

int main(int argc, char **argv) {
  setenv("DREIST_BACKGROUND_GUI", "1", 1);
  @autoreleasepool {
    [NSApplication sharedApplication];
    [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
    [NSApp finishLaunching];
    int n = argc - 1 > 8 ? 8 : argc - 1; Inst in[8] = {0};
    if (n == 0) { NSWindow *w = window(200, 100, "smoke"); pump(1.0); [w close]; printf("smoke OK\n"); return 0; }
    for (int i = 0; i < n; i++) { in[i].path = argv[i + 1]; in[i].vst3 = strstr(argv[i + 1], ".vst3") != NULL; }
    printf("== replace flow\n");
    for (int i = 0; i < n; i++) { open_inst(&in[i]); close_inst(&in[i]); }
    printf("== concurrent\n");
    for (int i = 0; i < n; i++) open_inst(&in[i]);
    for (int i = n - 1; i >= 0; i--) close_inst(&in[i]);
    printf(failed ? "FAILED\n" : "OK\n");
  }
  return failed ? 1 : 0;
}
