"""Signatures of functions that are NOT decompiled (Sony SDK, libc, libm, libgcc soft-float).
name -> (n_int_args, n_float_args, returns_float, dynamic)   dynamic: argument count taken from the call site."""
EXTERNS = {
    # libgcc soft-float double (doubles travel as 64-bit values in integer registers)
    'fptodp': (0, 1, False), 'dptofp': (1, 0, True), 'dpadd': (2, 0, False), 'dpsub': (2, 0, False),
    'dpmul': (2, 0, False), 'dpdiv': (2, 0, False), 'dpcmp': (2, 0, False), 'dptoli': (1, 0, False),
    'dptoul': (1, 0, False), 'fptoui': (0, 1, False), '__divdi3': (2, 0, False),
    # libm
    'sqrtf': (0, 1, True), 'sinf': (0, 1, True), 'cosf': (0, 1, True), 'tanf': (0, 1, True), 'acosf': (0, 1, True),
    'asinf': (0, 1, True), 'floorf': (0, 1, True), 'atan2f': (0, 2, True), 'powf': (0, 2, True),
    'sqrt': (1, 0, False), 'atan': (1, 0, False), 'floor': (1, 0, False), 'atan2': (2, 0, False),
    'pow': (2, 0, False),
    # libc
    'memset': (3, 0, False), 'memcpy': (3, 0, False), 'bcopy': (3, 0, False), 'strcpy': (2, 0, False),
    'strncpy': (3, 0, False), 'strlen': (1, 0, False), 'strcmp': (2, 0, False), 'strncmp': (3, 0, False),
    'strcat': (2, 0, False), 'strchr': (2, 0, False), 'strcasecmp': (2, 0, False), 'atoi': (1, 0, False),
    'malloc': (1, 0, False), 'free': (1, 0, False), 'exit': (1, 0, False), 'Exit': (1, 0, False),
    'vsprintf': (3, 0, False), '__main': (0, 0, False),
    # Sony vector math
    'sceVu0MulMatrix': (3, 0, False), 'sceVu0CopyMatrix': (2, 0, False), 'sceVu0UnitMatrix': (1, 0, False),
    'sceVu0InversMatrix': (2, 0, False), 'sceVu0TransposeMatrix': (2, 0, False),
    # kernel
    'WaitSema': (1, 0, False), 'SignalSema': (1, 0, False), 'iSignalSema': (1, 0, False),
    'CreateSema': (1, 0, False), 'iPollSema': (1, 0, False), 'FlushCache': (1, 0, False),
    'InvalidDCache': (2, 0, False),
}
DYNAMIC = {'sprintf', 'scePrintf', 'sscanf', 'sceSdRemote', 'sceSifCallRpc'}
