module dorm.model;

import dorm.annotations;
import dorm.declarative;
import dorm.declarative.conversion;
import dorm.api.db;

public import dorm.api.db : DormPatch;
import std.sumtype;
import std.traits;

abstract class Model
{
    /// Auto-included ID field that's assigned on every model.
    @Id @columnName("id") @modifiedIf("_modifiedId")
    public long _fallbackId;
    @ignored
    public bool _modifiedId;

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

    this(this This)()
    {
        applyConstructValue!This();
    }

    /// Sets all fields on `this` (with the compile-time class as context) to
    /// the values in the given Patch struct.
    void applyPatch(Patch, this This)(Patch patch)
    if (hasUDA!(Patch, DormPatch!This))
    {
        auto t = cast(This)this;
        foreach (i, ref field; patch.tupleof)
        {
            __traits(getMember, t, Patch.tupleof[i].stringof) = field;
            alias mods = getUDAs!(__traits(getMember, t, Patch.tupleof[i].stringof), modifiedIf);
            static foreach (m; mods)
                __traits(getMember, t, m.field) = m.equalsTo;
        }
    }

    void applyConstructValue(this This)()
    {
        enum constructorFuncs = {
            ConstructValueRef[] ret;
            foreach (ref field; DormFields!This)
            {
                foreach (ref annotation; field.internalAnnotations)
                {
                    annotation.match!(
                        (ConstructValueRef ctor) {
                            ret ~= ctor;
                        },
                        (_) {}
                    );
                }
            }
            return ret;
        }();
        static if (constructorFuncs.length)
        {
            auto t = cast(This)this;
            foreach (ref fn; constructorFuncs)
                fn.callback(t);
        }
    }

    /// Runs the defined `@validator` functions on fields, returns a list of
    /// failed fields.
    ModelFormat.Field[] runValidators(this This)()
    {
        ModelFormat.Field[] failedFields;
        enum validatorFuncs = {
            struct Ret {
                ValidatorRef validator;
                ModelFormat.Field field;
            }
            Ret[] ret;
            foreach (ref field; DormFields!This)
            {
                foreach (ref annotation; field.internalAnnotations)
                {
                    annotation.match!(
                        (ValidatorRef validator) {
                            ret ~= Ret(validator, field);
                        },
                        (_) {}
                    );
                }
            }
            return ret;
        }();
        static if (validatorFuncs.length)
        {
            auto t = cast(This)this;
            foreach (func; validatorFuncs)
                if (!func.validator.callback(t))
                    failedFields ~= func.field;
        }
        return failedFields;
    }
}
