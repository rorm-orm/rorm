module dorm.api.condition;

@safe:

import std.conv;
import std.datetime;
import std.sumtype;
import std.traits;
import std.typecons : Nullable;

import dorm.declarative;

public import dorm.lib.ffi : FFIValue;
import ffi = dorm.lib.ffi;

alias UnaryConditionType = ffi.FFIUnaryCondition.Type;
alias BinaryConditionType = ffi.FFIBinaryCondition.Type;
alias TernaryConditionType = ffi.FFITernaryCondition.Type;

struct Condition
{
	SumType!(
		FFIValue,
		UnaryCondition,
		BinaryCondition,
		TernaryCondition,
		AndCondition,
		OrCondition
	) impl;
	alias impl this;

	this(T)(T value)
	{
		impl = value;
	}

	auto opAssign(T)(T value)
	{
		impl = value;
		return this;
	}

	static Condition and(Condition[] conditions...)
	{
		return Condition(AndCondition(conditions.dup));
	}

	static Condition or(Condition[] conditions...)
	{
		return Condition(OrCondition(conditions.dup));
	}

	Condition not() const @trusted
	{
		Condition* c = new Condition();
		c.impl = impl;
		return Condition(UnaryCondition(UnaryConditionType.Not, c));
	}
}

struct AndCondition
{
	Condition[] conditions;
}

struct OrCondition
{
	Condition[] conditions;
}

struct UnaryCondition
{
	UnaryConditionType type;
	Condition* condition;
}

struct BinaryCondition
{
	BinaryConditionType type;
	Condition* lhs;
	Condition* rhs;
}

struct TernaryCondition
{
	TernaryConditionType type;
	Condition* first, second, third;
}

FFIValue conditionValue(ModelFormat.Field fieldInfo, T)(T c) @trusted
{
	FFIValue ret;
	static if (is(T == Nullable!U, U))
	{
		if (c.isNull)
			ret.type = FFIValue.Type.Null;
		else
			return conditionValue!fieldInfo(c.get);
	}
	else static if (fieldInfo.type == ModelFormat.Field.DBType.datetime
		&& (is(T == long) || is(T == ulong)))
	{
		ret = conditionValue!fieldInfo(cast(DateTime) SysTime(cast(long)c, UTC()));
	}
	else static if (is(T == enum))
	{
		ret.type = FFIValue.Type.String;
		static if (is(OriginalType!T == string))
			ret.string = ffi.ffi(cast(string)c);
		else
			ret.string = ffi.ffi(c.to!string); // std.conv : to gives us the enum field name
	}
	else static if (is(T == typeof(null)))
	{
		ret.type = FFIValue.Type.Null;
	}
	else static if (is(T == bool))
	{
		ret.type = FFIValue.Type.Bool;
		ret.boolean = c;
	}
	else static if (is(T == short))
	{
		ret.type = FFIValue.Type.I16;
		ret.i16 = c;
	}
	else static if (is(T == int))
	{
		ret.type = FFIValue.Type.I32;
		ret.i32 = c;
	}
	else static if (isIntegral!T && is(T : long))
	{
		ret.type = FFIValue.Type.I64;
		ret.i64 = c;
	}
	else static if (is(T == float))
	{
		ret.type = FFIValue.Type.F32;
		ret.f32 = c;
	}
	else static if (is(T : double))
	{
		ret.type = FFIValue.Type.F64;
		ret.f64 = c;
	}
	else static if (is(T : const(char)[]))
	{
		ret.type = FFIValue.Type.String;
		ret.string = ffi.ffi(c);
	}
	else static if (is(T : Date))
	{
		ret.type = FFIValue.Type.NaiveDate;
		ret.naiveDate = ffi.FFIDate(cast(uint)c.day, cast(uint)c.month, cast(int)c.year);
	}
	else static if (is(T : TimeOfDay))
	{
		ret.type = FFIValue.Type.NaiveTime;
		ret.naiveTime = ffi.FFITime(cast(uint)c.hour, cast(uint)c.minute, cast(uint)c.second);
	}
	else static if (is(T : DateTime))
	{
		ret.type = FFIValue.Type.NaiveDateTime;
		ret.naiveDateTime = ffi.FFIDateTime(cast(int)c.year, cast(uint)c.month, cast(uint)c.day, cast(uint)c.hour, cast(uint)c.minute, cast(uint)c.second);
	}
	else static if (is(T : SysTime))
	{
		ret.type = FFIValue.Type.NaiveDateTime;
		auto d = cast(DateTime) c.toUTC;
		ret.naiveDateTime = ffi.FFIDateTime(cast(int)d.year, cast(uint)d.month, cast(uint)d.day, cast(uint)d.hour, cast(uint)d.minute, cast(uint)d.second);
	}
	else static if (is(T == ffi.FFIString))
	{
		ret.type = FFIValue.Type.String;
		ret.string = c;
	}
	else
		static assert(false, "Unsupported condition value type: " ~ T.stringof
			~ text(" in column ", field.sourceColumn, " in file ", field.definedAt).idup);
	return ret;
}

