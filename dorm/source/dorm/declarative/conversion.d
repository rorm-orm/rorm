/**
 * This module converts a package containing D Model definitions to
 * $(REF SerializedModels, dorm,declarative)
 */
module dorm.declarative.conversion;

import dorm.annotations;
import dorm.declarative;
import dorm.model;

import std.conv;
import std.datetime;
import std.meta;
import std.traits;
import std.typecons;

version (unittest) import dorm.model;

/** 
 * Entry point to the Model (class) to SerializedModels (declarative) conversion
 * code. Manually calling this should not be neccessary as the
 * $(REF RegisterModels, dorm,declarative,entrypoint) mixin will call this instead.
 */
SerializedModels processModelsToDeclarations(alias mod)()
{
	SerializedModels ret;

	static foreach (member; __traits(allMembers, mod))
	{
		static if (__traits(compiles, is(__traits(getMember, mod, member) : Model))
			&& is(__traits(getMember, mod, member) : Model)
			&& !__traits(isAbstractClass, __traits(getMember, mod, member)))
		{
			processModel!(
				__traits(getMember, mod, member),
				SourceLocation(__traits(getLocation, __traits(getMember, mod, member)))
			)(ret);
		}
	}

	return ret;
}

private void processModel(TModel : Model, SourceLocation loc)(
	ref SerializedModels models)
{
	ModelFormat serialized = DormLayout!TModel;
	serialized.definedAt = loc;
	models.models ~= serialized;
}

private enum DormLayoutImpl(TModel : Model) = function() {
	ModelFormat serialized;
	serialized.tableName = TModel.stringof.toSnakeCase;

	alias attributes = __traits(getAttributes, TModel);

	static foreach (attribute; attributes)
	{
		static if (isDormModelAttribute!attribute)
		{
			static assert(false, "Unsupported attribute " ~ attribute.stringof ~ " on " ~ TModel.stringof ~ "." ~ member);
		}
	}

	string errors;

	static foreach (field; LogicalFields!TModel)
	{
		static if (__traits(getProtection, __traits(getMember, TModel, field)) == "public")
		{
			try
			{
				processField!(TModel, field, field)(serialized);
			}
			catch (Exception e)
			{
				errors ~= "\n\t" ~ e.msg;
			}
		}
	}

	errors ~= serialized.lint("\n\t");

	struct Ret
	{
		string errors;
		ModelFormat ret;
	}

	return Ret(errors, serialized);
}();

template DormLayout(TModel : Model)
{
	private enum Impl = DormLayoutImpl!TModel;
	static if (Impl.errors.length)
		static assert(false, "Model Definition contains errors:" ~ Impl.errors);
	else
		enum DormLayout = Impl.ret;
}

enum DormFields(TModel : Model) = DormLayout!TModel.fields;

enum DormFieldIndex(TModel : Model, string sourceName) = findFieldIdx(DormFields!TModel, sourceName, TModel.stringof);
enum hasDormField(TModel : Model, string sourceName) = DormFieldIndex!(TModel, sourceName) != -1;
enum DormField(TModel : Model, string sourceName) = DormFields!TModel[DormFieldIndex!(TModel, sourceName)];

private auto findFieldIdx(ModelFormat.Field[] fields, string name, string modelName)
{
	foreach (i, ref field; fields)
		if (field.sourceColumn == name)
			return i;
	return -1;
}

template LogicalFields(TModel)
{
	static if (is(TModel : Model))
		alias Classes = AliasSeq!(TModel, BaseClassesTuple!TModel);
	else
		alias Classes = AliasSeq!(TModel);
	alias LogicalFields = AliasSeq!();

	static foreach_reverse (Class; Classes)
		LogicalFields = AliasSeq!(LogicalFields, FieldNameTuple!Class);
}

