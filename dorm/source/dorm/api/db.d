module dorm.api.db;

import dorm.declarative;
import dorm.declarative.conversion;
import dorm.lib.util;
import ffi = dorm.lib.ffi;

import std.conv : text;
import std.datetime : Clock, Date, DateTime, DateTimeException, SysTime, TimeOfDay, UTC;
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
@safe:
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
@safe:
	private ffi.DBHandle handle;

	@disable this();

	/**
	 * Performs a Database connection (possibly in another thread) and returns
	 * the constructed DormDB handle once connected.
	 */
	this(DBConnectOptions options) @trusted
	{
		// TODO: think of how to make async waiting configurable, right now the thread is just blocked
		auto ffiOptions = options.ffiInto!(ffi.DBConnectOptions);

		scope dbHandleAsync = FreeableAsyncResult!(ffi.DBHandle).make;
		ffi.rorm_db_connect(ffiOptions, dbHandleAsync.callback.expand);
		handle = dbHandleAsync.result;
	}

	~this() @trusted
	{
		if (handle)
		{
			ffi.rorm_db_free(handle);
			handle = null;
		}
	}

	@disable this(this);

	void insert(T)(T value)
	{
		alias DB = DBType!T;
		ffi.FFIString[DormFields!DB.length] columns;
		ffi.FFIValue[DormFields!DB.length] values;
		int used;

		static if (is(T == DB))
			alias validatorObject = value;
		else
			auto validatorObject = new DB();

		static foreach (field; DormFields!DB)
		{{
			static if (is(typeof(mixin("value." ~ field.sourceColumn))))
			{
				enum modifiedIfsCode = {
					string ret;
					auto ifs = field.getModifiedIfs();
					foreach (m; ifs)
					{
						if (ret.length) ret ~= " || ";
						ret ~= m.makeCheckCode("value.");
					}
					return ret.length ? ret : "true";
				}();

				if (mixin(modifiedIfsCode)
					&& !isImplicitlyIgnoredValue(mixin("value." ~ field.sourceColumn)))
				{
					columns[used] = ffi.ffi(field.columnName);
					values[used] = conditionValue!field(mixin("value." ~ field.sourceColumn));
					used++;
				}
			}
			else static if (field.hasConstructValue)
			{
				// filled in by constructor
				columns[used] = ffi.ffi(field.columnName);
				values[used] = conditionValue!field(mixin("validatorObject." ~ field.sourceColumn));
				used++;
			}
			else static if (field.hasGeneratedDefaultValue)
			{
				// OK
			}
			else static if (!is(T == DB))
				static assert(false, "Trying to insert a patch " ~ T.stringof
					~ " into " ~ DB.stringof ~ ", but it is missing the required field "
					~ field.sourceReferenceName ~ "! "
					~ "Fields with auto-generated values may be omitted in patch types. "
					~ ModelFormat.Field.humanReadableGeneratedDefaultValueTypes);
			else
				static assert(false, "wat? (defined DormField not found inside the Model class that defined it)");
		}}

		static if (is(T == DB))
		{
			auto brokenFields = value.runValidators();

			string error;
			foreach (field; brokenFields)
				error ~= "Field " ~ field.sourceColumn ~ " defined in "
					~ field.definedAt.toString ~ " failed user validation.";
			if (error.length)
				throw new Exception(error);
		}
		else
		{
			validatorObject.applyPatch(value);
			auto brokenFields = validatorObject.runValidators();

			string error;
			foreach (field; brokenFields)
			{
				switch (field.columnName)
				{
					static foreach (sourceField; DormFields!DB)
					{
						static if (is(typeof(mixin("value." ~ sourceField.sourceColumn))))
						{
							case sourceField.columnName:
						}
					}
					error ~= "Field " ~ field.sourceColumn ~ " defined in "
						~ field.definedAt.toString ~ " failed user validation.";
					break;
				default:
					break;
				}
			}

			if (error.length)
				throw new Exception(error);
		}


		(() @trusted {
			auto ctx = FreeableAsyncResult!void.make;
			ffi.rorm_db_insert(handle,
				ffi.ffi(DormLayout!DB.tableName),
				ffi.ffi(columns[0 .. used]),
				ffi.ffi(values[0 .. used]), ctx.callback.expand);
			ctx.result();
		})();
	}
}

