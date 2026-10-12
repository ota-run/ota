#!/usr/bin/python3
"""Private operator-owned build custody. SOURCE ONLY; no installed qualification.

This is not an Ota command, installer, or generic execution service. An operator
must separately admit the installed platform, request, unit, caches and budgets.
No record produced here authorizes execution of the retained binary.
"""

import ctypes
import errno
import hashlib
import json
import os
import platform
import re
import resource
import selectors
import signal
import stat
import subprocess
import sys
import time
import zlib
from contextlib import ExitStack
from pathlib import PurePosixPath


CONTROLLER = "/opt/ota-bootstrap/controller.py"
PYTHON = "/usr/bin/python3"
BWRAP = "/usr/bin/bwrap"
SYSTEMCTL = "/usr/bin/systemctl"
REQUESTS = "/etc/ota-bootstrap/requests"
INVOCATIONS = "/var/lib/ota-bootstrap/invocations"
RETAINED = "/var/lib/ota-bootstrap/retained"
SCRATCH = "/run/ota-bootstrap-work"
CGROUP_ROOT = "/sys/fs/cgroup"
NONCE = re.compile(r"[0-9a-f]{32}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
GENERATION = re.compile(r"[0-9a-f]{32}\Z")
MAX_METADATA = 16 * 1024 * 1024
MAX_LOG = 1024 * 1024
MAX_ARTIFACT = 512 * 1024 * 1024
MAX_ENTRIES = 100_000
MAX_SOURCE = 32 * 1024**3
MAX_INVENTORY_METADATA = 14 * 1024**2
MAX_REQUEST_BYTES = 512 * 1024
MAX_GIT_OBJECT = 8 * 1024**2
MAX_GIT_DECOMPRESSED = 512 * 1024**2
BUILD_SECONDS = 960
CLEANUP_SECONDS = 120
POSTFLIGHT_SECONDS = 120
STATES = (
    "armed", "child_stopped", "placed", "building", "worker_terminal",
    "artifact_pending", "finalization_candidate",
)
NOT_PROVED = [
    "all_descendant_compiler_and_config_enforcement",
    "source_to_binary_semantic_correctness",
    "reproducible_compilation", "runtime_attestation", "provider_authority",
    "operator_postflight_unit_termination_and_resource_envelope_closure",
    "subsequent_driver_execution_admission",
    "independent_git_delta_reapplication",
]
LIBC = ctypes.CDLL(None, use_errno=True)
LIBC.syscall.restype = ctypes.c_long


class Refusal(Exception):
    pass


class LogRefusal(Refusal):
    def __init__(self, disposition):
        super().__init__("worker log incomplete/overflow")
        self.disposition = disposition


def require(condition, message):
    if not condition:
        raise Refusal(message)


def exact(value, keys):
    require(type(value) is dict and set(value) == set(keys), "unknown/missing fields")
    return value


def canonical(value):
    def visit(item):
        if item is None or type(item) is bool:
            return
        if type(item) is int:
            require(abs(item) <= 2**53 - 1, "integer outside canonical subset")
            return
        if type(item) is str:
            require(item.isascii() and all(32 <= ord(c) <= 126 for c in item),
                    "noncanonical string")
            return
        if type(item) is list:
            for child in item:
                visit(child)
            return
        require(type(item) is dict, "unsupported canonical value")
        for key, child in item.items():
            require(type(key) is str, "non-string key")
            visit(key)
            visit(child)
    visit(value)
    data = json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=True, allow_nan=False).encode("ascii")
    require(len(data) <= MAX_METADATA, "metadata capacity exceeded")
    return data


def decode(data):
    require(len(data) <= MAX_METADATA, "metadata capacity exceeded")
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON field")
            result[key] = value
        return result
    try:
        value = json.loads(data.decode("ascii"), object_pairs_hook=pairs,
                           parse_constant=lambda _: (_ for _ in ()).throw(Refusal("constant")))
    except (ValueError, UnicodeError) as error:
        raise Refusal("invalid JSON") from error
    require(canonical(value) == data, "noncanonical JSON bytes")
    return value


def digest(data, domain=b"ota.retained-driver.private.v1\0"):
    return hashlib.sha256(domain + data).hexdigest()


def nonce_from_path(path):
    prefix = REQUESTS + "/"
    require(type(path) is str and path.startswith(prefix) and path.endswith(".json"),
            "fixed request path required")
    nonce = path[len(prefix):-5]
    require(NONCE.fullmatch(nonce) is not None and int(nonce, 16) != 0, "invalid nonce")
    require(path == f"{REQUESTS}/{nonce}.json", "request path substitution")
    return nonce


def syscall(number, *args):
    result = LIBC.syscall(ctypes.c_long(number), *args)
    if result < 0:
        code = ctypes.get_errno()
        raise OSError(code, os.strerror(code))
    return result


class OpenHow(ctypes.Structure):
    _fields_ = [("flags", ctypes.c_uint64), ("mode", ctypes.c_uint64),
                ("resolve", ctypes.c_uint64)]


def beneath(fd, path, flags=os.O_RDONLY, mode=0):
    require(type(path) is str and path and not path.startswith("/")
            and all(p not in ("", ".", "..") for p in path.split("/")), "relative path")
    how = OpenHow(flags | os.O_CLOEXEC | os.O_NOFOLLOW, mode, 0x01 | 0x02 | 0x04 | 0x08)
    return syscall(437, ctypes.c_int(fd), ctypes.c_char_p(path.encode()),
                   ctypes.byref(how), ctypes.c_size_t(ctypes.sizeof(how)))


def object_id(fd):
    info = os.fstat(fd)
    return {"dev": info.st_dev, "ino": info.st_ino}


def stable_metadata(info):
    # Reading may update atime; content, ownership and mutation timestamps may not.
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def protected(info, kind, mode=None, uid=0, gid=0):
    require(kind(info.st_mode) and info.st_uid == uid and info.st_gid == gid
            and not info.st_mode & 0o022, "unprotected object")
    require(not stat.S_ISREG(info.st_mode) or info.st_nlink == 1, "linked regular file")
    if mode is not None:
        require(stat.S_IMODE(info.st_mode) == mode, "incorrect exact mode")


def directory(path):
    absolute_path(path)
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
    try:
        protected(os.fstat(fd), stat.S_ISDIR)
        for part in path.strip("/").split("/") if path != "/" else []:
            nxt = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
                          dir_fd=fd)
            protected(os.fstat(nxt), stat.S_ISDIR)
            os.close(fd)
            fd = nxt
        return fd
    except BaseException:
        os.close(fd)
        raise


def absolute_path(path, root=None):
    require(type(path) is str and path.startswith("/") and path.isascii()
            and all(32 <= ord(c) <= 126 for c in path) and len(path) <= 4096,
            "noncanonical absolute path")
    require(path == "/" or all(part not in ("", ".", "..") for part in path[1:].split("/")),
            "dot/parent/empty path component")
    if root is not None:
        require(path.startswith(root + "/"), "outside admitted root")
    return path


def validate_symlink_graph(entries):
    rows = {row["path"]: row for row in entries}
    for row in entries:
        if row["kind"] != "symlink":
            continue
        parts = list(PurePosixPath(row["path"]).parent.parts)
        todo = row["target"].split("/")
        hops = 1
        while todo:
            component, *todo = todo
            if component in ("", "."):
                continue
            if component == "..":
                require(parts, "transitive symlink escape")
                parts.pop()
                continue
            candidate = "/".join(parts + [component])
            require(candidate in rows, "unadmitted/dangling symlink referent")
            target = rows[candidate]
            if target["kind"] == "symlink":
                hops += 1
                require(hops <= 40 and not target["target"].startswith("/"), "symlink cycle/absolute target")
                todo = target["target"].split("/") + todo
            else:
                require(not todo or target["kind"] == "directory", "non-directory link traversal")
                parts.append(component)


def read_fd(fd, maximum=MAX_METADATA):
    chunks = []
    total = 0
    while True:
        data = os.read(fd, min(65536, maximum + 1 - total))
        if not data:
            return b"".join(chunks)
        total += len(data)
        require(total <= maximum, "read capacity exceeded")
        chunks.append(data)


def read_record(parent, name, mode=0o444):
    fd = beneath(parent, name)
    try:
        before = os.fstat(fd)
        protected(before, stat.S_ISREG, mode)
        data = read_fd(fd)
        after = os.fstat(fd)
        require(stable_metadata(before) == stable_metadata(after)
                and len(data) == before.st_size, "changing record")
        return decode(data)
    finally:
        os.close(fd)


