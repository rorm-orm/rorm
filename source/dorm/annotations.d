module dorm.annotations;

import std.traits;
import std.meta;

enum autoCreateTime;
enum autoUpdateTime;
enum timestamp;

struct constructValue(alias fn) {}
struct validator(alias fn) {}

struct maxLength { int maxLength; }

alias AllowedDefaultValueTypes = AliasSeq!(
	string, ubyte[], byte, short, int, long, ubyte, ushort, uint, ulong, float,
	double, bool, Date, DateTime, TimeOfDay
);
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

	this(T... args)
	{
		foreach (ref field; this.tupleof)
		{
			static foreach (arg; args)
			{
				static if (typeof(field) == typeof(arg))
					field = arg;
			}
		}
	}
}

enum embedded;
