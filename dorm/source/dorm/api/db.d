module dorm.api.db;

import ffi = dorm.lib.ffi;
import dorm.lib.util;

public import dorm.lib.ffi : DBBackend;

struct DBConnectOptions
{
	DBBackend backend;
	string name;
	string host;
	ushort port;
	string user;
	string password;
	uint minConnections;
	uint maxConnections;
}

struct DormDB
{
	private ffi.DBHandle handle;

	this(DBConnectOptions options)
	{
		// TODO: think of how to make async waiting configurable, right now the thread is just blocked
		auto ffiOptions = options.ffiInto!(ffi.DBConnectOptions);

		scope dbHandleAsync = FreeableAsyncResult!(ffi.DBHandle).make;
		ffi.rorm_db_connect(ffiOptions, dbHandleAsync.callback.expand);
		handle = dbHandleAsync.result;
	}

	~this()
	{
		ffi.rorm_db_discconnect(handle);
	}

	@disable this(this);
}

static SelectOperation!(T, SelectType!(T, Selection)) select(T, Selection...)(return ref DormDB db)
{
	return SelectOperation!(T, SelectType!(T, Selection))(&db);
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

private template BuildFieldsTuple(T, Selection...)
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

struct ConditionBuilder(T)
{
	static foreach (i, member; T.tupleof)
		mixin("ConditionBuilderField!(typeof(T.tupleof[i])) ",
			T.tupleof[i].stringof,
			" = ConditionBuilderField!(typeof(T.tupleof[i]))(`",
			T.Fields[i].columnName,
			"`)");
}

struct ConditionBuilderField(T)
{
	// TODO: all the type specific field to Condition thingies

	ffi.Condition lhs;
	this(string columnName)
	{
		lhs.type = ffi.Condition.value;
		lhs.value.type = ffi.ConditionValue.Type.Identifier;
		lhs.value.identifier = ffi.ffi(columnName);
	}

	ffi.Condition binaryCondition(ffi.BinaryCondition.Type type, ffi.Condition rhs)
	{
		ffi.Condition ret;
		ret.type = ffi.Condition.binaryCondition;
		// TODO: need to provide some way to use allocator or other things instead of GC here
		ret.binaryCondition = new ffi.BinaryCondition();
		ret.binaryCondition.type = type;
		ret.binaryCondition.lhs = lhs;
		ret.binaryCondition.rhs = rhs;
		return ret;
	}

	ffi.Condition equals(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.Equals, makeConditionConstant(value));
	}

	ffi.Condition notEquals(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.NotEquals, makeConditionConstant(value));
	}

	ffi.Condition lessThan(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.Less, makeConditionConstant(value));
	}

	ffi.Condition lessThanOrEqual(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.LessOrEquals, makeConditionConstant(value));
	}

	ffi.Condition greaterThan(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.Greater, makeConditionConstant(value));
	}

	ffi.Condition greaterThanOrEqual(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.GreaterOrEquals, makeConditionConstant(value));
	}

	ffi.Condition like(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.Like, makeConditionConstant(value));
	}

	ffi.Condition like(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.NotLike, makeConditionConstant(value));
	}

	ffi.Condition regexp(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.Regexp, makeConditionConstant(value));
	}

	ffi.Condition regexp(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.NotRegexp, makeConditionConstant(value));
	}

	ffi.Condition in_(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.In, makeConditionConstant(value));
	}

	ffi.Condition in_(V)(V value)
	{
		return binaryCondition(ffi.BinaryCondition.Type.In, makeConditionConstant(value));
	}

	ffi.Condition unaryCondition(ffi.UnaryCondition.Type type)
	{
		ffi.Condition ret;
		ret.type = ffi.Condition.Type.UnaryCondition;
		// TODO: need to provide some way to use allocator or other things instead of GC here
		ret.unaryCondition = new ffi.UnaryCondition();
		ret.unaryCondition.type = type;
		ret.unaryCondition.condition = lhs;
		return ret;
	}

	ffi.Condition isNull()
	{
		return unaryCondition(ffi.UnaryCondition.Type.IsNull);
	}

	alias equalsNull = isNull;

	ffi.Condition isNotNull()
	{
		return unaryCondition(ffi.UnaryCondition.Type.IsNotNull);
	}

	alias notEqualsNull = isNotNull;

	ffi.Condition exists()
	{
		return unaryCondition(ffi.UnaryCondition.Type.Exists);
	}

	ffi.Condition notExists()
	{
		return unaryCondition(ffi.UnaryCondition.Type.NotExists);
	}

	ffi.Condition ternaryCondition(ffi.TernaryCondition.Type type, ffi.Condition second, ffi.Condition third)
	{
		ffi.Condition ret;
		ret.type = ffi.Condition.ternaryCondition;
		// TODO: need to provide some way to use allocator or other things instead of GC here
		ret.ternaryCondition = new ffi.TernaryCondition();
		ret.ternaryCondition.type = type;
		ret.ternaryCondition.first = lhs;
		ret.ternaryCondition.second = second;
		ret.ternaryCondition.third = third;
		return ret;
	}

	ffi.Condition between(L, R)(L min, R max)
	{
		return ternaryCondition(ffi.TernaryCondition.Type.Between, makeConditionConstant(min), makeConditionConstant(max));
	}

	ffi.Condition notBetween(L, R)(L min, R max)
	{
		return ternaryCondition(ffi.TernaryCondition.Type.NotBetween, makeConditionConstant(min), makeConditionConstant(max));
	}
}

