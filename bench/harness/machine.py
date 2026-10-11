# machine metadata recorded with every run

import ctypes
import datetime
import os
import platform
import struct
import subprocess
import sys

from suite import ROOT

WINDOWS = sys.platform == "win32"


def describe(pinned_cpu):
	info = {
		"recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
		"os": operating_system(),
		"cpu": cpu(),
		"memory_bytes": memory_bytes(),
		"power": power(),
		"python": platform.python_version(),
		"repository": repository(),
		"pinned_cpu": pinned_cpu,
	}
	topology = info["cpu"].get("topology")
	if pinned_cpu is not None and topology:
		info["pinned_cpu_kind"] = topology.get(pinned_cpu)
	return info


def operating_system():
	info = {"system": platform.system(), "release": platform.release(), "version": platform.version(), "machine": platform.machine()}
	if WINDOWS:
		info["build"] = sys.getwindowsversion().build
	elif sys.platform == "linux":
		info["kernel"] = platform.release()
		try:
			with open("/etc/os-release") as f:
				fields = dict(line.rstrip("\n").split("=", 1) for line in f if "=" in line)
			info["distribution"] = fields.get("PRETTY_NAME", "").strip('"')
		except OSError:
			pass
		info["wsl"] = "microsoft" in platform.release().lower()
	return info


def cpu():
	info = {"logical_cpus": os.cpu_count()}
	if WINDOWS:
		import winreg
		with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
			info["model"] = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
			info["identifier"] = winreg.QueryValueEx(key, "Identifier")[0]
			info["nominal_mhz"] = winreg.QueryValueEx(key, "~MHz")[0]
		topology = windows_topology()
	elif sys.platform == "linux":
		with open("/proc/cpuinfo") as f:
			for line in f:
				key, _, value = line.partition(":")
				key = key.strip()
				if key in ("model name", "vendor_id", "cpu family", "model", "stepping", "microcode") and key not in info:
					info[key.replace(" ", "_")] = value.strip()
		topology = linux_topology()
	else:
		topology = {}
	if topology:
		info["topology"] = topology
		info["physical_cores"] = len({entry["core"] for entry in topology.values()})
	return info


def windows_topology():
	# GetSystemCpuSetInformation reports the efficiency class of each logical CPU
	# (P-cores rank above E-cores on hybrid parts)
	kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
	length = ctypes.c_ulong()
	kernel32.GetSystemCpuSetInformation(None, 0, ctypes.byref(length), None, 0)
	buffer = ctypes.create_string_buffer(length.value)
	if not kernel32.GetSystemCpuSetInformation(buffer, length, ctypes.byref(length), None, 0):
		return {}
	raw = buffer.raw[: length.value]
	entries = []
	offset = 0
	while offset < len(raw):
		size, kind = struct.unpack_from("<II", raw, offset)
		if kind == 0:
			logical, core, _, _, efficiency = struct.unpack_from("<BBBBB", raw, offset + 14)
			entries.append((logical, core, efficiency))
		offset += size
	classes = sorted({e for _, _, e in entries})
	hybrid = len(classes) > 1
	return {logical: {"core": core, "kind": ("performance" if efficiency == classes[-1] else "efficiency") if hybrid else "uniform"} for logical, core, efficiency in entries}


def linux_topology():
	topology = {}
	kinds = {}
	for kind, path in (("performance", "/sys/devices/cpu_core/cpus"), ("efficiency", "/sys/devices/cpu_atom/cpus")):
		try:
			with open(path) as f:
				for cpu_index in parse_cpu_list(f.read()):
					kinds[cpu_index] = kind
		except OSError:
			pass
	for index in range(os.cpu_count() or 0):
		try:
			with open(f"/sys/devices/system/cpu/cpu{index}/topology/core_id") as f:
				core = int(f.read())
			with open(f"/sys/devices/system/cpu/cpu{index}/topology/physical_package_id") as f:
				package = int(f.read())
		except OSError:
			continue
		topology[index] = {"core": f"{package}:{core}", "kind": kinds.get(index, "uniform")}
	return topology


def parse_cpu_list(text):
	cpus = []
	for part in text.strip().split(","):
		if "-" in part:
			first, last = part.split("-")
			cpus.extend(range(int(first), int(last) + 1))
		elif part:
			cpus.append(int(part))
	return cpus


def memory_bytes():
	if WINDOWS:
		class MemoryStatus(ctypes.Structure):
			_fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong)] + [(name, ctypes.c_ulonglong) for name in ("total", "available", "total_page", "available_page", "total_virtual", "available_virtual", "available_extended")]
		status = MemoryStatus()
		status.length = ctypes.sizeof(status)
		ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status))
		return status.total
	try:
		with open("/proc/meminfo") as f:
			for line in f:
				if line.startswith("MemTotal:"):
					return int(line.split()[1]) * 1024
	except OSError:
		pass
	return None


def power():
	if WINDOWS:
		result = subprocess.run(["powercfg", "/getactivescheme"], capture_output=True, text=True)
		return {"scheme": result.stdout.strip().split("(")[-1].rstrip(")") if result.returncode == 0 else None}
	info = {}
	for name, path in (
		("governor", "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"),
		("intel_no_turbo", "/sys/devices/system/cpu/intel_pstate/no_turbo"),
		("boost", "/sys/devices/system/cpu/cpufreq/boost"),
	):
		try:
			with open(path) as f:
				info[name] = f.read().strip()
		except OSError:
			pass
	return info


def repository():
	def git(*args):
		result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
		return result.stdout.strip() if result.returncode == 0 else None

	# tracked files only (local scratch files do not mark a run as modified)
	status = git("status", "--porcelain", "--untracked-files=no")
	return {"commit": git("rev-parse", "HEAD"), "modified": bool(status)}
