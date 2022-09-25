module dorm.api.db;

import ffi = dorm.lib.ffi;
import dorm.lib.util;
import dorm.declarative.conversion;

import std.meta;
import std.range.primitives;
import std.traits;

import core.time;

public import dorm.lib.ffi : DBBackend;

public import dorm.api.condition;

/**
 * Configuration operation to connect to a database.
 */
struct DBConnectOptions
{
	/// Specifies the driver that will be used.
	DBBackend backend;
	/// Name of the database, in case of `DatabaseBackend.SQLite` name of the file.
	string name;
	/// Host to connect to. Not used in case of `DatabaseBackend.SQLite`.
	string host;
	/// Port to connect to. Not used in case of `DatabaseBackend.SQLite`.
	ushort port;
	/// Username to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	string user;
	/// Password to authenticate with. Not used in case of `DatabaseBackend.SQLite`.
	string password;
	/// Minimal connections to initialize upfront. Must not be 0.
	uint minConnections = ffi.DBConnectOptions.init.minConnections;
	/// Maximum connections that allowed to be created. Must not be 0.
	uint maxConnections = ffi.DBConnectOptions.init.maxConnections;
}

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

/**
 * High-level wrapper around a database. Through the driver implementation layer
 * this handles connection pooling and distributes work across a thread pool
 * automatically.
 *
 * Use the UDA methods
 *
 * - $(LREF select)
 * - $(LREF insert)
 *
 * to access the database.
 *
 * This struct cannot be copied, to pass it around, use `ref`. Once the struct
 * goes out of scope or gets unset, the connection to the database will be freed.
 */
struct DormDB
{
	private ffi.DBHandle handle;

	@disable this();

	/**
	 * Performs a Database connection (possibly in another thread) and returns
	 * the constructed DormDB handle once connected.
	 */
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
		if (handle)
		{
			ffi.rorm_db_free(handle);
			handle = null;
		}
	}

	@disable this(this);
}

static SelectOperation!(DBType!(Selection), SelectType!(Selection)) select(Selection...)(return ref DormDB db)
{
	return typeof(return)(&db);
}

private template DBType(Selection...)
{
	static assert(Selection.length >= 1);

	static if (Selection.length > 1)
		alias DBType = Selection[0];
	else
	{
		alias PatchAttrs = getUDAs!(Selection[0], DormPatch);
		static if (PatchAttrs.length == 0)
			alias DBType = Selection[0];
		else static if (PatchAttrs.length == 1)
		{
			static if (is(PatchAttrs[0] == DormPatch!T, T))
				alias DBType = T;
			else
				static assert(false, "internal template error");
		}
		else
			static assert(false, "Cannot annotate DormPatch struct with multiple DormPatch UDAs.");
	}
}

private template SelectType(T, Selection...)
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
			DormField!(T, T.tupleof[i].stringof).columnName,
			"`);");

	static if (__traits(allMembers, NotConditionBuilder!T).length > 1)
		NotConditionBuilder!T not;
	else
		void not()() { static assert(false, "Model " ~ T.stringof
			~ " has no fields that can be used with .not"); }

	auto opDispatch(string member)()
	{
		import std.string : join;

		pragma(msg, supplErrorPrefix ~ "cannot access condition field '" ~ member ~ "'. Available members are: "
			~ [__traits(allMembers, ConditionBuilder)][0 .. ConditionBuilder.tupleof.length].join(", "));
		static assert(false);
	}
}

private enum supplErrorPrefix = "           \x1B[1;31mDORM Error:\x1B[m ";

struct NotConditionBuilder(T)
{
	static foreach (i, member; T.tupleof)
	{
		static if (is(typeof(T.tupleof[i]) : bool))
		{
			mixin("Condition ",
				T.tupleof[i].stringof,
				"() { return Condition(UnaryCondition(UnaryConditionType.Not,
					makeConditionIdentifier(`",
				DormField!(T, T.tupleof[i].stringof).columnName,
				"`))); }");
		}
	}

	auto opDispatch(string member)()
	{
		import std.string : join;

		pragma(msg, supplErrorPrefix ~ "cannot access negated condition field '" ~ member ~ "'. Available members are: "
			~ [__traits(allMembers, NotConditionBuilder)][0 .. $ - 1].join(", "));
		static assert(false);
	}
}

