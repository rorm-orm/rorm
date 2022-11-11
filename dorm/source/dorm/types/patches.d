module dorm.types.patches;

import dorm.api.db;
import dorm.declarative.conversion;
import dorm.model;

import std.meta;
import std.traits;

/**
 * UDA to mark patch structs with, to make selecting them easier.
 *
 * Examples:
 * ---
 * @DormPatch!User
 * struct UserSelection
 * {
 *     string username;
 * }
 * db.select!UserSelection;
 *
 * // is equivalent to
 * struct UserSelection
 * {
 *     string username;
 * }
 * db.select!(User, UserSelection);
 *
 * // is equivalent to
 * db.select!(User, "username");
 *
 * // is equivalent to
 * db.select!(User, User.username);
 *
 * // is equivalent to
 * db.select!(User, Tuple!(string, "username"));
 * ---
 */
struct DormPatch(User)
{
}

/// Helper to validate if a DormPatch annotated field is valid for the given
/// Model type.
mixin template ValidatePatch(Patch, TModel)
{
	import std.traits : hasUDA;
	import dorm.annotations : isDormFieldAttribute;

	static if (!isImplicitPatch!(Patch, TModel))
	{
		static assert (hasUDA!(Patch, DormPatch!TModel), "Patch struct " ~ Patch.stringof
			~ " must be annoated using DormPatch!(" ~ TModel.stringof ~ ") exactly once!");

		static foreach (i, field; Patch.tupleof)
		{
			static assert (__traits(hasMember, TModel.init, Patch.tupleof[i].stringof),
				"\n" ~ SourceLocation(__traits(getLocation, Patch.tupleof[i])).toErrorString
					~ "Patch field `" ~ Patch.tupleof[i].stringof
					~ "` is not defined on DB Type " ~ TModel.stringof
					~ ".\n\tAvailable usable fields: "
						~ DormFields!TModel.map!(f => f.sourceColumn).join(", "));

			static foreach (attr; __traits(getAttributes, Patch.tupleof[i]))
				static assert (!isDormFieldAttribute!attr,
					"\n" ~ SourceLocation(__traits(getLocation, Patch.tupleof[i])).toErrorString
						~ "Patch field `" ~ Patch.tupleof[i].stringof
						~ "` defines DB-related annotations, which is not"
						~ " supported. Put annotations on the Model field instead!");
		}
	}
}

/// Checks if Patch is an implicit patch of TModel. That is a child struct of
/// the given model class.
enum isImplicitPatch(Patch, TModel) = is(__traits(parent, Patch) == TModel);

/// Checks if the given type Patch is either an implicit or an explicit patch.
enum isSomePatch(Patch) =
	is(Patch == struct) && (is(__traits(parent, Patch) : Model)
		|| hasUDA!(Patch, DormPatch));
enum isSomePatch(alias other) = false;

static template PatchFromIdOrPatch(alias idOrPatch)
{
	static if (is(idOrPatch : Model))
		alias PatchFromIdOrPatch = idOrPatch;
	else static if (isSomePatch!idOrPatch)
		alias PatchFromIdOrPatch = idOrPatch;
	else static if (is(__traits(parent, idOrPatch) : Model)
		|| isSomePatch!(__traits(parent, idOrPatch)))
		alias PatchFromIdOrPatch = __traits(parent, idOrPatch);
	else
		static assert(false, "Invalid id or patch: " ~ idOrPatch.stringof);
}

static template IdAliasFromIdOrPatch(alias idOrPatch)
{
	static if (is(idOrPatch : Model))
		alias IdAliasFromIdOrPatch =
			__traits(getMember, idOrPatch, DormPrimaryKey!idOrPatch.sourceColumn);
	else static if (isSomePatch!idOrPatch)
		alias IdAliasFromIdOrPatch =
			__traits(getMember, idOrPatch,
				DormPrimaryKey!(ModelFromSomePatch!idOrPatch).sourceColumn);
	else static if (is(__traits(parent, idOrPatch) : Model)
		|| isSomePatch!(__traits(parent, idOrPatch)))
		alias IdAliasFromIdOrPatch = idOrPatch;
	else
		static assert(false, "Invalid id or patch: " ~ idOrPatch.stringof);
}