// defined this as global so that we can pass `Foo.fieldName` as alias argument,
// to have it be selected.
static SelectOperation!(DBType!(Selection), SelectType!(Selection)) select(Selection...)(return ref DormDB db) @trusted
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
		static if (hasDormField!(T, T.tupleof[i].stringof))
			mixin("ConditionBuilderField!(typeof(T.tupleof[i]), DormField!(T, T.tupleof[i].stringof)) ",
				T.tupleof[i].stringof,
				" = ConditionBuilderField!(typeof(T.tupleof[i]), DormField!(T, T.tupleof[i].stringof))(`",
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

private Condition* makeConditionIdentifier(T)(T value) @safe
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionIdentifier(value));
}

private Condition* makeConditionConstant(ModelFormat.Field fieldInfo, T)(T value) @safe
{
	// TODO: think of how we can abstract memory allocation here
	return new Condition(conditionValue!fieldInfo(value));
}

struct ConditionBuilderField(T, ModelFormat.Field field)
{
	// TODO: all the type specific field to Condition thingies

	private string columnName;

	this(string columnName) @safe
	{
		this.columnName = columnName;
	}

	private Condition* lhs() @safe
	{
		return makeConditionIdentifier(columnName);
	}

	Condition equals(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Equals, lhs, makeConditionConstant!field(value)));
	}

	Condition notEquals(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition lessThan(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Less, lhs, makeConditionConstant!field(value)));
	}

	Condition lessThanOrEqual(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.LessOrEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition greaterThan(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Greater, lhs, makeConditionConstant!field(value)));
	}

	Condition greaterThanOrEqual(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.GreaterOrEquals, lhs, makeConditionConstant!field(value)));
	}

	Condition like(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Like, lhs, makeConditionConstant!field(value)));
	}

	Condition notLike(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotLike, lhs, makeConditionConstant!field(value)));
	}

	Condition regexp(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.Regexp, lhs, makeConditionConstant!field(value)));
	}

	Condition notRegexp(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotRegexp, lhs, makeConditionConstant!field(value)));
	}

	Condition in_(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.In, lhs, makeConditionConstant!field(value)));
	}

	Condition notIn(V)(V value) @safe
	{
		return Condition(BinaryCondition(BinaryConditionType.NotIn, lhs, makeConditionConstant!field(value)));
	}

	Condition isNull() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNull, lhs));
	}

	alias equalsNull = isNull;

	Condition isNotNull() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.IsNotNull, lhs));
	}

	alias notEqualsNull = isNotNull;

	Condition exists() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.Exists, lhs));
	}

	Condition notExists() @safe
	{
		return Condition(UnaryCondition(UnaryConditionType.NotExists, lhs));
	}

	Condition between(L, R)(L min, R max) @safe
	{
		return Condition(TernaryCondition(TernaryConditionType.Between, lhs, makeConditionConstant!field(min), makeConditionConstant!field(max)));
	}

	Condition notBetween(L, R)(L min, R max) @safe
	{
		return Condition(TernaryCondition(TernaryConditionType.NotBetween, lhs, makeConditionConstant!field(min), makeConditionConstant!field(max)));
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
@safe:
	private DormDB* db;
	private ffi.FFICondition[] conditionTree;
	private long offset, limit;

	static if (!hasWhere)
	{
		alias SelectBuilder = Condition delegate(ConditionBuilder!T);

		SelectOperation!(T, TSelect, true, hasOrder, hasLimit) condition(SelectBuilder callback) return @trusted
		{
			ConditionBuilder!T builder;
			conditionTree = callback(builder).makeTree;
			return cast(typeof(return))this;
		}
	}

	static if (!hasOrder)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) orderBy(T...)(T) return @trusted
		{
			static assert(false, "not implemented");
		}
	}

	static if (!hasOffset)
	{
		SelectOperation!(T, TSelect, hasWhere, true, hasLimit) drop(long offset) return @trusted
		{
			this.offset = offset;
			return cast(typeof(return))this;
		}
	}

	static if (!hasLimit)
	{
		// may not .drop after take!
		SelectOperation!(T, TSelect, hasWhere, true, true) take(long limit) return @trusted
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

		SelectOperation!(T, TSelect, hasWhere, true, true) opIndex(size_t[2] slice) return @trusted
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

	auto stream() @trusted
	{
		enum fields = FilterLayoutFields!(T, TSelect);

		ffi.FFIString[fields.length] columns;
		static foreach (i, field; fields)
			columns[i] = ffi.ffi(field.columnName);
		auto stream = sync_call!(ffi.rorm_db_query_stream)(db.handle,
			ffi.ffi(DormLayout!T.tableName),
			ffi.ffi(columns),
			conditionTree.length ? &conditionTree[0] : null);

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

		void reset() @safe
		{
			impl.reset();
			done = false;
		}
	}

	extern(C) private static void rowCallback(void* data, ffi.DBRowHandle result, scope ffi.RormError error) nothrow @trusted
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

	this(ffi.DBStreamHandle handle) @trusted
	{
		this.handle = handle;
		currentHandle = RowHandleState(FreeableAsyncResult!(ffi.DBRowHandle).make);
	}

	~this() @trusted
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

	auto front() @property @trusted
	{
		if (!started) nextIteration();
		return unwrapRowResult!(T, TSelect)(currentHandle.result());
	}
	
	bool empty() @property @trusted
	{
		if (!started) nextIteration();
		currentHandle.impl.event.wait();
		return currentHandle.done;
	}

	void popFront() @trusted
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

	private void nextIteration() @trusted
	{
		started = true;
		ffi.rorm_stream_get_row(handle, &rowCallback, cast(void*)&currentHandle);
	}

	static assert(isInputRange!RormStream, "implementation error: did not become an input range");
}