private void processField(TModel, string fieldName, string directFieldName)(ref ModelFormat serialized)
{
	import uda = dorm.annotations;

	alias fieldAlias = __traits(getMember, TModel, directFieldName);

	alias attributes = __traits(getAttributes, fieldAlias);

	bool include = true;
	ModelFormat.Field field;

	field.definedAt = SourceLocation(__traits(getLocation, fieldAlias));
	field.columnName = directFieldName.toSnakeCase;
	field.sourceType = typeof(fieldAlias).stringof;
	field.sourceColumn = fieldName;
	field.type = guessDBType!(typeof(fieldAlias));

	bool explicitNotNull = false;
	bool nullable = false;
	bool mustBeNullable = false;
	bool hasNonNullDefaultValue = false;
	static if (is(typeof(fieldAlias) == Nullable!T, T)
		|| is(typeof(fieldAlias) : Model))
		nullable = true;

	static if (is(typeof(fieldAlias) == enum))
		field.annotations ~= DBAnnotation(Choices([
				__traits(allMembers, typeof(fieldAlias))
			]));

	void setDefaultValue(T)(T value)
	{
		static if (__traits(compiles, value !is null))
		{
			if (value !is null)
				hasNonNullDefaultValue = true;
		}
		else
			hasNonNullDefaultValue = true;

		field.annotations ~= DBAnnotation(defaultValue(value));
	}

	static foreach (attribute; attributes)
	{
		static if (__traits(isSame, attribute, uda.autoCreateTime))
		{
			field.type = ModelFormat.Field.DBType.datetime;
			field.annotations ~= DBAnnotation(AnnotationFlag.autoCreateTime);
			hasNonNullDefaultValue = true;
		}
		else static if (__traits(isSame, attribute, uda.autoUpdateTime))
		{
			field.type = ModelFormat.Field.DBType.datetime;
			field.annotations ~= DBAnnotation(AnnotationFlag.autoUpdateTime);
			mustBeNullable = true;
		}
		else static if (__traits(isSame, attribute, uda.timestamp))
		{
			field.type = ModelFormat.Field.DBType.datetime;
			mustBeNullable = true;
		}
		else static if (__traits(isSame, attribute, uda.primaryKey))
		{
			nullable = true;
			field.annotations ~= DBAnnotation(AnnotationFlag.primaryKey);
			if (!field.isBuiltinId)
			{
				if (serialized.fields[0].isBuiltinId)
					serialized.fields = serialized.fields[1 .. $];

				foreach (other; serialized.fields)
				{
					if (other.isPrimaryKey)
						throw new Exception("Duplicate primary key found in Model " ~ TModel.stringof
							~ ":\n- first defined here:\n"
							~ other.sourceColumn ~ " in " ~ other.definedAt.toString
							~ "\n- then attempted to redefine here:\n"
							~ typeof(fieldAlias).stringof ~ " " ~ fieldName ~ " in " ~ field.definedAt.toString
							~ "\nMaybe you wanted to define a `TODO: compositeKey`?");
				}
			}
		}
		else static if (__traits(isSame, attribute, uda.autoIncrement))
		{
			field.annotations ~= DBAnnotation(AnnotationFlag.autoIncrement);
			hasNonNullDefaultValue = true;
		}
		else static if (__traits(isSame, attribute, uda.unique))
		{
			field.annotations ~= DBAnnotation(AnnotationFlag.unique);
		}
		else static if (__traits(isSame, attribute, uda.embedded))
		{
			static if (is(typeof(fieldAlias) == struct))
			{
				serialized.embeddedStructs ~= fieldName;
				alias TSubModel = typeof(fieldAlias);
				static foreach (subfield; LogicalFields!TSubModel)
				{
					static if (__traits(getProtection, __traits(getMember, TSubModel, subfield)) == "public")
					{
						processField!(TSubModel, fieldName ~ "." ~ subfield, subfield)(serialized);
					}
				}
			}
			else
				static assert(false, "@embedded is only supported on structs");
			include = false;
		}
		else static if (__traits(isSame, attribute, uda.ignored))
		{
			include = false;
		}
		else static if (__traits(isSame, attribute, uda.notNull))
		{
			explicitNotNull = true;
		}
		else static if (is(attribute == constructValue!fn, alias fn))
		{
			field.internalAnnotations ~= InternalAnnotation(ConstructValueRef(fieldName));
		}
		else static if (is(attribute == validator!fn, alias fn))
		{
			field.internalAnnotations ~= InternalAnnotation(ValidatorRef(fieldName));
		}
		else static if (__traits(isSame, attribute, defaultFromInit))
		{
			static if (is(TModel == struct))
			{
				setDefaultValue(__traits(getMember, TModel.init, directFieldName));
			}
			else
			{
				setDefaultValue(__traits(getMember, new TModel(), directFieldName));
			}
		}
		else static if (is(typeof(attribute) == maxLength)
			|| is(typeof(attribute) == DefaultValue!T, T)
			|| is(typeof(attribute) == index))
		{
			static if (is(typeof(attribute) == DefaultValue!U, U)
				&& !is(U == typeof(null)))
			{
				hasNonNullDefaultValue = true;
			}
			field.annotations ~= DBAnnotation(attribute);
		}
		else static if (is(typeof(attribute) == Choices))
		{
			field.type = ModelFormat.Field.DBType.choices;
			field.annotations ~= DBAnnotation(attribute);
		}
		else static if (is(typeof(attribute) == columnName))
		{
			field.columnName = attribute.name;
		}
		else static if (isDormFieldAttribute!attribute)
		{
			static assert(false, "Unsupported attribute " ~ attribute.stringof ~ " on " ~ TModel.stringof ~ "." ~ fieldName);
		}
	}

	if (include)
	{
		if (!nullable || explicitNotNull)
		{
			if (mustBeNullable && !hasNonNullDefaultValue)
			{
				throw new Exception(field.sourceReferenceName(TModel.stringof)
					~ " may be null. Change it to Nullable!(" ~ typeof(fieldAlias).stringof
					~ ") or annotate with defaultValue, autoIncrement or autoCreateTime");
			}
			field.annotations = DBAnnotation(AnnotationFlag.notNull) ~ field.annotations;
		}

		// https://github.com/myOmikron/drorm/issues/8
		if (field.hasFlag(AnnotationFlag.autoIncrement) && !field.hasFlag(AnnotationFlag.primaryKey))
				throw new Exception(field.sourceReferenceName(TModel.stringof)
					~ " has @autoIncrement annotation, but is missing required @primaryKey annotation.");

		if (field.type == InvalidDBType)
			throw new Exception(SourceLocation(__traits(getLocation, fieldAlias)).toErrorString
				~ "Cannot resolve DORM Model DBType from " ~ typeof(fieldAlias).stringof
				~ " `" ~ directFieldName ~ "` in " ~ TModel.stringof);

		foreach (ai, lhs; field.annotations)
		{
			foreach (bi, rhs; field.annotations)
			{
				if (ai == bi) continue;
				if (!lhs.isCompatibleWith(rhs))
					throw new Exception("Incompatible annotation: "
						~ lhs.to!string ~ " conflicts with " ~ rhs.to!string
						~ " on " ~ field.sourceReferenceName(TModel.stringof));
			}
		}

		serialized.fields ~= field;
	}
}