private Condition* makeConditionIdentifier(T)(T value)
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionIdentifier(value));
}

private Condition* makeConditionConstant(T)(T value)
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionValue(value));
}

struct ConditionBuilderField(T)
{
	// TODO: all the type specific field to Condition thingies

	private string columnName;

	this(string columnName)
	{
		this.columnName = columnName;
	}

	private Condition* lhs()
	{
		return makeConditionIdentifier(columnName);
	}

	Condition equals(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.Equals, lhs, makeConditionConstant(value)));
	}

	Condition notEquals(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.NotEquals, lhs, makeConditionConstant(value)));
	}

	Condition lessThan(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.Less, lhs, makeConditionConstant(value)));
	}

	Condition lessThanOrEqual(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.LessOrEquals, lhs, makeConditionConstant(value)));
	}

	Condition greaterThan(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.Greater, lhs, makeConditionConstant(value)));
	}

	Condition greaterThanOrEqual(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.GreaterOrEquals, lhs, makeConditionConstant(value)));
	}

	Condition like(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.Like, lhs, makeConditionConstant(value)));
	}

	Condition notLike(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.NotLike, lhs, makeConditionConstant(value)));
	}

	Condition regexp(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.Regexp, lhs, makeConditionConstant(value)));
	}

	Condition notRegexp(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.NotRegexp, lhs, makeConditionConstant(value)));
	}

	Condition in_(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.In, lhs, makeConditionConstant(value)));
	}

	Condition notIn(V)(V value)
	{
		return Condition(BinaryCondition(BinaryConditionType.NotIn, lhs, makeConditionConstant(value)));
	}

	Condition isNull()
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNull, lhs));
	}

	alias equalsNull = isNull;

	Condition isNotNull()
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNotNull, lhs));
	}

	alias notEqualsNull = isNotNull;

	Condition exists()
	{
		return Condition(UnaryCondition(UnaryConditionType.Exists, lhs));
	}

	Condition notExists()
	{
		return Condition(UnaryCondition(UnaryConditionType.NotExists, lhs));
	}

	Condition between(L, R)(L min, R max)
	{
		return Condition(TernaryCondition(TernaryConditionType.Between, lhs, makeConditionConstant(min), makeConditionConstant(max)));
	}

	Condition notBetween(L, R)(L min, R max)
	{
		return Condition(TernaryCondition(TernaryConditionType.NotBetween, lhs, makeConditionConstant(min), makeConditionConstant(max)));
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

		SelectOperation!(T, TSelect, true, hasOrder, hasLimit) condition(SelectBuilder callback) return
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

	version(none)
	TSelect[] array()
	{
		scope handle = ffi.rorm_db_query_all();
	}

	auto stream()
	{
		enum fields = FilterLayoutFields!(T, TSelect);

		ffi.FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.columnName);
		auto stream = sync_call!(ffi.rorm_db_query_stream)(db.handle,
			ffi.ffi(DormLayout!T.tableName),
			ffi.ffi(columns),
			&conditionTree[0]);

		return RormStream!(T, TSelect)(stream);
	}
}

private struct RormStream(T, TSelect)
{
	import dorm.lib.util;

	private static struct RowHandleState
	{
		FreeableAsyncResult!(ffi.DBRowHandle) impl;
		alias impl this;
		bool done;

		void reset()
		{
			impl.reset();
			done = false;
		}
	}

	extern(C) private static void rowCallback(void* data, ffi.DBRowHandle result, scope ffi.RormError error) nothrow
	{
		auto res = cast(RowHandleState*)data;
		if (error.tag == ffi.RormError.Tag.NoRowsLeftInStream)
			res.done = true;
		else if (error)
			res.error = error.makeException;
		else
			res.raw_result = result;
		res.event.set();
	}