def publish(parent, name, data):
    require(len(data) <= MAX_METADATA or name == "driver.pending", "publication capacity")
    temporary = "." + name + ".tmp"
    fd = beneath(parent, temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        protected(os.fstat(fd), stat.S_ISREG, 0o600)
        view = memoryview(data)
        while view:
            written = os.write(fd, view)
            require(written > 0, "short publication write")
            view = view[written:]
        os.fsync(fd)
        os.fchmod(fd, 0o444)
        info = os.fstat(fd)
        protected(info, stat.S_ISREG, 0o444)
        require(info.st_size == len(data), "publication length")
        os.fsync(fd)
        syscall(316, ctypes.c_int(parent), ctypes.c_char_p(temporary.encode()),
                ctypes.c_int(parent), ctypes.c_char_p(name.encode()), ctypes.c_uint(1))
        os.fsync(parent)
    finally:
        os.close(fd)


def exclusive_directory(parent, name):
    os.mkdir(name, 0o700, dir_fd=parent)
    os.fsync(parent)
    fd = beneath(parent, name, os.O_RDONLY | os.O_DIRECTORY)
    protected(os.fstat(fd), stat.S_ISDIR, 0o700)
    return fd


def boot_id():
    with open("/proc/sys/kernel/random/boot_id", "r", encoding="ascii") as stream:
        value = stream.read(64).strip()
    require(re.fullmatch(r"[0-9a-f-]{36}", value) is not None, "boot identity")
    return value


def dedicated_principal(request):
    # Root/operator concurrency is excluded; an already active shared build UID is not.
    scanned = 0
    with os.scandir("/proc") as scan:
        for entry in scan:
            if not entry.name.isdigit():
                continue
            scanned += 1
            require(scanned <= MAX_ENTRIES, "process inventory capacity")
            try:
                with open(f"/proc/{entry.name}/status", "r", encoding="ascii") as stream:
                    rows = dict(line.split(":", 1) for line in stream.read(8192).splitlines() if ":" in line)
            except FileNotFoundError:
                continue
            require(request["uid"] not in [int(uid) for uid in rows["Uid"].split()],
                    "dedicated build UID already active")


def now():
    return time.clock_gettime_ns(time.CLOCK_BOOTTIME)


def before_deadline(request, reserve=0):
    require(boot_id() == request["boot_id"] and now() + reserve * 10**9 < request["deadline_ns"],
            "boot/deadline mismatch")


def process_birth(pid):
    with open(f"/proc/{pid}/stat", "r", encoding="ascii") as stream:
        raw = stream.read(4096)
    tail = raw[raw.rfind(")") + 2:].split()
    require(len(tail) >= 20, "process birth unavailable")
    return {"pid": pid, "start": int(tail[19]), "boot": boot_id()}


def validate_request(value, nonce):
    exact(value, {"version", "nonce", "boot_id", "issued_ns", "deadline_ns", "uid", "gid",
                  "qualification", "controller_sha256", "template_sha256", "host_files",
                  "trees", "budget", "rust_release", "host_tree_sha256", "host_links", "reviewed_source"})
    require(type(value["version"]) is int and value["version"] == 1
            and value["nonce"] == nonce, "request version/nonce")
    for field in ("issued_ns", "deadline_ns", "uid", "gid"):
        require(type(value[field]) is int and value[field] > 0, "integer request field")
    require(value["uid"] >= 1000 and value["gid"] >= 1000, "dedicated unprivileged IDs")
    require(value["rust_release"] == "1.95.0", "fixed Rust release")
    review = exact(value["reviewed_source"], {"base_head", "source_head", "delta_hex", "delta_sha256", "untracked"})
    for name in ("base_head", "source_head"):
        require(type(review[name]) is str and re.fullmatch(r"[0-9a-f]{40}", review[name]), "reviewed revision")
    require(review["base_head"] == review["source_head"], "unsupported review base/HEAD divergence")
    require(type(review["delta_hex"]) is str and len(review["delta_hex"]) <= MAX_METADATA
            and re.fullmatch(r"(?:[0-9a-f]{2})*", review["delta_hex"]), "review delta encoding")
    require(hashlib.sha256(bytes.fromhex(review["delta_hex"])).hexdigest() == review["delta_sha256"],
            "review delta digest")
    require(type(review["untracked"]) is list and len(review["untracked"]) <= MAX_ENTRIES,
            "untracked inventory")
    seen = set()
    for item in review["untracked"]:
        exact(item, {"path", "sha256"})
        require(type(item["path"]) is str and not item["path"].startswith("/")
                and all(p not in ("", ".", "..") for p in item["path"].split("/"))
                and item["path"] not in seen and type(item["sha256"]) is str
                and SHA.fullmatch(item["sha256"]), "untracked input binding")
        seen.add(item["path"])
    require(value["deadline_ns"] - value["issued_ns"] == 1200 * 10**9,
            "non-resettable total deadline")
    for field in ("qualification", "controller_sha256", "template_sha256", "host_tree_sha256"):
        require(type(value[field]) is str and SHA.fullmatch(value[field]), "admitted digest")
    require(type(value["host_files"]) is dict
            and set(value["host_files"]) == {PYTHON, BWRAP, SYSTEMCTL, "/etc/ld.so.cache",
                                            "/etc/nsswitch.conf", "/etc/passwd", "/etc/group"},
            "host executable/native lookup closure")
    for identity in value["host_files"].values():
        require(type(identity) is str and SHA.fullmatch(identity), "host file digest")
    require(value["host_links"] == {"/lib": "usr/lib", "/lib64": "usr/lib64", "/bin": "usr/bin"},
            "unsupported host library aliases")
    exact(value["trees"], {"source", "platform", "toolchain", "cargo"})
    paths = []
    for role, tree in value["trees"].items():
        exact(tree, {"path", "sha256"})
        absolute_path(tree["path"], "/opt/ota-bootstrap/inputs")
        require(type(tree["sha256"]) is str and SHA.fullmatch(tree["sha256"]),
                "frozen tree binding")
        paths.append(tree["path"])
    require(all(not a.startswith(b + "/") for i, a in enumerate(paths)
                for j, b in enumerate(paths) if i != j)
            and len(set(paths)) == 4, "overlapping input roots")
    exact(value["budget"], {"artifact_bytes", "metadata_bytes", "writable_bytes", "entries"})
    require(value["budget"] == {"artifact_bytes": MAX_ARTIFACT, "metadata_bytes": MAX_METADATA,
                                "writable_bytes": 8 * 1024**3, "entries": MAX_ENTRIES},
            "unsupported budget; no automatic expansion")
    require(len(canonical(value)) <= MAX_REQUEST_BYTES, "bounded reviewed request packet")
    return value


def load_request(path):
    nonce = nonce_from_path(path)
    parent = directory(REQUESTS)
    try:
        value = read_record(parent, nonce + ".json", 0o400)
    finally:
        os.close(parent)
    validate_request(value, nonce)
    before_deadline(value)
    return value


def hash_file(fd, maximum=MAX_SOURCE):
    before = os.fstat(fd)
    require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= maximum,
            "file type/link/capacity")
    h = hashlib.sha256()
    size = 0
    while True:
        data = os.read(fd, 65536)
        if not data:
            break
        size += len(data)
        require(size <= maximum, "file capacity exceeded")
        h.update(data)
    require(stable_metadata(before) == stable_metadata(os.fstat(fd)) and size == before.st_size,
            "file changed while hashing")
    return h.hexdigest(), size


def admitted_file(path, expected, mode=None):
    parent = directory(str(PurePosixPath(path).parent))
    try:
        fd = beneath(parent, PurePosixPath(path).name)
        try:
            protected(os.fstat(fd), stat.S_ISREG, mode)
            require(hash_file(fd)[0] == expected, "admitted file substitution")
        finally:
            os.close(fd)
    finally:
        os.close(parent)