private string toSnakeCase(string s)
{
	import std.array;
	import std.ascii;

	auto ret = appender!(char[]);
	int upperCount;
	char lastChar;
	foreach (char c; s)
	{
		scope (exit)
			lastChar = c;
		if (upperCount)
		{
			if (isUpper(c))
			{
				ret ~= toLower(c);
				upperCount++;
			}
			else
			{
				if (isDigit(c))
				{
					ret ~= '_';
					ret ~= c;
				}
				else if (isLower(c) && upperCount > 1 && ret.data.length)
				{
					auto last = ret.data[$ - 1];
					ret.shrinkTo(ret.data.length - 1);
					ret ~= '_';
					ret ~= last;
					ret ~= c;
				}
				else
				{
					ret ~= toLower(c);
				}
				upperCount = 0;
			}
		}
		else if (isUpper(c))
		{
			if (ret.data.length
				&& ret.data[$ - 1] != '_'
				&& !lastChar.isUpper)
			{
				ret ~= '_';
				ret ~= toLower(c);
				upperCount++;
			}
			else
			{
				if (ret.data.length)
					upperCount++;
				ret ~= toLower(c);
			}
		}
		else if (c == '_')
		{
			if (ret.data.length)
				ret ~= '_';
		}
		else if (isDigit(c))
		{
			if (ret.data.length && !ret.data[$ - 1].isDigit && ret.data[$ - 1] != '_')
				ret ~= '_';
			ret ~= c;
		}
		else
		{
			if (ret.data.length && ret.data[$ - 1].isDigit && ret.data[$ - 1] != '_')
				ret ~= '_';
			ret ~= c;
		}
	}

	auto slice = ret.data;
	while (slice.length && slice[$ - 1] == '_')
		slice = slice[0 .. $ - 1];
	return slice.idup;
}

