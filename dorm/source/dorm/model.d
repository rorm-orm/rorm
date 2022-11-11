module dorm.model;

import dorm.annotations;
import dorm.api.db;
import dorm.declarative;
import dorm.declarative.conversion;

public import dorm.types : DormPatch;
import std.algorithm;
import std.string;
import std.sumtype;
import std.traits;

/**
 * Base Model class for all user-defined DORM Models. This implements running
 * value constructors and validators as well as defining an optional default id
 * field. The default id field is only used / available as getter/setter if
 * there is no other `@Id` or `@primaryKey` field on the Model class.
 *
 * This is only checked at compile time using the `this This` template type, so
 * only blocking invalid usage when calling it on an actual instance of the
 * user-defined Model type. When casting to a base-class that uses the built-in
 * generated ID field, it may be possible to circumvent this safety check.
 */
abstract class Model
{
    /// Auto-included ID field that's assigned on every model. May be overriden
    /// by simply defining a custom `@Id` or `@primaryKey` annotated field.
    @Id @columnName("id")
    public long _fallbackId;

    /// Gets or sets the builtin id, only available on Model classes that don't
    /// define a custom `@Id` or `@primaryKey` field.
    public ref long id(this This)() const @property @safe nothrow @nogc pure
    if (!is(This == Model) && DormFields!This[0].isBuiltinId)
    {
        return _fallbackId;
    }

    /// Default constructor. Runs value constructors. (`@constructValue` UDAs)
    this(this This)()
    {
        applyConstructValue!This();
    }

    /// Sets all fields on `this` (with the compile-time class as context) to
    /// the values in the given Patch struct.
    void applyPatch(Patch, this This)(Patch patch)
    {
        mixin ValidatePatch!(Patch, This);

        auto t = cast(This)this;
        foreach (i, ref field; patch.tupleof)
            __traits(getMember, t, Patch.tupleof[i].stringof) = field;
    }

    /// Explicitly calls value constructors. (`@constructValue` UDAs)
    /// This is already implicitly called by the default constructor and is
    /// probably not needed to be called manually.
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
            static foreach (fn; constructorFuncs)
                runValueConstructorImpl!(fn.rid)(t);
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
                    if (!runValidatorImpl!(func.validator.rid)(t))
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

private static bool runValidatorImpl(string field, T)(T t)
{
    alias fieldAlias = mixin("t." ~ field);
    alias attributes = __traits(getAttributes, fieldAlias);

    static foreach (attribute; attributes)
    {
        static if (is(attribute == validator!fn, alias fn))
        {
            return fn(mixin("t." ~ field));
        }
    }
}

private static bool runValueConstructorImpl(string field, T)(T t)
{
    alias fieldAlias = mixin("t." ~ field);
    alias attributes = __traits(getAttributes, fieldAlias);

    static foreach (attribute; attributes)
    {
        static if (is(attribute == constructValue!fn, alias fn))
        {
            mixin("t." ~ field) = fn();
            return true; // dummy return value
        }
    }
}