template FilterLayoutFields(T, TSelect)
{
	static if (is(T == TSelect))
		enum FilterLayoutFields = DormFields!T;
	else static if (is(TSelect == Model))
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

TSelect unwrapRowResult(T, TSelect)(ffi.DBRowHandle row) @safe
{
	TSelect res;
	static if (is(TSelect == class))
		res = new TSelect();
	ffi.RormError rowError;
	enum fields = FilterLayoutFields!(T, TSelect);
	static foreach (field; fields)
		mixin("res." ~ field.sourceColumn) = extractField!(field, typeof(mixin("res." ~ field.sourceColumn)),
			text(" from model ", T.stringof, " in column ", field.sourceColumn, " in file ", field.definedAt).idup)(row, rowError);
	if (rowError)
		throw rowError.makeException;
	return res;
}

private T extractField(alias field, T, string errInfo)(ffi.DBRowHandle row, ref ffi.RormError error) @trusted
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
		case floatNumber:
			static if (field.type == floatNumber) return fieldInto!(T, errInfo)(mixin(pre, "f32", suf), error);
			else assert(false);
		case doubleNumber:
			static if (field.type == doubleNumber) return fieldInto!(T, errInfo)(mixin(pre, "f64", suf), error);
			else assert(false);
		case boolean:
			static if (field.type == boolean) return fieldInto!(T, errInfo)(mixin(pre, "bool", suf), error);
			else assert(false);
		case date:
			static if (field.type == date) return fieldInto!(T, errInfo)(mixin(pre, "date", suf), error);
			else assert(false);
		case time:
			static if (field.type == time) return fieldInto!(T, errInfo)(mixin(pre, "time", suf), error);
			else assert(false);
		case datetime:
			static if (field.type == datetime) return fieldInto!(T, errInfo)(mixin(pre, "datetime", suf), error);
			else assert(false);

		static assert(
			field.type != set,
			"field type " ~ field.type.to!string ~ " not yet implemented for reading");

		case choices:
			static if (field.type == choices) return fieldInto!(T, errInfo)(mixin(pre, "str", suf), error);
			else assert(false);
		case set: assert(false);
	}
}