FFIValue conditionIdentifier(return string identifier) @safe
{
	FFIValue ret;
	ret.type = FFIValue.Type.Identifier;
	(() @trusted {
		ret.identifier = ffi.ffi(identifier);
	})();
	return ret;
}

ffi.FFICondition[] makeTree(Condition c) @trusted
{
	// we store all conditions sequentially in a flat list, this function may be
	// run at CTFE, where it can then efficiently be put on the stack for
	// building the whole tree pointers. Otherwise everything is closer together
	// in memory as well, so we might even get performance improvements in the
	// runtime case.
	ffi.FFICondition[] ret;

	// as `ret` may be moved when resizing, we only store indices at first when
	// constructing the tree inside the pointer fields. Afterwards we go through
	// all generated items and replace the pointers, which hold list indices as
	// pointer values, instead of valid memory locations, and replace these
	// indices with the actual memory locations, to allow lookup on the other
	// side of the FFI boundary, which expects pointers as children.
	static void recurse(ref ffi.FFICondition[] ret, size_t dst, ref Condition c)
	{
		ffi.FFICondition dstret;
		c.match!(
			(FFIValue v)
			{
				dstret.type = ffi.FFICondition.Type.Value;
				dstret.value = v;
			},
			(UnaryCondition v)
			{
				size_t index = ret.length;
				ret.length++;
				recurse(ret, index, *v.condition);
				dstret.type = ffi.FFICondition.Type.UnaryCondition;
				dstret.unaryCondition = ffi.FFIUnaryCondition(v.type,
						cast(ffi.FFICondition*)(index));
			},
			(BinaryCondition v)
			{
				size_t index = ret.length;
				ret.length += 2;
				recurse(ret, index, *v.lhs);
				recurse(ret, index + 1, *v.rhs);
				dstret.type = ffi.FFICondition.Type.BinaryCondition;
				dstret.binaryCondition = ffi.FFIBinaryCondition(v.type,
						cast(ffi.FFICondition*)(index),
						cast(ffi.FFICondition*)(index + 1));
			},
			(TernaryCondition v)
			{
				size_t index = ret.length;
				ret.length += 3;
				recurse(ret, index, *v.first);
				recurse(ret, index + 1, *v.second);
				recurse(ret, index + 2, *v.third);
				dstret.type = ffi.FFICondition.Type.TernaryCondition;
				dstret.ternaryCondition = ffi.FFITernaryCondition(v.type,
						cast(ffi.FFICondition*)(index),
						cast(ffi.FFICondition*)(index + 1),
						cast(ffi.FFICondition*)(index + 2));
			},
			(AndCondition v)
			{
				size_t start = ret.length;
				ret.length += v.conditions.length;
				foreach (i, ref c; v.conditions)
					recurse(ret, start + i, c);
				dstret.type = ffi.FFICondition.Type.Conjunction;
				dstret.conjunction.content = cast(ffi.FFICondition*)start;
				dstret.conjunction.size = v.conditions.length;
			},
			(OrCondition v)
			{
				size_t start = ret.length;
				ret.length += v.conditions.length;
				foreach (i, ref c; v.conditions)
					recurse(ret, start + i, c);
				dstret.type = ffi.FFICondition.Type.Disjunction;
				dstret.disjunction.content = cast(ffi.FFICondition*)start;
				dstret.disjunction.size = v.conditions.length;
			}
		);
		ret[dst] = dstret;
	}

	ret.length = 1;
	recurse(ret, 0, c);

	// now fix the pointer values:
	foreach (ref fixup; ret)
	{
		final switch (fixup.type)
		{
		case ffi.FFICondition.Type.Value: break;
		case ffi.FFICondition.Type.UnaryCondition:
			fixup.unaryCondition.condition = &ret[cast(size_t)fixup.unaryCondition.condition];
			break;
		case ffi.FFICondition.Type.BinaryCondition:
			fixup.binaryCondition.lhs = &ret[cast(size_t)fixup.binaryCondition.lhs];
			fixup.binaryCondition.rhs = &ret[cast(size_t)fixup.binaryCondition.rhs];
			break;
		case ffi.FFICondition.Type.TernaryCondition:
			fixup.ternaryCondition.first = &ret[cast(size_t)fixup.ternaryCondition.first];
			fixup.ternaryCondition.second = &ret[cast(size_t)fixup.ternaryCondition.second];
			fixup.ternaryCondition.third = &ret[cast(size_t)fixup.ternaryCondition.third];
			break;
		case ffi.FFICondition.Type.Conjunction:
			fixup.conjunction.content = &ret[cast(size_t)fixup.conjunction.content];
			break;
		case ffi.FFICondition.Type.Disjunction:
			fixup.disjunction.content = &ret[cast(size_t)fixup.disjunction.content];
			break;
		}
	}

	return ret;
}