unittest
{
	assert("".toSnakeCase == "");
	assert("a".toSnakeCase == "a");
	assert("A".toSnakeCase == "a");
	assert("AB".toSnakeCase == "ab");
	assert("JsonValue".toSnakeCase == "json_value");
	assert("HTTPHandler".toSnakeCase == "http_handler");
	assert("Test123".toSnakeCase == "test_123");
	assert("foo_bar".toSnakeCase == "foo_bar");
	assert("foo_Bar".toSnakeCase == "foo_bar");
	assert("Foo_Bar".toSnakeCase == "foo_bar");
	assert("FOO_bar".toSnakeCase == "foo_bar");
	assert("FOO__bar".toSnakeCase == "foo__bar");
	assert("do_".toSnakeCase == "do");
	assert("fooBar_".toSnakeCase == "foo_bar");
	assert("_do".toSnakeCase == "do");
	assert("_fooBar".toSnakeCase == "foo_bar");
	assert("_FooBar".toSnakeCase == "foo_bar");
	assert("HTTP2".toSnakeCase == "http_2");
	assert("HTTP2Foo".toSnakeCase == "http_2_foo");
	assert("HTTP2foo".toSnakeCase == "http_2_foo");
}

private enum InvalidDBType = cast(ModelFormat.Field.DBType)int.max;

private template guessDBType(T)
{
	static if (is(T == enum))
		enum guessDBType = ModelFormat.Field.DBType.choices;
	else static if (is(T == BitFlags!U, U))
		enum guessDBType = ModelFormat.Field.DBType.set;
	else static if (is(T == Nullable!U, U))
	{
		static if (__traits(compiles, guessDBType!U))
			enum guessDBType = guessDBType!U;
		else
			static assert(false, "cannot resolve DBType from nullable " ~ U.stringof);
	}
	else static if (__traits(compiles, guessDBTypeBase!T))
		enum guessDBType = guessDBTypeBase!T;
	else
		enum guessDBType = InvalidDBType;
}

private enum guessDBTypeBase(T : const(char)[]) = ModelFormat.Field.DBType.varchar;
private enum guessDBTypeBase(T : const(ubyte)[]) = ModelFormat.Field.DBType.varbinary;
private enum guessDBTypeBase(T : byte) = ModelFormat.Field.DBType.int8;
private enum guessDBTypeBase(T : short) = ModelFormat.Field.DBType.int16;
private enum guessDBTypeBase(T : int) = ModelFormat.Field.DBType.int32;
private enum guessDBTypeBase(T : long) = ModelFormat.Field.DBType.int64;
private enum guessDBTypeBase(T : ubyte) = ModelFormat.Field.DBType.int16;
private enum guessDBTypeBase(T : ushort) = ModelFormat.Field.DBType.int32;
private enum guessDBTypeBase(T : uint) = ModelFormat.Field.DBType.int64;
private enum guessDBTypeBase(T : float) = ModelFormat.Field.DBType.floatNumber;
private enum guessDBTypeBase(T : double) = ModelFormat.Field.DBType.doubleNumber;
private enum guessDBTypeBase(T : bool) = ModelFormat.Field.DBType.boolean;
private enum guessDBTypeBase(T : Date) = ModelFormat.Field.DBType.date;
private enum guessDBTypeBase(T : DateTime) = ModelFormat.Field.DBType.datetime;
private enum guessDBTypeBase(T : SysTime) = ModelFormat.Field.DBType.datetime;
private enum guessDBTypeBase(T : TimeOfDay) = ModelFormat.Field.DBType.time;

