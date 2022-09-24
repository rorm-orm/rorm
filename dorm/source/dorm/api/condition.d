module dorm.api.condition;

import std.sumtype;
import std.traits;

public import dorm.lib.ffi : ConditionValue;
import ffi = dorm.lib.ffi;

alias UnaryConditionType = ffi.FFIUnaryCondition.Type;
alias BinaryConditionType = ffi.FFIBinaryCondition.Type;
alias TernaryConditionType = ffi.FFITernaryCondition.Type;

alias Condition = SumType!(
	ConditionValue,
	UnaryCondition,
	BinaryCondition,
	TernaryCondition,
	AndCondition,
	OrCondition
);

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

ConditionValue conditionValue(T)(T c)
{
	ConditionValue ret;
	static if (is(T == typeof(null)))
	{
		ret.type = ConditionValue.Type.Null;
	}
	else static if (is(T == bool))
	{
		ret.type = ConditionValue.Type.Bool;
		ret.boolean = c;
	}
	else static if (is(T == short))
	{
		ret.type = ConditionValue.Type.I16;
		ret.i16 = c;
	}
	else static if (is(T == int))
	{
		ret.type = ConditionValue.Type.I32;
		ret.i32 = c;
	}
	else static if (isIntegral!T && is(T : long))
	{
		ret.type = ConditionValue.Type.I64;
		ret.i64 = c;
	}
	else static if (is(T == float))
	{
		ret.type = ConditionValue.Type.F32;
		ret.f32 = c;
	}
	else static if (is(T : double))
	{
		ret.type = ConditionValue.Type.F64;
		ret.f64 = c;
	}
	else static if (is(T : const(char)[]))
	{
		ret.type = ConditionValue.Type.String;
		ret.string = ffi.ffi(c);
	}
	else static assert(false, "Unsupported condition value type: " ~ T.stringof);
	return ret;
}

ConditionValue conditionIdentifier(string identifier)
{
	ConditionValue ret;
	ret.type = ConditionValue.Type.Identifier;
	ret.identifier = ffi.ffi(identifier);
	return ret;
}

ffi.FFICondition[] makeTree(Condition c)
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
			(ConditionValue v)
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
		return new Condition(conditionValue(value));
	}

	auto condition = and(
		*binary(i("foo"), BinaryConditionType.Equals, v("wert")),
		*binary(i("bar"), BinaryConditionType.Greater, v(5)),
		*unary(UnaryConditionType.Not, or(
			*binary(i("baz"), BinaryConditionType.Equals, v(1)),
			*binary(i("baz"), BinaryConditionType.Equals, v(4)),
		))
	);

	auto query = appender!string;
	query ~= "WHERE";
	auto tree = makeTree(*condition);
	void recurse(ref ffi.FFICondition c)
	{
		import std.conv;

		final switch (c.type)
		{
		case ffi.FFICondition.Type.Value:
			query ~= " Value(";
			final switch (c.value.type)
			{
				case ConditionValue.Type.String: query ~= '`' ~ c.value.string[] ~ '`'; break;
				case ConditionValue.Type.Identifier: query ~= "ident:" ~ c.value.identifier[]; break;
				case ConditionValue.Type.Bool: query ~= c.value.boolean.to!string; break;
				case ConditionValue.Type.I16: query ~= "i16:" ~ c.value.i16.to!string; break;
				case ConditionValue.Type.I32: query ~= "i32:" ~ c.value.i32.to!string; break;
				case ConditionValue.Type.I64: query ~= "i64:" ~ c.value.i64.to!string; break;
				case ConditionValue.Type.F32: query ~= "f32:" ~ c.value.f32.to!string; break;
				case ConditionValue.Type.F64: query ~= "f64:" ~ c.value.f64.to!string; break;
				case ConditionValue.Type.Null: query ~= "[null]"; break;
				case ConditionValue.Type.Binary: query ~= "(binary)"; break;
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
	recurse(tree[0]);
	assert(query.data == "WHERE ( Value(ident:foo) = Value(`wert`) AND Value(ident:bar) > Value(i32:5) AND NOT ( Value(ident:baz) = Value(i32:1) OR Value(ident:baz) = Value(i32:4) ) )");
}