string dumpTree(ffi.FFICondition[] c)
{
	import std.array : appender;
	import std.format : format;

	auto query = appender!string;
	query ~= "WHERE";
	void recurse(ref ffi.FFICondition c) @trusted
	{
		import std.conv;

		final switch (c.type)
		{
		case ffi.FFICondition.Type.Value:
			query ~= " Value(";
			final switch (c.value.type)
			{
				case FFIValue.Type.String: query ~= '`' ~ c.value.string[] ~ '`'; break;
				case FFIValue.Type.Identifier: query ~= "ident:" ~ c.value.identifier[]; break;
				case FFIValue.Type.Bool: query ~= c.value.boolean.to!string; break;
				case FFIValue.Type.I16: query ~= "i16:" ~ c.value.i16.to!string; break;
				case FFIValue.Type.I32: query ~= "i32:" ~ c.value.i32.to!string; break;
				case FFIValue.Type.I64: query ~= "i64:" ~ c.value.i64.to!string; break;
				case FFIValue.Type.F32: query ~= "f32:" ~ c.value.f32.to!string; break;
				case FFIValue.Type.F64: query ~= "f64:" ~ c.value.f64.to!string; break;
				case FFIValue.Type.Null: query ~= "[null]"; break;
				case FFIValue.Type.Binary: query ~= "(binary)"; break;
				case FFIValue.Type.NaiveTime: auto t = c.value.naiveTime; query ~= format!"%02d:%02d:%02d"(t.hour, t.min, t.sec); break;
				case FFIValue.Type.NaiveDate: auto d = c.value.naiveDate; query ~= format!"%04d-%02d-%02d"(d.year, d.month, d.day); break;
				case FFIValue.Type.NaiveDateTime: auto dt = c.value.naiveDateTime; query ~= format!"%04d-%02d-%02dT%02d:%02d:%02d"(dt.year, dt.month, dt.day, dt.hour, dt.min, dt.sec); break;
			}
			query ~= ")";
			break;
		case ffi.FFICondition.Type.UnaryCondition:
			final switch (c.unaryCondition.type)
			{
				case UnaryConditionType.Not:
					query ~= " NOT";
					recurse(*c.unaryCondition.condition);
					break;
				case UnaryConditionType.Exists:
					recurse(*c.unaryCondition.condition);
					query ~= " EXISTS";
					break;
				case UnaryConditionType.NotExists:
					recurse(*c.unaryCondition.condition);
					query ~= " NOT EXISTS";
					break;
				case UnaryConditionType.IsNull:
					recurse(*c.unaryCondition.condition);
					query ~= " IS NULL";
					break;
				case UnaryConditionType.IsNotNull:
					recurse(*c.unaryCondition.condition);
					query ~= " IS NOT NULL";
					break;
			}
			break;
		case ffi.FFICondition.Type.BinaryCondition:
			recurse(*c.binaryCondition.lhs);
			final switch (c.binaryCondition.type)
			{
				case BinaryConditionType.Equals:
					query ~= " =";
					break;
				case BinaryConditionType.NotEquals:
					query ~= " !=";
					break;
				case BinaryConditionType.Greater:
					query ~= " >";
					break;
				case BinaryConditionType.GreaterOrEquals:
					query ~= " >=";
					break;
				case BinaryConditionType.Less:
					query ~= " <";
					break;
				case BinaryConditionType.LessOrEquals:
					query ~= " <=";
					break;
				case BinaryConditionType.Like:
					query ~= " LIKE";
					break;
				case BinaryConditionType.NotLike:
					query ~= " NOT LIKE";
					break;
				case BinaryConditionType.In:
					query ~= " IN";
					break;
				case BinaryConditionType.NotIn:
					query ~= " NOT IN";
					break;
				case BinaryConditionType.Regexp:
					query ~= " REGEXP";
					break;
				case BinaryConditionType.NotRegexp:
					query ~= " NOT REGEXP";
					break;
			}
			recurse(*c.binaryCondition.rhs);
			break;
		case ffi.FFICondition.Type.TernaryCondition:
			recurse(*c.ternaryCondition.first);
			final switch (c.ternaryCondition.type)
			{
				case TernaryConditionType.Between:
					query ~= " BETWEEN";
					break;
				case TernaryConditionType.NotBetween:
					query ~= " NOT BETWEEN";
					break;
			}
			recurse(*c.ternaryCondition.second);
			query ~= " AND";
			recurse(*c.ternaryCondition.third);
			break;
		case ffi.FFICondition.Type.Conjunction:
		case ffi.FFICondition.Type.Disjunction:
			string op = c.type == ffi.FFICondition.Type.Conjunction ? " AND" : " OR";
			query ~= " (";
			foreach (i, ref subc; c.conjunction.data)
			{
				if (i != 0) query ~= op;
				recurse(subc);
			}
			query ~= " )";
			break;
		}
	}
	recurse(c[0]);
	return query.data;
}