unittest
{
	import std.sumtype;

	struct Mod
	{
		import std.datetime;
		import std.typecons;

		enum State : string
		{
			ok = "ok",
			warn = "warn",
			critical = "critical",
			unknown = "unknown"
		}

		class User : Model
		{
			@maxLength(255)
			string username;

			@maxLength(255)
			string password;

			@maxLength(255)
			Nullable!string email;

			ubyte age;

			Nullable!DateTime birthday;

			@autoCreateTime
			SysTime createdAt;

			@autoUpdateTime
			Nullable!SysTime updatedAt;

			@autoCreateTime
			ulong createdAt2;

			@autoUpdateTime
			Nullable!ulong updatedAt2;

			State state;

			@choices("ok", "warn", "critical", "unknown")
			string state2;

			@columnName("admin")
			bool isAdmin;

			@constructValue!(() => Clock.currTime + 4.hours)
			SysTime validUntil;

			@maxLength(255)
			@defaultValue("")
			string comment;

			@defaultValue(1337)
			int counter;

			@primaryKey
			long ownPrimaryKey;

			@timestamp
			Nullable!ulong someTimestamp;

			@unique
			int uuid;

			@validator!(x => x >= 18)
			int someInt;

			@ignored
			int imNotIncluded;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.models.length == 1);

	auto m = mod.models[0];

	// Length is always len(m.fields + 1) as dorm.model.Model adds the id field,
	// unless you define your own primary key field.
	assert(m.fields.length == 19);

	int i = 0;

	assert(m.fields[i].columnName == "username");
	assert(m.fields[i].type == ModelFormat.Field.DBType.varchar);
	assert(m.fields[i].annotations == [DBAnnotation(AnnotationFlag.notNull), DBAnnotation(maxLength(255))]);

	assert(m.fields[++i].columnName == "password");
	assert(m.fields[i].type == ModelFormat.Field.DBType.varchar);
	assert(m.fields[i].annotations == [DBAnnotation(AnnotationFlag.notNull), DBAnnotation(maxLength(255))]);

	assert(m.fields[++i].columnName == "email");
	assert(m.fields[i].type == ModelFormat.Field.DBType.varchar);
	assert(m.fields[i].annotations == [DBAnnotation(maxLength(255))]);

	assert(m.fields[++i].columnName == "age");
	assert(m.fields[i].type == ModelFormat.Field.DBType.int16);
	assert(m.fields[i].annotations == [DBAnnotation(AnnotationFlag.notNull)]);

	assert(m.fields[++i].columnName == "birthday");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == []);

	assert(m.fields[++i].columnName == "created_at");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(AnnotationFlag.autoCreateTime),
		]);

	assert(m.fields[++i].columnName == "updated_at");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.autoUpdateTime)
		]);

	assert(m.fields[++i].columnName == "created_at_2");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(AnnotationFlag.autoCreateTime),
		]);

	assert(m.fields[++i].columnName == "updated_at_2");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.autoUpdateTime)
		]);

	assert(m.fields[++i].columnName == "state");
	assert(m.fields[i].type == ModelFormat.Field.DBType.choices);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(Choices(["ok", "warn", "critical", "unknown"])),
		]);

	assert(m.fields[++i].columnName == "state_2");
	assert(m.fields[i].type == ModelFormat.Field.DBType.choices);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(Choices(["ok", "warn", "critical", "unknown"])),
		]);

	assert(m.fields[++i].columnName == "admin");
	assert(m.fields[i].type == ModelFormat.Field.DBType.boolean);
	assert(m.fields[i].annotations == [DBAnnotation(AnnotationFlag.notNull)]);

	assert(m.fields[++i].columnName == "valid_until");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull)
		]);
	assert(m.fields[i].internalAnnotations.length == 1);
	assert(m.fields[i].internalAnnotations[0].match!((ConstructValueRef r) => true, _ => false));

	assert(m.fields[++i].columnName == "comment");
	assert(m.fields[i].type == ModelFormat.Field.DBType.varchar);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(maxLength(255)),
			DBAnnotation(defaultValue("")),
		]);

	assert(m.fields[++i].columnName == "counter");
	assert(m.fields[i].type == ModelFormat.Field.DBType.int32);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(defaultValue(1337)),
		]);

	assert(m.fields[++i].columnName == "own_primary_key");
	assert(m.fields[i].type == ModelFormat.Field.DBType.int64);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.primaryKey),
		]);

	assert(m.fields[++i].columnName == "some_timestamp");
	assert(m.fields[i].type == ModelFormat.Field.DBType.datetime);
	assert(m.fields[i].annotations == []);

	assert(m.fields[++i].columnName == "uuid");
	assert(m.fields[i].type == ModelFormat.Field.DBType.int32);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull),
			DBAnnotation(AnnotationFlag.unique),
		]);

	assert(m.fields[++i].columnName == "some_int");
	assert(m.fields[i].type == ModelFormat.Field.DBType.int32);
	assert(m.fields[i].annotations == [
			DBAnnotation(AnnotationFlag.notNull)
		]);
	assert(m.fields[i].internalAnnotations.length == 1);
	assert(m.fields[i].internalAnnotations[0].match!((ValidatorRef r) => true, _ => false));
}

