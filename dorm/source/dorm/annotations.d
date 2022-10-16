module dorm.annotations;

import std.datetime;
import std.traits;
import std.meta;

enum autoCreateTime;
enum autoUpdateTime;
enum autoIncrement;
enum timestamp;
enum notNull;

alias Id = AliasSeq!(primaryKey, autoIncrement);

struct constructValue(alias fn) {}
struct validator(alias fn) {}

struct maxLength { int maxLength; }

alias AllowedDefaultValueTypes = AliasSeq!(
	string, ubyte[], byte, short, int, long, ubyte, ushort, uint, ulong, float,
	double, bool, Date, DateTime, TimeOfDay, SysTime
);
enum isAllowedDefaultValueType(T) = staticIndexOf!(T, AllowedDefaultValueTypes) != -1;
struct DefaultValue(T) { T value; }
auto defaultValue(T)(T value) if (isAllowedDefaultValueType!T)
{
	return DefaultValue!T(value);
}
alias PossibleDefaultValueTs = staticMap!(DefaultValue, AllowedDefaultValueTypes);

enum primaryKey;
enum unique;

struct Choices { string[] choices; }
Choices choices(string[] choices...) { return Choices(choices.dup); }

struct columnName { string name; }

struct index
{
	// part of ctor
	static struct priority { int priority = 10; }
	static struct composite { string name; }

	// careful: never duplicate types here, otherwise the automatic ctor doesn't work
	priority _priority;
	composite _composite;

	this(T...)(T args)
	{
		foreach (ref field; this.tupleof)
		{
			static foreach (arg; args)
			{
				static if (is(typeof(field) == typeof(arg)))
					field = arg;
			}
		}
	}
}

enum embedded;
enum ignored;

/**
 * When saving a model, only include the annotated column if the here referenced
 * field equals to `equalsTo`.
 */
struct modifiedIf
{
	string field;
	bool equalsTo = true;

	string makeCheckCode(string prefix) const
	{
		return prefix ~ field ~ " == " ~ (equalsTo ? "true" : "false");
	}
}

/// Checks if the given attribute affects DORM Fields
template isDormFieldAttribute(alias attr)
{
	pragma(msg, "TODO: check if " ~ attr.stringof ~ " is a DORM Field annotation");
	enum isDormFieldAttribute = true;
}

/// Checks if the given attribute affects DORM Models (classes)
template isDormModelAttribute(alias attr)
{
	pragma(msg, "TODO: check if " ~ attr.stringof ~ " is a DORM Model annotation");
	enum isDormFieldAttribute = true;
}