def tree_inventory(root, request, capacity=None):
    """Complete inventory, including Git/embedded inputs, never importing target code."""
    entries = []
    capacity = capacity if capacity is not None else {"entries": 0, "bytes": 0, "metadata": 0}
    def walk(fd, prefix, depth):
        require(depth <= 128, "tree depth")
        protected(os.fstat(fd), stat.S_ISDIR)
        names = []
        with os.scandir(fd) as scan:
            for item in scan:
                require(len(names) + capacity["entries"] < MAX_ENTRIES, "inventory entry limit")
                names.append(item.name)
        names.sort()
        for name in names:
            before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
            require(name.isascii() and name not in (".", "..") and "/" not in name, "tree name")
            capacity["entries"] += 1
            require(capacity["entries"] <= MAX_ENTRIES, "inventory entry limit")
            info = os.stat(name, dir_fd=fd, follow_symlinks=False)
            relative = prefix + name
            row = {"path": relative, "mode": stat.S_IMODE(info.st_mode)}
            if stat.S_ISLNK(info.st_mode):
                require(info.st_uid == 0 and info.st_gid == 0, "unprotected symlink")
                target = os.readlink(name, dir_fd=fd)
                require(not target.startswith("/") and target.isascii(), "absolute symlink refused")
                parts = list(PurePosixPath(relative).parent.parts)
                for part in target.split("/"):
                    if part == "..":
                        require(parts, "symlink escapes tree")
                        parts.pop()
                    elif part not in ("", "."):
                        parts.append(part)
                row.update(kind="symlink", target=target)
                require(stable_metadata(info) == stable_metadata(os.stat(name, dir_fd=fd, follow_symlinks=False)),
                        "changing symlink")
            elif stat.S_ISDIR(info.st_mode):
                child = beneath(fd, name, os.O_RDONLY | os.O_DIRECTORY)
                try:
                    protected(os.fstat(child), stat.S_ISDIR)
                    row.update(kind="directory")
                    capacity["metadata"] += len(canonical(row)) + 1
                    require(capacity["metadata"] <= MAX_INVENTORY_METADATA, "inventory metadata limit")
                    entries.append(row)
                    walk(child, relative + "/", depth + 1)
                finally:
                    os.close(child)
                continue
            else:
                protected(info, stat.S_ISREG)
                child = beneath(fd, name)
                try:
                    require(object_id(child) == {"dev": info.st_dev, "ino": info.st_ino}, "tree race")
                    identity, length = hash_file(child)
                finally:
                    os.close(child)
                capacity["bytes"] += length
                require(capacity["bytes"] <= MAX_SOURCE, "raw inventory byte limit")
                row.update(kind="file", sha256=identity, length=length)
            entries.append(row)
            capacity["metadata"] += len(canonical(row)) + 1
            require(capacity["metadata"] <= MAX_INVENTORY_METADATA, "inventory metadata limit")
    walk(root, "", 0)
    validate_symlink_graph(entries)
    return entries


def git_loose_object(objects, oid, request, capacity):
    require(re.fullmatch(r"[0-9a-f]{40}", oid) is not None, "invalid Git object identifier")
    fd = beneath(objects, oid[:2] + "/" + oid[2:], os.O_RDONLY | os.O_NONBLOCK)
    decoder = zlib.decompressobj()
    raw = bytearray()
    try:
        initial = os.fstat(fd)
        protected(initial, stat.S_ISREG)
        compressed = 0
        while True:
            before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
            data = os.read(fd, 65536)
            if not data:
                break
            compressed += len(data)
            require(compressed <= MAX_SOURCE and not decoder.eof, "trailing/oversize Git compressed data")
            while data:
                decoded = decoder.decompress(data, MAX_GIT_OBJECT + 129 - len(raw))
                raw.extend(decoded)
                require(len(raw) <= MAX_GIT_OBJECT + 128, "Git object decompression capacity")
                capacity["bytes"] += len(decoded)
                require(capacity["bytes"] <= MAX_GIT_DECOMPRESSED, "Git aggregate decompression capacity")
                require(not decoder.unused_data, "trailing Git compressed stream")
                tail = decoder.unconsumed_tail
                require(not tail or len(tail) < len(data) or decoded, "Git decompression made no progress")
                data = tail
        require(decoder.eof and not decoder.unconsumed_tail and not decoder.unused_data,
                "incomplete Git compressed stream")
        require(stable_metadata(initial) == stable_metadata(os.fstat(fd)), "changing Git object")
    except zlib.error as error:
        raise Refusal("invalid Git compression") from error
    finally:
        os.close(fd)
    header, separator, body = bytes(raw).partition(b"\0")
    require(separator and len(header) <= 128, "Git object framing")
    match = re.fullmatch(rb"(commit|tree|blob) (0|[1-9][0-9]*)", header)
    require(match is not None and int(match[2]) == len(body) and len(body) <= MAX_GIT_OBJECT,
            "Git object type/length")
    # SHA1 is legacy format consistency ONLY. Root-approved SHA256 raw inventories
    # are the input authority anchor; this must never become a security trust key.
    require(hashlib.sha1(raw).hexdigest() == oid, "Git object identifier/content mismatch")
    return match[1].decode("ascii"), body, hashlib.sha256(raw).hexdigest()


def git_edges(kind, body):
    edges = []
    if kind == "blob":
        return edges
    if kind == "commit":
        headers, separator, _ = body.partition(b"\n\n")
        require(separator, "Git commit header boundary")
        trees = []
        for line in headers.splitlines():
            if line.startswith(b"tree ") or line.startswith(b"parent "):
                key, oid = line.split(b" ", 1)
                require(re.fullmatch(rb"[0-9a-f]{40}", oid) is not None, "Git commit reference")
                edge = (oid.decode("ascii"), "tree" if key == b"tree" else "commit")
                edges.append(edge)
                if key == b"tree":
                    trees.append(edge)
            require(len(edges) <= MAX_ENTRIES, "Git commit edge capacity")
        require(len(trees) == 1, "Git commit tree cardinality")
        return edges
    require(kind == "tree", "unsupported Git object kind")
    cursor = 0
    names = set()
    while cursor < len(body):
        space = body.find(b" ", cursor)
        nul = body.find(b"\0", space + 1) if space >= 0 else -1
        require(space > cursor and nul > space + 1 and nul + 21 <= len(body), "Git tree entry framing")
        mode, name = body[cursor:space], body[space + 1:nul]
        require(name not in (b".", b"..") and b"/" not in name and name not in names,
                "Git tree entry name/duplication")
        names.add(name)
        require(mode in (b"40000", b"100644", b"100755", b"120000"),
                "unsupported Git tree mode/submodule")
        edges.append((body[nul + 1:nul + 21].hex(), "tree" if mode == b"40000" else "blob"))
        require(len(edges) <= MAX_ENTRIES, "Git tree edge capacity")
        cursor = nul + 21
    return edges


def git_head_closure(objects, head, request):
    done, active = {}, set()
    tasks = [(False, head, "commit", 0)]
    capacity = {"bytes": 0}
    graph = hashlib.sha256(b"ota.retained-driver.loose-head-closure.v1\0")
    while tasks:
        before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
        leaving, oid, expected, depth = tasks.pop()
        if leaving:
            active.remove(oid)
            done[oid] = expected
            continue
        require(depth <= MAX_ENTRIES and oid not in active, "Git closure depth/cycle")
        if oid in done:
            require(done[oid] == expected, "Git object used with conflicting types")
            continue
        require(len(done) + len(active) < MAX_ENTRIES, "Git closure object capacity")
        kind, body, raw_sha256 = git_loose_object(objects, oid, request, capacity)
        require(kind == expected, "Git referenced object has wrong type")
        graph.update(canonical({"oid": oid, "kind": kind, "sha256": raw_sha256, "length": len(body)}))
        active.add(oid)
        tasks.append((True, oid, kind, depth))
        for child, child_kind in reversed(git_edges(kind, body)):
            require(len(tasks) < MAX_ENTRIES, "Git closure work queue capacity")
            tasks.append((False, child, child_kind, depth + 1))
    return {"encoding": "loose_only", "head": head, "objects": len(done),
            "decompressed_bytes": capacity["bytes"], "graph_sha256": graph.hexdigest()}