private T fieldInto(T, string errInfo, From)(scope From v, ref ffi.RormError error) @safe
{
	import dorm.lib.ffi : FFIArray, FFIOption;
	import std.typecons : Nullable;

	static if (is(T == From))
		return v;
	else static if (is(T == enum))
	{
		auto s = fieldInto!(string, errInfo, From)(v, error);
		static if (is(OriginalType!T == string))
			return cast(T)s;
		else
		{
			switch (s)
			{
				static foreach (f; __traits(allMembers, T))
				{
				case f:
					return __traits(getMember, T, f);
				}
				default:
					error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
					return T.init;
			}
		}
	}
	else static if (is(From == FFIArray!U, U))
	{
		static if (is(T == Res[], Res))
		{
			static if (is(immutable Res == immutable U))
				return (() @trusted => cast(T)v.data.dup)();
			else
				static assert(false, "can't auto-wrap array element type " ~ Res.stringof ~ " into " ~ U.stringof ~ errInfo);
		}
		else static if (is(T == Nullable!V, V))
		{
			return T(fieldInto!(V, errInfo, From)(v, error));
		}
		else
			static assert(false, "can't auto-wrap " ~ U.stringof ~ "[] into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From == FFIOption!U, U))
	{
		static if (is(T == Nullable!V, V))
		{
			if (v.isNull)
				return T.init;
			else
				return T(fieldInto!(V, errInfo)(v.raw_value, error));
		}
		else static if (__traits(compiles, T(null)))
		{
			if (v.isNull)
				return T(null);
			else
				return fieldInto!(T, errInfo)(v.raw_value, error);
		}
		else
		{
			if (v.isNull)
			{
				error = ffi.RormError(ffi.RormError.Tag.ColumnDecodeError);
				return T.init;
			}
			else
			{
				return fieldInto!(T, errInfo)(v.raw_value, error);
			}
		}
	}
	else static if (is(T == Nullable!U, U))
	{
		return T(fieldInto!(U, errInfo, From)(v, error));
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
	else static if (is(From : ffi.FFITime))
	{
		static if (is(T == TimeOfDay))
		{
			try
			{
				return TimeOfDay(cast(int)v.hour, cast(int)v.min, cast(int)v.sec);
			}
			catch (DateTimeException)
			{
				error = ffi.RormError(ffi.RormError.Tag.InvalidTimeError);
				return T.init;
			}
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From : ffi.FFIDate))
	{
		static if (is(T == Date))
		{
			try
			{
				return Date(cast(int)v.year, cast(int)v.month, cast(int)v.day);
			}
			catch (DateTimeException)
			{
				error = ffi.RormError(ffi.RormError.Tag.InvalidDateError);
				return T.init;
			}
		}
		else
			static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
	}
	else static if (is(From : ffi.FFIDateTime))
	{
		try
		{
			static if (is(T == DateTime))
			{
				return DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec);
			}
			else static if (is(T == SysTime))
			{
				return SysTime(DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec), UTC());
			}
			else static if (is(T == long) || is(T == ulong))
			{
				return cast(T)SysTime(DateTime(cast(int)v.year, cast(int)v.month, cast(int)v.day,
					cast(int)v.hour, cast(int)v.min, cast(int)v.sec), UTC()).stdTime;
			}
			else
				static assert(false, "can't put " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
		}
		catch (DateTimeException)
		{
			error = ffi.RormError(ffi.RormError.Tag.InvalidDateTimeError);
			return T.init;
		}
	}
	else
		static assert(false, "did not implement conversion from " ~ From.stringof ~ " into " ~ T.stringof ~ errInfo);
}

private bool isImplicitlyIgnoredValue(SysTime value) { return value == SysTime.init; }
private bool isImplicitlyIgnoredValue(T)(T value) { return false; }

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