static template ModelFromIdOrModel(alias idOrModel)
{
	static if (is(idOrModel : Model))
		alias ModelFromIdOrModel = idOrModel;
	else static if (is(__traits(parent, idOrModel) : Model))
		alias ModelFromIdOrModel = __traits(parent, idOrModel);
	else
		static assert(false, "Invalid id or model: " ~ idOrModel.stringof);
}

static template IdAliasFromIdOrModel(alias idOrModel)
{
	static if (is(idOrModel : Model))
		alias IdAliasFromIdOrModel =
			__traits(getMember, idOrModel, DormPrimaryKey!idOrModel.sourceColumn);
	else static if (is(__traits(parent, idOrModel) : Model))
		alias IdAliasFromIdOrModel = idOrModel;
	else
		static assert(false, "Invalid id or model: " ~ idOrModel.stringof);
}

static template ModelFromSomePatch(TModel)
{
	static if (is(__traits(parent, TModel) : Model))
		alias ModelFromSomePatch = __traits(parent, TModel);
	else
		alias ModelFromSomePatch = DBType!TModel;
}

template DBType(Selection...)
{
	static assert(Selection.length >= 1);

	static if (Selection.length > 1)
	{
		alias DBType = GetModelDBType!(Selection[0]);
	}
	else
	{
		alias PatchAttrs = getUDAs!(Selection[0], DormPatch);
		static if (PatchAttrs.length == 0)
			alias DBType = GetModelDBType!(Selection[0]);
		else static if (PatchAttrs.length == 1)
		{
			static if (is(PatchAttrs[0] == DormPatch!T, T))
			{
				mixin ValidatePatch!(Selection[0], T);
				alias DBType = T;
			}
			else
				static assert(false, "internal template error");
		}
		else
			static assert(false, "Cannot annotate DormPatch struct with multiple DormPatch UDAs.");
	}
}

template GetModelDBType(T)
{
	import dorm.model : Model;

	static if (is(T : Model))
		alias GetModelDBType = T;
	else static if (is(__traits(parent, T) : Model))
		alias GetModelDBType = __traits(parent, T);
	else
		static assert(false, "Passed in Non-Model Type where a Model was expected");
}

template SelectType(T, Selection...)
{
	import std.traits : isAggregateType;

	static if (Selection.length == 0)
		alias SelectType = T;
	else static if (Selection.length == 1 && isAggregateType!(Selection[0]))
		alias SelectType = Selection[0];
	else
		alias SelectType = BuildFieldsTuple!(T, Selection);
}

template BuildFieldsTuple(T, Selection...)
{
	import std.meta : AliasSeq;
	import std.typecons : Tuple;

	alias TupleArgs = AliasSeq!();
	static foreach (alias Field; Selection)
	{
		static if (__traits(compiles, { string s = Field; }))
			alias TupleArgs = AliasSeq!(TupleArgs, typeof(__traits(getMember, T, Field)), Field);
		else
			alias TupleArgs = AliasSeq!(TupleArgs, typeof(Field), __traits(identifier, Field));
	}
	alias BuildFieldsTuple = Tuple!TupleArgs;
}

template FilterLayoutFields(T, TSelect)
{
	static if (is(T == TSelect))
		enum FilterLayoutFields = DormFields!T;
	else static if (is(TSelect : Model))
		static assert(false, "Cannot filter for fields of Model class on a Model class");
	else
		enum FilterLayoutFields = filterFields!T(selectionFieldNames!(T, TSelect));
}

private auto filterFields(T)(string[] sourceNames...)
{
	import std.algorithm : canFind;

	enum fields = DormFields!T;
	typeof(fields) ret;
	foreach (ref field; fields)
		if (sourceNames.canFind(field.sourceColumn))
			ret ~= field;
	return ret;
}

private string[] selectionFieldNames(T, TSelect)(string prefix = "")
{
	import std.algorithm : canFind;

	enum layout = DormLayout!T;

	string[] ret;
	static foreach (field; __traits(allMembers, TSelect))
	{
		static if (layout.embeddedStructs.canFind(field))
			ret ~= selectionFieldNames!(T, typeof(__traits(getMember, TSelect, field)))(
				prefix ~ field ~ ".");
		else
			ret ~= (prefix ~ field);
	}
	return ret;
}