def self_contained_git(source, review, inventory, request):
    git = beneath(source, ".git", os.O_RDONLY | os.O_DIRECTORY)
    objects = refs = None
    try:
        objects = beneath(git, "objects", os.O_RDONLY | os.O_DIRECTORY)
        refs = beneath(git, "refs", os.O_RDONLY | os.O_DIRECTORY)
        protected(os.fstat(objects), stat.S_ISDIR)
        protected(os.fstat(refs), stat.S_ISDIR)
        try:
            pack = beneath(objects, "pack", os.O_RDONLY | os.O_DIRECTORY)
        except FileNotFoundError:
            pack = None
        if pack is not None:
            try:
                with os.scandir(pack) as entries:
                    require(next(entries, None) is None, "packed/other Git encoding unsupported")
            finally:
                os.close(pack)
        for path in ("commondir", "gitdir", "objects/info/alternates", "objects/info/http-alternates"):
            try:
                fd = beneath(git, path)
            except FileNotFoundError:
                continue
            else:
                os.close(fd)
                raise Refusal("external Git indirection")
        config = beneath(git, "config")
        try:
            data = read_fd(config).decode("ascii")
        finally:
            os.close(config)
        for line in data.splitlines():
            text = line.strip().lower()
            require(not text.endswith("\\") and not re.match(r"\[\s*include", text)
                    and not re.match(r"(?:worktree|worktreeconfig|gitdir|commondir)\s*(?:=|$)", text),
                    "unsupported Git configuration indirection")
        head_fd = beneath(git, "HEAD")
        try:
            head = read_fd(head_fd, 4096).decode("ascii").strip()
        finally:
            os.close(head_fd)
        if head.startswith("ref: "):
            ref = head[5:]
            require(ref.startswith("refs/") and all(p not in ("", ".", "..") for p in ref.split("/")),
                    "Git ref outside sealed directory")
            try:
                ref_fd = beneath(git, ref)
            except FileNotFoundError:
                packed = beneath(git, "packed-refs")
                try:
                    rows = read_fd(packed).decode("ascii").splitlines()
                finally:
                    os.close(packed)
                matches = [line.split()[0] for line in rows if not line.startswith(("#", "^"))
                           and len(line.split()) == 2 and line.split()[1] == ref]
                require(len(matches) == 1, "Git ref missing/ambiguous")
                head = matches[0]
            else:
                try:
                    head = read_fd(ref_fd, 4096).decode("ascii").strip()
                finally:
                    os.close(ref_fd)
        require(head == review["source_head"], "sealed Git HEAD differs from reviewed HEAD")
        rows = {row["path"]: row for row in inventory}
        require(all(path in rows and rows[path]["kind"] == "directory"
                    for path in (".git/objects", ".git/refs")) and ".gitmodules" not in rows,
                "unsupported external/submodule Git layout")
        for item in review["untracked"]:
            require(item["path"] in rows and rows[item["path"]]["kind"] == "file"
                    and rows[item["path"]]["sha256"] == item["sha256"], "untracked reviewed bytes differ")
        return {"source_head": head, "git_directory": ".git", "external_indirection": False,
                "reviewed_delta_sha256": review["delta_sha256"], "untracked": review["untracked"],
                "head_closure": git_head_closure(objects, head, request)}
    finally:
        if objects is not None:
            os.close(objects)
        if refs is not None:
            os.close(refs)
        os.close(git)


def platform_admission(request):
    require(sys.platform == "linux" and platform.machine() == "x86_64"
            and os.getuid() == 0 and os.geteuid() == 0, "qualified Linux/X64 root operator only")
    require(sys.flags.isolated and sys.flags.dont_write_bytecode, "isolated admitted Python required")
    admitted_file(CONTROLLER, request["controller_sha256"], 0o444)
    admitted_file("/etc/systemd/system/ota-bootstrap-owner@.service", request["template_sha256"], 0o444)
    for path, identity in request["host_files"].items():
        admitted_file(path, identity)
        info = os.lstat(path)
        require(not info.st_mode & (stat.S_ISUID | stat.S_ISGID), "set-ID host executable")
    for path, target in request["host_links"].items():
        info = os.lstat(path)
        require(stat.S_ISLNK(info.st_mode) and info.st_uid == info.st_gid == 0
                and os.readlink(path) == target, "host library alias substitution")
    require(not os.path.lexists("/etc/ld.so.preload"), "ambient native preload")
    require(all(path.startswith("/usr/") for path in sys.path), "unadmitted Python search root")
    # A normal symlinked /usr/bin/python3 is intentionally not an admitted installation.
    # The operator must freeze the interpreter and its native closure; no path fallback.
    require(resource.getrlimit(resource.RLIMIT_NOFILE)[0] <= 1024, "FD ceiling")
    require(request["issued_ns"] <= now(), "future request")


def command(argv, request, maximum=65536):
    """Only trusted manager queries; shared deadline and bounded output."""
    before_deadline(request)
    require(argv[0] == SYSTEMCTL, "not a manager query")
    proc = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, env={"PATH": "/usr/bin", "LC_ALL": "C",
                                                         "SYSTEMD_PAGER": "cat"}, close_fds=True)
    output = bytearray()
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(proc.stdout, selectors.EVENT_READ)
            while selector.get_map():
                before_deadline(request)
                for key, _ in selector.select(0.1):
                    data = os.read(key.fileobj.fileno(), 4096)
                    if not data:
                        selector.unregister(key.fileobj)
                    else:
                        output.extend(data)
                        require(len(output) <= maximum, "manager output capacity")
        status = proc.wait(timeout=max(0.01, (request["deadline_ns"] - now()) / 10**9))
        require(status == 0, "manager command failed")
        return output.decode("ascii")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        proc.stdout.close()


PROPERTIES = {
    "Id", "InvocationID", "ControlGroup", "Type", "User", "Group", "Restart",
    "RemainAfterExit", "KillMode", "MemoryMax", "MemorySwapMax", "CPUQuotaPerSecUSec",
    "TasksMax", "LimitNOFILE", "RuntimeMaxUSec", "TimeoutStopUSec", "PrivateNetwork",
    "PrivateMounts", "Delegate", "MainPID", "ExecMainCode", "ExecMainStatus",
    "Result", "ActiveState", "SubState",
}


def manager(unit, request):
    output = command([SYSTEMCTL, "show", "--no-pager", "--property=" + ",".join(sorted(PROPERTIES)),
                      "--", unit], request)
    result = {}
    for line in output.splitlines():
        key, separator, value = line.partition("=")
        require(separator and key in PROPERTIES and key not in result, "manager property shape")
        result[key] = value
    require(set(result) == PROPERTIES and result["Id"] == unit
            and GENERATION.fullmatch(result["InvocationID"]), "manager generation missing")
    return result


def manager_limits(record, postflight=False):
    require(record["Type"] == "exec" and record["User"] == "root"
            and record["Group"] == "root" and record["Restart"] == "no"
            and record["KillMode"] == "control-group"
            and record["MemoryMax"] == str((1 if postflight else 3) * 1024**3)
            and record["MemorySwapMax"] == "0" and record["CPUQuotaPerSecUSec"] == "2s"
            and record["TasksMax"] == "256" and record["LimitNOFILE"] == "1024"
            and record["PrivateNetwork"] == "yes" and record["PrivateMounts"] == "yes",
            "effective manager containment mismatch")
    require(record["RemainAfterExit"] == ("no" if postflight else "yes"), "manager retention policy")
    require(record["TimeoutStopUSec"] == "10s", "manager stop ceiling")
    if postflight:
        require(re.fullmatch(r"[1-9][0-9]*s", record["RuntimeMaxUSec"])
                and int(record["RuntimeMaxUSec"][:-1]) <= POSTFLIGHT_SECONDS,
                "bounded postflight runtime required")
    else:
        require(record["RuntimeMaxUSec"] == "18min", "manager operational/cleanup ceiling")
    if not postflight:
        require(record["Delegate"] == "yes", "root cgroup delegation missing")


def own_cgroup():
    with open("/proc/self/cgroup", "r", encoding="ascii") as stream:
        lines = stream.read(4096).splitlines()
    require(len(lines) == 1 and lines[0].startswith("0::/"), "unified cgroup v2 required")
    return lines[0][3:]


def cgroup_open(path):
    require(path.startswith("/") and path != "/" and str(PurePosixPath(path)) == path, "cgroup path")
    root = directory(CGROUP_ROOT)
    try:
        # Do not infer cgroup-v2 from a directory name.
        superblock = ctypes.create_string_buffer(256)
        require(LIBC.fstatfs(ctypes.c_int(root), ctypes.byref(superblock)) == 0,
                "cgroup superblock unavailable")
        require(ctypes.c_long.from_buffer(superblock).value == 0x63677270, "not cgroup-v2")
        fd = beneath(root, path[1:], os.O_RDONLY | os.O_DIRECTORY)
        protected(os.fstat(fd), stat.S_ISDIR)
        return fd
    finally:
        os.close(root)


def control_read(fd, name):
    child = beneath(fd, name)
    try:
        return read_fd(child, 4096).decode("ascii").strip()
    finally:
        os.close(child)


def control_write(fd, name, value):
    child = beneath(fd, name, os.O_WRONLY)
    try:
        protected(os.fstat(child), stat.S_ISREG)
        require(os.write(child, value.encode("ascii")) == len(value), "short cgroup control write")
    finally:
        os.close(child)


def populated(fd):
    events = dict(line.split() for line in control_read(fd, "cgroup.events").splitlines())
    require(set(events) == {"populated", "frozen"} and events["populated"] in ("0", "1"),
            "recursive cgroup state unavailable")
    return events["populated"] == "1"


def pidfd_signal(pidfd, sig):
    syscall(424, ctypes.c_int(pidfd), ctypes.c_int(sig), ctypes.c_void_p(), ctypes.c_uint(0))


def wait_owned(pidfd, request, stopped=False):
    options = os.WNOWAIT | os.WNOHANG | (os.WSTOPPED | os.WEXITED if stopped else os.WEXITED)
    while True:
        before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS if stopped else POSTFLIGHT_SECONDS)
        result = os.waitid(os.P_PIDFD, pidfd, options)
        if result is not None:
            if stopped:
                require(result.si_code == os.CLD_STOPPED and result.si_status == signal.SIGSTOP,
                        "child did not stop before placement")
            return result
        time.sleep(0.02)


