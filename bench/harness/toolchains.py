# pinned toolchains (bench/toolchains.toml): fetch/locate/compile commands

import dataclasses
import hashlib
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.request
import zipfile

from suite import ROOT

PINS = ROOT / "bench" / "toolchains.toml"
STORE = ROOT / ".toolchains"
MARKER = ".pinned"
WINDOWS = sys.platform == "win32"
EXE = ".exe" if WINDOWS else ""


def host_key():
	arch = {"amd64": "x86_64", "x86_64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}[platform.machine().lower()]
	system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}[platform.system()]
	return f"{arch}-{system}"


def load_pins():
	with open(PINS, "rb") as f:
		return tomllib.load(f)


def install_dir(name, pin):
	return STORE / f"{name}-{pin['version']}"


# ================================================================
# FETCHING
# ================================================================

def sha256_of(path):
	digest = hashlib.sha256()
	with open(path, "rb") as f:
		for chunk in iter(lambda: f.read(1 << 20), b""):
			digest.update(chunk)
	return digest.hexdigest()


def download(url, sha256):
	downloads = STORE / "downloads"
	downloads.mkdir(parents=True, exist_ok=True)
	path = downloads / url.rsplit("/", 1)[1]
	if path.exists() and sha256_of(path) == sha256:
		return path
	print(f"downloading {url}")
	partial = path.with_name(path.name + ".part")
	with urllib.request.urlopen(url) as response, open(partial, "wb") as f:
		shutil.copyfileobj(response, f, 1 << 20)
	actual = sha256_of(partial)
	if actual != sha256:
		partial.unlink()
		raise SystemExit(f"sha256 mismatch for {url}\n  expected {sha256}\n  got      {actual}")
	partial.replace(path)
	return path


def strip_top(name):
	# every pinned archive has one top-level directory. It gets stripped here so
	# files land in place (Defender holds a freshly unpacked tree hostage and the
	# rename fails with Access denied)
	parts = pathlib.PurePosixPath(name).parts
	if ".." in parts:
		raise SystemExit(f"refusing archive entry {name!r}")
	return "/".join(parts[1:])


def unpack(archive, destination):
	if destination.exists():
		shutil.rmtree(destination)
	destination.mkdir(parents=True)
	print(f"unpacking {archive.name}")
	if not archive.name.endswith((".zip", ".tar.xz", ".tar.gz")):
		# a single file such as a header
		shutil.copyfile(archive, destination / archive.name)
	elif archive.name.endswith(".zip"):
		with zipfile.ZipFile(archive) as z:
			for info in z.infolist():
				name = strip_top(info.filename)
				if not name:
					continue
				target = destination / name
				if info.is_dir():
					target.mkdir(parents=True, exist_ok=True)
					continue
				target.parent.mkdir(parents=True, exist_ok=True)
				with z.open(info) as source, open(target, "wb") as f:
					shutil.copyfileobj(source, f, 1 << 20)
	else:
		# stream mode (seeking in a compressed tar decompresses it all over again)
		with tarfile.open(archive, "r|*") as t:
			for member in t:
				member.name = strip_top(member.name)
				if not member.name:
					continue
				if member.islnk():
					member.linkname = strip_top(member.linkname)
				t.extract(member, destination, filter="data")


def fetch(names=None):
	pins = load_pins()
	host = host_key()
	for name in names or list(pins):
		if name not in pins:
			raise SystemExit(f"unknown toolchain {name!r} (known: {', '.join(pins)})")
		pin = pins[name]
		if name == "rust":
			rustup = find_rustup()
			if rustup is None:
				print(f"rust: rustup not found. Install it from https://rustup.rs, then: rustup toolchain install {pin['version']} --profile minimal")
				continue
			subprocess.run([rustup, "toolchain", "install", pin["version"], "--profile", "minimal"], check=True)
			continue
		# a pin with its own url fits every host
		entry = pin if "url" in pin else pin.get(host)
		if entry is None:
			print(f"{name}: no pinned build for {host}, PATH will be used")
			continue
		target = install_dir(name, pin)
		marker = target / MARKER
		if marker.exists() and marker.read_text().strip() == entry["sha256"]:
			print(f"{name} {pin['version']}: present")
			continue
		unpack(download(entry["url"], entry["sha256"]), target)
		marker.write_text(entry["sha256"] + "\n")
		print(f"{name} {pin['version']}: {target.relative_to(ROOT)}")


# ================================================================
# LOCATING
# ================================================================

@dataclasses.dataclass
class Tool:
	path: str
	pinned: bool
	version: str
	env: dict = dataclasses.field(default_factory=dict)


def find_rustup():
	found = shutil.which("rustup")
	if found:
		return found
	fallback = pathlib.Path.home() / ".cargo" / "bin" / f"rustup{EXE}"
	return str(fallback) if fallback.exists() else None


def version_text(command, env=None):
	merged = dict(os.environ, **(env or {}))
	result = subprocess.run(command, capture_output=True, text=True, env=merged)
	return (result.stdout or result.stderr).strip()


def first_version_line(text):
	lines = text.splitlines()
	return next((line.strip() for line in lines if any(c.isdigit() for c in line)), lines[0] if lines else "")


def pinned_bin(name, pins, host, use_pins):
	pin = pins[name]
	if not use_pins or (host not in pin and "url" not in pin):
		return None
	directory = install_dir(name, pin)
	if not (directory / MARKER).exists():
		return None
	return directory / pin.get("bin", ".")


def locate(program, directory):
	if directory is not None:
		candidate = directory / f"{program}{EXE}"
		if candidate.exists():
			return str(candidate), True
	found = shutil.which(program)
	return found, False


