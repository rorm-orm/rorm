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
                int type;
                ValidatorRef validator;
                Choices choices;
                ModelFormat.Field field;
            }
            Ret[] ret;
            foreach (ref field; DormFields!This)
            {
                foreach (ref annotation; field.internalAnnotations)
                {
                    annotation.match!(
                        (ValidatorRef validator) {
                            ret ~= Ret(0, validator, Choices.init, field);
                        },
                        (_) {}
                    );
                }
                foreach (ref annotation; field.annotations)
                {
                    annotation.value.match!(
                        (Choices choices) {
                            ret ~= Ret(1, ValidatorRef.init, choices, field);
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
            static foreach (func; validatorFuncs)
            {{
                static if (func.type == 0)
                {
                    // validator
                    if (!func.validator.callback(t))
                        failedFields ~= func.field;
                }
                else static if (func.type == 1)
                {
                    // choices
                    alias fieldRef = __traits(getMember, cast(This)this, func.field.sourceColumn);
                    alias FieldT = typeof(fieldRef);

                    static if (is(FieldT == enum))
                    {
                        // we assume that the enum value is simply valid for now.
                    }
                    else static if (is(FieldT : string))
                    {
                        import std.algorithm : canFind;

                        if (!func.choices.choices.canFind(__traits(getMember, cast(This)this, func.field.sourceColumn)))
                            failedFields ~= func.field;
                    }
                    else static assert(false,
                        "Missing DORM implementation: Cannot validate inferred @choices from "
                        ~ This.stringof ~ " -> " ~ func.field.sourceColumn ~ " of type "
                        ~ FieldT.stringof
                        ~ " (choices should only apply to string and enums, don't know what to do with this type)");
                }
                else static assert(false);
            }}
        }
        return failedFields;
    }
}
