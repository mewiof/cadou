// how the C runtime sees stdout. The harness sends it to the null device:
//
//   python -c "import subprocess, sys; subprocess.run([sys.argv[1]], stdout=subprocess.DEVNULL)" PROBE.exe
//
// prints to stderr. Windows only

#include <io.h>
#include <stdio.h>
#include <windows.h>

int main(void) {
	DWORD type = GetFileType(GetStdHandle(STD_OUTPUT_HANDLE));
	fprintf(stderr, "_isatty(stdout) = %d\nGetFileType(stdout) = %lu (FILE_TYPE_CHAR = %d, FILE_TYPE_DISK = %d, FILE_TYPE_PIPE = %d)\n", _isatty(_fileno(stdout)), type, FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE);
	return 0;
}