	private ffi.DBStreamHandle handle;
	private RowHandleState currentHandle;
	private bool started;

	this(ffi.DBStreamHandle handle)
	{
		this.handle = handle;
		currentHandle = RowHandleState(FreeableAsyncResult!(ffi.DBRowHandle).make);
	}

	~this()
	{
		if (started)
		{
			currentHandle.impl.event.wait();
			if (currentHandle.impl.raw_result !is null)
				ffi.rorm_row_free(currentHandle.impl.raw_result);
			ffi.rorm_stream_free(handle);
		}
	}

	@disable this(this);

	int opApply(scope int delegate(TSelect) dg)
	{
		int result = 0;
		for (; !this.empty; this.popFront())
		{
			result = dg(this.front);
			if (result)
				break;
		}
		return result;
	}

	int opApply(scope int delegate(size_t i, TSelect) dg)
	{
		int result = 0;
		size_t i;
		for (; !this.empty; this.popFront())
		{
			result = dg(i++, this.front);
			if (result)
				break;
		}
		return result;
	}

	auto front() @property
	{
		if (!started) nextIteration();
		return unwrapRowResult!(T, TSelect)(currentHandle.result());
	}
	
	bool empty() @property
	{
		if (!started) nextIteration();
		currentHandle.impl.event.wait();
		return currentHandle.done;
	}
	
	void popFront()
	{
		if (!started) nextIteration();
		currentHandle.impl.event.wait();
		if (currentHandle.done)
			throw new Exception("attempted to run popFront on ended stream");
		else if (currentHandle.impl.error)
			throw currentHandle.impl.error;
		else
		{
			ffi.rorm_row_free(currentHandle.impl.raw_result);
			currentHandle.reset();
			nextIteration();
		}
	}

	private void nextIteration()
	{
		started = true;
		ffi.rorm_stream_get_row(handle, &rowCallback, cast(void*)&currentHandle);
	}

	static assert(isInputRange!RormStream, "implementation error: did not become an input range");
}


template FilterLayoutFields(T, TSelect)
{
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
				field ~ ".");
		else
			ret ~= (prefix ~ field);
	}
	return ret;
}

TSelect unwrapRowResult(T, TSelect)(ffi.DBRowHandle row)
{
	import std.conv : text;

	TSelect res;
	ffi.RormError rowError;
	enum layout = DormLayout!T;
	enum fields = FilterLayoutFields!(T, TSelect);
	static foreach (field; fields)
		mixin("res." ~ field.sourceColumn) = extractField!(field, typeof(mixin("res." ~ field.sourceColumn)),
			text(" from model ", T.stringof, " in column ", field.sourceColumn, " in file ", field.definedAt))(row, rowError);
	if (rowError)
		throw rowError.makeException;
	return res;
}

