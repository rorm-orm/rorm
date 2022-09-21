module dorm.api.db;

import ffi = dorm.lib.ffi;
import dorm.lib.util;

public import dorm.lib.ffi : DBBackend;

public import dorm.api.condition;

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

	@disable this();

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
		ffi.rorm_db_free(handle);
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
			DormFields!T[i].columnName,
			"`)");
}

struct ConditionBuilderField(T)
{
	// TODO: all the type specific field to Condition thingies

	Condition lhs;
	this(string columnName)
	{
		lhs.type = Condition.value;
		lhs.value.type = ConditionValue.Type.Identifier;
		lhs.value.identifier = ffi.ffi(columnName);
	}

	Condition equals(V)(V value)
	{
		return binaryCondition(BinaryConditionType.Equals, makeConditionConstant(value));
	}

	Condition notEquals(V)(V value)
	{
		return binaryCondition(BinaryConditionType.NotEquals, makeConditionConstant(value));
	}

	Condition lessThan(V)(V value)
	{
		return binaryCondition(BinaryConditionType.Less, makeConditionConstant(value));
	}

	Condition lessThanOrEqual(V)(V value)
	{
		return binaryCondition(BinaryConditionType.LessOrEquals, makeConditionConstant(value));
	}

	Condition greaterThan(V)(V value)
	{
		return binaryCondition(BinaryConditionType.Greater, makeConditionConstant(value));
	}

	Condition greaterThanOrEqual(V)(V value)
	{
		return binaryCondition(BinaryConditionType.GreaterOrEquals, makeConditionConstant(value));
	}

	Condition like(V)(V value)
	{
		return binaryCondition(BinaryConditionType.Like, makeConditionConstant(value));
	}

	Condition like(V)(V value)
	{
		return binaryCondition(BinaryConditionType.NotLike, makeConditionConstant(value));
	}

	Condition regexp(V)(V value)
	{
		return binaryCondition(BinaryConditionType.Regexp, makeConditionConstant(value));
	}

	Condition regexp(V)(V value)
	{
		return binaryCondition(BinaryConditionType.NotRegexp, makeConditionConstant(value));
	}

	Condition in_(V)(V value)
	{
		return binaryCondition(BinaryConditionType.In, makeConditionConstant(value));
	}

	Condition in_(V)(V value)
	{
		return binaryCondition(BinaryConditionType.In, makeConditionConstant(value));
	}

	Condition unaryCondition(UnaryConditionType type)
	{
		Condition ret;
		ret.type = Condition.Type.UnaryCondition;
		// TODO: need to provide some way to use allocator or other things instead of GC here
		ret.unaryCondition = new ffi.UnaryCondition();
		ret.unaryCondition.type = type;
		ret.unaryCondition.condition = lhs;
		return ret;
	}

	Condition isNull()
	{
		return unaryCondition(UnaryConditionType.IsNull);
	}

	alias equalsNull = isNull;

	Condition isNotNull()
	{
		return unaryCondition(UnaryConditionType.IsNotNull);
	}

	alias notEqualsNull = isNotNull;

	Condition exists()
	{
		return unaryCondition(UnaryConditionType.Exists);
	}

	Condition notExists()
	{
		return unaryCondition(UnaryConditionType.NotExists);
	}

	Condition ternaryCondition(TernaryConditionType type, Condition second, Condition third)
	{
		Condition ret;
		ret.type = Condition.ternaryCondition;
		// TODO: need to provide some way to use allocator or other things instead of GC here
		ret.ternaryCondition = new ffi.TernaryCondition();
		ret.ternaryCondition.type = type;
		ret.ternaryCondition.first = lhs;
		ret.ternaryCondition.second = second;
		ret.ternaryCondition.third = third;
		return ret;
	}

	Condition between(L, R)(L min, R max)
	{
		return ternaryCondition(TernaryConditionType.Between, makeConditionConstant(min), makeConditionConstant(max));
	}

	Condition notBetween(L, R)(L min, R max)
	{
		return ternaryCondition(TernaryConditionType.NotBetween, makeConditionConstant(min), makeConditionConstant(max));
	}
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
	private ffi.FFICondition[] conditionTree;
	private long offset, limit;

	static if (!hasWhere)
	{
		alias SelectBuilder = Condition delegate(ConditionBuilder!T);

		SelectOperation!(T, TSelect, true, hasOrder, hasLimit) select(SelectBuilder callback) return
		{
			ConditionBuilder!T builder;
			conditionTree = callback(builder).makeTree;
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