def worker_environment():
    return {
        "PATH": "/toolchain/bin:/usr/bin:/bin", "HOME": "/work/home",
        "TMPDIR": "/work/tmp", "CARGO_HOME": "/cargo", "CARGO_TARGET_DIR": "/work/target",
        "CARGO": "/toolchain/bin/cargo", "RUSTC": "/toolchain/bin/rustc",
        "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "2", "CARGO_INCREMENTAL": "0",
        "GIT_OPTIONAL_LOCKS": "0", "LC_ALL": "C", "LANG": "C",
    }


def sandbox_argv(request, work):
    trees = request["trees"]
    args = [BWRAP, "--unshare-all", "--die-with-parent", "--new-session", "--cap-drop", "ALL",
            "--ro-bind", trees["platform"]["path"], "/", "--proc", "/proc", "--dev", "/dev",
            "--ro-bind", trees["source"]["path"], "/source",
            "--ro-bind", trees["toolchain"]["path"], "/toolchain",
            "--ro-bind", trees["cargo"]["path"], "/cargo",
            "--bind", work, "/work",
            "--bind", work + "/.package-cache", "/cargo/.package-cache",
            "--bind", work + "/.package-cache-mutate", "/cargo/.package-cache-mutate",
            "--clearenv"]
    for key, value in sorted(worker_environment().items()):
        args.extend(["--setenv", key, value])
    args.extend(["--chdir", "/source", "--", "/toolchain/bin/cargo", "build", "--locked",
                 "--offline", "--bin", "ota", "--target", "x86_64-unknown-linux-gnu"])
    return args


def binding(request, unit):
    return {"request": digest(canonical(request)), "nonce": request["nonce"],
            "boot_id": request["boot_id"], "deadline_ns": request["deadline_ns"],
            "unit": unit["Id"], "generation": unit["InvocationID"],
            "cgroup": unit["ControlGroup"], "controller": request["controller_sha256"],
            "qualification": request["qualification"], "trees": request["trees"],
            "budget": request["budget"], "environment": worker_environment(),
            "reviewed_source": {"head": request["reviewed_source"]["source_head"],
                                "base": request["reviewed_source"]["base_head"],
                                "delta_sha256": request["reviewed_source"]["delta_sha256"],
                                "untracked_sha256": digest(canonical(request["reviewed_source"]["untracked"]))},
            "not_proved": NOT_PROVED}


class Journal:
    def __init__(self, fd, binding_value):
        self.fd = fd
        self.binding = binding_value
        self.state = None

    def advance(self, state, evidence):
        expected = STATES[0] if self.state is None else (
            STATES[STATES.index(self.state) + 1] if self.state in STATES[:-1] else None)
        require(state == expected or (state in ("failed", "interrupted")
                                     and self.state not in ("failed", "interrupted", "finalization_candidate")),
                "illegal state promotion")
        record = {"version": 1, "state": state, "previous": self.state,
                  "binding": self.binding, "evidence": evidence}
        publish(self.fd, state + ".json", canonical(record))
        self.state = state
        return record


def verify_record(record, state, bound):
    exact(record, {"version", "state", "previous", "binding", "evidence"})
    require(type(record["version"]) is int and record["version"] == 1
            and record["state"] == state and record["binding"] == bound
            and record["previous"] == STATES[STATES.index(state) - 1], "stale/forged state")
    require(type(record["evidence"]) is dict, "record evidence shape")
    return record["evidence"]


def verify_child(pid, birth, path):
    require(process_birth(pid) == birth, "child identity changed")
    with open(f"/proc/{pid}/status", "r", encoding="ascii") as stream:
        status = dict(line.split(":", 1) for line in stream.read(8192).splitlines() if ":" in line)
    require(status["State"].strip().startswith("T") and int(status["PPid"]) == os.getpid(),
            "child continuation/parent mismatch")
    with open(f"/proc/{pid}/cgroup", "r", encoding="ascii") as stream:
        require(stream.read(4096) == f"0::{path}\n", "child placement mismatch")


def scratch_ancestry(fd):
    # Search-only for the isolated worker; no enumeration, write or ownership grant.
    protected(os.fstat(fd), stat.S_ISDIR, 0o711)


def cgroup_leaf(parent, name):
    # cgroupfs controls are kernel state, not fsyncable journal files.
    os.mkdir(name, 0o755, dir_fd=parent)
    child = beneath(parent, name, os.O_RDONLY | os.O_DIRECTORY)
    protected(os.fstat(child), stat.S_ISDIR)
    require(not control_read(child, "cgroup.procs") and not populated(child), "foreign leaf tasks")
    return child


def verify_cgroup_limits(cg):
    require(control_read(cg, "memory.max") == str(3 * 1024**3)
            and control_read(cg, "memory.swap.max") == "0"
            and control_read(cg, "pids.max") == "256", "kernel resource ceiling mismatch")
    quota, period = control_read(cg, "cpu.max").split()
    require(quota.isdigit() and period.isdigit() and int(quota) <= 2 * int(period),
            "kernel CPU ceiling mismatch")


def mount_private_scratch(work, request):
    mounts = []
    for name, size in (("home", 16 * 1024**2), ("tmp", 256 * 1024**2)):
        path = work + "/" + name
        options = f"size={size},mode=0700,uid={request['uid']},gid={request['gid']}".encode()
        require(LIBC.mount(ctypes.c_char_p(b"tmpfs"), ctypes.c_char_p(path.encode()),
                           ctypes.c_char_p(b"tmpfs"), ctypes.c_ulong(2 | 4 | 8),
                           ctypes.c_char_p(options)) == 0, "noexec scratch mount failed")
        fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
        mounts.append((path, fd, object_id(fd)))
    return mounts


def unmount_private_scratch(mounts):
    for path, fd, identity in reversed(mounts):
        current = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
        try:
            require(object_id(current) == identity == object_id(fd), "scratch mount substitution")
        finally:
            os.close(current)
        # An open directory reference would itself keep the mount busy.
        os.close(fd)
        require(LIBC.umount2(ctypes.c_char_p(path.encode()), ctypes.c_int(0)) == 0,
                "scratch unmount incomplete")


def input_geometry(roots):
    # Only these two Cargo housekeeping files are writable. Missing cache geometry
    # or a normal rustup shim requires a separately approved input packet.
    for name in (".package-cache", ".package-cache-mutate"):
        fd = beneath(roots["cargo"], name)
        try:
            protected(os.fstat(fd), stat.S_ISREG)
            require(os.fstat(fd).st_size == 0, "cache lock geometry")
        finally:
            os.close(fd)
    for name in ("bin/cargo", "bin/rustc"):
        fd = beneath(roots["toolchain"], name)
        try:
            protected(os.fstat(fd), stat.S_ISREG)
            require(os.fstat(fd).st_mode & 0o111, "native toolchain executable")
        finally:
            os.close(fd)
    fd = beneath(roots["toolchain"], "lib/rustlib/multirust-channel-manifest.toml")
    try:
        import tomllib
        release = tomllib.loads(read_fd(fd).decode("utf-8"))["pkg"]["rust"]["version"]
        require(type(release) is str and release.startswith("1.95.0 "), "toolchain release data")
    finally:
        os.close(fd)
    for name in ("source", "toolchain", "cargo", "work", "proc", "dev"):
        fd = beneath(roots["platform"], name, os.O_RDONLY | os.O_DIRECTORY)
        os.close(fd)


def recursive_remove(fd, request):
    """Only the retained private scratch root; no symlink following or mount traversal."""
    for name in os.listdir(fd):
        before_deadline(request, POSTFLIGHT_SECONDS)
        info = os.stat(name, dir_fd=fd, follow_symlinks=False)
        if stat.S_ISDIR(info.st_mode):
            child = beneath(fd, name, os.O_RDONLY | os.O_DIRECTORY)
            try:
                recursive_remove(child, request)
            finally:
                os.close(child)
            os.rmdir(name, dir_fd=fd)
        else:
            os.unlink(name, dir_fd=fd)
    os.fsync(fd)