ffi.Condition makeConditionConstant(T : ffi.Condition)(T c) { return c; }
ffi.Condition makeConditionConstant(T)(T c)
{
	ffi.Condition ret;
	ret.type = ffi.Condition.Type.Value;
	static if (is(T == typeof(null)))
	{
		ret.value.type = ffi.ConditionValue.Type.Null;
	}
	else static if (is(T == bool))
	{
		ret.value.type = ffi.ConditionValue.Type.Bool;
		ret.value.boolean = c;
	}
	else static if (is(T == short))
	{
		ret.value.type = ffi.ConditionValue.Type.I16;
		ret.value.i16 = c;
	}
	else static if (is(T == int))
	{
		ret.value.type = ffi.ConditionValue.Type.I32;
		ret.value.i32 = c;
	}
	else static if (is(T == long))
	{
		ret.value.type = ffi.ConditionValue.Type.I64;
		ret.value.i64 = c;
	}
	else static if (is(T == float))
	{
		ret.value.type = ffi.ConditionValue.Type.F32;
		ret.value.f32 = c;
	}
	else static if (is(T == double))
	{
		ret.value.type = ffi.ConditionValue.Type.F64;
		ret.value.f64 = c;
	}
	else static if (is(T : const(char)[]))
	{
		ret.value.type = ffi.ConditionValue.Type.String;
		ret.value.string = ffi.ffi(c);
	}
	else static assert(false, "Unsupported condition value type: " ~ T.stringof);
	return c;
}

ffi.Condition and(ffi.Condition[] conjunction...)
{
	ffi.Condition ret;
	ret.type = ffi.Condition.Type.Conjunction;
	// TODO: might want to use allocator instead of GC
	ret.conjunction = ffi.ffi(conjunction.dup);
	return ret;
}

ffi.Condition or(ffi.Condition[] disjunction...)
{
	ffi.Condition ret;
	ret.type = ffi.Condition.Type.Disjunction;
	// TODO: might want to use allocator instead of GC
	ret.disjunction = ffi.ffi(disjunction.dup);
	return ret;
}

struct SelectOperation(
	T,
	TSelect,
	bool hasWhere = false,
	bool hasOrder = false,
	bool hasOffset = false,
	bool hasLimit = false,
)
{
	private DormDB* db;
	private ffi.Condition condition;
	private long offset, limit;

	static if (!hasWhere)
	{
		SelectOperation!(T, TSelect, true, hasOrder, hasLimit) select(SelectBuilder callback) return
		{
			condition = callback(ConditionBuilder!T.init);
			return cast(typeof(return))this;
		}
	}

	static if (!hasOrder)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) orderBy(T...)(T) return
		{
			static assert(false, "not implemented");
		}
	}

	static if (!hasOffset)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) drop(long offset) return
		{
			this.offset = offset;
			return cast(typeof(return))this;
		}
	}

	static if (!hasLimit)
	{
		// may not .drop after take!
		SelectOperation!(T, TSelect, hasWhere, true, true) take(long limit) return
		{
			this.limit = limit;
			return cast(typeof(return))this;
		}
	}

	static if (!hasOffset && !hasLimit)
	{
		size_t[2] opSlice(size_t start, size_t end)
		{
			return [start, end];
		}

		SelectOperation!(T, TSelect, hasWhere, true, true) opIndex(size_t[2] slice) return
		{
			this.offset = slice[0];
			this.limit = cast(long)slice[1] - cast(long)slice[0];
			return cast(typeof(return))this;
		}
	}

	TSelect[] array()
	{
		scope handle = ffi.rorm_db_query_all();
	}

	auto stream()
	{
		int result = 0;

		enum fields = T.ModelInfo.filterFields!TSelect;

		FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.name);
		scope stream = ffi.rorm_db_query_stream(db.handle,
			ffi.ffi(T.ModelInfo.name),
			ffi.ffi(columns));
		scope (exit)
			rorm_stream_free(stream);
	
		while (!rorm_stream_empty(stream))
		{
			async_call!rorm_stream_next(stream, (rowResult) {
				writeln("Hello ", rorm_row_get_data_varchar(rowResult.expect, 0));
			}).wait;
		}
	
		return result;
	}
}