class Toolchains:
	def __init__(self, use_pins=True):
		pins = load_pins()
		host = host_key()
		self.pins = pins
		self.tools = {}

		llvm = pinned_bin("llvm", pins, host, use_pins)
		for program in ("clang", "clang++", "llvm-strip"):
			path, pinned = locate(program, llvm)
			if path:
				self.tools[program] = Tool(path, pinned, first_version_line(version_text([path, "--version"])))

		gcc = pinned_bin("gcc", pins, host, use_pins)
		for program in ("gcc", "g++", "strip"):
			path, pinned = locate(program, gcc)
			if path:
				self.tools[program] = Tool(path, pinned, first_version_line(version_text([path, "--version"])))

		# khash.h for the C programs (no fallback: C has no hash table to fall back to)
		klib = pinned_bin("klib", pins, host, use_pins)
		if klib is not None:
			self.tools["klib"] = Tool(str(klib), True, f"{pins['klib']['version']} ({pins['klib']['commit'][:7]})")

		zig = pinned_bin("zig", pins, host, use_pins)
		path, pinned = locate("zig", zig)
		if path:
			self.tools["zig"] = Tool(path, pinned, version_text([path, "version"]))

		# rustup picks the pinned release through RUSTUP_TOOLCHAIN
		rustc = shutil.which("rustc") or str(pathlib.Path.home() / ".cargo" / "bin" / f"rustc{EXE}")
		if pathlib.Path(rustc).exists():
			env = {"RUSTUP_TOOLCHAIN": pins["rust"]["version"]} if use_pins and find_rustup() else {}
			version = version_text([rustc, "-vV"], env)
			pinned = bool(env) and f"release: {pins['rust']['version']}" in version
			self.tools["rustc"] = Tool(rustc, pinned, version, env)

	def require(self, name):
		tool = self.tools.get(name)
		if tool is None:
			raise SystemExit(f"{name} not found: run `python bench/harness/bench.py fetch` or put it on PATH")
		return tool

	def describe(self):
		return {name: {"path": display_path(tool.path), "pinned": tool.pinned, "version": tool.version} for name, tool in self.tools.items()}


def display_path(text):
	# keep the user's home directory out of committed results
	text = str(text)
	for base, label in ((ROOT, "."), (pathlib.Path.home(), "~")):
		text = text.replace(str(base), label)
	return text


# ================================================================
# COMPILE COMMANDS
# ================================================================

def compile_command(toolchains, implementation, output, cache):
	# flags follow docs/decisions/0003-benchmark-methodology.md
	variant = implementation.variant
	source = str(implementation.source)
	include = [f"-I{toolchains.tools['klib'].path}"] if variant.language == "c" and "klib" in toolchains.tools else []
	if variant.toolchain == "llvm":
		compiler = toolchains.require("clang" if variant.language == "c" else "clang++")
		standard = "-std=c17" if variant.language == "c" else "-std=c++23"
		command = [compiler.path, standard, "-O3", "-march=native", "-flto", "-ffp-contract=off", "-fuse-ld=lld", *include, source, "-o", str(output)]
		if not WINDOWS:
			command.append("-lm")
		return command, compiler.env
	if variant.toolchain == "gcc":
		compiler = toolchains.require("gcc" if variant.language == "c" else "g++")
		standard = "-std=c17" if variant.language == "c" else "-std=c++23"
		command = [compiler.path, standard, "-O3", "-march=native", "-flto", "-flto-partition=one", "-ffp-contract=off", *include, source, "-o", str(output)]
		if WINDOWS:
			# -static keeps stray MinGW runtime DLLs on PATH out of it. libstdc++exp holds
			# the Windows terminal code of std::print (the link fails without it)
			command.append("-static")
			if variant.language == "cpp":
				command.append("-lstdc++exp")
		command.append("-lm")
		return command, compiler.env
	if variant.toolchain == "rust":
		compiler = toolchains.require("rustc")
		command = [
			compiler.path, "--edition=2024",
			"-C", "opt-level=3", "-C", "codegen-units=1", "-C", "lto=fat", "-C", "panic=abort",
			"-C", "target-cpu=native", "-C", "strip=debuginfo",
			source, "-o", str(output),
		]
		return command, compiler.env
	if variant.toolchain == "zig":
		compiler = toolchains.require("zig")
		command = [
			compiler.path, "build-exe", source, "-O", "ReleaseFast", "-mcpu=native", f"-femit-bin={output}",
			"--cache-dir", str(cache / "local"), "--global-cache-dir", str(cache / "global"),
		]
		return command, compiler.env
	raise ValueError(variant.toolchain)


def strip_command(toolchains, binary, output):
	tool = toolchains.tools.get("llvm-strip") or toolchains.tools.get("strip")
	if tool is None:
		return None
	return [tool.path, "--strip-all", str(binary), "-o", str(output)]


def target_cpu(toolchains):
	# what each compiler family resolves -march=native to on this machine
	found = {}
	clang = toolchains.tools.get("clang")
	if clang:
		text = version_text([clang.path, "-march=native", "-###", "-x", "c", "-c", os.devnull])
		marker = '"-target-cpu" "'
		if marker in text:
			found["clang"] = text.split(marker, 1)[1].split('"', 1)[0]
	gcc = toolchains.tools.get("gcc")
	if gcc:
		for line in version_text([gcc.path, "-march=native", "-Q", "--help=target"]).splitlines():
			parts = line.split()
			if len(parts) == 2 and parts[0] == "-march=":
				found["gcc"] = parts[1]
	return found
