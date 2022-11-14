module dorm.lib.ffi;

version (unittest)
{
    // no DebugFFI in unittest, dunno it doesn't compile
}
else
{
    // version = DebugFFI;
}

version (DebugFFI)
{
    public import dorm.lib.ffi_wrap;

    enum DormFFITrace = true;
}
else
{
    public import dorm.lib.ffi_impl;

    enum DormFFITrace = false;
}

