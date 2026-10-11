# run one process and measure it (wall time/CPU time/peak memory/counters
# where the platform has them)

import contextlib
import ctypes
import dataclasses
import os
import shutil
import subprocess
import sys
import tempfile
import time

WINDOWS = sys.platform == "win32"


@dataclasses.dataclass
class Sample:
	wall_ns: int
	user_ns: int | None
	system_ns: int | None
	peak_memory_bytes: int | None
	# only on Windows: TSC ticks charged to the process (fine-grained CPU time, not core clocks)
	reference_cycles: int | None
	exit_code: int


if WINDOWS:
	from ctypes import wintypes

	_kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

	class _MemoryCounters(ctypes.Structure):
		_fields_ = [
			("cb", wintypes.DWORD),
			("PageFaultCount", wintypes.DWORD),
			("PeakWorkingSetSize", ctypes.c_size_t),
			("WorkingSetSize", ctypes.c_size_t),
			("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
			("QuotaPagedPoolUsage", ctypes.c_size_t),
			("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
			("QuotaNonPagedPoolUsage", ctypes.c_size_t),
			("PagefileUsage", ctypes.c_size_t),
			("PeakPagefileUsage", ctypes.c_size_t),
		]

	_kernel32.GetCurrentProcess.restype = wintypes.HANDLE
	_kernel32.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
	_kernel32.GetProcessTimes.restype = wintypes.BOOL
	_kernel32.K32GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(_MemoryCounters), wintypes.DWORD]
	_kernel32.K32GetProcessMemoryInfo.restype = wintypes.BOOL
	_kernel32.QueryProcessCycleTime.argtypes = [wintypes.HANDLE, ctypes.POINTER(ctypes.c_uint64)]
	_kernel32.QueryProcessCycleTime.restype = wintypes.BOOL
	_kernel32.GetProcessAffinityMask.argtypes = [wintypes.HANDLE, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
	_kernel32.GetProcessAffinityMask.restype = wintypes.BOOL
	_kernel32.SetProcessAffinityMask.argtypes = [wintypes.HANDLE, ctypes.c_size_t]
	_kernel32.SetProcessAffinityMask.restype = wintypes.BOOL

	def _check(ok, what):
		if not ok:
			raise OSError(ctypes.get_last_error(), f"{what} failed")

	def _filetime_ns(value):
		return ((value.dwHighDateTime << 32) | value.dwLowDateTime) * 100

	def _process_counters(handle):
		creation, exit, kernel, user = (wintypes.FILETIME() for _ in range(4))
		_check(_kernel32.GetProcessTimes(handle, ctypes.byref(creation), ctypes.byref(exit), ctypes.byref(kernel), ctypes.byref(user)), "GetProcessTimes")
		memory = _MemoryCounters()
		memory.cb = ctypes.sizeof(memory)
		_check(_kernel32.K32GetProcessMemoryInfo(handle, ctypes.byref(memory), memory.cb), "K32GetProcessMemoryInfo")
		cycles = ctypes.c_uint64()
		_check(_kernel32.QueryProcessCycleTime(handle, ctypes.byref(cycles)), "QueryProcessCycleTime")
		return _filetime_ns(user), _filetime_ns(kernel), memory.PeakWorkingSetSize, cycles.value

	def _all_cpus_mask():
		process, system = ctypes.c_size_t(), ctypes.c_size_t()
		_check(_kernel32.GetProcessAffinityMask(_kernel32.GetCurrentProcess(), ctypes.byref(process), ctypes.byref(system)), "GetProcessAffinityMask")
		return system.value


def pin(cpu):
	# children inherit the affinity of this process on both platforms
	if cpu is None:
		unpin()
	elif WINDOWS:
		_check(_kernel32.SetProcessAffinityMask(_kernel32.GetCurrentProcess(), 1 << cpu), "SetProcessAffinityMask")
	else:
		os.sched_setaffinity(0, {cpu})


def unpin():
	if WINDOWS:
		_check(_kernel32.SetProcessAffinityMask(_kernel32.GetCurrentProcess(), _all_cpus_mask()), "SetProcessAffinityMask")
	elif hasattr(os, "sched_setaffinity"):
		os.sched_setaffinity(0, range(os.cpu_count()))


def run(command, stdout=None, env=None, high_priority=False, stdin=None):
	# stdin is a file path (None reads from the null device)
	extra = {}
	if WINDOWS and high_priority:
		extra["creationflags"] = subprocess.HIGH_PRIORITY_CLASS
	merged = dict(os.environ, **env) if env else None
	with (
		tempfile.TemporaryFile() as stderr,
		open(os.devnull, "wb") if stdout is None else contextlib.nullcontext(stdout) as out,
		open(stdin, "rb") if stdin else contextlib.nullcontext(subprocess.DEVNULL) as source,
	):
		start = time.perf_counter_ns()
		process = subprocess.Popen(command, stdin=source, stdout=out, stderr=stderr, env=merged, **extra)
		if WINDOWS:
			process.wait()
			wall = time.perf_counter_ns() - start
			# NOTE: _handle is CPython's process handle (still open after wait)
			user, system, peak, cycles = _process_counters(int(process._handle))
			sample = Sample(wall, user, system, peak, cycles, process.returncode)
		else:
			_, status, usage = os.wait4(process.pid, 0)
			wall = time.perf_counter_ns() - start
			process.returncode = os.waitstatus_to_exitcode(status)
			scale = 1 if sys.platform == "darwin" else 1024
			sample = Sample(wall, round(usage.ru_utime * 1e9), round(usage.ru_stime * 1e9), usage.ru_maxrss * scale, None, process.returncode)
		if sample.exit_code != 0:
			stderr.seek(0)
			message = stderr.read().decode(errors="replace").strip()
			raise RuntimeError(f"{' '.join(map(str, command))} exited with {sample.exit_code}: {message[-2000:]}")
		return sample


# ================================================================
# HARDWARE COUNTERS (LINUX PERF)
# ================================================================

PERF_EVENTS = ("cycles", "instructions", "branches", "branch-misses", "cache-references", "cache-misses")


def perf_available():
	if WINDOWS or shutil.which("perf") is None:
		return False
	try:
		return bool(perf_counters(["true"]).get("cycles"))
	except (OSError, subprocess.CalledProcessError):
		return False


def perf_counters(command, env=None, stdin=None):
	merged = dict(os.environ, **env) if env else None
	with tempfile.TemporaryDirectory() as directory, open(stdin, "rb") if stdin else contextlib.nullcontext(subprocess.DEVNULL) as source:
		path = os.path.join(directory, "perf.csv")
		perf = ["perf", "stat", "-x", ",", "-e", ",".join(PERF_EVENTS), "-o", path, "--"] + list(command)
		subprocess.run(perf, stdin=source, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=merged, check=True)
		with open(path) as f:
			return parse_perf_csv(f.read())


def parse_perf_csv(text):
	# hybrid CPUs report cpu_core/cycles/ and cpu_atom/cycles/ separately (summed here)
	counters = {}
	for line in text.splitlines():
		fields = line.split(",")
		if line.startswith("#") or len(fields) < 3 or fields[0].startswith("<"):
			continue
		event = fields[2]
		if "/" in event:
			event = event.split("/")[1]
		event = event.split(":")[0]
		try:
			value = int(float(fields[0]))
		except ValueError:
			continue
		counters[event] = counters.get(event, 0) + value
	return counters
