"""Drive `rea mcp` over stdio without a client-side timeout.

The first Ghidra analysis of the engine outlasts the MCP client's request timeout, so this opens
the binary once in its own `rea mcp` process and runs queued jobs against that session.

Usage: rea_drive.py <binary> <outdir>
Drop job lists into <outdir>/queue/*.json, touch <outdir>/STOP to end.
Job list: [{"id": "name", "tool": "procedure_pseudo_code", "args": {"procedure": "0x23ce0"}}, ...]
Each result is written to <outdir>/<id>.json; progress goes to <outdir>/_progress.log.
"""
import glob, json, os, subprocess, sys, time

REA = ["npx", "-y", "rea-agents@6.7.0", "mcp"]
binary, outdir = sys.argv[1:3]
os.makedirs(outdir, exist_ok=True)
env = dict(os.environ,
           GHIDRA_INSTALL_DIR=os.path.expanduser("~/tools/ghidra_12.1.4_PUBLIC"),
           JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home",
           REA_GHIDRA_STARTUP_TIMEOUT_MS="3600000")
p = subprocess.Popen(REA, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                     stderr=open(os.path.join(outdir, "_stderr.log"), "w"), env=env, text=True)
nid = 0


def rpc(method, params, notify=False):
    global nid
    msg = {"jsonrpc": "2.0", "method": method, "params": params}
    if not notify:
        nid += 1
        msg["id"] = nid
    p.stdin.write(json.dumps(msg) + "\n")
    p.stdin.flush()
    if notify:
        return None
    while True:
        line = p.stdout.readline()
        if not line:
            raise SystemExit("rea exited")
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get("id") == nid:
            return r
        # server->client requests (roots/list etc.)
        if "method" in r and "id" in r:
            res = {"roots": []} if r["method"] == "roots/list" else {}
            p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": r["id"], "result": res}) + "\n")
            p.stdin.flush()


def log(s):
    with open(os.path.join(outdir, "_progress.log"), "a") as f:
        f.write(time.strftime("%H:%M:%S ") + s + "\n")


rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                   "clientInfo": {"name": "einst-rea-driver", "version": "1"}})
rpc("notifications/initialized", {}, notify=True)
r = rpc("tools/call", {"name": "open_binary", "arguments": {"path": binary, "provider_id": "ghidra"}})
log("open: " + json.dumps(r)[:300])
done = set()
while not os.path.exists(os.path.join(outdir, "STOP")):
    files = sorted(glob.glob(os.path.join(outdir, "queue", "*.json")))
    pending = [f for f in files if f not in done]
    if not pending:
        time.sleep(2)
        continue
    for jf in pending:
        done.add(jf)
        for job in json.load(open(jf)):
            out = os.path.join(outdir, job["id"] + ".json")
            if os.path.exists(out):
                continue
            t = time.time()
            r = rpc("tools/call", {"name": job["tool"], "arguments": job["args"]})
            json.dump(r, open(out, "w"))
            log("%s %.0fs" % (job["id"], time.time() - t))
        log("BATCH DONE " + os.path.basename(jf))
p.stdin.close()