unittest
{
	struct Mod
	{
		abstract class NamedThing : Model
		{
			@maxLength(255)
			string name;
		}

		class User : NamedThing
		{
			int age;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.models.length == 1);

	auto m = mod.models[0];
	assert(m.tableName == "user");
	// As Model also adds the id field, length is 3
	assert(m.fields.length == 3);
	assert(m.fields[0].columnName == "id");
	assert(m.fields[1].columnName == "name");
	assert(m.fields[2].columnName == "age");
}

unittest
{
	struct Mod
	{
		struct SuperCommon
		{
			int superCommonField;
		}

		struct Common
		{
			string commonName;
			@embedded
			SuperCommon superCommon;
		}

		class NamedThing : Model
		{
			@embedded
			Common common;

			@maxLength(255)
			string name;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.models.length == 1);
	auto m = mod.models[0];
	assert(m.tableName == "named_thing");
	// As Model also adds the id field, length is 3
	assert(m.fields.length == 4);
	assert(m.fields[1].columnName == "common_name");
	assert(m.fields[1].sourceColumn == "common.commonName");
	assert(m.fields[2].columnName == "super_common_field");
	assert(m.fields[2].sourceColumn == "common.superCommon.superCommonField");
	assert(m.fields[3].columnName == "name");
	assert(m.fields[3].sourceColumn == "name");
	assert(m.embeddedStructs == ["common", "common.superCommon"]);
}

// https://github.com/myOmikron/drorm/issues/6
unittest
{
	struct Mod
	{
		class NamedThing : Model
		{
			@timestamp
			Nullable!long timestamp1;

			@autoUpdateTime
			Nullable!long timestamp2;

			@autoCreateTime
			long timestamp3;

			@autoUpdateTime @autoCreateTime
			long timestamp4;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.models.length == 1);
	auto m = mod.models[0];
	assert(m.tableName == "named_thing");
	// As Model also adds the id field, length is 5
	assert(m.fields.length == 5);

	assert(m.fields[1].columnName == "timestamp_1");
	assert(m.fields[1].isNullable);
	assert(m.fields[2].isNullable);
	assert(!m.fields[3].isNullable);
	assert(!m.fields[4].isNullable);
}

unittest
{
	struct Mod
	{
		class DefaultValues : Model
		{
			@defaultValue(10)
			int f1;

			@defaultValue(2)
			int f2 = 1337;

			@defaultFromInit
			int f3 = 1337;
		}
	}

	auto mod = processModelsToDeclarations!Mod;
	assert(mod.models.length == 1);
	auto m = mod.models[0];
	// As Model also adds the id field
	assert(m.fields.length == 4);

	assert(m.fields[1].columnName == "f_1");
	assert(m.fields[1].annotations == [
		DBAnnotation(AnnotationFlag.notNull),
		DBAnnotation(defaultValue(10))
	]);

	assert(m.fields[2].columnName == "f_2");
	assert(m.fields[2].annotations == [
		DBAnnotation(AnnotationFlag.notNull),
		DBAnnotation(defaultValue(2))
	]);

	assert(m.fields[3].columnName == "f_3");
	assert(m.fields[3].annotations == [
		DBAnnotation(AnnotationFlag.notNull),
		DBAnnotation(defaultValue(1337))
	]);
}
