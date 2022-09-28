module dorm.model;

import dorm.annotations;
import dorm.declarative.conversion;
import dorm.api.db;

public import dorm.api.db : DormPatch;
import std.traits : hasUDA;

abstract class Model
{
    /// Auto-included ID field that's assigned on every model.
    @Id @columnName("id") @modifiedIf("_modifiedId")
    public long _fallbackId;
    protected bool _modifiedId;

    public long id(this This)() const @property @safe nothrow @nogc pure
    if (DormFields!This[0].isBuiltinId)
    {
        return _fallbackId;
    }

    public long id(this This)(long value) @property @safe nothrow @nogc pure
    if (DormFields!This[0].isBuiltinId)
    {
        _modifiedId = true;
        return _fallbackId = value;
    }

    this()
    {
    }

    this(Patch, this This)(Patch patch)
    {
        applyPatch!(Patch, This)(patch);
    }

    /// Sets all fields on `this` (with the compile-time class as context) to
    /// the values in the given Patch struct.
    void applyPatch(Patch, this This)(Patch patch)
    if (hasUDA!(Patch, DormPatch!This))
    {
        foreach (i, ref field; patch.tupleof)
            __traits(getMember, cast(This)this, Patch.tupleof[i].stringof) = field;
    }
}