private T extractField(alias field, T, string errInfo)(ffi.DBRowHandle row, ref ffi.RormError error)
{
	import std.conv;
	import dorm.declarative;

	auto columnName = ffi.ffi(field.columnName);
	enum pre = field.isNullable() ? "ffi.rorm_row_get_null_" : "ffi.rorm_row_get_";
	enum suf = "(row, columnName, error)";

	final switch (field.type) with (ModelFormat.Field.DBType)
	{
		case varchar:
			static if (field.type == varchar) return fieldInto!(T, errInfo)(mixin(pre, "str", suf), error);
			else assert(false);
		case varbinary:
			static if (field.type == varbinary) return fieldInto!(T, errInfo)(mixin(pre, "binary", suf), error);
			else assert(false);
		case int8:
			static if (field.type == int8) return fieldInto!(T, errInfo)(mixin(pre, "i16", suf), error);
			else assert(false);
		case int16:
			static if (field.type == int16) return fieldInto!(T, errInfo)(mixin(pre, "i16", suf), error);
			else assert(false);
		case int32:
			static if (field.type == int32) return fieldInto!(T, errInfo)(mixin(pre, "i32", suf), error);
			else assert(false);
		case int64:
			static if (field.type == int64) return fieldInto!(T, errInfo)(mixin(pre, "i64", suf), error);
			else assert(false);
		case uint8:
			static if (field.type == uint8) return fieldInto!(T, errInfo)(mixin(pre, "i16", suf), error);
			else assert(false);
		case uint16:
			static if (field.type == uint16) return fieldInto!(T, errInfo)(mixin(pre, "i32", suf), error);
			else assert(false);
		case uint32:
			static if (field.type == uint32) return fieldInto!(T, errInfo)(mixin(pre, "i64", suf), error);
			else assert(false);
		case floatNumber:
			static if (field.type == floatNumber) return fieldInto!(T, errInfo)(mixin(pre, "f32", suf), error);
			else assert(false);
		case doubleNumber:
			static if (field.type == doubleNumber) return fieldInto!(T, errInfo)(mixin(pre, "f64", suf), error);
			else assert(false);
		case boolean:
			static if (field.type == boolean) return fieldInto!(T, errInfo)(mixin(pre, "bool", suf), error);
			else assert(false);

		static assert(field.type != date &&
			field.type != datetime &&
			field.type != timestamp &&
			field.type != time &&
			field.type != set,
			"field type " ~ field.type.to!string ~ " not yet implemented for reading");
		case date: assert(false);
		case datetime: assert(false);
		case timestamp: assert(false);
		case time: assert(false);

		case choices:
			static if (field.type == choices) return fieldInto!T(mixin(pre, "str", suf), error);
			else assert(false);
		case set: assert(false);
	}
}

private T fieldInto(T, string errInfo, From)(From v, ref ffi.RormError error)
{
	import dorm.lib.ffi : FFIArray, FFIOption;

	static if (is(From == FFIArray!U, U))
	{
		auto data = v[];
		static if (is(T == Res[], Res))
		{
			static if (is(immutable Res == immutable U))
				return cast(Res[])data;
			else
				static assert(false, "can't auto-wrap array element type " ~ Res.stringof ~ " into " ~ U.stringof ~ errInfo);
		}
		else
			static assert(false, "can't auto-wrap " ~ U.stringof ~ "[] into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From == FFIOption!U, U))
	{
		static if (__traits(compiles, T(null)))
		{
			if (v.isNull)
				return T(null);
			else
				return fieldInto!(T, errInfo)(v.raw_value, error);
		}
		else
			static assert(false, "can't put optional " ~ U.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (isIntegral!From)
	{
		static if (isIntegral!T && From.sizeof >= T.sizeof)
		{
			if (v < cast(From)T.min || v > cast(From)T.max)
			{
				error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
				return T.init;
			}
			else
			{
				return cast(T)v;
			}
		}
		else static if (isFloatingPoint!T)
		{
			return cast(T)v;
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (isFloatingPoint!From)
	{
		static if (isFloatingPoint!T)
			return cast(T)v;
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else
		static assert(false, "did not implement conversion from " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
}

mixin template SetupDormRuntime(alias timeout = 10.seconds)
{
	__gshared bool _initializedDormRuntime;

	shared static this()
	{
		import dorm.lib.util : sync_call;
		import dorm.lib.ffi : rorm_runtime_start;

		sync_call!(rorm_runtime_start)();
		_initializedDormRuntime = true;
	}

	shared static ~this()
	{
		import core.time : Duration;
		import dorm.lib.util;
		import dorm.lib.ffi : rorm_runtime_shutdown;

		if (_initializedDormRuntime)
		{
			static if (is(typeof(timeout) == Duration))
				sync_call!(rorm_runtime_shutdown)(timeout.total!"msecs");
			else
				sync_call!(rorm_runtime_shutdown)(timeout);
		}
	}
}