def retain_candidate(target, retained, request):
    fd = beneath(target, "x86_64-unknown-linux-gnu/debug/ota", os.O_RDONLY | os.O_NONBLOCK)
    output = None
    try:
        info = os.fstat(fd)
        require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1
                and info.st_uid == request["uid"] and info.st_gid == request["gid"]
                and stat.S_IMODE(info.st_mode) == 0o755 and 0 < info.st_size <= MAX_ARTIFACT,
                "candidate metadata/capacity")
        output = beneath(retained, ".driver.pending.tmp", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        protected(os.fstat(output), stat.S_ISREG, 0o600)
        h = hashlib.sha256()
        total = 0
        while True:
            before_deadline(request, POSTFLIGHT_SECONDS)
            data = os.read(fd, 65536)
            if not data:
                break
            total += len(data)
            require(total <= MAX_ARTIFACT, "candidate capacity")
            h.update(data)
            view = memoryview(data)
            while view:
                count = os.write(output, view)
                require(count > 0, "short candidate write")
                view = view[count:]
        require(stable_metadata(info) == stable_metadata(os.fstat(fd)) and total == info.st_size,
                "changing candidate")
        os.fsync(output)
        os.fchmod(output, 0o444)
        protected(os.fstat(output), stat.S_ISREG, 0o444)
        require(os.fstat(output).st_size == total, "retained length mismatch")
        os.fsync(output)
        syscall(316, ctypes.c_int(retained), ctypes.c_char_p(b".driver.pending.tmp"),
                ctypes.c_int(retained), ctypes.c_char_p(b"driver.pending"), ctypes.c_uint(1))
        os.fsync(retained)
        return {"length": total, "sha256": h.hexdigest(), "candidate_object": object_id(fd),
                "retained_object": object_id(output), "relative_path": "driver.pending"}
    finally:
        if output is not None:
            os.close(output)
        os.close(fd)


def clean_worker(worker_fd, request):
    limit = min(request["deadline_ns"] - POSTFLIGHT_SECONDS * 10**9,
                now() + CLEANUP_SECONDS * 10**9)
    control_write(worker_fd, "cgroup.kill", "1")
    while populated(worker_fd):
        require(now() < limit and boot_id() == request["boot_id"], "worker cleanup uncertain")
        time.sleep(0.02)


def seal_log(fd, retention, disposition):
    os.fsync(fd)
    os.fchmod(fd, 0o444)
    protected(os.fstat(fd), stat.S_ISREG, 0o444)
    require(os.fstat(fd).st_size == disposition["retained_bytes"], "log retention length")
    os.fsync(fd)
    syscall(316, ctypes.c_int(retention), ctypes.c_char_p(b".worker.log.tmp"),
            ctypes.c_int(retention), ctypes.c_char_p(b"worker.log"), ctypes.c_uint(1))
    os.fsync(retention)
    return {**disposition, "path": "worker.log", "object": object_id(fd)}


def build_owner(request):
    unit_name = f"ota-bootstrap-owner@{request['nonce']}.service"
    observed = manager(unit_name, request)
    manager_limits(observed)
    require(int(observed["MainPID"]) == os.getpid() and own_cgroup() == observed["ControlGroup"],
            "not the admitted owner invocation")
    bound = binding(request, observed)
    with ExitStack() as stack:
        def own(fd):
            stack.callback(os.close, fd)
            return fd
        inv_root = own(directory(INVOCATIONS))
        journal_fd = own(exclusive_directory(inv_root, request["nonce"]))
        publish(journal_fd, "started", canonical(bound))
        publish(journal_fd, "request.json", canonical(request))
        journal = Journal(journal_fd, bound)
        journal.advance("armed", {"owner": process_birth(os.getpid())})
        pid = pidfd = worker_fd = None
        scratch_mounts = []
        log_fd = retention = None
        log_disposition = None
        retained_log = None
        try:
            before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
            dedicated_principal(request)
            roots = {}
            capacity = {"entries": 0, "bytes": 0, "metadata": 0}
            host = own(directory("/usr"))
            host_inventory = tree_inventory(host, request, capacity)
            require(digest(canonical(host_inventory), b"ota.retained-driver.tree.v1\0") == request["host_tree_sha256"],
                    "interpreter/manager/native platform closure substitution")
            publish(journal_fd, "host.inventory.json", canonical(host_inventory))
            del host_inventory
            source_inventory = None
            for role, tree in request["trees"].items():
                roots[role] = own(directory(tree["path"]))
                inventory = tree_inventory(roots[role], request, capacity)
                require(digest(canonical(inventory), b"ota.retained-driver.tree.v1\0") == tree["sha256"],
                        "input tree substitution")
                publish(journal_fd, role + ".inventory.json", canonical(inventory))
                if role == "source":
                    source_inventory = inventory
            publish(journal_fd, "git-layout.json",
                    canonical(self_contained_git(roots["source"], request["reviewed_source"], source_inventory, request)))
            input_geometry(roots)
            cg = own(cgroup_open(observed["ControlGroup"]))
            verify_cgroup_limits(cg)
            require(control_read(cg, "cgroup.type") == "domain"
                    and set(control_read(cg, "cgroup.procs").split()) == {str(os.getpid())}
                    and not any(stat.S_ISDIR(os.stat(n, dir_fd=cg).st_mode) for n in os.listdir(cg)),
                    "foreign cgroup state")
            owner_leaf = own(cgroup_leaf(cg, "owner"))
            control_write(owner_leaf, "cgroup.procs", str(os.getpid()))
            require(own_cgroup() == observed["ControlGroup"] + "/owner", "owner leaf placement")
            require({"cpu", "memory", "pids"} <= set(control_read(cg, "cgroup.controllers").split()),
                    "controllers unavailable")
            control_write(cg, "cgroup.subtree_control", "+cpu +memory +pids")
            require({"cpu", "memory", "pids"} <= set(control_read(cg, "cgroup.subtree_control").split()),
                    "controllers not enabled")
            worker_fd = own(cgroup_leaf(cg, "worker"))
            worker_path = observed["ControlGroup"] + "/worker"
            cg_object = object_id(cg)
            worker_object = object_id(worker_fd)
            work_root = own(directory(SCRATCH))
            scratch_ancestry(work_root)
            require(control_scratch_mount(), "scratch must be bounded nosuid/nodev tmpfs")
            work_fd = own(exclusive_directory(work_root, request["nonce"]))
            work = SCRATCH + "/" + request["nonce"]
            for name in ("target", "home", "tmp"):
                os.mkdir(name, 0o700, dir_fd=work_fd)
                os.chown(name, request["uid"], request["gid"], dir_fd=work_fd)
            os.fchown(work_fd, request["uid"], request["gid"])
            for name in (".package-cache", ".package-cache-mutate"):
                lock = beneath(work_fd, name, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
                os.fchown(lock, request["uid"], request["gid"])
                os.close(lock)
            scratch_mounts = mount_private_scratch(work, request)
            target = own(beneath(work_fd, "target", os.O_RDONLY | os.O_DIRECTORY))
            retention = own(exclusive_directory(own(directory(RETAINED)), request["nonce"]))
            log_fd = own(beneath(retention, ".worker.log.tmp", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600))
            protected(os.fstat(log_fd), stat.S_ISREG, 0o600)
            read_pipe, write_pipe = os.pipe2(os.O_CLOEXEC)
            stack.callback(os.close, read_pipe)
            actions = [(os.POSIX_SPAWN_OPEN, 0, "/dev/null", os.O_RDONLY, 0),
                       (os.POSIX_SPAWN_DUP2, write_pipe, 1), (os.POSIX_SPAWN_DUP2, write_pipe, 2),
                       (os.POSIX_SPAWN_CLOSE, read_pipe), (os.POSIX_SPAWN_CLOSE, write_pipe)]
            try:
                pid = os.posix_spawn(PYTHON, [PYTHON, "-I", "-B", CONTROLLER, "worker",
                                             f"{REQUESTS}/{request['nonce']}.json", str(os.getpid())],
                                     {"PATH": "/usr/bin", "LC_ALL": "C"}, file_actions=actions)
            finally:
                os.close(write_pipe)
            pidfd = own(os.pidfd_open(pid))
            birth = process_birth(pid)
            wait_owned(pidfd, request, stopped=True)
            journal.advance("child_stopped", {"child": birth, "worker_object": worker_object})
            control_write(worker_fd, "cgroup.procs", str(pid))
            verify_child(pid, birth, worker_path)
            placed = {"child": birth, "owner": process_birth(os.getpid()), "worker_path": worker_path,
                      "cgroup_object": cg_object, "worker_object": worker_object,
                      "target_object": object_id(target), "work": work}
            journal.advance("placed", placed)
            verify_child(pid, birth, worker_path)
            journal.advance("building", {"argv": sandbox_argv(request, work), "placement": placed})
            pidfd_signal(pidfd, signal.SIGCONT)
            terminal, log_disposition, leaked = collect_worker(pidfd, read_pipe, request, log_fd, worker_fd)
            retained_log = seal_log(log_fd, retention, log_disposition)
            require(not populated(worker_fd), "worker not recursively terminal")
            require(process_birth(pid) == birth, "unreaped child generation mismatch")
            os.waitid(os.P_PIDFD, pidfd, os.WEXITED)
            pid = None
            os.rmdir("worker", dir_fd=cg)
            evidence = {"exit_code": terminal.si_code, "exit_status": terminal.si_status,
                        "forced_cleanup": leaked, "recursive_terminal": True,
                        "log": retained_log,
                        "placement": placed}
            journal.advance("worker_terminal", evidence)
            require(terminal.si_code == os.CLD_EXITED and terminal.si_status == 0 and not leaked,
                    "nonzero worker or surviving descendants")
            artifact = retain_candidate(target, retention, request)
            unmount_private_scratch(scratch_mounts)
            scratch_mounts = []
            recursive_remove(work_fd, request)
            os.rmdir(request["nonce"], dir_fd=work_root)
            os.fsync(work_root)
            evidence.update(artifact=artifact, scratch_removed=True)
            journal.advance("artifact_pending", evidence)
        except BaseException as error:
            if isinstance(error, LogRefusal):
                log_disposition = error.disposition
            if worker_fd is not None:
                try:
                    if pidfd is not None and pid is not None:
                        try:
                            pidfd_signal(pidfd, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    clean_worker(worker_fd, request)
                    if pidfd is not None and pid is not None:
                        limit = min(request["deadline_ns"] - POSTFLIGHT_SECONDS * 10**9,
                                    now() + CLEANUP_SECONDS * 10**9)
                        while os.waitid(os.P_PIDFD, pidfd, os.WEXITED | os.WNOWAIT | os.WNOHANG) is None:
                            require(now() < limit, "owned child exit uncertain")
                            time.sleep(0.02)
                        os.waitid(os.P_PIDFD, pidfd, os.WEXITED)
                except BaseException:
                    error = Refusal("cleanup uncertain; manager emergency cleanup required")
            for _, mount_fd, _ in scratch_mounts:
                try:
                    os.close(mount_fd)
                except OSError:
                    pass
            try:
                if retained_log is None and log_disposition is not None and log_fd is not None and retention is not None:
                    try:
                        retained_log = seal_log(log_fd, retention, log_disposition)
                    except BaseException:
                        pass
                journal.advance("failed", {"reason": type(error).__name__, "success": False,
                                           "log": retained_log, "log_disposition": log_disposition})
            except BaseException:
                pass  # Absence of terminal publication can never be success.
            raise error


def control_scratch_mount():
    with open("/proc/self/mountinfo", "r", encoding="ascii") as stream:
        lines = stream.read(MAX_METADATA).splitlines()
    rows = [line.split() for line in lines if len(line.split()) > 6 and line.split()[4] == SCRATCH]
    if len(rows) != 1:
        return False
    row = rows[0]
    separator = row.index("-")
    options = set(row[5].split(",")) | set(row[separator + 3].split(","))
    sizes = [int(item[5:]) for item in options if item.startswith("size=") and item[5:].isdigit()]
    return row[separator + 1] == "tmpfs" and {"rw", "nosuid", "nodev"} <= options and (
        len(sizes) == 1 and sizes[0] <= 7 * 1024**3 and "noexec" not in options)


def collect_worker(pidfd, pipe, request, log_fd, worker_fd):
    h = hashlib.sha256()
    retained_bytes = observed_bytes = 0
    terminal = None
    eof = False
    survivors = False
    cleanup_deadline = None
    operation_deadline = request["issued_ns"] + BUILD_SECONDS * 10**9
    def disposition(complete=False, truncated=False):
        return {"sha256": h.hexdigest(), "retained_bytes": retained_bytes,
                "observed_bytes": observed_bytes, "complete": complete, "truncated": truncated}
    with selectors.DefaultSelector() as selector:
        selector.register(pipe, selectors.EVENT_READ)
        try:
            while terminal is None or not eof:
                before_deadline(request, POSTFLIGHT_SECONDS if terminal else CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
                require(now() < (cleanup_deadline if terminal else operation_deadline), "worker log deadline")
                for key, _ in selector.select(0.02):
                    data = os.read(key.fd, 65536)
                    if not data:
                        selector.unregister(key.fd)
                        eof = True
                        continue
                    observed_bytes += len(data)
                    accepted = data[:MAX_LOG - retained_bytes]
                    view = memoryview(accepted)
                    while view:
                        count = os.write(log_fd, view)
                        require(count > 0, "short log write")
                        h.update(view[:count])
                        retained_bytes += count
                        view = view[count:]
                    if observed_bytes > MAX_LOG:
                        raise LogRefusal(disposition(truncated=True))
                if terminal is None:
                    terminal = os.waitid(os.P_PIDFD, pidfd, os.WEXITED | os.WNOWAIT | os.WNOHANG)
                    if terminal is not None:
                        cleanup_deadline = min(request["deadline_ns"] - POSTFLIGHT_SECONDS * 10**9,
                                               now() + CLEANUP_SECONDS * 10**9)
                        survivors = populated(worker_fd)
                        clean_worker(worker_fd, request)
        except LogRefusal:
            raise
        except BaseException as error:
            raise LogRefusal(disposition()) from error
    return terminal, disposition(complete=True), survivors


def worker(request, owner):
    require(os.getppid() == owner and owner > 1, "worker parent changed")
    parent_birth = process_birth(owner)
    os.kill(os.getpid(), signal.SIGSTOP)
    require(os.getppid() == owner and process_birth(owner) == parent_birth, "owner lost")
    root = directory(INVOCATIONS + "/" + request["nonce"])
    try:
        placed = read_record(root, "placed.json")
        armed = read_record(root, "armed.json")
        exact(armed, {"version", "state", "previous", "binding", "evidence"})
        require(type(armed["version"]) is int and armed["version"] == 1
                and armed["state"] == "armed" and armed["previous"] is None
                and armed["binding"]["request"] == digest(canonical(request)), "placement binding")
        proof = verify_record(placed, "placed", armed["binding"])
        exact(proof, {"child", "owner", "worker_path", "cgroup_object", "worker_object",
                      "target_object", "work"})
        require(proof["owner"] == parent_birth and proof["child"] == process_birth(os.getpid())
                and own_cgroup() == proof["worker_path"], "worker placement identity")
        cg = cgroup_open(proof["worker_path"])
        try:
            require(object_id(cg) == proof["worker_object"], "worker cgroup generation")
        finally:
            os.close(cg)
        work = proof["work"]
        require(work == SCRATCH + "/" + request["nonce"], "scratch substitution")
    finally:
        os.close(root)
    before_deadline(request, CLEANUP_SECONDS + POSTFLIGHT_SECONDS)
    with open("/proc/sys/kernel/cap_last_cap", "r", encoding="ascii") as stream:
        last_cap = int(stream.read(32))
    require(0 <= last_cap < 64, "unsupported capability ABI")
    for capability in range(last_cap + 1):
        require(LIBC.prctl(24, capability, 0, 0, 0) == 0, "bounding capability drop")
    require(LIBC.prctl(8, 0, 0, 0, 0) == 0, "clear keepcaps")
    os.setgroups([])
    os.setresgid(request["gid"], request["gid"], request["gid"])
    os.setresuid(request["uid"], request["uid"], request["uid"])
    require(os.getresuid() == (request["uid"],) * 3 and os.getresgid() == (request["gid"],) * 3
            and not os.getgroups(), "principal drop incomplete")
    class CapHeader(ctypes.Structure):
        _fields_ = [("version", ctypes.c_uint32), ("pid", ctypes.c_int)]
    class CapData(ctypes.Structure):
        _fields_ = [("effective", ctypes.c_uint32), ("permitted", ctypes.c_uint32),
                    ("inheritable", ctypes.c_uint32)]
    header = CapHeader(0x20080522, 0)
    empty = (CapData * 2)()
    require(LIBC.capset(ctypes.byref(header), ctypes.byref(empty)) == 0, "capability sets clear")
    require(LIBC.prctl(38, 1, 0, 0, 0) == 0 and LIBC.prctl(39, 0, 0, 0, 0) == 1,
            "no_new_privs unavailable")
    require(LIBC.prctl(47, 4, 0, 0, 0) == 0, "ambient capability clear")
    with open("/proc/self/status", "r", encoding="ascii") as stream:
        status = dict(line.split(":", 1) for line in stream.read(8192).splitlines() if ":" in line)
    require(all(int(status[key].strip(), 16) == 0
                for key in ("CapInh", "CapPrm", "CapEff", "CapAmb", "CapBnd")),
            "capability drop incomplete")
    # Existing inherited FDs can exceed the soft limit; never bound closure by rlimit.
    syscall(436, ctypes.c_uint(3), ctypes.c_uint(0xFFFFFFFF), ctypes.c_uint(0))
    os.umask(0o022)
    os.execve(BWRAP, sandbox_argv(request, work), {"PATH": "/usr/bin", "LC_ALL": "C"})


def finalize_owner(request):
    nonce = request["nonce"]
    current = manager(f"ota-bootstrap-postflight-{nonce}.service", request)
    manager_limits(current, postflight=True)
    require(int(current["MainPID"]) == os.getpid() and own_cgroup() == current["ControlGroup"],
            "unbounded postflight invocation")
    require(int(current["RuntimeMaxUSec"][:-1]) * 10**9 <= request["deadline_ns"] - now(),
            "postflight unit exceeds remaining deadline")
    observed = manager(f"ota-bootstrap-owner@{nonce}.service", request)
    manager_limits(observed)
    bound = binding(request, observed)
    with ExitStack() as stack:
        def own(fd):
            stack.callback(os.close, fd)
            return fd
        journal_fd = own(directory(INVOCATIONS + "/" + nonce))
        publish(journal_fd, "finalization-consumed", canonical({"binding": bound,
                                                               "postflight": current["InvocationID"]}))
        pending = read_record(journal_fd, "artifact_pending.json")
        require(read_record(journal_fd, "request.json") == request, "retained request substitution")
        for role, expected in [("host", request["host_tree_sha256"])] + [
                (role, tree["sha256"]) for role, tree in request["trees"].items()]:
            inventory = read_record(journal_fd, role + ".inventory.json")
            require(type(inventory) is list and digest(canonical(inventory), b"ota.retained-driver.tree.v1\0") == expected,
                    "retained complete inventory substitution")
        layout = read_record(journal_fd, "git-layout.json")
        exact(layout, {"source_head", "git_directory", "external_indirection", "reviewed_delta_sha256", "untracked", "head_closure"})
        require(layout["source_head"] == request["reviewed_source"]["source_head"]
                and layout["git_directory"] == ".git" and layout["external_indirection"] is False
                and layout["reviewed_delta_sha256"] == request["reviewed_source"]["delta_sha256"]
                and layout["untracked"] == request["reviewed_source"]["untracked"], "retained Git/review provenance")
        closure = exact(layout["head_closure"], {"encoding", "head", "objects", "decompressed_bytes", "graph_sha256"})
        require(closure["encoding"] == "loose_only" and closure["head"] == layout["source_head"]
                and type(closure["objects"]) is int and 0 < closure["objects"] <= MAX_ENTRIES
                and type(closure["decompressed_bytes"]) is int and 0 <= closure["decompressed_bytes"] <= MAX_GIT_DECOMPRESSED
                and type(closure["graph_sha256"]) is str and SHA.fullmatch(closure["graph_sha256"]),
                "retained Git closure shape/bounds")
        evidence = verify_record(pending, "artifact_pending", bound)
        exact(evidence, {"exit_code", "exit_status", "forced_cleanup", "recursive_terminal",
                         "log", "placement", "artifact", "scratch_removed"})
        require(evidence["exit_code"] == os.CLD_EXITED and evidence["exit_status"] == 0
                and evidence["forced_cleanup"] is False and evidence["recursive_terminal"] is True
                and evidence["scratch_removed"] is True, "failed/pending cleanup cannot be promoted")
        require(observed["ExecMainCode"] == "1" and observed["ExecMainStatus"] == "0"
                and observed["MainPID"] == "0" and observed["Result"] == "success"
                and observed["ActiveState"] == "active" and observed["SubState"] == "exited",
                "original owner not successfully terminal")
        cg = own(cgroup_open(observed["ControlGroup"]))
        require(object_id(cg) == evidence["placement"]["cgroup_object"] and not populated(cg),
                "retained cgroup identity/quiescence mismatch")
        retention = own(directory(RETAINED + "/" + nonce))
        log = exact(evidence["log"], {"sha256", "retained_bytes", "observed_bytes", "complete",
                                      "truncated", "path", "object"})
        require(log["complete"] is True and log["truncated"] is False and log["path"] == "worker.log"
                and type(log["retained_bytes"]) is int and 0 <= log["retained_bytes"] <= MAX_LOG
                and log["observed_bytes"] == log["retained_bytes"], "incomplete raw log")
        log_reader = own(beneath(retention, "worker.log", os.O_RDONLY | os.O_NONBLOCK))
        protected(os.fstat(log_reader), stat.S_ISREG, 0o444)
        require(object_id(log_reader) == log["object"]
                and hash_file(log_reader, MAX_LOG) == (log["sha256"], log["retained_bytes"]), "raw log changed")
        artifact = evidence["artifact"]
        exact(artifact, {"length", "sha256", "candidate_object", "retained_object", "relative_path"})
        require(artifact["relative_path"] == "driver.pending", "artifact path substitution")
        driver = own(beneath(retention, "driver.pending"))
        protected(os.fstat(driver), stat.S_ISREG, 0o444)
        require(object_id(driver) == artifact["retained_object"]
                and hash_file(driver, MAX_ARTIFACT) == (artifact["sha256"], artifact["length"]),
                "retained bytes changed")
        command([SYSTEMCTL, "stop", "--", observed["Id"]], request)
        terminal = manager(observed["Id"], request)
        require(terminal["InvocationID"] == observed["InvocationID"]
                and terminal["ActiveState"] == "inactive" and terminal["MainPID"] == "0"
                and terminal["Result"] == "success", "build unit cleanup incomplete")
        try:
            reopened = cgroup_open(observed["ControlGroup"])
        except FileNotFoundError:
            reopened = None
            require(os.fstat(cg).st_nlink == 0, "unlinked exact cgroup not witnessed")
        if reopened is not None:
            try:
                require(object_id(reopened) == object_id(cg) and not populated(reopened),
                        "cgroup path reused")
            finally:
                os.close(reopened)
        before_deadline(request)
        result = {"version": 1, "state": "finalization_candidate", "previous": "artifact_pending",
                  "binding": bound, "evidence": {"artifact": artifact, "log": log, "build_unit_terminal": True,
                                                  "postflight_generation": current["InvocationID"],
                                                  "usable_driver": False, "not_proved": NOT_PROVED}}
        # One carrier, deliberately provisional. Even rename-success/fsync-failure
        # cannot certify completion: successful same-generation manager exit is a
        # required external observation, unavailable while this owner is alive.
        publish(journal_fd, "finalization-candidate.json", canonical(result))
        require(not completion_predicate(read_record(journal_fd, "finalization-candidate.json"), current, bound),
                "live finalizer cannot certify its own completion")


def completion_predicate(candidate, observed_postflight, bound):
    """Operator consumption predicate, not a new role or driver-execution authority.

    Candidate must be independently acquired from the protected journal and the
    manager observation from PID1. Caller/worker-supplied JSON is not admissible.
    The outer postflight cgroup/resource closure remains an additional open gate.
    """
    exact(candidate, {"version", "state", "previous", "binding", "evidence"})
    require(type(candidate["version"]) is int and candidate["version"] == 1
            and candidate["state"] == "finalization_candidate"
            and candidate["previous"] == "artifact_pending" and candidate["binding"] == bound,
            "completion candidate binding")
    evidence = exact(candidate["evidence"], {"artifact", "log", "build_unit_terminal",
                                             "postflight_generation", "usable_driver", "not_proved"})
    require(evidence["usable_driver"] is False and evidence["not_proved"] == NOT_PROVED
            and evidence["build_unit_terminal"] is True, "completion evidence boundary")
    require(observed_postflight["Id"] == f"ota-bootstrap-postflight-{bound['nonce']}.service"
            and observed_postflight["InvocationID"] == evidence["postflight_generation"],
            "completion manager generation")
    return (observed_postflight["MainPID"] == "0" and observed_postflight["ExecMainCode"] == "1"
            and observed_postflight["ExecMainStatus"] == "0" and observed_postflight["Result"] == "success"
            and observed_postflight["ActiveState"] in ("inactive", "active")
            and observed_postflight["SubState"] in ("dead", "exited"))


def main(argv):
    require(len(argv) in (3, 4) and argv[1] in ("build-owner", "worker", "finalize-owner"),
            "fixed internal role required")
    role = argv[1]
    require(len(argv) == (4 if role == "worker" else 3), "role argument count")
    os.umask(0o077)
    request = load_request(argv[2])
    platform_admission(request)
    if role == "build-owner":
        build_owner(request)
    elif role == "finalize-owner":
        finalize_owner(request)
    else:
        require(argv[3].isdigit(), "worker owner PID")
        worker(request, int(argv[3]))


if __name__ == "__main__":
    try:
        main(sys.argv)
    except (Refusal, OSError, ValueError, KeyError) as failure:
        print("retained-driver refused: " + type(failure).__name__, file=sys.stderr)
        raise SystemExit(1)