unittest
{
	import std.array;

	Condition* and(scope Condition[] conditions...)
	{
		return new Condition(AndCondition(conditions.dup));
	}
	Condition* or(scope Condition[] conditions...)
	{
		return new Condition(OrCondition(conditions.dup));
	}

	Condition* unary(UnaryConditionType t, Condition* c)
	{
		return new Condition(UnaryCondition(t, c));
	}

	Condition* binary(Condition* lhs, BinaryConditionType t, Condition* rhs)
	{
		return new Condition(BinaryCondition(t, lhs, rhs));
	}

	Condition* ternary(Condition* first, TernaryConditionType t, Condition* second, Condition* third)
	{
		return new Condition(TernaryCondition(t, first, second, third));
	}

	Condition* i(string s)
	{
		return new Condition(conditionIdentifier(s));
	}

	Condition* v(T)(T value)
	{
		return new Condition(conditionValue!(ModelFormat.Field.init)(value));
	}

	auto condition = and(
		*binary(i("foo"), BinaryConditionType.Equals, v("wert")),
		*binary(i("bar"), BinaryConditionType.Greater, v(5)),
		*unary(UnaryConditionType.Not, or(
			*binary(i("baz"), BinaryConditionType.Equals, v(1)),
			*binary(i("baz"), BinaryConditionType.Equals, v(4)),
		))
	);

	auto tree = makeTree(*condition);
	auto query = dumpTree(tree);
	assert(query == "WHERE ( Value(ident:foo) = Value(`wert`) AND Value(ident:bar) > Value(i32:5) AND NOT ( Value(ident:baz) = Value(i32:1) OR Value(ident:baz) = Value(i32:4) ) )");
}
